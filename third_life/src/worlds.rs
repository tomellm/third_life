mod config;
mod food;
mod population;
mod ui;


use bevy::prelude::*;

use crate::{
    SimulationState, animation::{AnimationIndex, SpriteSize, AnimationTimer}, config::SelectedConfigPath,
};

use self::{
    config::{SpriteConfig, WorldConfig, WorldsConfig, WorldsConfigPlugin},
    food::FoodPlugin, population::{components::Population, PopulationPlugin}, ui::WorldsUiPlugin,
};

pub struct WorldsPlugin;

impl Plugin for WorldsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SimulationState::Running), init_colonies)
            .add_plugins((WorldsConfigPlugin, PopulationPlugin, FoodPlugin, WorldsUiPlugin))
            /*.add_systems(
                Update,
                display_colonies.run_if(in_state(SimulationState::Running)),
            )*/;
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
        println!(
            "{}/sprite_sheets/{}",
            config_path.0,
            world.sprite().sprite_sheet()
        );
        let texture = asset_server.load(format!(
            "{}/sprite_sheets/{}",
            config_path.0,
            world.sprite().sprite_sheet()
        ));
        commands.spawn(WorldColonyBundle::new(
                world.name(),
                texture,
                world.world_position(),
                &mut texture_atlas_layouts, 
                world.sprite().frames(),
                world.sprite().frames_layout(),
                world.sprite().shape(),
                world.sprite().animation_timer(),
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
    sprite_sheet_bundle: SpriteSheetBundle,
    animation_index: AnimationIndex,
    sprite_size: SpriteSize,
    animation_timer: AnimationTimer,
    config: WorldConfig
}

impl WorldColonyBundle {
    pub fn new(
        name: String,
        sprite_sheet: Handle<Image>,
        (world_position_x, world_position_y): (isize, isize),
        mut texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        num_frames: usize,
        (frames_layout_x, frames_layout_y): (usize, usize),
        (shape_x, shape_y): (usize, usize),
        animation_timer: f32,
        config: WorldConfig
    ) -> Self {
        let layout = TextureAtlasLayout::from_grid(
            Vec2::new(shape_x as f32, shape_y as f32), 
            frames_layout_x, frames_layout_y,
            None, None
        );
        let sprite_size = SpriteSize(layout.size / layout.len() as f32);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        let animation_index = AnimationIndex::new(num_frames);
        Self { 
            colony: WorldColony,
            entity: WorldEntity::new(name),
            population: Population::default(),
            sprite_sheet_bundle: SpriteSheetBundle {
                transform: Transform::from_xyz(
                    world_position_x as f32, world_position_y as f32, 0.
                ).with_scale(Vec3::splat(2.)),
                texture: sprite_sheet,
                atlas: TextureAtlas { 
                    layout: texture_atlas_layout,
                    index: animation_index.first
                },
                ..default()
            },
            animation_index,
            sprite_size,
            animation_timer: AnimationTimer(Timer::from_seconds(
                    animation_timer,
                    TimerMode::Repeating
            )),
            config
        }
    }
}



/*
let texture = asset_server.load("rotating-earth-spritesheet.png");
let layout = TextureAtlasLayout::from_grid(Vec2::new(32.0, 32.0), 5, 6, None, None);
let sprite_size = PlanetSpriteSize(layout.size / layout.len() as f32);
let texture_atlas_layout = texture_atlas_layouts.add(layout);
let animation_index = PlanetAnimationIndex { first: 0, last: 29 };


SpriteSheetBundle {
        transform: Transform::from_translation(Vec3::splat(0.))
            .with_scale(Vec3::splat(5.)),
        texture,
        atlas: TextureAtlas {
            layout: texture_atlas_layout,
            index: animation_index.first
        },
        ..default()
    }



*/
