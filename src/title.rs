//! Title screen (original mainMenuD symbol, "init_load" pose: frame 38).
//! Background is the original render cropped to the 640x480 stage;
//! NEW GAME / LOAD GAME / BACK buttons use the original DefineButton2
//! up/over/down art at pixel-measured stage positions.
//!
//! Original flow (Application.as / mainMenuD_470.as): after preloading the
//! menu shows PLAY/OPTION/CREDIT/MORE ("stand"), then clicking PLAY plays
//! "init_load" revealing NEW GAME / LOAD GAME / BACK. Our port jumps
//! straight to the init_load pose (matching the reference capture, which
//! starts there). LOAD GAME is disabled while there is no save support,
//! mirroring `mainMenuD.btn_load.mouseEnabled = false` when no save exists.

use crate::data::AppState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Original Flash stage size the title art was designed for.
const TITLE_W: f32 = 640.0;
const TITLE_H: f32 = 480.0;

/// Button geometry measured on the original render (stage coordinates).
const BTN_SIZE: (f32, f32) = (106.0, 26.0);
const BTN_X: f32 = 484.0;
const BTN_NEW_Y: f32 = 240.0;
const BTN_LOAD_Y: f32 = 275.0;
const BTN_BACK_Y: f32 = 349.0;

#[derive(Component, Clone, Copy, PartialEq)]
enum TitleButton {
    NewGame,
    LoadGame,
    Back,
}

#[derive(Component)]
struct TitleRoot;

/// The three state images for one button (swapped on Interaction).
#[derive(Component)]
struct ButtonArt {
    up: Handle<Image>,
    over: Handle<Image>,
    down: Handle<Image>,
}

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Title), spawn_title)
            .add_systems(OnExit(AppState::Title), despawn_title)
            .add_systems(
                Update,
                (title_scale, title_buttons).run_if(in_state(AppState::Title)),
            );
    }
}

/// Scale the whole UI so the 640x480 title stage fits the window
/// (letterboxed like the original 4:3 stage).
fn title_scale(windows: Query<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    if let Ok(w) = windows.single() {
        let s = (w.width() / TITLE_W).min(w.height() / TITLE_H).max(0.1);
        if (scale.0 - s).abs() > 0.001 {
            scale.0 = s;
        }
    }
}

fn spawn_title(mut commands: Commands, assets: Res<AssetServer>) {
    let bg = assets.load("sprites/title/menu_bg.png");

    let buttons: [(TitleButton, &str, f32); 3] = [
        (TitleButton::NewGame, "btn_new_game", BTN_NEW_Y),
        (TitleButton::LoadGame, "btn_load_game", BTN_LOAD_Y),
        (TitleButton::Back, "btn_back", BTN_BACK_Y),
    ];

    commands
        .spawn((
            TitleRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            // Solid backdrop behind the letterboxed stage.
            BackgroundColor(Color::BLACK),
            GlobalZIndex(50),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(TITLE_W),
                    height: Val::Px(TITLE_H),
                    ..default()
                },
                ImageNode::new(bg),
            ))
            .with_children(|stage| {
                for (which, name, y) in buttons {
                    let up = assets.load(format!("sprites/title/{name}_up.png"));
                    let over = assets.load(format!("sprites/title/{name}_over.png"));
                    let down = assets.load(format!("sprites/title/{name}_down.png"));
                    stage.spawn((
                        which,
                        Button,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(BTN_X),
                            top: Val::Px(y),
                            width: Val::Px(BTN_SIZE.0),
                            height: Val::Px(BTN_SIZE.1),
                            ..default()
                        },
                        ImageNode::new(up.clone()),
                        ButtonArt { up, over, down },
                    ));
                }
            });
        });
}

fn despawn_title(mut commands: Commands, roots: Query<Entity, With<TitleRoot>>) {
    for e in &roots {
        commands.entity(e).despawn();
    }
}

fn title_buttons(
    mut q: Query<
        (&TitleButton, &Interaction, &ButtonArt, &mut ImageNode),
        (Changed<Interaction>, With<Button>),
    >,
    mut next: ResMut<NextState<AppState>>,
) {
    for (which, interaction, art, mut img) in &mut q {
        // LOAD GAME is disabled (no save data yet), like the original's
        // greyed-out btn_load: keep the up art and ignore clicks.
        if *which == TitleButton::LoadGame {
            img.image = art.up.clone();
            continue;
        }
        match interaction {
            Interaction::Hovered => img.image = art.over.clone(),
            Interaction::Pressed => {
                img.image = art.down.clone();
                if *which == TitleButton::NewGame {
                    next.set(AppState::Playing);
                }
                // BACK: the original returns to the sponsor "stand" menu;
                // our port has no previous screen, so it is a visual no-op.
            }
            Interaction::None => img.image = art.up.clone(),
        }
    }
}
