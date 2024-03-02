use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct WorldUi;

#[derive(Component)]
pub struct WorldUiName(pub String);

#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct WorldUiEntity(pub Entity);

#[derive(Component)]
pub struct PopulationHistorgram {
    pub count: usize,
    pub average_age: usize,
    pub ages: HashMap<usize, usize>,
    pub average_children_per_mother: f32, 
}



#[derive(Component)]
pub struct ResourceStorage {
    pub timer: Timer,
    pub meat: Vec<f32>,
    pub carb: Vec<f32>,
    pub food: Vec<f32>,
}

impl ResourceStorage {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            meat: vec![0.], carb: vec![0.], food: vec![0.]
        }
    }
}

#[derive(Bundle)]
pub struct WorldUiBundle {
    pub ui: WorldUi,
    pub name: WorldUiName,
    pub entity: WorldUiEntity,
    pub pop: PopulationHistorgram,
    pub stor: ResourceStorage,
}

impl WorldUiBundle {
    pub fn new(name: String, entity: Entity) -> Self {
        Self { 
            ui: WorldUi,
            name: WorldUiName(name),
            entity: WorldUiEntity(entity),
            pop: PopulationHistorgram { count: 0, average_age: 0, ages: HashMap::new(), average_children_per_mother: 0.0},
            stor: ResourceStorage::new(),
        }
    }
}
