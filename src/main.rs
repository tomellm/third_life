mod time;
mod worlds;
use bevy::{log::LogPlugin, prelude::*};
mod common;
use bevy_egui::EguiPlugin;
use time::TimeDatePlugin;
use worlds::WorldsPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Third Life".into(),
                        resolution: (800.0, 600.0).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(
                    LogPlugin {
                        level: bevy::log::Level::DEBUG,
                        ..default()
                    }
                )
        )
        .add_plugins(EguiPlugin)
        .add_plugins((TimeDatePlugin, WorldsPlugin))
        .run();
}
