//! HUD built from the original SWF art (navigator1/2/3 clips).
//! nav1_top: top panel (logo, title bar, RP bar, level star, counter buttons)
//! nav2_tools: left vertical toolbar
//! nav3_bottom: bottom bar (clock, DAY, speed buttons, sliders, popularity, money)

use crate::build::BuildMode;
use crate::data::AppState;
use crate::game::{GameClock, Wallet};
use bevy::prelude::*;
use std::collections::HashMap;

// ---- panel geometry (pixel-measured on the extracted art) ----
const NAV1_SIZE: (f32, f32) = (640.0, 78.0);
const NAV2_SIZE: (f32, f32) = (47.0, 346.0);
const NAV3_SIZE: (f32, f32) = (676.0, 74.0);

#[derive(Component)]
struct HudMoney;
#[derive(Component)]
struct HudPop;
#[derive(Component)]
struct HudDay;
#[derive(Component)]
struct HudRp;
#[derive(Component)]
struct HudTicker;

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
            .add_systems(OnEnter(AppState::Playing), spawn_hud)
            .add_systems(
                Update,
                (update_hud, speed_buttons, toolbar_buttons, sync_badges)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn spawn_hud(mut commands: Commands, assets: Res<AssetServer>) {
    // ---------- top panel (navigator1) ----------
    commands
        .spawn((
            ImageNode::new(assets.load("sprites/ui/nav1_top.png")),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Px(NAV1_SIZE.0),
                height: Val::Px(NAV1_SIZE.1),
                ..default()
            },
            ZIndex(10),
        ))
        .with_children(|p| {
            // resort title over the gray title bar (52,35)-(188,50)
            p.spawn((
                Text::new("RESORT EMPIRE"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(56.0),
                    top: Val::Px(36.0),
                    ..default()
                },
            ));
            // RP label over the green strip (52,52)-(133,68)
            p.spawn((
                Text::new("RP 0/1400"),
                TextFont {
                    font_size: 9.0,
                    ..default()
                },
                TextColor(Color::srgb(0.1, 0.25, 0.05)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(56.0),
                    top: Val::Px(54.0),
                    ..default()
                },
                HudRp,
            ));
            // red RP progress fill inside the black bar (134,54)-(192,64)
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(135.0),
                    top: Val::Px(55.0),
                    width: Val::Px(0.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.85, 0.1, 0.1)),
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
                // lock overlay on the expand button (centered)
                if tool == Tool::Expand {
                    p.spawn((
                        ImageNode::new(assets.load("sprites/ui/buttons/badge_lock.png")),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(8.0 + (w - 10.0) / 2.0),
                            top: Val::Px(y + (h - 12.0) / 2.0),
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

    // ---------- bottom bar (navigator3) ----------
    commands
        .spawn((
            ImageNode::new(assets.load("sprites/ui/nav3_bottom.png")),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Px(NAV3_SIZE.0),
                height: Val::Px(NAV3_SIZE.1),
                ..default()
            },
            ZIndex(10),
        ))
        .with_children(|p| {
            // DAY text in the block under the clock (10,56)-(66,70)
            p.spawn((
                Text::new("DAY 1"),
                TextFont {
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
                HudDay,
            ));
            // news ticker text over the light band (93..520, rows 12..22)
            p.spawn((
                Text::new("Welcome to Resort Empire!"),
                TextFont {
                    font_size: 9.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(96.0),
                    top: Val::Px(12.0),
                    ..default()
                },
                HudTicker,
            ));
            // POPULARITY value in the blue box (524,28)-(631,43)
            p.spawn((
                Text::new("POPULARITY 000.0"),
                TextFont {
                    font_size: 9.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(527.0),
                    top: Val::Px(30.0),
                    ..default()
                },
                HudPop,
            ));
            // money value in the white box (532,49)-(628,66); "$" prefix is baked
            p.spawn((
                Text::new("10.000"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.05, 0.05, 0.05)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(546.0),
                    top: Val::Px(51.0),
                    ..default()
                },
                HudMoney,
            ));
            // invisible clickable zones over the speed buttons
            // play (293..319), 1x (327..353), 2x (355..381), 3x (381..407), rows 37..61
            for (x, w, which) in [
                (291.0, 30.0, SpeedBtn::Play),
                (325.0, 28.0, SpeedBtn::X1),
                (353.0, 28.0, SpeedBtn::X2),
                (381.0, 26.0, SpeedBtn::X3),
            ] {
                p.spawn((
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
}

/// Format money like the original: thousands separated by dots.
fn fmt_money(v: f64) -> String {
    let n = v.max(0.0) as u64;
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push('.');
        }
        out.push(c);
    }
    out
}

fn update_hud(
    clock: Res<GameClock>,
    wallet: Res<Wallet>,
    mut q_money: Query<&mut Text, (With<HudMoney>, Without<HudDay>, Without<HudPop>)>,
    mut q_day: Query<&mut Text, (With<HudDay>, Without<HudMoney>, Without<HudPop>)>,
    mut q_pop: Query<&mut Text, (With<HudPop>, Without<HudMoney>, Without<HudDay>)>,
) {
    if let Ok(mut t) = q_money.single_mut() {
        t.0 = fmt_money(wallet.money);
    }
    if let Ok(mut t) = q_day.single_mut() {
        t.0 = format!("DAY {}", clock.day() + 1);
    }
    if let Ok(mut t) = q_pop.single_mut() {
        t.0 = format!("POPULARITY {:05.1}", wallet.popularity);
    }
}

/// Hover = swap to the _2 frame; click = clear the NEW badge (matikanNewIcon)
/// and run the tool action.
fn toolbar_buttons(
    mut q: Query<(&Interaction, &ToolBtn, &mut ImageNode), Changed<Interaction>>,
    mut state: ResMut<ToolbarState>,
    mut build_mode: ResMut<BuildMode>,
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
                match btn.tool {
                    // arrow = default cursor: cancel any pending build
                    Tool::ArrowMouse | Tool::DragMouse => build_mode.selected = None,
                    // expand stays locked until Expand1 is purchased
                    Tool::Expand if !state.expand_unlocked => {}
                    // room/facility/scenery/etc. panels: to be wired to the
                    // original build windows in a follow-up
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
