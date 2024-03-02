

use bevy::{prelude::*, utils::HashMap};

use crate::SimulationState;

use super::{
    food::{FoodResource, ResourceOf},
    population::components::{Citizen, CitizenOf, Starving},
};

pub struct ConsumptionPlugin;
impl Plugin for ConsumptionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (consume).run_if(in_state(SimulationState::Running)));
    }
}

fn consume(
    mut commands: Commands,
    mut citizens: Query<(Entity, &Citizen, &CitizenOf, Option<&mut Starving>)>,
    mut food_resources: Query<(&mut FoodResource, &ResourceOf)>,
) {
    let mut food_map = food_resources
        .iter_mut()
        .map(|(food_resource, resource_of)| (resource_of.colony, food_resource))
        .collect::<HashMap<_, _>>();
    citizens.iter_mut().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, f32>, (entity, _, citizen_of, starving)| {
            let food_eaten = 1.0*1.0;
            *acc.entry(citizen_of.colony).or_insert(0.0) += food_eaten;
            let food_resource = food_map.get_mut(&citizen_of.colony).unwrap();
            if food_resource.amount < *acc.get(&citizen_of.colony).unwrap() {
                starving.map_or_else(
                    || { commands.get_entity(entity).map(|mut e| {
                        e.try_insert(Starving { days_since_last_meal: 1 });
                    });},
                    |mut starving| {
                        starving.days_since_last_meal += 1;
                    }
                );
            } else {
                starving.map(|_| {
                    commands.get_entity(entity).map(|mut e| {
                        e.remove::<Starving>();
                    });
                });
                food_resource.amount -= food_eaten;
            }
            
            acc
        },
    );
}
