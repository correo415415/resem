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
}

#[derive(Resource, Default)]
pub struct BuildMode {
    pub selected: Option<BuildItem>,
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

#[derive(Component)]
struct BuildButton(BuildItem);

pub struct BuildPlugin;

impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildMode>()
            .init_resource::<Occupancy>()
            .add_systems(OnEnter(AppState::Playing), spawn_build_menu)
            .add_systems(
                Update,
                (menu_buttons, ghost_and_place).run_if(in_state(AppState::Playing)),
            );
    }
}

// ---------------- build menu (bottom bar) ----------------

fn spawn_build_menu(mut commands: Commands, gamedata: Res<Assets<GameData>>, handles: Res<DataHandles>) {
    let gd = gamedata.get(&handles.gamedata).unwrap();
    // initial unlocks from Application.DefaultGameVars
    let booths = ["Cottage", "Sauna", "Icecream"];
    let plants: [u32; 3] = [1, 2, 3];

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            // above the original nav3 bottom bar (74 px tall)
            bottom: Val::Px(82.0),
            left: Val::Px(55.0),
            column_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(6.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)))
        .with_children(|p| {
            for name in booths {
                let price = gd
                    .booth_def(name)
                    .and_then(|b| b.price.first().copied())
                    .unwrap_or(0.0);
                button(p, format!("{name}\n${price:.0}"), BuildItem::Booth(name.into()));
            }
            for j in plants {
                let (nm, price) = gd
                    .scenery_def(j)
                    .map(|s| (s.nama.unwrap_or_default(), s.price))
                    .unwrap_or_default();
                button(p, format!("{nm}\n${price:.0}"), BuildItem::Scenery(j));
            }
            button(p, "Cancelar\n[Esc]".into(), BuildItem::Scenery(0));
        });
}

fn button(p: &mut ChildSpawnerCommands, label: String, item: BuildItem) {
    p.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.3, 0.4)),
        BuildButton(item),
    ))
    .with_children(|b| {
        b.spawn((
            Text::new(label),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

fn menu_buttons(
    mut q: Query<(&Interaction, &BuildButton, &mut BackgroundColor), Changed<Interaction>>,
    mut mode: ResMut<BuildMode>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        mode.selected = None;
    }
    for (it, btn, mut bg) in q.iter_mut() {
        match *it {
            Interaction::Pressed => {
                mode.selected = match &btn.0 {
                    BuildItem::Scenery(0) => None, // cancel button
                    other => Some(other.clone()),
                };
                *bg = BackgroundColor(Color::srgb(0.3, 0.55, 0.3));
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.2, 0.4, 0.55)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.15, 0.3, 0.4)),
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
    }
}
