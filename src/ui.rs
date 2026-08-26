//! HUD: money, day/time, speed indicator.

use crate::data::AppState;
use crate::game::{GameClock, Wallet};
use bevy::prelude::*;

#[derive(Component)]
struct HudMoney;

#[derive(Component)]
struct HudClock;

#[derive(Component)]
struct HudSpeed;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_hud)
            .add_systems(Update, update_hud.run_if(in_state(AppState::Playing)));
    }
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)))
        .with_children(|p| {
            p.spawn((
                Text::new("$ 10000"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.3)),
                HudMoney,
            ));
            p.spawn((
                Text::new("Día 0  08:00"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                HudClock,
            ));
            p.spawn((
                Text::new("x1  [1/2/3 velocidad, espacio pausa]"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.85, 0.95)),
                HudSpeed,
            ));
        });
}

fn update_hud(
    clock: Res<GameClock>,
    wallet: Res<Wallet>,
    mut q_money: Query<&mut Text, (With<HudMoney>, Without<HudClock>, Without<HudSpeed>)>,
    mut q_clock: Query<&mut Text, (With<HudClock>, Without<HudMoney>, Without<HudSpeed>)>,
    mut q_speed: Query<&mut Text, (With<HudSpeed>, Without<HudMoney>, Without<HudClock>)>,
) {
    if let Ok(mut t) = q_money.single_mut() {
        t.0 = format!("$ {:.0}   Pop {:.0}", wallet.money, wallet.popularity);
    }
    if let Ok(mut t) = q_clock.single_mut() {
        t.0 = format!(
            "Día {}  {:02}:{:02} {}",
            clock.day(),
            clock.hour_of_day(),
            clock.minute_of_hour(),
            if clock.is_day() { "☀" } else { "🌙" }
        );
    }
    if let Ok(mut t) = q_speed.single_mut() {
        t.0 = if clock.paused {
            "PAUSA  [espacio]".to_string()
        } else {
            format!("x{:.0}  [1/2/3 velocidad, espacio pausa]", clock.speed)
        };
    }
}
