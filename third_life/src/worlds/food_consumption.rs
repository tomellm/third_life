use std::usize;

use bevy::{prelude::*, utils::HashMap};

use crate::SimulationState;

use super::{
    food::{FoodResource, ResourceOf},
    population::{Citizen, CitizenOf},
};

pub struct ConsumptionPlugin;
impl Plugin for ConsumptionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (consume).run_if(in_state(SimulationState::Running)));
    }
}

fn consume(
    mut citizens: Query<(Entity, &mut Citizen, &CitizenOf)>,
    mut food_resources: Query<(&mut FoodResource, &ResourceOf)>,
) {
    let mut food_map = food_resources
        .iter_mut()
        .map(|(food_resource, resource_of)| (resource_of.colony, food_resource))
        .collect::<HashMap<_, _>>();
    citizens.iter_mut().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, f32>, (_, mut citizen, citizen_of)| {
            let food_eaten = 1.0*1.0;
            *acc.entry(citizen_of.colony).or_insert(0.0) += food_eaten;
            let food_resource = food_map.get_mut(&citizen_of.colony).unwrap();
            if food_resource.amount < *acc.get(&citizen_of.colony).unwrap()
            {
                citizen.days_since_meal += 1
            } else {
                food_resource.amount -= food_eaten;
                citizen.days_since_meal = 0;
            }
            acc
        },
    );
}
