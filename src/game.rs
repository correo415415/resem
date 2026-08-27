//! Core game state: clock + money. Port of the timing logic in Application.as.
//!
//! Original: every frame `counter += speed_effect` (1/3/5). With FR=30 fps,
//! `minute = counter / 9`, `hour = minute / 60`, `hourDay = hour % 24`.
//! Start: day 0, 08:00, $10,000, popularity 0.

use bevy::prelude::*;

pub const TICKS_PER_MINUTE: f64 = 9.0;
pub const FRAME_HZ: f64 = 30.0;
pub const START_HOUR: u64 = 8;

#[derive(Resource)]
pub struct GameClock {
    /// accumulated tick counter (original `counter`)
    pub counter: f64,
    /// game speed multiplier (1, 3, 5)
    pub speed: f64,
    pub paused: bool,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            counter: (START_HOUR * 60) as f64 * TICKS_PER_MINUTE,
            speed: 1.0,
            paused: false,
        }
    }
}

impl GameClock {
    pub fn total_minutes(&self) -> u64 {
        (self.counter / TICKS_PER_MINUTE) as u64
    }
    pub fn total_hours(&self) -> u64 {
        self.total_minutes() / 60
    }
    pub fn day(&self) -> u64 {
        self.total_hours() / 24
    }
    pub fn hour_of_day(&self) -> u64 {
        self.total_hours() % 24
    }
    pub fn minute_of_hour(&self) -> u64 {
        self.total_minutes() % 60
    }
    /// Fractional hour of day (original `hourDay`), drives the clock hands:
    /// panjang.rotation = hourDay*360, pendek.rotation = hourDay*(360/12).
    pub fn hour_frac(&self) -> f64 {
        (self.counter / TICKS_PER_MINUTE / 60.0) % 24.0
    }
    /// true between 06:00 and 18:00 (original day/night split)
    pub fn is_day(&self) -> bool {
        let h = self.hour_of_day();
        (6..18).contains(&h)
    }
}

#[derive(Resource)]
pub struct Wallet {
    pub money: f64,
    pub popularity: f64,
    pub xp: f64,
}

impl Default for Wallet {
    fn default() -> Self {
        Self {
            money: 10_000.0,
            popularity: 0.0,
            xp: 0.0,
        }
    }
}

/// Fired once per game-minute boundary (carriers of periodic logic).
#[derive(Event)]
pub struct MinuteTick {
    pub total_minutes: u64,
}

/// Fired once per game-hour boundary.
#[derive(Event)]
pub struct HourTick {
    pub hour_of_day: u64,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameClock>()
            .init_resource::<Wallet>()
            .add_event::<MinuteTick>()
            .add_event::<HourTick>()
            .add_systems(Update, (tick_clock, speed_keys));
    }
}

fn tick_clock(
    time: Res<Time>,
    mut clock: ResMut<GameClock>,
    mut minute_ev: EventWriter<MinuteTick>,
    mut hour_ev: EventWriter<HourTick>,
) {
    if clock.paused {
        return;
    }
    let before_min = clock.total_minutes();
    let before_hr = clock.total_hours();
    // frame-rate independent: original added `speed` per frame at 30fps
    clock.counter += clock.speed * FRAME_HZ * time.delta_secs_f64();
    let after_min = clock.total_minutes();
    if after_min > before_min {
        minute_ev.write(MinuteTick {
            total_minutes: after_min,
        });
    }
    if clock.total_hours() > before_hr {
        hour_ev.write(HourTick {
            hour_of_day: clock.hour_of_day(),
        });
    }
}

fn speed_keys(keys: Res<ButtonInput<KeyCode>>, mut clock: ResMut<GameClock>) {
    if keys.just_pressed(KeyCode::Digit1) {
        clock.speed = 1.0;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        clock.speed = 3.0;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        clock.speed = 5.0;
    }
    if keys.just_pressed(KeyCode::Space) {
        clock.paused = !clock.paused;
    }
}
