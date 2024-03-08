pub mod events;
pub mod components;
mod giving_birth;
mod dying;
mod relationships;
mod food_consumption;

use events::*;
use components::*;
use giving_birth::*;
use dying::*;
use relationships::*;


use crate::{
    common::utils::roll_chance,
    time::{GameDate},
    SimulationState,
};
use bevy::{prelude::*};
use bevy_egui::{egui::{ahash::{HashMapExt}}};
use chrono::{Datelike, NaiveDate};
use rand::{thread_rng, Rng};
use rand_distr::{num_traits::{Float, real::Real}, Distribution, SkewNormal};
use rnglib::{Language, RNG};

use self::food_consumption::FoodConsumptionPlugin;

use super::{config::{WorldConfig, WorldsConfig}, init_colonies, WorldColony, WorldEntity};

pub struct PopulationPlugin;

impl Plugin for PopulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            (init_citizens).chain().after(init_colonies),
        )
        .add_systems(
            Update,(
                update_population,
            ).run_if(in_state(SimulationState::Running)),
        )
        .add_plugins((GivingBirthPlugin, DeathsPlugin, RelationshipsPlugin, FoodConsumptionPlugin));
    }
}



pub fn init_citizens(
    colonies: Query<(Entity, &WorldConfig), With<WorldColony>>,
    mut commands: Commands,
    mut event_writer: EventWriter<CitizenCreated>,
    game_date: Res<GameDate>,
) {
    for (colony, pop_config) in colonies.iter() {
        let mut rng = thread_rng();
        let name_rng = RNG::try_from(&Language::Roman).unwrap();
        let skew_normal = SkewNormal::new(
            pop_config.location(), pop_config.scale(), pop_config.shape()
        ).unwrap();
        let mut age_gen = skew_normal.sample_iter(&mut rng);
        let year = game_date.date.year_ce().1 as usize;

        for _ in 0..pop_config.size() {
            let age = age_gen.next().unwrap().floor() as usize;
            let birthday = NaiveDate::from_yo_opt(
                (year - age).try_into().unwrap(),
                thread_rng().gen_range(1..=365),
            )
            .unwrap();

            let citizen = Citizen {
                name: name_rng.generate_name(),
                birthday
            };
            match roll_chance(50) {
                true => commands.spawn((citizen, CitizenOf { colony }, Male)),
                false => commands.spawn((citizen, CitizenOf { colony }, Female{children_had: 0})),
            };

            event_writer.send(CitizenCreated { age, colony });
        }
        commands.entity(colony).try_insert(Population::default());
    }
}



pub fn update_population(
    mut event_reader: EventReader<CitizenCreated>,
    mut populations: Query<(Entity, &mut Population)>,
    citizens: Query<(&Citizen, &CitizenOf)>,
    women: Query<(&Citizen, &CitizenOf, &Female)>,
    game_date: Res<GameDate>,
) {
    for event in event_reader.read() {
        for (colony, mut population) in &mut populations.iter_mut() {
            if colony == event.colony {
                population.count += 1;

                let all_citizen_ages: Vec<usize> = citizens
                    .iter()
                    .filter_map(|(citizen, citizen_of)| {
                        if citizen_of.colony == colony {
                            Some(game_date.date.years_since(citizen.birthday).unwrap() as usize)
                        } else {
                            None
                        }
                    })
                    .collect();

                let all_women_children_had: Vec<f32> = women
                    .iter()
                    .filter_map(|(_, citizen_of, female)| {
                        if citizen_of.colony == colony {
                            Some(female.children_had as f32)
                        } else {
                            None
                        }
                    })
                    .collect();
                population.average_children_per_mother = 
                    all_women_children_had.iter().sum::<f32>() / all_women_children_had.len() as f32;
                
                population.average_age =
                    all_citizen_ages.iter().sum::<usize>() / all_citizen_ages.len();
            }
        }
    }
}
