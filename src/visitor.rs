//! Visitors: probabilistic spawning (port of Application.visitorFunction) and
//! simple walking along the road/sidewalk toward the lobby.
//!
//! Original spawn logic (Application.as):
//!   PROB_RANGE = [50,40,30,25,15,5] indexed by hour band:
//!     10-15h -> 0, 16-19h -> 1, 7-9h -> 2, 20-23h -> 3, 0-1h & 5-6h -> 4, 2-4h -> 5
//!   rand(0..100) <= prob + popularity*0.5 -> spawn (1-5am extra gate: 25% pass)
//!   After a spawn, cooldown (jeda) = random minutes range shrinking with popularity.

use crate::data::{AppState, DataHandles, GameData};
use crate::game::{GameClock, MinuteTick, Wallet};
use crate::iso;
use bevy::prelude::*;
use bevy::sprite::Anchor;

const PROB_RANGE: [f64; 6] = [50.0, 40.0, 30.0, 25.0, 15.0, 5.0];
const NUM_SKINS: u32 = 29;

/// Entry path: spawn at the map edge on the asphalt road (row 4/5),
/// walk along the road to the sidewalk column, then to the lobby front.
/// Simplified from serbi.enterArray.
const ENTRY: (i32, i32) = (4, 0);
const WAYPOINTS: &[(i32, i32)] = &[(4, 20), (10, 20), (13, 21)];

#[derive(Component)]
pub struct Visitor {
    pub skin: u32,
    pub speed: f32,
    /// waypoint list in tile coords; walks toward each in turn
    pub route: Vec<(i32, i32)>,
    pub next: usize,
    /// minutes remaining at destination before leaving
    pub stay: f64,
    pub leaving: bool,
}

#[derive(Resource, Default)]
pub struct SpawnState {
    /// cooldown in game-minutes until next spawn roll
    pub jeda: f64,
    pub rng: u64,
}

impl SpawnState {
    fn rand(&mut self) -> f64 {
        // xorshift64*
        let mut x = self.rng.wrapping_add(0x9E3779B97F4A7C15);
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        (x.wrapping_mul(0x2545F4914F6CDD1D) >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn prob_index(hour: u64) -> usize {
    match hour {
        10..=15 => 0,
        16..=19 => 1,
        7..=9 => 2,
        20..=23 => 3,
        0 | 1 | 5 | 6 => 4,
        _ => 5, // 2-4
    }
}

pub struct VisitorPlugin;

impl Plugin for VisitorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnState {
            jeda: 1.0,
            rng: 0xC0FFEE,
        })
        .add_systems(
            Update,
            (spawn_roll, walk).run_if(in_state(AppState::Playing)),
        );
    }
}

fn spawn_roll(
    mut minute_ev: EventReader<MinuteTick>,
    mut st: ResMut<SpawnState>,
    clock: Res<GameClock>,
    wallet: Res<Wallet>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    handles: Res<DataHandles>,
    gamedata: Res<Assets<GameData>>,
) {
    let ticks = minute_ev.read().count() as f64;
    if ticks == 0.0 {
        return;
    }
    st.jeda -= ticks;
    if st.jeda > 0.0 {
        return;
    }
    let hour = clock.hour_of_day();
    let prob = PROB_RANGE[prob_index(hour)] + wallet.popularity * 0.5;
    let roll = st.rand() * 100.0;
    let night_gate = if (1..=5).contains(&hour) {
        st.rand() < 0.25
    } else {
        true
    };
    if roll <= prob && night_gate {
        let skin = (st.rand() * NUM_SKINS as f64) as u32 % NUM_SKINS + 1;
        // per-skin speed from gamedata (Visitor{n}.speed), fallback 1.15
        let speed = gamedata
            .get(&handles.gamedata)
            .and_then(|gd| gd.visitor_speed(skin))
            .unwrap_or(1.15);
        let (wx, wy) = iso::tile_center(ENTRY.0, ENTRY.1);
        let mut route: Vec<(i32, i32)> = WAYPOINTS.to_vec();
        // small per-visitor offset target near lobby
        let jitter = ((st.rand() * 3.0) as i32) - 1;
        if let Some(last) = route.last_mut() {
            last.1 = (last.1 + jitter).clamp(20, 23);
        }
        commands.spawn((
            Sprite {
                image: assets.load(format!("sprites/visitor{skin}/1.png")),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            Transform::from_xyz(wx, wy, 400.0),
            Visitor {
                skin,
                speed,
                route,
                next: 0,
                stay: 60.0 + st.rand() * 240.0,
                leaving: false,
            },
        ));
        // reload cooldown: shrinks as popularity grows (approx of reloadMaxJedaVisitor)
        let base = if wallet.popularity < 20.0 {
            (4.0, 9.0)
        } else if wallet.popularity < 50.0 {
            (3.0, 7.0)
        } else if wallet.popularity < 100.0 {
            (2.0, 5.0)
        } else {
            (1.0, 3.0)
        };
        st.jeda = base.0 + st.rand() * (base.1 - base.0);
    } else {
        st.jeda = 1.0;
    }
}

fn walk(
    time: Res<Time>,
    clock: Res<GameClock>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Visitor, &mut Transform, &mut Sprite)>,
    mut minute_ev: EventReader<MinuteTick>,
) {
    if clock.paused {
        return;
    }
    let minutes = minute_ev.read().count() as f64;
    let dt = time.delta_secs() * clock.speed as f32;
    for (e, mut v, mut tf, mut sprite) in q.iter_mut() {
        if v.next >= v.route.len() {
            // at destination: count stay, then leave
            if !v.leaving {
                v.stay -= minutes;
                if v.stay <= 0.0 {
                    v.leaving = true;
                    let mut back: Vec<(i32, i32)> = WAYPOINTS.to_vec();
                    back.reverse();
                    back.push(ENTRY);
                    v.route = back;
                    v.next = 0;
                }
            }
            if v.leaving && v.next >= v.route.len() {
                commands.entity(e).despawn();
            }
            continue;
        }
        let (gx, gy) = iso::tile_center(v.route[v.next].0, v.route[v.next].1);
        let goal = Vec2::new(gx, gy);
        let pos = tf.translation.truncate();
        let delta = goal - pos;
        let dist = delta.length();
        let step = v.speed * 30.0 * dt; // ~ px/s scaled like original
        if dist <= step {
            tf.translation.x = gx;
            tf.translation.y = gy;
            v.next += 1;
            if v.leaving && v.next >= v.route.len() {
                commands.entity(e).despawn();
                continue;
            }
        } else {
            let dir = delta / dist;
            tf.translation.x += dir.x * step;
            tf.translation.y += dir.y * step;
            // face direction: frames 1..4 are down/up/left/right-ish; use flip for now
            sprite.flip_x = dir.x < 0.0;
        }
        // painter depth by y
        tf.translation.z = 400.0 - tf.translation.y * 0.001;
    }
}
