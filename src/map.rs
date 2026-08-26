//! Default map generation + isometric rendering + camera.
//! Ports the map-build loop of `pack/MapContainer.as` (buildMap) and the
//! default layout of `pack/zmap.as` for EXPAND level 0 (37x37).

use crate::data::{Anchors, AppState, DataHandles, GameData};
use crate::iso;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::collections::HashMap;

pub const ROWS_MAX: i32 = 50;
pub const COLS_MAX: i32 = 50;
const CTR_X: i32 = 10;
const CTR_Y: i32 = 11;

// z layers
const Z_TILE: f32 = 0.0;
const Z_TILE_DECOR: f32 = 1.0; // grid, crabs
const Z_OBJ: f32 = 10.0;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(OnEnter(AppState::Playing), spawn_map)
            .add_systems(
                Update,
                (camera_pan_zoom,).run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    let (cx, cy) = iso::tile_center(14, 22); // lobby
    commands.spawn((
        Camera2d,
        Transform::from_xyz(cx, cy, 1000.0),
        MainCamera,
    ));
}

/// Normalized sprite anchor for a pixel point (top-left origin) in a WxH image.
fn px_anchor(ax: f32, ay: f32, w: f32, h: f32) -> Anchor {
    Anchor::Custom(Vec2::new(ax / w - 0.5, 0.5 - ay / h))
}

struct Ctx<'a> {
    assets: &'a AssetServer,
    tile_anchor: Anchor,
}

impl<'a> Ctx<'a> {
    fn tile_img(&self, fr: u32) -> Handle<Image> {
        self.assets.load(format!("sprites/tiles/{fr}.png"))
    }
}

fn spawn_tile(commands: &mut Commands, ctx: &Ctx, tx: i32, ty: i32, fr: u32, z: f32) {
    let (wx, wy) = iso::tile_to_world(tx, ty);
    commands.spawn((
        Sprite {
            image: ctx.tile_img(fr),
            anchor: ctx.tile_anchor,
            ..default()
        },
        Transform::from_xyz(wx, wy, z),
    ));
}

fn spawn_map(
    mut commands: Commands,
    assets: Res<AssetServer>,
    handles: Res<DataHandles>,
    gamedata: Res<Assets<GameData>>,
    anchors: Res<Assets<Anchors>>,
    mut occupancy: ResMut<crate::build::Occupancy>,
) {
    let gd = gamedata.get(&handles.gamedata).unwrap();
    let an = anchors.get(&handles.anchors).unwrap();
    let expand = 0usize;

    let ta = &an.tile;
    let ctx = Ctx {
        assets: &assets,
        tile_anchor: px_anchor(ta.anchor[0], ta.anchor[1], ta.size[0], ta.size[1]),
    };

    let rows = gd.expand[expand].rows; // 37
    let cols = gd.expand[expand].cols; // 37
    let lahan = gd.expand[expand]
        .lahan
        .as_ref()
        .map(|l| (l[0] as i32, l[1] as i32))
        .unwrap_or((16, 16));
    let bound_x = CTR_X + lahan.0; // 26
    let bound_y = CTR_Y + lahan.1; // 27
    let bound2_x = bound_x + 3; // 29
    let bound2_y = bound_y + 3; // 30
    let bound3_x = bound2_x + 2; // 31
    let bound3_y = bound2_y + 2; // 32

    // frame lookup per tile jenis (PNG number == REF.fr, see BlittingSingle)
    let fr_of = |jenis: u32| gd.tile_def(jenis).map(|t| t.fr).unwrap_or(24);

    // zmap default overrides: roads
    let mut tile_default: HashMap<(i32, i32), u32> = HashMap::new();
    for i in 10..16 {
        for j in 20..=21 {
            tile_default.insert((i, j), 1);
        }
    }
    tile_default.insert((10, 22), 1);
    tile_default.insert((10, 23), 1);

    // ---- ground tiles (mirror of buildMap branches) ----
    for i in 0..ROWS_MAX {
        for j in 0..COLS_MAX {
            let is_lahan =
                i > CTR_X && i < bound_x && j > CTR_Y && j < bound_y;
            let jenis: u32 = if i == 4 || i == 5 || j == 5 || j == 6 {
                83 // asphalt roads
            } else if (j > bound2_y && j <= bound3_y) || (i > bound2_x && i <= bound3_x) {
                83
            } else if j > 6 && j <= bound2_y && i > 5 && i <= bound2_x {
                if is_lahan {
                    80 // sand (buildable land)
                } else {
                    82 // sidewalk
                }
            } else {
                0 // grass base
            };
            let jenis = tile_default.get(&(i, j)).copied().unwrap_or(jenis);
            spawn_tile(&mut commands, &ctx, i, j, fr_of(jenis), Z_TILE);
            if is_lahan && !tile_default.contains_key(&(i, j)) {
                // grid overlay on buildable land (captureArray[24] -> 25.png)
                spawn_tile(&mut commands, &ctx, i, j, 25, Z_TILE_DECOR);
            }
        }
    }

    // ---- zmap decorations, with expand shifts ----
    let shift = |i: i32, j: i32| -> (i32, i32) {
        let l7 = COLS_MAX - 4; // 46
        let l8 = COLS_MAX - cols; // 13
        let l9 = ROWS_MAX - 5; // 45
        let l10 = ROWS_MAX - rows; // 13
        if i > 5 {
            if j >= l7 {
                (i, j - l8)
            } else if j > 6 && i >= l9 {
                (i - l10, j)
            } else {
                (i, j)
            }
        } else {
            (i, j)
        }
    };

    // crabs (kepiting): tiles/28.png on tile canvas
    for &(i, j) in KEPITING {
        let (si, sj) = shift(i, j);
        spawn_tile(&mut commands, &ctx, si, sj, 28, Z_TILE_DECOR);
    }

    // sceneries
    for &(i, j, jenis) in SCENERIES {
        let (si, sj) = shift(i, j);
        spawn_scenery_at(&mut commands, &assets, an, si, sj, jenis);
    }

    // ---- Lobby booth at (14,22) ----
    spawn_booth_at(&mut commands, &assets, gd, an, "Lobby", 14, 22);
    if let Some(lobby) = gd.booth_def("Lobby") {
        occupancy.occupy_booth(14, 22, lobby.rows, lobby.cols);
    }
}

pub fn spawn_scenery_at(
    commands: &mut Commands,
    assets: &AssetServer,
    an: &Anchors,
    tx: i32,
    ty: i32,
    jenis: u32,
) {
    let Some(sa) = an.scenery.get(&format!("PLANT_{jenis}")) else {
        return;
    };
    let (cx, cy) = iso::tile_center(tx, ty);
    // base (bottom-center) sits slightly below tile center, like the original art
    commands.spawn((
        Sprite {
            image: assets.load(format!("sprites/scenery/{}.png", sa.fr)),
            anchor: Anchor::Custom(Vec2::new(sa.base[0] / 49.0 - 0.5, 0.5 - sa.base[1] / 54.0)),
            ..default()
        },
        Transform::from_xyz(cx, cy - 6.0, Z_OBJ + (tx + ty) as f32 * 0.01),
    ));
}

pub fn spawn_booth_at(
    commands: &mut Commands,
    assets: &AssetServer,
    gd: &GameData,
    an: &Anchors,
    name: &str,
    tx: i32,
    ty: i32,
) {
    let Some(b) = gd.booth_def(name) else { return };
    let Some(ba) = an.booth.get(name) else { return };
    // footprint: tx-ROWS+1..=tx, ty..=ty+COLS-1; its N vertex belongs to
    // tile (tx-ROWS+1, ty)
    let (wx, wy) = iso::tile_to_world(tx - b.rows + 1, ty);
    let depth_sum = tx + ty + b.cols - 1; // southernmost footprint tile
    // floor / platform sprite (booths/fr.png)
    commands.spawn((
        Sprite {
            image: assets.load(format!("sprites/booths/{}.png", b.fr)),
            anchor: px_anchor(ba.anchor[0], ba.anchor[1], 218.0, 112.0),
            ..default()
        },
        Transform::from_xyz(wx, wy, Z_OBJ + depth_sum as f32 * 0.01),
    ));
    // building / wall sprite (walls/N.png) drawn on top of the floor.
    // Its measured `n_corner` pixel (intersection of the two wall base edges)
    // must sit on the same footprint N vertex as the floor anchor.
    if let Some(wall_fr) = b.wall {
        if let Some(wa) = an.wall.get(&wall_fr.to_string()) {
            commands.spawn((
                Sprite {
                    image: assets.load(format!("sprites/walls/{wall_fr}.png")),
                    anchor: px_anchor(wa.n_corner[0], wa.n_corner[1], wa.size[0], wa.size[1]),
                    ..default()
                },
                Transform::from_xyz(wx, wy, Z_OBJ + depth_sum as f32 * 0.01 + 0.005),
            ));
        }
    }
}

// ---------------- camera ----------------

fn camera_pan_zoom(
    mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut wheel: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    let Ok((mut tf, mut proj)) = cam.single_mut() else {
        return;
    };
    let Projection::Orthographic(ortho) = &mut *proj else {
        return;
    };

    // zoom
    for ev in wheel.read() {
        let factor = if ev.y > 0.0 { 0.9 } else { 1.1 };
        ortho.scale = (ortho.scale * factor).clamp(0.35, 3.0);
    }

    // drag pan
    if buttons.pressed(MouseButton::Left) || buttons.pressed(MouseButton::Middle) {
        for ev in motion.read() {
            tf.translation.x -= ev.delta.x * ortho.scale;
            tf.translation.y += ev.delta.y * ortho.scale;
        }
    } else {
        motion.clear();
    }

    // keyboard pan
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        dir.y -= 1.0;
    }
    if dir != Vec2::ZERO {
        let speed = 600.0 * ortho.scale;
        tf.translation.x += dir.x * speed * time.delta_secs();
        tf.translation.y += dir.y * speed * time.delta_secs();
    }
}

// ---------------- zmap static data ----------------

/// Crab decoration positions (pack/zmap.as MAP.kepiting).
const KEPITING: &[(i32, i32)] = &[
    (-3, 28),
    (-5, 22),
    (-4, 22),
    (0, 19),
    (23, -4),
    (3, 14),
    (3, 19),
    (1, 28),
    (2, 35),
    (17, 0),
    (29, 0),
    (22, 4),
    (13, 3),
    (24, 3),
    (27, 49),
    (29, 47),
    (24, 46),
    (45, 25),
    (45, 39),
    (47, 19),
    (49, 33),
    (47, 33),
    (36, 48),
    (55, 26),
    (54, 27),
    (52, 24),
    (30, 52),
    (24, 55),
    (21, 50),
    (26, 56),
    (14, 46),
];

/// Default sceneries (pack/zmap.as MAP.sceneries): (tx, ty, PLANT_jenis).
const SCENERIES: &[(i32, i32, u32)] = &[
    (0, 17, 15),
    (0, 18, 15),
    (18, 0, 11),
    (2, 16, 10),
    (2, 23, 12),
    (22, 3, 13),
    (2, 29, 13),
    (2, 30, 14),
    (0, 26, 15),
    (15, 4, 16),
    (14, 4, 16),
    (18, 1, 14),
    (15, 1, 17),
    (21, 4, 17),
    (1, 21, 17),
    (16, 46, 12),
    (17, 46, 12),
    (23, 48, 13),
    (36, 47, 11),
    (30, 49, 16),
    (45, 16, 15),
    (45, 17, 13),
    (48, 22, 12),
    (48, 23, 15),
    (45, 27, 16),
    (45, 28, 16),
    (45, 35, 16),
    (46, 37, 14),
    (46, 38, 14),
    (7, 8, 7),
    (6, 30, 7),
    (19, 1, 7),
    (29, 7, 7),
    (12, 4, 7),
    (16, 4, 7),
    (23, 4, 7),
    (6, 7, 7),
    (7, 7, 7),
    (6, 8, 7),
    (6, 11, 7),
    (6, 12, 7),
    (3, 11, 7),
    (3, 12, 7),
    (3, 18, 7),
    (1, 23, 7),
    (3, 27, 7),
    (3, 28, 7),
    (2, 22, 7),
    (19, 0, 7),
    (23, 0, 7),
    (26, 3, 7),
    (6, 29, 7),
    (6, 24, 7),
    (6, 25, 7),
    (6, 26, 7),
    (34, 4, 7),
    (23, 49, 7),
    (21, 48, 7),
    (19, 46, 7),
    (15, 46, 7),
    (19, 49, 7),
    (45, 29, 7),
    (47, 27, 7),
    (45, 21, 7),
    (48, 21, 7),
    (46, 17, 7),
    (35, 4, 7),
    (36, 4, 7),
    (45, 34, 7),
    (45, 36, 7),
    (45, 37, 7),
    (11, 46, 7),
    (3, 35, 7),
    (3, 36, 7),
    (3, 34, 7),
    (12, 46, 7),
    (10, 46, 7),
    (34, 48, 7),
    (32, 47, 7),
    (45, 12, 7),
];
