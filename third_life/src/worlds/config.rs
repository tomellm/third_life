#![allow(dead_code)]

use std::fs;
use proc_macros::{ConfigFile, Config};
use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::{config::{ConfigurationLoader, RegisterConfigReaderEvent, ConfigReaderFinishedEvent, SelectedConfigPath}, SimulationState};
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
    name: String,
    #[def(PopulationConfig::def_conf())]
    population: Option<PopulationConfig>,
    #[def(EnvironmentConfig::def_conf())]
    environment: Option<EnvironmentConfig>
}

#[derive(Deserialize, Debug, Clone, Resource, Default, Config)]
pub struct PopulationConfig {
    #[def(1000)]
    size: Option<u32>,
    #[def(18.)]
    location: Option<f32>,
    #[def(6.)]
    scale: Option<f32>,
    #[def(10.)]
    shape: Option<f32>,
}

#[derive(Deserialize, Debug, Clone, Resource, Default, Config)]
pub struct EnvironmentConfig {
    #[def(0.5)]
    urbanization: Option<f32>,
    #[def(1.)]
    env_health: Option<f32>,
    #[def(1.)]
    ecosystem_vitylity: Option<f32>,
}


