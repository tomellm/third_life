pub mod components;
mod dying;
pub mod events;
mod food_consumption;
mod giving_birth;
mod relationships;

use components::*;
use dying::*;
use events::*;
use giving_birth::*;
use relationships::*;

use crate::{
    common::utils::roll_chance,
    time::{DateChanged, GameDate},
    SimulationState,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_egui::egui::ahash::HashMapExt;
use chrono::{Datelike, NaiveDate};
use rand::{thread_rng, Rng};
use rand_distr::{
    num_traits::{real::Real, Float},
    Distribution, SkewNormal,
};
use rnglib::{Language, RNG};

use self::food_consumption::FoodConsumptionPlugin;

use super::{
    config::{WorldConfig, WorldsConfig},
    init_colonies, WorldColony, WorldEntity,
};

pub struct PopulationPlugin;

impl Plugin for PopulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            (init_citizens).chain().after(init_colonies),
        )
        .add_systems(
            Update,
            (update_population, check_birthdays, come_of_age, retirement)
                .run_if(in_state(SimulationState::Running)),
        )
        .add_event::<CitizenBirthday>()
        .add_plugins((
            GivingBirthPlugin,
            DeathsPlugin,
            RelationshipsPlugin,
            FoodConsumptionPlugin,
        ));
    }
}

pub fn init_citizens(
    colonies: Query<(Entity, &WorldConfig), With<WorldColony>>,
    mut commands: Commands,
    mut event_writer: EventWriter<CitizenCreated>,
    game_date: Res<GameDate>,
) {
    for (colony, pop_config) in colonies.iter() {
        let pop_config = pop_config.population();
        let mut rng = thread_rng();
        let name_rng = RNG::try_from(&Language::Roman).unwrap();
        let skew_normal = SkewNormal::new(
            pop_config.location(),
            pop_config.scale(),
            pop_config.shape(),
        )
        .unwrap();
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
                birthday,
            };
            if game_date.years_since(birthday).unwrap() >= 18 as u32 {
                match roll_chance(50) {
                    true => commands.spawn((citizen, Employable, CitizenOf { colony }, Male)),
                    false => commands.spawn((
                        citizen,
                        Employable,
                        CitizenOf { colony },
                        Female { children_had: 0 },
                    )),
                };
            } else {
                match roll_chance(50) {
                    true => commands.spawn((citizen, CitizenOf { colony }, Male)),
                    false => {
                        commands.spawn((citizen, CitizenOf { colony }, Female { children_had: 0 }))
                    }
                };
            }
            event_writer.send(CitizenCreated { age, colony });
        }
        commands.entity(colony).try_insert(Population::default());
    }
}

pub fn update_population(
    mut event_reader: EventReader<CitizenCreated>,
    mut populations: Query<(Entity, &mut Population)>,
    citizens: Query<(&Citizen, &CitizenOf)>,
    working_pop: Query<&CitizenOf, (Without<Youngling>, Without<Retiree>, Without<Pregnancy>)>,
    younglings: Query<&CitizenOf, With<Youngling>>,
    retirees: Query<&CitizenOf, With<Retiree>>,
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

                population.younglings = younglings.iter().filter(|&y| y.colony == colony).count();
                population.retirees = retirees.iter().filter(|&y| y.colony == colony).count();
                population.working_pop = working_pop.iter().filter(|&y| y.colony == colony).count();

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
                population.average_children_per_mother = all_women_children_had.iter().sum::<f32>()
                    / all_women_children_had.len() as f32;

                population.average_age =
                    all_citizen_ages.iter().sum::<usize>() / all_citizen_ages.len();
            }
        }
    }
}

pub fn check_birthdays(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut birthday_events: EventWriter<CitizenBirthday>,
    citizens: Query<(Entity, &Citizen)>,
) {
    for day_changed_event in day_changed_event_reader.read() {
        let game_date = day_changed_event.date;
        for (citizen_entity, citizen) in citizens.iter() {
            if game_date.month() == citizen.birthday.month()
                && game_date.day() == citizen.birthday.day()
            {
                birthday_events.send(CitizenBirthday {
                    entity: citizen_entity,
                    age: game_date.years_since(citizen.birthday).unwrap() as usize,
                });
            }
        }
    }
}

pub fn come_of_age(
    mut commands: Commands,
    mut birthday_event_reader: EventReader<CitizenBirthday>,
) {
    for birthday in birthday_event_reader.read() {
        if birthday.age == 18 {
            commands.get_entity(birthday.entity).map(|mut e| {
                e.remove::<Youngling>();
            });
            warn!("{:?} became of age", birthday.entity);
        }
    }
}

pub fn retirement(mut commands: Commands, mut birthday_event_reader: EventReader<CitizenBirthday>) {
    for birthday in birthday_event_reader.read() {
        if birthday.age == 65 {
            commands.get_entity(birthday.entity).map(|mut e| {
                e.remove::<Youngling>();
            });
            warn!("{:?} retired", birthday.entity);
        }
    }
}
