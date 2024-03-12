use std::{f32::consts, iter::zip};
use crate::{
    common::utils::roll_chance,
    time::{DateChanged, GameDate, MonthChanged},
    SimulationState, worlds::WorldColony,
};
use bevy::{prelude::*, transform::commands};
use bevy_egui::{egui::{Window, ahash::{HashMap, HashMapExt}}, EguiContexts};
use chrono::{Datelike, NaiveDate};
use rand::{thread_rng, Rng, rngs::ThreadRng};
use rand_distr::{num_traits::{Float, real::Real}, Distribution, SkewNormal};
use rnglib::{Language, RNG};

use super::{events::*, components::*};

pub struct GivingBirthPlugin;

impl Plugin for GivingBirthPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,(
                    init_ovulation,
                    end_ovulation,
                    init_miscarriage,
                    init_pregnancies,
                    citizen_births,
                ).run_if(in_state(SimulationState::Running))
            )
            .add_event::<CitizenCreated>();
    }
}

pub fn citizen_births(
    mut commands: Commands,
    mut event_reader: EventReader<DateChanged>,
    mut event_writer: EventWriter<CitizenCreated>,
    mut pregnant_women: Query<(Entity, &mut Citizen, &mut Pregnancy, &CitizenOf), With<Pregnancy>>,
    colonies: Query<Entity, With<WorldColony>>,
    game_date: Res<GameDate>,
) {
    for _ in event_reader.read() {
        for (entity, _, pregnancy, citizen_of) in &mut pregnant_women.iter_mut() {
            if pregnancy.baby_due_date == game_date.date {
                for colony in &colonies {
                    if citizen_of.colony == colony {
                        let name_rng = RNG::try_from(&Language::Roman).unwrap();

                        let new_born = Citizen {
                            name: name_rng.generate_name(),
                            birthday: game_date.date
                        };

                        match roll_chance(50) {
                            true => commands.spawn((new_born, CitizenOf { colony }, Youngling, Male)),
                            false => commands.spawn((new_born, CitizenOf { colony }, Youngling, Female { children_had: 0 } )),
                        };

                        event_writer.send(CitizenCreated { age: 0, colony });
                    }
                }
                commands.get_entity(entity).map(|mut e| {
                    e.remove::<Pregnancy>();
                    e.try_insert(Employable);
                });
            }
        }
    }
}

pub fn init_miscarriage(
    mut commands: Commands,
    mut event_reader: EventReader<MonthChanged>,
    mut pregnant_women: Query<(Entity, &Citizen), With<Pregnancy>>,
    game_date: Res<GameDate>,
) {
    for _ in event_reader.read() {
        for (entity, w_citizen) in &mut pregnant_women {
            if miscarriage_chance(game_date.date.years_since(w_citizen.birthday).unwrap() as u8) {
                commands.get_entity(entity).map(|mut e| {
                    e.remove::<Pregnancy>();
                    e.try_insert(Employable);
                });
            }
        }
    }
}

pub fn miscarriage_chance(age: u8) -> bool {
    match age {
        18..=19 => roll_chance(17),
        20..=24 => roll_chance(11),
        25..=29 => roll_chance(10),
        30..=34 => roll_chance(11),
        35..=39 => roll_chance(17),
        40..=44 => roll_chance(33),
        45.. => roll_chance(57),
        _ => false,
    }
}

pub fn init_ovulation(
    mut commands: Commands,
    mut event_reader: EventReader<MonthChanged>,
    game_date: Res<GameDate>,
    women: Query<(Entity, &mut Citizen), (With<Female>, Without<Pregnancy>, Without<Ovulation>)>,
) {
    for _ in event_reader.read() {
        for (entity, _) in &women {
            let ovulation_start_date =
                game_date.date + chrono::Duration::days(thread_rng().gen_range(5..=20) as i64);

            commands.get_entity(entity).map(|mut e| {
                e.try_insert(Ovulation {
                    ovulation_start_date,
                });
            });
        }
    }
}

pub fn end_ovulation(
    mut commands: Commands,
    mut event_reader: EventReader<DateChanged>,
    game_date: Res<GameDate>,
    women: Query<(Entity, &Citizen, &Ovulation)>,
) {
    for _ in event_reader.read() {
        for (entity, _, ovulation) in &women {
            if ovulation.ovulation_start_date
                + chrono::Duration::days(thread_rng().gen_range(5..=6))
                == game_date.date
            {
                commands.get_entity(entity).map(|mut e| {
                    e.remove::<Ovulation>();
                });
            }
        }
    }
}

pub fn init_pregnancies(
    mut commands: Commands,
    game_date: Res<GameDate>,
    mut event_reader: EventReader<DateChanged>,
    mut citizens: Query<
        (Entity, &mut Citizen),
        (
            With<Ovulation>,
            With<Female>,
            With<Spouse>,
            Without<Pregnancy>,
        ),
    >,
) {
    for _ in event_reader.read() {
        for (w_entity, w_citizen) in &mut citizens {
            if pregnancy_desire() {
                if pregnancy_chance(game_date.date.years_since(w_citizen.birthday).unwrap() as u8) {
                    let pregnancy_term = thread_rng().gen_range(270..=280);
                    commands.get_entity(w_entity).map(|mut e| {
                        e.try_insert(Pregnancy {
                            baby_due_date: game_date
                                .date
                                .checked_add_signed(chrono::Duration::days(pregnancy_term))
                                .unwrap(),
                        });
                    });
                }
            }
        }
    }
}

pub fn pregnancy_chance(age: u8) -> bool {
    let age_f32 = age as f32;
    let pregnancy_chance = -0.0005893368566 * age_f32.powf(4.0)
        + 0.0730945581099 * age_f32.powf(3.0)
        - 3.3813849411076 * age_f32.powf(2.0)
        + 66.904528373158 * age_f32
        - 390.6749280259455;
    roll_chance(pregnancy_chance as u8)
}

pub fn pregnancy_desire() -> bool {
    let economy: f32 = thread_rng().gen_range(0.0..=1.0);
    let urbanization: f32 = thread_rng().gen_range(0.0..=1.0);
    let demand: f32 = thread_rng().gen_range(0.0..=1.0);
    let survivability: f32 = thread_rng().gen_range(0.0..=1.0);

    let mut preg_chance =
        2.1 * urbanization * (economy / economy) * demand * (1.0 - urbanization) * survivability;

    preg_chance = preg_chance * 100.0;

    roll_chance(preg_chance as u8)
}


