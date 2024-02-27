#[allow(dead_code)]

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

#[derive(Deserialize, Debug, Clone, Resource, ConfigFile, Default)]
pub struct WorldsConfig {
    worlds: Vec<WorldConfig>
}

#[derive(Deserialize, Debug, Clone, Resource, Default, Config)]
pub struct WorldConfig {
    name: String,
    population: PopulationConfig
}

#[derive(Deserialize, Debug, Clone, Resource, Default, Config)]
pub struct PopulationConfig {
    #[def(1000.)]
    size: Option<f32>,
    #[def(30.)]
    median: Option<f32>,
    #[def(4.)]
    stdev: Option<f32>
}


