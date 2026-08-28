//! HUD built from the original SWF art (navigator1/2/3 clips).
//! The top and bottom bars stretch to the full window width like the
//! original (left block anchored left, info boxes anchored right, a 1px
//! plain strip stretched in between). A global UiScale reproduces the
//! original stage scaling (726px logical width).

use crate::build::{BuildMode, DialogKind, DialogState};
use crate::data::AppState;
use crate::game::{GameClock, Wallet};
use crate::visitor::Visitor;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

// ---- panel geometry (pixel-measured on the extracted art) ----
const NAV1_H: f32 = 78.0;
const NAV2_SIZE: (f32, f32) = (47.0, 346.0);
const NAV3_H: f32 = 74.0;
/// Original Flash logical stage width the art was designed for.
const STAGE_W: f32 = 726.0;

/// Every dynamic HUD text carries one of these.
#[derive(Component, Clone, Copy, PartialEq)]
enum HudField {
    Money,
    Day,
    Pop,
    Rp,
    Ticker,
    Gift,
    Star,
    /// 0=janitor 1=visitor 2=room 3=plant 4=facility
    Counter(u8),
}

/// Placed-object counters shown in the top bar (rooms/plants/facilities
/// are incremented by the build system; janitors not implemented yet).
#[derive(Resource, Default)]
pub struct HudCounts {
    pub janitors: u32,
    pub janitors_max: u32,
    pub rooms: u32,
    pub plants: u32,
    pub facilities: u32,
}

/// Animated clock pieces (original animasiJam clip: panjang/pendek hands
/// rotate with hourDay; siang/malam faces swap at 06:00/18:00).
#[derive(Component, Clone, Copy, PartialEq)]
enum ClockPiece {
    FaceDay,
    FaceNight,
    HandMinute,
    HandHour,
}

/// Speed control buttons on nav3 (play toggles pause; 1x/2x/3x set speed).
#[derive(Component, Clone, Copy, PartialEq)]
enum SpeedBtn {
    Play,
    X1,
    X2,
    X3,
}

/// Left toolbar tools (navigator2 buttons, same order as the original clip).
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Tool {
    Tips,
    ArrowMouse,
    DragMouse,
    Room,
    Facility,
    Scenery,
    Tile,
    JanitorPos,
    Destroy,
    Expand,
    Extra,
}

/// Toolbar button: swaps between the _1 (normal) and _2 (hover) frames.
#[derive(Component)]
struct ToolBtn {
    tool: Tool,
    normal: Handle<Image>,
    hover: Handle<Image>,
}

/// NEW badge overlay attached to a tool button (original: setNewUnlocked/matikanNewIcon).
#[derive(Component)]
struct NewBadge(Tool);

/// Lock overlay (original: btn_expand._locked, hidden once UNLOCKED.Expand1).
#[derive(Component)]
struct LockIcon(Tool);

/// Mirrors main.game.UNLOCKED_new + UNLOCKED.Expand1 from the original.
#[derive(Resource)]
pub struct ToolbarState {
    pub new_flags: HashMap<Tool, bool>,
    pub expand_unlocked: bool,
}

impl Default for ToolbarState {
    fn default() -> Self {
        // Verified against the original reference screenshot: on a fresh
        // game only the Tips button shows a NEW badge; the others appear
        // once their content unlocks, and Expand starts locked.
        let mut new_flags = HashMap::new();
        for t in [Tool::Room, Tool::Facility, Tool::Scenery, Tool::Expand, Tool::Extra] {
            new_flags.insert(t, false);
        }
        new_flags.insert(Tool::Tips, true);
        Self {
            new_flags,
            expand_unlocked: false,
        }
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolbarState>()
            .init_resource::<HudCounts>()
            .add_systems(OnEnter(AppState::Playing), spawn_hud)
            .add_systems(
                Update,
                (
                    update_ui_scale,
                    update_hud,
                    animate_clock,
                    speed_buttons,
                    toolbar_buttons,
                    sync_badges,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Reproduce the original stage scaling: the Flash stage is ~726px of
/// logical width scaled up to the window, so UI pixels grow with it.
fn update_ui_scale(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut scale: ResMut<UiScale>,
) {
    if let Ok(w) = windows.single() {
        let s = (w.width() / STAGE_W).max(1.0);
        if (scale.0 - s).abs() > 0.01 {
            scale.0 = s;
        }
    }
}

fn spawn_hud(mut commands: Commands, assets: Res<AssetServer>) {
    // Original embedded game font (Starmap Truetype, exported from the SWF).
    let font = assets.load("fonts/starmap.ttf");
    // ---------- top panel (navigator1): full-width ----------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(NAV1_H),
                ..default()
            },
            ZIndex(10),
        ))
        .with_children(|p| {
            // stretched 1px strip: green top band + white counter panel
            p.spawn((
                ImageNode::new(assets.load("sprites/ui/nav1_mid.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    height: Val::Px(NAV1_H),
                    ..default()
                },
            ));
            // left block: dragon logo, title bar, RP bar, level star
            p.spawn((
                ImageNode::new(assets.load("sprites/ui/nav1_left.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(222.0),
                    height: Val::Px(NAV1_H),
                    ..default()
                },
            ))
            .with_children(|l| {
                // resort title, vertically centered in the gray title bar
                // (measured bbox in nav1_left.png: (50,33)-(192,52))
                l.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(56.0),
                        top: Val::Px(33.0),
                        width: Val::Px(136.0),
                        height: Val::Px(20.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|t| {
                    t.spawn((
                        Text::new("GREEN ISLAND"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
                // RP label over the green strip (52,52)-(133,68)
                l.spawn((
                    Text::new("RP (0%)"),
                    TextFont {
                        font: font.clone(),
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.1, 0.25, 0.05)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(58.0),
                        top: Val::Px(55.0),
                        ..default()
                    },
                    HudField::Rp,
                ));
                // red RP progress fill inside the black bar
                // (measured bbox in nav1_left.png: (134,54)-(192,66))
                l.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(136.0),
                        top: Val::Px(56.0),
                        width: Val::Px(0.0),
                        height: Val::Px(9.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.85, 0.1, 0.1)),
                ));
                // star level number, centered on the star art
                // (measured orange bbox in nav1_left.png: (189,23)-(220,54))
                l.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(189.0),
                        top: Val::Px(23.0),
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|s| {
                    s.spawn((
                        Text::new("0"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.1, 0.08, 0.02)),
                        HudField::Star,
                    ));
                });
            });
            // counters block: stretched between the left block and right box.
            // Labels are baked in the art; the value texts sit in each box.
            p.spawn((
                ImageNode::new(assets.load("sprites/ui/nav1_counters.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(222.0),
                    right: Val::Px(40.0),
                    top: Val::Px(0.0),
                    height: Val::Px(NAV1_H),
                    ..default()
                },
            ))
            .with_children(|c| {
                // blue box interiors in the 378px counters image (percent):
                // (13..81)(86..154)(159..227)(232..300)(305..373)
                const BOX_LEFTS: [f32; 5] = [3.4, 22.8, 42.1, 61.4, 80.7];
                for (i, left_pct) in BOX_LEFTS.iter().enumerate() {
                    c.spawn((Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(*left_pct),
                        width: Val::Percent(18.0),
                        top: Val::Px(46.0),
                        height: Val::Px(14.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(if i == 0 { "0(0)" } else { "0" }),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 9.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                HudField::Counter(i as u8),
                            ));
                        });
                }
            });
            // right box button (item storage) anchored to the right edge
            p.spawn((
                ImageNode::new(assets.load("sprites/ui/nav1_box.png")),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(40.0),
                    height: Val::Px(NAV1_H),
                    ..default()
                },
            ));
        });

    // ---------- left toolbar (navigator2) ----------
    // Clean panel + separate button sprites, so hover frames and the
    // NEW/lock overlays can toggle like the original MovieClips.
    // (tool, sprite name, y, w, h) — x is always 8, measured on the export.
    const TOOLS: [(Tool, &str, f32, f32, f32); 11] = [
        (Tool::Tips, "btn_tips", 8.0, 26.0, 25.0),
        (Tool::ArrowMouse, "btn_arrowMouse", 35.0, 24.0, 24.0),
        (Tool::DragMouse, "btn_dragMouse", 62.0, 26.0, 25.0),
        (Tool::Room, "btn_room", 101.0, 26.0, 25.0),
        (Tool::Facility, "btn_facility", 131.0, 26.0, 25.0),
        (Tool::Scenery, "btn_scenery", 161.0, 26.0, 25.0),
        (Tool::Tile, "btn_tile", 191.0, 26.0, 25.0),
        (Tool::JanitorPos, "btn_janitorPos", 221.0, 26.0, 25.0),
        (Tool::Destroy, "btn_destroy", 251.0, 26.0, 25.0),
        (Tool::Expand, "btn_expand", 280.0, 26.0, 25.0),
        (Tool::Extra, "btn_extra", 311.0, 26.0, 25.0),
    ];
    // NEW badge bands measured on the original baked panel (x=17).
    const BADGES: [(Tool, f32); 6] = [
        (Tool::Tips, 6.0),
        (Tool::Room, 97.0),
        (Tool::Facility, 129.0),
        (Tool::Scenery, 158.0),
        (Tool::Expand, 278.0),
        (Tool::Extra, 311.0),
    ];

    commands
        .spawn((
            ImageNode::new(assets.load("sprites/ui/nav2_panel.png")),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(90.0),
                left: Val::Px(0.0),
                width: Val::Px(NAV2_SIZE.0),
                height: Val::Px(NAV2_SIZE.1),
                ..default()
            },
            ZIndex(10),
        ))
        .with_children(|p| {
            for (tool, name, y, w, h) in TOOLS {
                let normal: Handle<Image> =
                    assets.load(format!("sprites/ui/buttons/{name}_1.png"));
                let hover: Handle<Image> =
                    assets.load(format!("sprites/ui/buttons/{name}_2.png"));
                p.spawn((
                    Button,
                    ImageNode::new(normal.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(8.0),
                        top: Val::Px(y),
                        width: Val::Px(w),
                        height: Val::Px(h),
                        ..default()
                    },
                    ToolBtn {
                        tool,
                        normal,
                        hover,
                    },
                ));
                // lock overlay on the expand button (bottom-right corner,
                // matching where the original SWF placed the `_locked` child)
                if tool == Tool::Expand {
                    p.spawn((
                        ImageNode::new(assets.load("sprites/ui/buttons/badge_lock.png")),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(8.0 + w - 10.0),
                            top: Val::Px(y + h - 12.0),
                            width: Val::Px(10.0),
                            height: Val::Px(12.0),
                            ..default()
                        },
                        ZIndex(2),
                        LockIcon(tool),
                    ));
                }
            }
            // NEW badges (drawn above the buttons, toggled by ToolbarState)
            for (tool, y) in BADGES {
                p.spawn((
                    ImageNode::new(assets.load("sprites/ui/buttons/badge_new.png")),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(17.0),
                        top: Val::Px(y),
                        width: Val::Px(30.0),
                        height: Val::Px(14.0),
                        ..default()
                    },
                    ZIndex(3),
                    NewBadge(tool),
                ));
            }
        });

    // ---------- bottom bar (navigator3): full-width ----------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(NAV3_H),
                ..default()
            },
            ZIndex(10),
        ))
        .with_children(|p| {
            // stretched 1px strip: ticker band + green body
            p.spawn((
                ImageNode::new(assets.load("sprites/ui/nav3_mid.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    height: Val::Px(NAV3_H),
                    ..default()
                },
            ));
            // left block: clock, DAY, menu buttons, speed buttons, sliders
            p.spawn((
                ImageNode::new(assets.load("sprites/ui/nav3_left.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(517.0),
                    height: Val::Px(NAV3_H),
                    ..default()
                },
            ))
            .with_children(|l| {
                // animated clock over the baked one; dial center at (36,31)
                // in nav3_left coords (white dial spans 15..57 x 10..52).
                // layering matches the original clip: faces (siang/malam),
                // then the dial tick marks (shape 3562), then the hands.
                for (piece, path, w, h, z) in [
                    (Some(ClockPiece::FaceDay), "sprites/ui/clock/face_day.png", 47.0, 47.0, 0),
                    (Some(ClockPiece::FaceNight), "sprites/ui/clock/face_night.png", 47.0, 47.0, 0),
                    (None, "sprites/ui/clock/dial_marks.png", 41.0, 41.0, 1),
                    (Some(ClockPiece::HandMinute), "sprites/ui/clock/hand_minute.png", 3.0, 32.0, 2),
                    (Some(ClockPiece::HandHour), "sprites/ui/clock/hand_hour.png", 5.0, 20.0, 2),
                ] {
                    let mut e = l.spawn((
                        ImageNode::new(assets.load(path)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(36.0 - w / 2.0),
                            top: Val::Px(31.0 - h / 2.0),
                            width: Val::Px(w),
                            height: Val::Px(h),
                            ..default()
                        },
                        ZIndex(z),
                    ));
                    if let Some(p) = piece {
                        e.insert(p);
                    }
                }
                // DAY text in the block under the clock (10,56)-(66,70)
                l.spawn((
                    Text::new("DAY 1"),
                    TextFont {
                        font: font.clone(),
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(14.0),
                        top: Val::Px(56.0),
                        ..default()
                    },
                    HudField::Day,
                ));
                // invisible clickable zones over the speed buttons
                for (x, w, which) in [
                    (291.0, 30.0, SpeedBtn::Play),
                    (325.0, 28.0, SpeedBtn::X1),
                    (353.0, 28.0, SpeedBtn::X2),
                    (381.0, 26.0, SpeedBtn::X3),
                ] {
                    l.spawn((
                        Button,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(x),
                            top: Val::Px(37.0),
                            width: Val::Px(w),
                            height: Val::Px(24.0),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        which,
                    ));
                }
            });
            // right block: gift box, POPULARITY, money — anchored right
            p.spawn((
                ImageNode::new(assets.load("sprites/ui/nav3_right.png")),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(159.0),
                    height: Val::Px(NAV3_H),
                    ..default()
                },
            ))
            .with_children(|r| {
                // gift amount next to the baked gift icon (icon x4..17)
                r.spawn((
                    Text::new("$ 500"),
                    TextFont {
                        font: font.clone(),
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(22.0),
                        top: Val::Px(11.0),
                        ..default()
                    },
                    HudField::Gift,
                ));
                // POPULARITY in the blue box (block coords ~7..114)
                r.spawn((
                    Text::new("POPULARITY 0%"),
                    TextFont {
                        font: font.clone(),
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(12.0),
                        top: Val::Px(30.0),
                        ..default()
                    },
                    HudField::Pop,
                ));
                // money value right-aligned in the white box ("$" is baked)
                r.spawn((
                    Text::new("10000"),
                    TextFont {
                        font: font.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.05, 0.05, 0.05)),
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(52.0),
                        top: Val::Px(51.0),
                        ..default()
                    },
                    HudField::Money,
                ));
            });
            // news ticker: centered between the clock block and the gift
            p.spawn((Node {
                position_type: PositionType::Absolute,
                left: Val::Px(90.0),
                right: Val::Px(165.0),
                top: Val::Px(10.0),
                height: Val::Px(14.0),
                justify_content: JustifyContent::Center,
                ..default()
            },))
                .with_children(|t| {
                    t.spawn((
                        Text::new("Build a room on your resort"),
                        TextFont {
                            font: font.clone(),
                            font_size: 9.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        HudField::Ticker,
                    ));
                });
        });
}

/// Original animasiJam: jalanJam(hourDay) sets panjang.rotation = hourDay*360
/// and pendek.rotation = hourDay*(360/12); toSiang/toMalam swap the face.
fn animate_clock(
    clock: Res<GameClock>,
    mut q: Query<(&ClockPiece, &mut Transform, &mut Visibility)>,
) {
    let hour = clock.hour_frac() as f32;
    let day = clock.is_day();
    for (piece, mut tf, mut vis) in &mut q {
        match piece {
            ClockPiece::FaceDay => {
                *vis = if day {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
            ClockPiece::FaceNight => {
                *vis = if day {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                };
            }
            // Flash rotation is clockwise-positive; Bevy Z+ is CCW, so negate.
            ClockPiece::HandMinute => {
                tf.rotation = Quat::from_rotation_z(-(hour * 360.0f32).to_radians());
            }
            ClockPiece::HandHour => {
                tf.rotation = Quat::from_rotation_z(-(hour * 30.0f32).to_radians());
            }
        }
    }
}

fn update_hud(
    clock: Res<GameClock>,
    wallet: Res<Wallet>,
    counts: Res<HudCounts>,
    visitors: Query<(), With<Visitor>>,
    mut texts: Query<(&HudField, &mut Text)>,
) {
    let n_visitors = visitors.iter().count() as u32;
    for (field, mut t) in &mut texts {
        let new = match field {
            HudField::Money => format!("{}", wallet.money.max(0.0) as u64),
            HudField::Day => format!("DAY {}", clock.day() + 1),
            HudField::Pop => format!("POPULARITY {}%", wallet.popularity.round() as i64),
            HudField::Counter(0) => format!("{}({})", counts.janitors, counts.janitors_max),
            HudField::Counter(1) => format!("{n_visitors}"),
            HudField::Counter(2) => format!("{}", counts.rooms),
            HudField::Counter(3) => format!("{}", counts.plants),
            HudField::Counter(4) => format!("{}", counts.facilities),
            // static for now (RP/level/gift/ticker systems come later)
            _ => continue,
        };
        if t.0 != new {
            t.0 = new;
        }
    }
}

/// Hover = swap to the _2 frame; click = clear the NEW badge (matikanNewIcon)
/// and run the tool action.
fn toolbar_buttons(
    mut q: Query<(&Interaction, &ToolBtn, &mut ImageNode), Changed<Interaction>>,
    mut state: ResMut<ToolbarState>,
    mut build_mode: ResMut<BuildMode>,
    mut dialogs: ResMut<DialogState>,
) {
    for (interaction, btn, mut img) in &mut q {
        match interaction {
            Interaction::Hovered => img.image = btn.hover.clone(),
            Interaction::None => img.image = btn.normal.clone(),
            Interaction::Pressed => {
                img.image = btn.hover.clone();
                // original matikanNewIcon: using a tool clears its NEW flag
                if let Some(f) = state.new_flags.get_mut(&btn.tool) {
                    *f = false;
                }
                // toggle behaviour: pressing the same tool again closes its
                // dialog (original navigator2 semuaDialogOff/opened flow)
                let toggle = |dialogs: &mut DialogState, kind: DialogKind| {
                    dialogs.open = if dialogs.open == Some(kind) {
                        None
                    } else {
                        Some(kind)
                    };
                };
                match btn.tool {
                    // arrow = default cursor: cancel any pending build
                    Tool::ArrowMouse | Tool::DragMouse => {
                        build_mode.selected = None;
                        dialogs.open = None;
                    }
                    Tool::Room => toggle(&mut dialogs, DialogKind::Room),
                    Tool::Facility => toggle(&mut dialogs, DialogKind::Facility),
                    Tool::Scenery => toggle(&mut dialogs, DialogKind::Scenery),
                    Tool::Tile => toggle(&mut dialogs, DialogKind::Tile),
                    // expand stays locked until Expand1 is purchased
                    Tool::Expand if !state.expand_unlocked => {}
                    _ => {}
                }
            }
        }
    }
}

/// Show/hide NEW badges and the expand lock from ToolbarState.
fn sync_badges(
    state: Res<ToolbarState>,
    mut badges: Query<(&NewBadge, &mut Visibility), Without<LockIcon>>,
    mut locks: Query<(&LockIcon, &mut Visibility), Without<NewBadge>>,
) {
    if !state.is_changed() {
        return;
    }
    for (badge, mut vis) in &mut badges {
        *vis = if state.new_flags.get(&badge.0).copied().unwrap_or(false) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (lock, mut vis) in &mut locks {
        let locked = matches!(lock.0, Tool::Expand) && !state.expand_unlocked;
        *vis = if locked {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn speed_buttons(
    mut q: Query<(&Interaction, &SpeedBtn, &mut BackgroundColor), Changed<Interaction>>,
    mut clock: ResMut<GameClock>,
) {
    for (interaction, btn, mut bg) in &mut q {
        match interaction {
            Interaction::Pressed => {
                match btn {
                    SpeedBtn::Play => clock.paused = !clock.paused,
                    SpeedBtn::X1 => {
                        clock.paused = false;
                        clock.speed = 1.0;
                    }
                    SpeedBtn::X2 => {
                        clock.paused = false;
                        clock.speed = 3.0;
                    }
                    SpeedBtn::X3 => {
                        clock.paused = false;
                        clock.speed = 5.0;
                    }
                }
                bg.0 = Color::NONE;
            }
            Interaction::Hovered => bg.0 = Color::srgba(1.0, 1.0, 1.0, 0.15),
            Interaction::None => bg.0 = Color::NONE,
        }
    }
}
