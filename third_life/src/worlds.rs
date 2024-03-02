mod config;
mod food;
mod population;
mod ui;
mod food_consumption;


use bevy::prelude::*;

use crate::{
    SimulationState,
};

use self::{
    config::{WorldsConfig, WorldsConfigPlugin},
    food::FoodPlugin, population::{PopulationPlugin, components::Population}, ui::WorldsUiPlugin, food_consumption::ConsumptionPlugin
};

pub struct WorldsPlugin;

impl Plugin for WorldsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SimulationState::Running), init_colonies)
            .add_plugins((WorldsConfigPlugin, PopulationPlugin, FoodPlugin, WorldsUiPlugin, ConsumptionPlugin))
            /*.add_systems(
                Update,
                display_colonies.run_if(in_state(SimulationState::Running)),
            )*/;
    }
}

#[derive(Component, PartialEq)]
pub struct WorldColony;

#[derive(Component)]
pub struct WorldEntity {
    name: String,
}

impl WorldEntity {
    fn new(name: &str) -> Self {
        let name = name.to_string();
        WorldEntity { name }
    }
}

#[derive(Component)]
pub struct ResourceAmount(f64);

fn init_colonies(
    worlds_config: Res<WorldsConfig>,
    mut commands: Commands
) {
    for world in worlds_config.worlds() {
        commands.spawn((WorldColony, WorldEntity::new(&world.name()), Population::default()));
    }
}
