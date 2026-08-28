//! Mission engine: simplified port of Application.CheckMissions / Mission_Temp.
//!
//! The original keeps an ordered queue (serbi.Mission.listing minus cleared
//! ones) and always shows the FIRST pending mission in the navigator3 ticker,
//! with its bonus next to the gift icon. When the condition is met it plays
//! "bonus1", adds the bonus to money, shifts the queue and shows the next one.
//!
//! Missions whose systems aren't ported yet (bookings, janitors, upgrades...)
//! are skipped so the queue keeps progressing; they're listed in todo.md.

use crate::data::{AppState, DataHandles, GameData};
use crate::game::Wallet;
use crate::ui::HudCounts;
use crate::visitor::Visitor;
use bevy::prelude::*;

/// What a mission measures, mapped from its `initial` key.
/// Mirrors the big if/else chain in Application.CheckMissions_sub.
#[derive(Clone, Copy)]
enum Metric {
    Rooms,
    Facilities,
    Plants,
    RoomsPlusFacilities,
    Money,
    Visitors,
    /// player-painted path tiles (approximation of CheckRoomWithPath)
    TilesPainted,
}

/// Key -> (metric, goal override). Override 0 = use `batas` from gamedata
/// (or 1 when batas is 0, for the one-shot tutorial missions).
fn metric_for(key: &str) -> Option<(Metric, f64)> {
    Some(match key {
        "build_room" => (Metric::Rooms, 1.0),
        "connect_tiles" => (Metric::TilesPainted, 1.0),
        "put_scenery" => (Metric::Plants, 1.0),
        "build_facility" => (Metric::Facilities, 1.0),
        "x_money" | "x_money2" => (Metric::Money, 0.0),
        "x_tree" | "xx_tree" | "xx_tree2" | "xx_tree3" => (Metric::Plants, 0.0),
        "x_facilityRoom" | "x_booths" | "x_booths2" => (Metric::RoomsPlusFacilities, 0.0),
        "x_room" | "xx_room" => (Metric::Rooms, 0.0),
        "x_facility" | "xx_facility" | "xxx_facility" => (Metric::Facilities, 0.0),
        "x_visitors" | "x_visitors2" => (Metric::Visitors, 0.0),
        _ => return None,
    })
}

/// The live mission queue + what the HUD should currently display.
#[derive(Resource, Default)]
pub struct MissionState {
    pub queue: Vec<String>,
    pub initialized: bool,
    /// ticker line for the active mission ("desc" or "desc(n/goal)")
    pub ticker: String,
    /// gift bonus label ("$ 500")
    pub gift: String,
    /// seconds left showing MISSION CLEAR! after a completion
    pub flash: f32,
}

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MissionState>()
            .add_systems(Update, check_missions.run_if(in_state(AppState::Playing)));
    }
}

fn check_missions(
    time: Res<Time>,
    mut st: ResMut<MissionState>,
    mut wallet: ResMut<Wallet>,
    counts: Res<HudCounts>,
    visitors: Query<(), With<Visitor>>,
    handles: Res<DataHandles>,
    gamedata: Res<Assets<GameData>>,
) {
    let Some(gd) = gamedata.get(&handles.gamedata) else {
        return;
    };
    if !st.initialized {
        st.queue = gd.mission_listing();
        st.initialized = true;
    }
    if st.flash > 0.0 {
        st.flash -= time.delta_secs();
        return; // keep showing MISSION CLEAR! briefly, like acceptFlow()
    }

    // find the first mission we can evaluate (skip unported ones)
    let mut skipped = 0usize;
    let mut found: Option<(String, Metric, f64, f64)> = None;
    for key in &st.queue {
        if let Some((metric, goal_override)) = metric_for(key) {
            if let Some(def) = gd.mission_def(key) {
                let goal = if goal_override > 0.0 {
                    goal_override
                } else if def.batas > 0.0 {
                    def.batas
                } else {
                    1.0
                };
                found = Some((key.clone(), metric, goal, def.bonus));
                break;
            }
        }
        skipped += 1;
    }
    // drop unevaluable heads so the visible mission is always the queue head
    if skipped > 0 {
        st.queue.drain(0..skipped);
    }
    let Some((key, metric, goal, bonus)) = found else {
        if st.ticker != "ALL MISSIONS COMPLETE" {
            st.ticker = "ALL MISSIONS COMPLETE".into();
            st.gift = String::new();
        }
        return;
    };

    let current: f64 = match metric {
        Metric::Rooms => counts.rooms as f64,
        Metric::Facilities => counts.facilities as f64,
        Metric::Plants => counts.plants as f64,
        Metric::RoomsPlusFacilities => (counts.rooms + counts.facilities) as f64,
        Metric::Money => wallet.money,
        Metric::Visitors => visitors.iter().count() as f64,
        Metric::TilesPainted => counts.tiles_painted as f64,
    };

    if current >= goal {
        // cleared: bonus money, brief CLEAR flash, then next mission
        wallet.money += bonus;
        st.queue.retain(|k| k != &key);
        st.ticker = "MISSION CLEAR!".into();
        st.gift = String::new();
        st.flash = 2.0;
        return;
    }

    let Some(def) = gd.mission_def(&key) else {
        return;
    };
    let ticker = if goal > 1.0 {
        format!("{}({}/{})", def.desc.trim(), current as u64, goal as u64)
    } else {
        def.desc.trim().to_string()
    };
    if st.ticker != ticker {
        st.ticker = ticker;
    }
    let gift = format!("$ {}", bonus as u64);
    if st.gift != gift {
        st.gift = gift;
    }
}
