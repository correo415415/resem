//! Construction system: pick an item from the build menu, ghost preview
//! follows the cursor over buildable land (lahan), click to place.
//! Ports the placement rules of MapContainer/Booth (footprint, occupancy, cost).

use crate::data::{Anchors, AppState, DataHandles, GameData};
use crate::game::Wallet;
use crate::ui::HudCounts;
use crate::iso;
use crate::map::{spawn_booth_at, spawn_scenery_at, MainCamera};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

// Buildable land bounds for EXPAND 0 (from MapContainer: ctr+1 .. ctr+lahan-1)
pub const LAHAN_X: (i32, i32) = (11, 25);
pub const LAHAN_Y: (i32, i32) = (12, 26);

#[derive(Clone, Debug, PartialEq)]
pub enum BuildItem {
    Booth(String),
    Scenery(u32),
    Tile(u32),
}

#[derive(Resource, Default)]
pub struct BuildMode {
    pub selected: Option<BuildItem>,
}

/// Which build dialog (original dialog_room/facility/scenery/tile clip) is open.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DialogKind {
    Room,
    Facility,
    Scenery,
    Tile,
}

#[derive(Resource, Default)]
pub struct DialogState {
    pub open: Option<DialogKind>,
}

/// Which tiles are occupied by placed objects (booths, sceneries, lobby).
#[derive(Resource, Default)]
pub struct Occupancy {
    pub tiles: HashMap<(i32, i32), ()>,
}

impl Occupancy {
    pub fn occupy_booth(&mut self, tx: i32, ty: i32, rows: i32, cols: i32) {
        for i in (tx - rows + 1)..=tx {
            for j in ty..(ty + cols) {
                self.tiles.insert((i, j), ());
            }
        }
    }
    pub fn booth_free(&self, tx: i32, ty: i32, rows: i32, cols: i32) -> bool {
        for i in (tx - rows + 1)..=tx {
            for j in ty..(ty + cols) {
                if self.tiles.contains_key(&(i, j)) || !in_lahan(i, j) {
                    return false;
                }
            }
        }
        true
    }
}

pub fn in_lahan(tx: i32, ty: i32) -> bool {
    (LAHAN_X.0..=LAHAN_X.1).contains(&tx) && (LAHAN_Y.0..=LAHAN_Y.1).contains(&ty)
}

#[derive(Component)]
struct Ghost;

/// A ground tile repainted by the player (original: MapContainer tile layer).
#[derive(Component)]
struct PaintedTile(i32, i32);

#[derive(Component)]
struct DialogRoot(DialogKind);

/// Item button inside a dialog: swaps 1_up / 3_down frames like DefineButton2.
#[derive(Component)]
struct ItemBtn {
    item: BuildItem,
    up: Handle<Image>,
    down: Handle<Image>,
}

pub struct BuildPlugin;

impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildMode>()
            .init_resource::<Occupancy>()
            .init_resource::<DialogState>()
            .add_systems(OnEnter(AppState::Playing), spawn_build_dialogs)
            .add_systems(
                Update,
                (menu_buttons, sync_dialogs, ghost_and_place)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

// ---------------- build dialogs (original dialog_* MovieClips) ----------------
//
// Geometry decoded from the SWF PlaceObject2 tags of sprites 3205 (room),
// 3142 (facility), 3230 (scenery) and 3184 (tile). Each dialog opens at
// stage x = PX(42) with its own PY (room 137, facility 85, scenery 150,
// tile 140); the exported panel PNG's top-left sits at (4.3, 2.9) in clip
// space (11.3, 9.9 for the tile dialog), hence the adjusted lefts/tops.
// Frame 1 of the panel art has the locked state baked in (lock + "00"
// boxes), so unlocked items simply draw their DefineButton2 art on top.

/// (kind, panel file, stage left, stage top, width, height)
const DIALOGS: [(DialogKind, &str, f32, f32, f32, f32); 4] = [
    (DialogKind::Room, "panel_room", 46.3, 139.9, 86.0, 90.0),
    (DialogKind::Facility, "panel_facility", 46.3, 87.9, 148.0, 290.0),
    (DialogKind::Scenery, "panel_scenery", 46.3, 152.9, 86.0, 147.0),
    (DialogKind::Tile, "panel_tile", 53.3, 149.9, 72.0, 217.0),
];

/// Room dialog: (button id, booth name, slot x, slot y) in panel space.
const ROOM_ITEMS: [(u32, &str, f32, f32); 2] =
    [(3204, "Cottage", 16.0, 19.0), (3202, "Lodge", 16.0, 48.0)];

/// Facility dialog: instance name -> button id mapping from PlaceObject2.
const FACILITY_ITEMS: [(u32, &str, f32, f32); 17] = [
    (3132, "Icecream", 20.0, 18.0),
    (3126, "Sauna", 81.0, 18.0),
    (3124, "Gym", 20.0, 47.0),
    (3122, "Bar", 81.0, 47.0),
    (3120, "Hotdog", 20.0, 77.0),
    (3118, "Jacuzi", 81.0, 77.0),
    (3116, "Arcade", 20.0, 106.0),
    (3114, "Taco", 81.0, 106.0),
    (3112, "IndiaResto", 20.0, 135.0),
    (3106, "Spa", 81.0, 135.0),
    (3110, "JapanResto", 20.0, 163.0),
    (3130, "Giftshop", 81.0, 163.0),
    (3108, "Medical", 20.0, 192.0),
    (3128, "BaratResto", 81.0, 192.0),
    (3134, "Minimarket", 20.0, 221.0),
    (3136, "Pool", 81.0, 221.0),
    (3138, "Golf", 20.0, 249.0),
];

/// Scenery dialog: note the PLANT 6/7/8 button-id swap in the original clip.
const SCENERY_ITEMS: [(u32, u32, f32, f32); 8] = [
    (3208, 1, 16.0, 17.0),
    (3211, 2, 46.0, 17.0),
    (3214, 3, 16.0, 45.0),
    (3217, 4, 46.0, 45.0),
    (3220, 5, 16.0, 80.0),
    (3229, 6, 46.0, 80.0),
    (3223, 7, 16.0, 108.0),
    (3226, 8, 46.0, 108.0),
];

/// Tile dialog: TILE_n uses button id 3147 + 3*(n-1); two-column layout.
const TILE_SLOTS: [(f32, f32); 13] = [
    (9.0, 10.0),
    (39.0, 10.0),
    (9.0, 38.0),
    (39.0, 38.0),
    (9.0, 73.0),
    (39.0, 73.0),
    (9.0, 101.0),
    (39.0, 101.0),
    (9.0, 130.0),
    (39.0, 130.0),
    (9.0, 157.0),
    (39.0, 157.0),
    (9.0, 185.0),
];

/// Initial unlocks from Application DefaultGameVars (game.UNLOCKED).
fn initially_unlocked(name: &str) -> bool {
    matches!(
        name,
        "Cottage" | "Sauna" | "Icecream" | "PLANT_1" | "PLANT_2" | "PLANT_3"
    )
}

fn spawn_build_dialogs(mut commands: Commands, assets: Res<AssetServer>) {
    for (kind, panel, left, top, w, h) in DIALOGS {
        commands
            .spawn((
                ImageNode::new(assets.load(format!("sprites/ui/dialogs/{panel}.png"))),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(left),
                    top: Val::Px(top),
                    width: Val::Px(w),
                    height: Val::Px(h),
                    ..default()
                },
                ZIndex(30),
                Visibility::Hidden,
                DialogRoot(kind),
            ))
            .with_children(|p| {
                match kind {
                    DialogKind::Room => {
                        for (id, name, x, y) in ROOM_ITEMS {
                            if initially_unlocked(name) {
                                item_button(p, &assets, id, BuildItem::Booth(name.into()), x, y);
                            }
                        }
                    }
                    DialogKind::Facility => {
                        for (id, name, x, y) in FACILITY_ITEMS {
                            if initially_unlocked(name) {
                                item_button(p, &assets, id, BuildItem::Booth(name.into()), x, y);
                            }
                        }
                    }
                    DialogKind::Scenery => {
                        for (id, jenis, x, y) in SCENERY_ITEMS {
                            if initially_unlocked(&format!("PLANT_{jenis}")) {
                                item_button(p, &assets, id, BuildItem::Scenery(jenis), x, y);
                            }
                        }
                    }
                    DialogKind::Tile => {
                        // all floor tiles are available from the start
                        for (i, (x, y)) in TILE_SLOTS.iter().enumerate() {
                            let n = i as u32 + 1;
                            item_button(
                                p,
                                &assets,
                                3147 + 3 * (n - 1),
                                BuildItem::Tile(n),
                                *x,
                                *y,
                            );
                        }
                    }
                }
            });
    }
}

fn item_button(
    p: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    id: u32,
    item: BuildItem,
    x: f32,
    y: f32,
) {
    let up: Handle<Image> = assets.load(format!("sprites/ui/dialogs/items/btn_{id}_up.png"));
    let down: Handle<Image> = assets.load(format!("sprites/ui/dialogs/items/btn_{id}_down.png"));
    p.spawn((
        Button,
        ImageNode::new(up.clone()),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x),
            top: Val::Px(y),
            width: Val::Px(24.0),
            height: Val::Px(24.0),
            ..default()
        },
        ItemBtn { item, up, down },
    ));
}

/// Show only the dialog selected on the toolbar (original opened()/closing()).
fn sync_dialogs(state: Res<DialogState>, mut q: Query<(&DialogRoot, &mut Visibility)>) {
    if !state.is_changed() {
        return;
    }
    for (root, mut vis) in &mut q {
        *vis = if state.open == Some(root.0) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn menu_buttons(
    mut q: Query<(&Interaction, &ItemBtn, &mut ImageNode), Changed<Interaction>>,
    mut mode: ResMut<BuildMode>,
    mut dialogs: ResMut<DialogState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        mode.selected = None;
        dialogs.open = None;
    }
    for (it, btn, mut img) in q.iter_mut() {
        match *it {
            Interaction::Pressed => {
                img.image = btn.down.clone();
                mode.selected = Some(btn.item.clone());
            }
            Interaction::Hovered | Interaction::None => img.image = btn.up.clone(),
        }
    }
}

// ---------------- ghost preview + placement ----------------

#[allow(clippy::too_many_arguments)]
fn ghost_and_place(
    mut commands: Commands,
    mode: Res<BuildMode>,
    mut occupancy: ResMut<Occupancy>,
    mut wallet: ResMut<Wallet>,
    assets: Res<AssetServer>,
    handles: Res<DataHandles>,
    gamedata: Res<Assets<GameData>>,
    anchors: Res<Assets<Anchors>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ghosts: Query<Entity, With<Ghost>>,
    painted: Query<(Entity, &PaintedTile)>,
    buttons: Res<ButtonInput<MouseButton>>,
    ui_hover: Query<&Interaction, With<Button>>,
    mut counts: ResMut<HudCounts>,
) {
    // clear previous ghost every frame (simple + robust)
    for e in ghosts.iter() {
        commands.entity(e).despawn();
    }
    let Some(item) = &mode.selected else { return };
    let Some(gd) = gamedata.get(&handles.gamedata) else { return };
    let Some(an) = anchors.get(&handles.anchors) else { return };
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_tf)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world) = cam.viewport_to_world_2d(cam_tf, cursor) else { return };
    let (tx, ty) = iso::world_to_tile(world.x, world.y);

    let over_ui = ui_hover.iter().any(|i| *i != Interaction::None);

    match item {
        BuildItem::Booth(name) => {
            let Some(b) = gd.booth_def(name) else { return };
            let Some(ba) = an.booth.get(name) else { return };
            let free = occupancy.booth_free(tx, ty, b.rows, b.cols);
            let price = b.price.first().copied().unwrap_or(0.0);
            let affordable = wallet.money >= price;
            let ok = free && affordable;
            // ghost sprite: anchor at the screen-topmost footprint corner
            // (lattice point of (tx, ty+cols-1)), same as spawn_booth_at
            let (wx, wy) = iso::tile_to_world(tx, ty + b.cols - 1);
            commands.spawn((
                Sprite {
                    image: assets.load(format!("sprites/booths/{}.png", b.fr)),
                    anchor: Anchor::Custom(Vec2::new(
                        ba.anchor[0] / 218.0 - 0.5,
                        0.5 - ba.anchor[1] / 112.0,
                    )),
                    color: if ok {
                        Color::srgba(0.6, 1.0, 0.6, 0.7)
                    } else {
                        Color::srgba(1.0, 0.4, 0.4, 0.7)
                    },
                    ..default()
                },
                Transform::from_xyz(wx, wy, 500.0),
                Ghost,
            ));
            if ok && !over_ui && buttons.just_pressed(MouseButton::Left) {
                wallet.money -= price;
                occupancy.occupy_booth(tx, ty, b.rows, b.cols);
                spawn_booth_at(&mut commands, &assets, gd, an, name, tx, ty);
                // ROOM(S) counts blok 51/56 (bookable rooms); everything
                // else placed from the booth list is a FACILITY.
                match b.blok {
                    51 | 56 => counts.rooms += 1,
                    _ => counts.facilities += 1,
                }
            }
        }
        BuildItem::Scenery(jenis) => {
            let Some(s) = gd.scenery_def(*jenis) else { return };
            let free = !occupancy.tiles.contains_key(&(tx, ty)) && in_lahan(tx, ty);
            let affordable = wallet.money >= s.price;
            let ok = free && affordable;
            let Some(sa) = an.scenery.get(&format!("PLANT_{jenis}")) else { return };
            let (cx, cy) = iso::tile_center(tx, ty);
            commands.spawn((
                Sprite {
                    image: assets.load(format!("sprites/scenery/{}.png", sa.fr)),
                    anchor: Anchor::Custom(Vec2::new(
                        sa.base[0] / 49.0 - 0.5,
                        0.5 - sa.base[1] / 54.0,
                    )),
                    color: if ok {
                        Color::srgba(0.6, 1.0, 0.6, 0.7)
                    } else {
                        Color::srgba(1.0, 0.4, 0.4, 0.7)
                    },
                    ..default()
                },
                Transform::from_xyz(cx, cy - 6.0, 500.0),
                Ghost,
            ));
            if ok && !over_ui && buttons.just_pressed(MouseButton::Left) {
                wallet.money -= s.price;
                occupancy.tiles.insert((tx, ty), ());
                spawn_scenery_at(&mut commands, &assets, an, tx, ty, *jenis);
                counts.plants += 1;
            }
        }
        BuildItem::Tile(jenis) => {
            // Repaint a ground tile (original Tile.changeJenis): allowed on
            // any lahan tile, replaces the previous paint on that tile.
            let Some(t) = gd.tile_def(*jenis) else { return };
            let free = in_lahan(tx, ty) && !occupancy.tiles.contains_key(&(tx, ty));
            let affordable = wallet.money >= t.price;
            let ok = free && affordable;
            let ta = &an.tile;
            let (wx, wy) = iso::tile_to_world(tx, ty);
            let anchor = Anchor::Custom(Vec2::new(
                ta.anchor[0] / ta.size[0] - 0.5,
                0.5 - ta.anchor[1] / ta.size[1],
            ));
            commands.spawn((
                Sprite {
                    image: assets.load(format!("sprites/tiles/{}.png", t.fr)),
                    anchor,
                    color: if ok {
                        Color::srgba(0.6, 1.0, 0.6, 0.7)
                    } else {
                        Color::srgba(1.0, 0.4, 0.4, 0.7)
                    },
                    ..default()
                },
                Transform::from_xyz(wx, wy, 500.0),
                Ghost,
            ));
            if ok && !over_ui && buttons.just_pressed(MouseButton::Left) {
                wallet.money -= t.price;
                // remove any previous paint on this tile, then draw the new one
                for (e, pt) in painted.iter() {
                    if pt.0 == tx && pt.1 == ty {
                        commands.entity(e).despawn();
                    }
                }
                commands.spawn((
                    Sprite {
                        image: assets.load(format!("sprites/tiles/{}.png", t.fr)),
                        anchor,
                        ..default()
                    },
                    // just above the base tile layer (Z_TILE=0) but below decor
                    Transform::from_xyz(wx, wy, 0.5),
                    PaintedTile(tx, ty),
                ));
            }
        }
    }
}
