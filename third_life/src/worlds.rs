pub mod config;
mod food;
mod population;
mod ui;
mod env_and_infra;
mod wealth;



use bevy::{prelude::*, ecs::world};

use crate::{
    SimulationState, animation::{AnimationIndex, SpriteSize, AnimationTimer, ColonyAnimationBundle}, config::SelectedConfigPath,
};

use self::{
    config::{SpriteConfig, WorldConfig, WorldsConfig, WorldsConfigPlugin}, env_and_infra::{components::ColonyInfraAndEnvBundle, InfrastructurePlugin}, food::FoodPlugin, population::{components::Population, PopulationPlugin}, ui::WorldsUiPlugin, wealth::{components::{ColonyWealthBundle, WealthAndSpending}, WealthPlugin}
};

pub struct WorldsPlugin;

impl Plugin for WorldsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SimulationState::Running), init_colonies)
            .add_plugins((
                WorldsConfigPlugin, PopulationPlugin, FoodPlugin, WorldsUiPlugin, 
                InfrastructurePlugin, WealthPlugin
            ));

    }
}



fn init_colonies(
    mut commands: Commands,
    worlds_config: Res<WorldsConfig>,
    asset_server: Res<AssetServer>,
    config_path: Res<SelectedConfigPath>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let font = asset_server.load("fonts/VictorMonoNerdFontMono-Medium.ttf");
    for world in worlds_config.worlds() {
        let texture = asset_server.load(format!(
            "{}/sprite_sheets/{}",
            config_path.0,
            world.sprite().sprite_sheet()
        ));
        commands.spawn(WorldColonyBundle::new(
                texture,
                &mut texture_atlas_layouts,
                world.clone()
        )).with_children(|parent| {
            parent.spawn(
                Text2dBundle {
                    text: Text::from_section(world.name(), TextStyle { 
                        font: font.clone(), font_size: 24., color: Color::WHITE 
                    }).with_justify(JustifyText::Center),
                    transform: Transform::from_xyz(0.,-1. * world.sprite().shape().0 as f32, 0.),
                    ..default()
                }
            );
        });
    }
}

#[derive(Component, PartialEq)]
pub struct WorldColony;

#[derive(Component)]
pub struct WorldEntity {
    name: String,
}

impl WorldEntity {
    fn new(name: String) -> Self {
        WorldEntity { name }
    }
}

#[derive(Component)]
pub struct ResourceAmount(f64);

#[derive(Bundle)]
pub struct WorldColonyBundle {
    colony: WorldColony,
    entity: WorldEntity,
    population: Population,
    animation: ColonyAnimationBundle,
    wealth: ColonyWealthBundle,
    infra_and_env: ColonyInfraAndEnvBundle,
    config: WorldConfig
}

impl WorldColonyBundle {
    pub fn new(
        sprite_sheet: Handle<Image>,
        mut texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        world: WorldConfig
    ) -> Self {
        Self { 
            colony: WorldColony,
            entity: WorldEntity::new(world.name()),
            population: Population::default(),
            animation: ColonyAnimationBundle::new(
                world.name(), world.world_position(), 
                sprite_sheet, texture_atlas_layouts,
                world.sprite()
            ),
            wealth: ColonyWealthBundle::new(world.government()),
            infra_and_env: ColonyInfraAndEnvBundle::default(),
            config: world
        }
    }
}
