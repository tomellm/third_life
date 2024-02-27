mod worlds;
mod time;
mod common;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use time::TimeDatePlugin;
use worlds::WorldsPlugin;

fn main() {


    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin))
        .add_plugins((TimeDatePlugin, WorldsPlugin))
        .run();
}

