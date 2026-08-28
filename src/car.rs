//! Road traffic: port of `pack/Instance/Mobil.as` dummy cars/buses driving on
//! the entry roads (rows 4-5 / cols 5-6). Frames car/1..6.png:
//!   1 = Bus arah 1 (color A), 2 = Bus arah -1 (color A) — actually the export
//!   collapsed colors: 1/2 = buses both directions, 3/4 = second bus color or
//!   vans both directions, 5/6 = vans. Visual check: 1&2 buses (orange/blue),
//!   3&4 vans dir 1/-1, 5&6 remaining pair. We assign by direction so each
//!   route uses a frame whose art faces the travel direction.
//!
//! Original spawn (Mobil.as, dummy mode): every few seconds a car spawns with
//! bantuRand = rand(0..100):
//!   >= 60      -> arah=1, io=1  : drives row 4 from far edge toward exit (j: max->1)
//!   <  20      -> io=4         : passenger bus on row 5, stops at stop_point (5,19)
//!   otherwise  -> io=2/3       : drives col 5 or 6 from far edge (i: max->1)
//! speed starts at 2 px/frame and accelerates each frame up to
//! speedMax = rand(7..10) px/frame (30 fps).
//!
//! Routes derive from serbi.as enterCar/outerCar with the expand-0 substitution
//! (y==0 -> COLS-5, x==0 -> ROWS-3); with the default 37x37 map that gives
//! COLS-5 = 32 and ROWS-3 = 34.

use crate::data::AppState;
use crate::game::{GameClock, Wallet};
use crate::iso::{HALF, SIZE};
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// Fired when a passenger bus finishes its stop and unloads visitors.
/// Port of Mobil.activitiesOnTick: while stopped (udahStop && penumpang) one
/// visitor is created per tick until jumlahV, via parent_.newVisitor(false,true).
#[derive(Event)]
pub struct BusArrival {
    /// number of visitors stepping off (jumlahV)
    pub count: u32,
    /// sidewalk tile where they appear (next to the stop_point)
    pub tile: (i32, i32),
}

/// Base z for cars: above tiles/objects band, below HUD. Depth within the band
/// follows the painter rule (bigger tx+ty = farther back = smaller z).
const Z_CAR: f32 = 380.0;

/// Anchor point of the 188x115 car frames: where the sprite's registration
/// (the tile lattice point of its position) sits inside the bitmap.
/// Estimated from frame composition; verify against a build screenshot.
const CAR_ANCHOR: (f32, f32) = (94.0, 96.0);

/// Fractional tile -> world (same formula as iso::tile_to_world but f32 tiles
/// so cars can start slightly off-map and glide smoothly).
fn tile_f_to_world(tx: f32, ty: f32) -> (f32, f32) {
    ((ty - tx) * SIZE, (tx + ty) * HALF)
}

#[derive(Component)]
pub struct Car {
    /// waypoints in fractional tile coords
    pub route: Vec<(f32, f32)>,
    pub next: usize,
    /// px per original frame (30 fps), accelerates to speed_max
    pub speed: f32,
    pub speed_max: f32,
    /// waypoint index at which the bus pauses (passenger stop)
    pub stop_at: Option<usize>,
    /// seconds of pause left at the stop
    pub pause: f32,
    /// visitors to unload at the stop (jumlahV); 0 for dummy traffic
    pub passengers: u32,
    /// seconds until the next passenger steps off (paces 1-per-tick unloading)
    pub unload_timer: f32,
}

#[derive(Resource)]
pub struct CarSpawn {
    pub cooldown: f32,
    pub rng: u64,
}

impl CarSpawn {
    fn rand(&mut self) -> f64 {
        let mut x = self.rng.wrapping_add(0x9E3779B97F4A7C15);
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        (x.wrapping_mul(0x2545F4914F6CDD1D) >> 11) as f64 / (1u64 << 53) as f64
    }
}

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CarSpawn {
            cooldown: 3.0,
            rng: 0xCA7,
        })
        .add_event::<BusArrival>()
        .add_systems(
            Update,
            (spawn_cars, drive).run_if(in_state(AppState::Playing)),
        );
    }
}

fn spawn_cars(
    time: Res<Time>,
    clock: Res<GameClock>,
    mut st: ResMut<CarSpawn>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    wallet: Res<Wallet>,
) {
    if clock.paused {
        return;
    }
    st.cooldown -= time.delta_secs() * clock.speed as f32;
    if st.cooldown > 0.0 {
        return;
    }
    // next car in 6-18 s (scaled by clock speed like the original jeda)
    st.cooldown = 6.0 + st.rand() as f32 * 12.0;

    let roll = st.rand() * 100.0;
    let speed_max = 7.0 + st.rand() as f32 * 3.0; // rand(7..10) px/frame
    let color = st.rand() < 0.5;

    // Route + frame per Mobil.as branches. Default map: COLS-5=32, ROWS-3=34.
    let mut passengers = 0u32;
    let (route, frame, stop_at): (Vec<(f32, f32)>, u32, Option<usize>) = if roll >= 60.0 {
        // arah=1, io=1: along row 4 from the far edge toward the exit (j 32 -> 1)
        let f = if color { 1 } else { 2 };
        (vec![(4.0, 32.0), (4.0, 1.0)], f, None)
    } else if roll < 20.0 {
        // io=4: passenger vehicle on row 5, stops at (5,19), then continues out.
        // newMobil(): rand(0..100) > 45 -> "Bus" else "Box" van.
        // maxVisitor = clamp(pop*0.1, 1, 8) for Bus, clamp(.., 1, 4) for Box;
        // jumlahV = randomRange(1, maxVisitor).
        let bus = st.rand() * 100.0 > 45.0;
        let cap = if bus { 8 } else { 4 };
        let max_v = ((wallet.popularity * 0.1) as u32).clamp(1, cap);
        passengers = 1 + (st.rand() * max_v as f64) as u32;
        // Bus art: frames 1/2 (orange/blue); Box van art facing the same
        // direction: frame 5 (white van).
        let f = if bus {
            if color {
                1
            } else {
                2
            }
        } else {
            5
        };
        (vec![(5.0, 32.0), (5.0, 19.0), (5.0, 1.0)], f, Some(1))
    } else {
        // io=2/3: vans along col 5 or 6 from the far edge (i 34 -> 1).
        // Columns run the opposite screen direction to the rows, so use the
        // arah -1 art: frames 4/6 are the vans facing that way.
        let col = if st.rand() < 0.5 { 5.0 } else { 6.0 };
        let f = if color { 4 } else { 6 };
        (vec![(34.0, col), (1.0, col)], f, None)
    };

    let (sx, sy) = tile_f_to_world(route[0].0, route[0].1);
    let anchor = Anchor::Custom(Vec2::new(
        0.5 - CAR_ANCHOR.0 / 188.0,
        CAR_ANCHOR.1 / 115.0 - 0.5,
    ));
    commands.spawn((
        Sprite {
            image: assets.load(format!("sprites/car/{frame}.png")),
            anchor,
            ..default()
        },
        Transform::from_xyz(sx, sy, Z_CAR),
        Car {
            route,
            next: 1,
            speed: 2.0,
            speed_max,
            stop_at,
            pause: 0.0,
            passengers,
            unload_timer: 0.0,
        },
    ));
}

fn drive(
    time: Res<Time>,
    clock: Res<GameClock>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Car, &mut Transform)>,
    mut arrivals: EventWriter<BusArrival>,
) {
    if clock.paused {
        return;
    }
    let dt = time.delta_secs() * clock.speed as f32;
    for (e, mut car, mut tf) in q.iter_mut() {
        if car.pause > 0.0 {
            // Unload 1 visitor per tick while stopped (activitiesOnTick:
            // one newVisitor(false, true) per tick until jumlah_turun == jumlahV).
            if car.passengers > 0 {
                car.unload_timer -= dt;
                if car.unload_timer <= 0.0 {
                    car.unload_timer += 0.25;
                    arrivals.write(BusArrival {
                        count: 1,
                        tile: (4, 19),
                    });
                    car.passengers -= 1;
                }
                // hold at the stop until everyone is off
                car.pause = car.pause.max(0.5);
            }
            car.pause -= dt;
            car.speed = 2.0; // pull away slowly after the stop
            continue;
        }
        if car.next >= car.route.len() {
            commands.entity(e).despawn();
            continue;
        }
        // accelerate: original adds a bit each 30fps frame up to speedMax
        car.speed = (car.speed + 4.0 * dt).min(car.speed_max);
        let (gx, gy) = tile_f_to_world(car.route[car.next].0, car.route[car.next].1);
        let goal = Vec2::new(gx, gy);
        let pos = tf.translation.truncate();
        let delta = goal - pos;
        let dist = delta.length();
        let step = car.speed * 30.0 * dt; // px/frame * 30fps
        if dist <= step {
            tf.translation.x = gx;
            tf.translation.y = gy;
            if car.stop_at == Some(car.next) {
                // dwell at the stop; unloading happens in the pause branch,
                // paced 1 visitor per tick like the original
                car.pause = 2.5;
                car.unload_timer = 0.25;
            }
            car.next += 1;
        } else {
            let dir = delta / dist;
            tf.translation.x += dir.x * step;
            tf.translation.y += dir.y * step;
        }
        // painter depth from the lattice sum s = wy/HALF (= tx+ty)
        let s = tf.translation.y / HALF;
        tf.translation.z = Z_CAR + (200.0 - s) * 0.001;
    }
}
