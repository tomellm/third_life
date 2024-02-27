use std::{hash::Hash, usize};

use bevy::{log::tracing_subscriber::fmt::time, prelude::*, transform::commands, utils::HashMap};

use crate::{time::DateChanged, SimulationState};

use super::{init_colonies, population::{self, Citizen, Population}, WorldColony};

pub struct FoodPlugin;
impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SimulationState::Running), init_farms.after(init_colonies))
            .add_systems(Update, (check_farm_workers).run_if(in_state(SimulationState::Running)));
    }
}

#[derive(Component)]
pub struct FoodResource;

#[derive(Component)]
pub struct MeatResource;

#[derive(Component)]
pub struct CarbResource {
    amount: f32,
}

#[derive(Component)]
pub struct WheatFarm {
    available_resource: f32,
}

#[derive(Component)]
pub struct WheatFarmer {
    farm: Entity,
}

fn init_farms(mut commands: Commands, colonies: Query<Entity, With<WorldColony>>) {
    for colony in colonies.iter() {
        commands.entity(colony).with_children(|commands| {
            commands.spawn(WheatFarm {
                available_resource: 100.0,
            });
        });
    }
}

fn check_farm_workers(
    mut commands: Commands,
    mut population: Query<(&mut Population, &Parent)>,
    farmers: Query<(&WheatFarmer)>, 
) {
    let farms_map = farmers.iter()
        .fold(HashMap::new(), |mut acc: HashMap<Entity, usize>, farmer | {
            *acc.entry(farmer.farm).or_insert(0) += 1;
            acc
        });
    
    
}

/*
fn create_meat(
    farms: Query<(&CowFarm, &Parent)>>,
    worlds: Query<(Entity, &Children), With<Planet>>,
    meats: Query<(Entity, &ResourceAmount), With<MeatResource>>,
) {

}


fn work_on_cow_farm(
    farms: Query<(&CowFarm, &Parent)>,
    planet: Query<(&ColonyPopulation, &Children)>
) {

}

fn create_carb(
    query: Query<&ResourceAmount, With<CarbResource>>
) {

}

fn create_food(
    meats: Query<&ResourceAmount, With<MeatResource>>,
    carbs: Query<&ResourceAmount, With<CarbResource>>,
    food: Query<&ResourceAmount, With<FoodResource>>
) {

}
*/
