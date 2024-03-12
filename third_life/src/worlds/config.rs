#![allow(dead_code)]


use proc_macros::{ConfigFile, Config};
use bevy::prelude::*;
use serde::{Deserialize};

use crate::{config::{ConfigurationLoader}};
impl ConfigurationLoader for WorldsConfig {
    fn path_with_name() -> &'static str {
        "worlds"
    }
}

#[derive(Deserialize, Debug, Clone, Resource, ConfigFile, Default, Config)]
pub struct WorldsConfig {
    worlds: Vec<WorldConfig>
}

#[derive(Deserialize, Debug, Clone, Resource, Default, Config, Component)]
pub struct WorldConfig {
    /// Name should be unique, since its used for identification of multiple 
    /// things.
    name: String,
    world_position: (isize, isize),
    #[def(PopulationConfig::def_conf())]
    population: Option<PopulationConfig>,
    #[def(GovernmentConfig::def_conf())]
    government: Option<GovernmentConfig>,
    #[def(EnvironmentConfig::def_conf())]
    environment: Option<EnvironmentConfig>,
    #[def(FoodConfig::def_conf())]
    food: Option<FoodConfig>,
    sprite: SpriteConfig,
}

/// Different parameters affecting the population directly
#[derive(Deserialize, Debug, Clone, Resource, Default, Config)]
pub struct PopulationConfig {
    /// Starting number of Peple. Any Real number
    #[def(1000)]
    size: Option<u32>,
    /// Location of the Skew normal distribution. Any Positive number
    #[def(18.)]
    location: Option<f32>,
    /// Scale of the Skew normal distribution. Any Real number
    #[def(6.)]
    scale: Option<f32>,
    /// Shape of the Skew normal distribution
    #[def(10.)]
    shape: Option<f32>,
    /// The spread in this case refers to at which age the probability of death
    /// starts to increase. Does not affect the actual average life expectancy.
    #[def(6.)]
    life_expectancy_spread: Option<f32>
}

#[derive(Deserialize, Debug, Clone, Resource, Default, Config)]
pub struct GovernmentConfig {
    #[def(0.1)]
    citizen_payout: Option<f32>,
    #[def(25)]
    civil_spending: Option<usize>,
    #[def(25)]
    sanitation_spending: Option<usize>,
    #[def(25)]
    social_spending: Option<usize>,
    #[def(25)]
    environmental_spending: Option<usize>,
}

/// General factors of the environment of the world
#[derive(Deserialize, Debug, Clone, Resource, Default, Config)]
pub struct EnvironmentConfig {
    #[def(0.5)]
    urbanization: Option<f32>,
    #[def(1.)]
    env_health: Option<f32>,
    #[def(1.)]
    ecosystem_vitylity: Option<f32>,
}


#[derive(Deserialize, Debug, Clone, Resource, Default, Config)]
pub struct FoodConfig {
    #[def(6)]
    cow_farms: Option<usize>,
     #[def(4)]
    wheat_farms: Option<usize>,
    #[def(5000.0)]
    starting_beef: Option<f32>,
     #[def(5000.0)]
    starting_carb: Option<f32>,
}

#[derive(Deserialize, Debug, Clone, Resource, Default, Config)]
pub struct SpriteConfig {
    sprite_sheet: String,
    frames: usize,
    frames_layout: (usize, usize),
    shape: (usize, usize),
    animation_timer: f32
}


