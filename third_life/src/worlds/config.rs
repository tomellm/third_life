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

#[derive(Deserialize, Debug, Clone, Resource, Default, Config)]
pub struct WorldConfig {
    /// Name should be unique, since its used for identification of multiple 
    /// things.
    name: String,
    #[def(PopulationConfig::def_conf())]
    population: Option<PopulationConfig>,
    #[def(EnvironmentConfig::def_conf())]
    environment: Option<EnvironmentConfig>
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


