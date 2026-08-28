//! Resort Empire — Rust/Bevy port of the Flash original (Little Giant World).
//! Milestone 2: isometric map render + camera + default map (roads, lobby, sceneries).

mod build;
mod car;
mod data;
mod game;
mod iso;
mod map;
mod missions;
mod ui;
mod visitor;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::PresentMode;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // The original assets have no .meta files; skip probing them
                    // (avoids hundreds of harmless 404s on the web build).
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Resort Empire".into(),
                        present_mode: PresentMode::AutoVsync,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(Color::srgb(0.15, 0.55, 0.72)))
        .add_plugins((
            data::DataPlugin,
            map::MapPlugin,
            game::GamePlugin,
            ui::UiPlugin,
            build::BuildPlugin,
            visitor::VisitorPlugin,
            car::CarPlugin,
            missions::MissionPlugin,
        ))
        .run();
}
