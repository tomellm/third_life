use bevy::{prelude::*, reflect::List};
use std::{collections::HashMap, time::Duration};

#[derive(Component)]
pub struct WorldUi;

#[derive(Component)]
pub struct WorldUiName(pub String);

#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct WorldUiEntity(pub Entity);

#[derive(Component)]
pub struct PopulationHistorgram {
    pub count: usize,
    pub working_pop: usize,
    pub younglings: usize,
    pub retirees: usize,
    pub average_age: usize,
    pub ages: HashMap<usize, usize>,
    pub average_children_per_mother: f32,
}

#[derive(Component)]
pub struct PopulationDeathLines {
    pub timer: Timer,
    pub old_age_deaths: Vec<usize>,
    pub starvation_deaths: Vec<usize>,
    pub infant_deaths: Vec<usize>,
}

impl PopulationDeathLines {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            old_age_deaths: vec![0],
            starvation_deaths: vec![0],
            infant_deaths: vec![0]
        }
    }
    pub fn new_step(&mut self, delta: Duration) {
        if self.timer.tick(delta).just_finished() {
            self.old_age_deaths.push(0);
            self.starvation_deaths.push(0);
            self.infant_deaths.push(0);
        }
        if self.old_age_deaths.len() > 25 {
            self.old_age_deaths.remove(0);
        }
        if self.starvation_deaths.len() > 25 {
            self.starvation_deaths.remove(0);
        }
        if self.infant_deaths.len() > 25 {
            self.infant_deaths.remove(0);
        }
    }
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
    pub deaths: PopulationDeathLines,
    pub stor: ResourceStorage,
}

impl WorldUiBundle {
    pub fn new(name: String, entity: Entity) -> Self {
        Self { 
            ui: WorldUi,
            name: WorldUiName(name),
            entity: WorldUiEntity(entity),
            pop: PopulationHistorgram { count: 0, working_pop: 0, younglings: 0, retirees: 0, average_age: 0, ages: HashMap::new(), average_children_per_mother: 0.0},
            deaths: PopulationDeathLines::new(),
            stor: ResourceStorage::new(),
        }
    }
}
