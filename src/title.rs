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
use bevy::input::keyboard::{Key, KeyboardInput};
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

/// Input-name dialog art size and baked button rects (measured on the
/// original Box_inputName render).
const DLG_SIZE: (f32, f32) = (212.0, 120.0);
const DLG_CANCEL: (f32, f32, f32, f32) = (37.0, 84.0, 52.0, 19.0);
const DLG_OK: (f32, f32, f32, f32) = (122.0, 84.0, 52.0, 19.0);

/// Random default resort names (serbi.resortName in the original).
const RESORT_NAMES: [&str; 11] = [
    "SUMMER ISLAND",
    "BULABULA RESORT",
    "FANTASY LAND",
    "HAPPY PARADISE",
    "MY WONDERLAND",
    "HOME SWEET LAND",
    "GREEN ISLAND",
    "RAINBOW ISLE",
    "SILVER PARADISE",
    "DIAMOND RESORT",
    "LITTLEGIANT WORLD",
];

/// The player's resort name, set by the input dialog (original playerName).
#[derive(Resource, Clone)]
pub struct ResortName(pub String);

impl Default for ResortName {
    fn default() -> Self {
        Self("HOME SWEET LAND".into())
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
enum TitleButton {
    NewGame,
    LoadGame,
    Back,
    DialogOk,
    DialogCancel,
}

#[derive(Component)]
struct TitleRoot;

/// Root node of the INPUT YOUR NAME dialog (visibility toggled).
#[derive(Component)]
struct NameDialog;

/// The editable name text inside the dialog. `select_all` mirrors the
/// original `input_t.setSelection(0, 20)`: the first keystroke replaces
/// the whole pre-filled name.
#[derive(Component)]
struct NameText {
    select_all: bool,
}

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
        app.init_resource::<ResortName>()
            .add_systems(OnEnter(AppState::Title), spawn_title)
            .add_systems(OnExit(AppState::Title), despawn_title)
            .add_systems(
                Update,
                (title_scale, title_buttons, name_typing)
                    .run_if(in_state(AppState::Title)),
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

                // INPUT YOUR NAME dialog (hidden until NEW GAME is clicked;
                // original Box_inputName.showing(playerName)).
                let dlg_bg = assets.load("sprites/title/input_name_bg.png");
                let font = assets.load("fonts/starmap.ttf");
                stage
                    .spawn((
                        NameDialog,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px((TITLE_W - DLG_SIZE.0) / 2.0),
                            top: Val::Px((TITLE_H - DLG_SIZE.1) / 2.0),
                            width: Val::Px(DLG_SIZE.0),
                            height: Val::Px(DLG_SIZE.1),
                            ..default()
                        },
                        ImageNode::new(dlg_bg),
                        Visibility::Hidden,
                        GlobalZIndex(60),
                    ))
                    .with_children(|dlg| {
                        // Editable name over the cleared text field.
                        dlg.spawn((
                            NameText { select_all: true },
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(15.0),
                                top: Val::Px(38.0),
                                width: Val::Px(182.0),
                                height: Val::Px(28.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            Text::new(""),
                            TextFont {
                                font,
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::BLACK),
                            TextLayout::new_with_justify(JustifyText::Center),
                        ));
                        // OK / CANCEL buttons over the baked art.
                        let dlg_buttons: [(TitleButton, &str, (f32, f32, f32, f32)); 2] = [
                            (TitleButton::DialogOk, "btn_ok", DLG_OK),
                            (TitleButton::DialogCancel, "btn_cancel", DLG_CANCEL),
                        ];
                        for (which, name, (x, y, w, h)) in dlg_buttons {
                            let up = assets.load(format!("sprites/title/{name}_up.png"));
                            let over = assets.load(format!("sprites/title/{name}_over.png"));
                            let down = assets.load(format!("sprites/title/{name}_down.png"));
                            dlg.spawn((
                                which,
                                Button,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(x),
                                    top: Val::Px(y),
                                    width: Val::Px(w),
                                    height: Val::Px(h),
                                    ..default()
                                },
                                ImageNode::new(up.clone()),
                                ButtonArt { up, over, down },
                            ));
                        }
                    });
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
    mut dialog: Query<&mut Visibility, With<NameDialog>>,
    mut name_text: Query<(&mut Text, &mut NameText)>,
    mut resort: ResMut<ResortName>,
    time: Res<Time>,
    mut next: ResMut<NextState<AppState>>,
) {
    let dialog_open = dialog
        .single()
        .map(|v| *v == Visibility::Visible)
        .unwrap_or(false);

    for (which, interaction, art, mut img) in &mut q {
        // LOAD GAME is disabled (no save data yet), like the original's
        // greyed-out btn_load: keep the up art and ignore clicks.
        if *which == TitleButton::LoadGame {
            img.image = art.up.clone();
            continue;
        }
        // The dialog is modal: while it is open the menu buttons behind
        // it are inert (original showing() covers the menu).
        let is_dialog_btn =
            matches!(which, TitleButton::DialogOk | TitleButton::DialogCancel);
        if dialog_open && !is_dialog_btn {
            img.image = art.up.clone();
            continue;
        }
        match interaction {
            Interaction::Hovered => img.image = art.over.clone(),
            Interaction::Pressed => {
                img.image = art.down.clone();
                match which {
                    TitleButton::NewGame => {
                        // Original randomPlayerName(): random pick from
                        // serbi.resortName, then BoxInputName.showing(name).
                        let idx = (time.elapsed_secs_f64() * 997.0) as usize
                            % RESORT_NAMES.len();
                        if let Ok((mut text, mut nt)) = name_text.single_mut() {
                            text.0 = RESORT_NAMES[idx].to_string();
                            nt.select_all = true;
                        }
                        if let Ok(mut vis) = dialog.single_mut() {
                            *vis = Visibility::Visible;
                        }
                    }
                    TitleButton::DialogOk => {
                        // Original btn_ok: playerName = input_t.text, GoToGame0().
                        if let Ok((text, _)) = name_text.single() {
                            if !text.0.trim().is_empty() {
                                resort.0 = text.0.trim().to_string();
                            }
                        }
                        next.set(AppState::Playing);
                    }
                    TitleButton::DialogCancel => {
                        // Original btn_cancel: hide dialog, menu stays put.
                        if let Ok(mut vis) = dialog.single_mut() {
                            *vis = Visibility::Hidden;
                        }
                    }
                    _ => {}
                }
                // BACK: the original returns to the sponsor "stand" menu;
                // our port has no previous screen, so it is a visual no-op.
            }
            Interaction::None => img.image = art.up.clone(),
        }
    }
}

/// Keyboard entry for the name dialog. Mirrors the original TextField:
/// `input_t.restrict = "A-Z a-z"` (letters and spaces only), max 20 chars,
/// initial select-all so the first keystroke replaces the suggestion.
/// Enter confirms like OK.
fn name_typing(
    mut evr: EventReader<KeyboardInput>,
    mut dialog: Query<&mut Visibility, With<NameDialog>>,
    mut name_text: Query<(&mut Text, &mut NameText)>,
    mut resort: ResMut<ResortName>,
    mut next: ResMut<NextState<AppState>>,
) {
    let open = dialog
        .single()
        .map(|v| *v == Visibility::Visible)
        .unwrap_or(false);
    if !open {
        evr.clear();
        return;
    }
    let Ok((mut text, mut nt)) = name_text.single_mut() else {
        return;
    };
    for ev in evr.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Character(s) => {
                let filtered: String = s
                    .chars()
                    .filter(|c| c.is_ascii_alphabetic())
                    .map(|c| c.to_ascii_uppercase())
                    .collect();
                if filtered.is_empty() {
                    continue;
                }
                if nt.select_all {
                    text.0.clear();
                    nt.select_all = false;
                }
                for c in filtered.chars() {
                    if text.0.len() < 20 {
                        text.0.push(c);
                    }
                }
            }
            Key::Space => {
                if nt.select_all {
                    text.0.clear();
                    nt.select_all = false;
                }
                if text.0.len() < 20 && !text.0.is_empty() {
                    text.0.push(' ');
                }
            }
            Key::Backspace => {
                if nt.select_all {
                    text.0.clear();
                    nt.select_all = false;
                } else {
                    text.0.pop();
                }
            }
            Key::Enter => {
                if !text.0.trim().is_empty() {
                    resort.0 = text.0.trim().to_string();
                }
                if let Ok(mut vis) = dialog.single_mut() {
                    *vis = Visibility::Hidden;
                }
                next.set(AppState::Playing);
            }
            _ => {}
        }
    }
}
