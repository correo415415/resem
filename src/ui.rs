//! HUD built from the original SWF art (navigator1/2/3 clips).
//! nav1_top: top panel (logo, title bar, RP bar, level star, counter buttons)
//! nav2_tools: left vertical toolbar
//! nav3_bottom: bottom bar (clock, DAY, speed buttons, sliders, popularity, money)

use crate::data::AppState;
use crate::game::{GameClock, Wallet};
use bevy::prelude::*;

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

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_hud).add_systems(
            Update,
            (update_hud, speed_buttons).run_if(in_state(AppState::Playing)),
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
    commands.spawn((
        ImageNode::new(assets.load("sprites/ui/nav2_tools.png")),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(90.0),
            left: Val::Px(0.0),
            width: Val::Px(NAV2_SIZE.0),
            height: Val::Px(NAV2_SIZE.1),
            ..default()
        },
        ZIndex(10),
    ));

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
