//! Title screen (original mainMenuD symbol). Background is the original
//! render cropped to the 640x480 stage; buttons use the original
//! DefineButton2 up/over/down art at pixel-measured stage positions.
//!
//! Original flow (Application.as / mainMenuD_470.as): after preloading the
//! menu stops on "stand" (PLAY/OPTIONS/CREDIT/MORE GAMES, frame 27);
//! clicking PLAY plays "init_load" revealing NEW GAME / LOAD GAME / BACK
//! (frame 38), and BACK returns to "stand". LOAD GAME is disabled while
//! there is no save support, mirroring `mainMenuD.btn_load.mouseEnabled =
//! false` when no save exists. OPTIONS/CREDIT/MORE GAMES are visual
//! no-ops for now (original: quality settings / credits / sponsor link).

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
const BTN_CREDIT_Y: f32 = 313.0;
const BTN_BACK_Y: f32 = 349.0;

/// CHECKING...DATA panel (shape 2863) stage position and size, measured
/// on the reference capture (panel bbox x227-412 y216-327, 1:1 bitmap).
const CHECK_POS: (f32, f32) = (227.0, 216.0);
const CHECK_SIZE: (f32, f32) = (186.0, 112.0);
/// Curtain gray sampled from the reference capture (tutup covers the
/// whole 640x480 stage while CheckLocalData runs).
const CHECK_CURTAIN: Color = Color::srgb(0.4, 0.4, 0.4);
/// How long the overlay stays up. The original tutup "anim" is 13 frames
/// @30fps plus the SharedObject read; the reference shows it ~1.3s.
const CHECK_SECS: f32 = 1.3;

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
    // "stand" pose (mainMenuD frame 27)
    Play,
    Options,
    Credit,
    MoreGames,
    // "init_load" pose (frame 38)
    NewGame,
    LoadGame,
    Back,
    // input-name dialog
    DialogOk,
    DialogCancel,
}

/// Which mainMenuD pose a menu button belongs to (visibility toggled
/// when the timeline would gotoAndPlay between poses).
#[derive(Component, Clone, Copy, PartialEq)]
enum PoseTag {
    Stand,
    InitLoad,
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

/// The CHECKING...DATA startup overlay (original tutup.showing() +
/// CheckLocalData() in Application.initMainMenu). Despawned on timeout.
#[derive(Component)]
struct CheckingOverlay {
    timer: Timer,
    /// The timer only starts once these are actually loaded: assets load
    /// asynchronously and on wasm the panel PNG/font can take longer than
    /// the overlay's lifetime, despawning the curtain before its contents
    /// ever painted (matches the original, which hides the curtain only
    /// after CheckLocalData completes).
    panel: Handle<Image>,
    font: Handle<Font>,
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
                (title_scale, title_buttons, name_typing, checking_tick)
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

    // Both poses share the same panel slots; the original swaps them by
    // moving along the timeline (stand: frame 27, init_load: frame 38).
    let buttons: [(TitleButton, PoseTag, &str, f32); 7] = [
        (TitleButton::Play, PoseTag::Stand, "btn_play", BTN_NEW_Y),
        (TitleButton::Options, PoseTag::Stand, "btn_options", BTN_LOAD_Y),
        (TitleButton::Credit, PoseTag::Stand, "btn_credit", BTN_CREDIT_Y),
        (TitleButton::MoreGames, PoseTag::Stand, "btn_more_games", BTN_BACK_Y),
        (TitleButton::NewGame, PoseTag::InitLoad, "btn_new_game", BTN_NEW_Y),
        (TitleButton::LoadGame, PoseTag::InitLoad, "btn_load_game", BTN_LOAD_Y),
        (TitleButton::Back, PoseTag::InitLoad, "btn_back", BTN_BACK_Y),
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
                for (which, pose, name, y) in buttons {
                    let up = assets.load(format!("sprites/title/{name}_up.png"));
                    let over = assets.load(format!("sprites/title/{name}_over.png"));
                    let down = assets.load(format!("sprites/title/{name}_down.png"));
                    // The menu starts on the "stand" pose like the original
                    // (frame 27 stop() after the intro animation).
                    let vis = if pose == PoseTag::Stand {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                    stage.spawn((
                        which,
                        pose,
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
                        vis,
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
                            // Same GlobalZIndex-descendant fix as the
                            // CHECKING overlay: lift above the dialog art.
                            GlobalZIndex(61),
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
                                GlobalZIndex(61),
                            ));
                        }
                    });

                // CHECKING...DATA overlay: gray curtain over the whole
                // stage + the semi-transparent panel with pixel text,
                // shown briefly before the menu (initMainMenu ->
                // tutup.showing() + CheckLocalData()).
                let panel = assets.load("sprites/title/checking_panel.png");
                let font = assets.load("fonts/starmap.ttf");
                stage
                    .spawn((
                        CheckingOverlay {
                            timer: Timer::from_seconds(CHECK_SECS, TimerMode::Once),
                            panel: panel.clone(),
                            font: font.clone(),
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Px(TITLE_W),
                            height: Val::Px(TITLE_H),
                            ..default()
                        },
                        BackgroundColor(CHECK_CURTAIN),
                        GlobalZIndex(70),
                    ))
                    .with_children(|curtain| {
                        curtain
                            .spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(CHECK_POS.0),
                                    top: Val::Px(CHECK_POS.1),
                                    width: Val::Px(CHECK_SIZE.0),
                                    height: Val::Px(CHECK_SIZE.1),
                                    ..default()
                                },
                                ImageNode::new(panel.clone()),
                                // Children of a non-root GlobalZIndex node
                                // paint in the *stage* context, i.e. below
                                // the lifted parent -- lift them too or the
                                // curtain covers its own panel/text.
                                GlobalZIndex(71),
                            ))
                            .with_children(|p| {
                                // Text 2865: two centered lines in the
                                // panel's upper half (rows 43-67 measured
                                // on the reference).
                                p.spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Px(0.0),
                                        top: Val::Px(40.0),
                                        width: Val::Px(CHECK_SIZE.0),
                                        ..default()
                                    },
                                    Text::new("CHECKING...\nDATA..."),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    TextLayout::new_with_justify(JustifyText::Center),
                                    GlobalZIndex(72),
                                ));
                            });
                    });
            });
        });
}

/// Despawn the CHECKING...DATA overlay once its time is up (original
/// tutup.hiding() after the "anim" frames + data check).
fn checking_tick(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<AssetServer>,
    mut q: Query<(Entity, &mut CheckingOverlay)>,
) {
    for (e, mut c) in &mut q {
        // Hold the curtain until the panel art + font are actually
        // renderable; only then run the original ~1.3s check delay.
        if !assets.is_loaded_with_dependencies(&c.panel)
            || !assets.is_loaded_with_dependencies(&c.font)
        {
            continue;
        }
        if c.timer.tick(time.delta()).finished() {
            commands.entity(e).despawn();
        }
    }
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
    mut dialog: Query<&mut Visibility, (With<NameDialog>, Without<PoseTag>)>,
    mut poses: Query<(&PoseTag, &mut Visibility), (With<PoseTag>, Without<NameDialog>)>,
    mut name_text: Query<(&mut Text, &mut NameText)>,
    mut resort: ResMut<ResortName>,
    time: Res<Time>,
    mut next: ResMut<NextState<AppState>>,
    checking: Query<(), With<CheckingOverlay>>,
) {
    // While the CHECKING...DATA curtain is up the menu is not
    // interactive (original tutup covers the stage).
    let checking_up = !checking.is_empty();
    let dialog_open = dialog
        .single()
        .map(|v| *v == Visibility::Visible)
        .unwrap_or(false);

    // Emulates mainMenuD.gotoAndPlay("init_load") / back to "stand".
    let mut set_pose = |poses: &mut Query<
        (&PoseTag, &mut Visibility),
        (With<PoseTag>, Without<NameDialog>),
    >,
                        show: PoseTag| {
        for (tag, mut vis) in poses.iter_mut() {
            *vis = if *tag == show {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    };

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
        if checking_up || (dialog_open && !is_dialog_btn) {
            img.image = art.up.clone();
            continue;
        }
        match interaction {
            Interaction::Hovered => img.image = art.over.clone(),
            Interaction::Pressed => {
                img.image = art.down.clone();
                match which {
                    TitleButton::Play => {
                        // Original ClickMainMenu btn_play:
                        // mainMenuD.gotoAndPlay("init_load").
                        set_pose(&mut poses, PoseTag::InitLoad);
                    }
                    TitleButton::Back => {
                        // Original btn_back: return to the "stand" pose.
                        set_pose(&mut poses, PoseTag::Stand);
                    }
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
                    // OPTIONS (quality settings), CREDIT (credits scene) and
                    // MORE GAMES (sponsor link) are visual no-ops for now.
                    _ => {}
                }
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
