use crate::{
    common::utils::percentage_chance,
    time::{DateChanged, GameDate},
    SimulationState,
};
use bevy::prelude::*;
use bevy_egui::{egui::Window, EguiContexts};
use chrono::{Datelike, NaiveDate};
use rand::{thread_rng, Rng};
use rand_distr::{num_traits::Float, Distribution, SkewNormal};
use rnglib::{Language, RNG};

use super::{init_colonies, WorldColony};

pub struct PopulationPlugin;

impl Plugin for PopulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            (init_citizens).chain().after(init_colonies),
        )
        .add_systems(
            Update,
            (
                init_couples,
                init_pregnancies,
                citizen_births,
                update_population,
                population_info_windows,
            )
                .run_if(in_state(SimulationState::Running)),
        )
        .add_event::<CitizenCreated>();
    }
}

#[derive(Component, Default)]
pub struct Population {
    pub count: usize,
    pub average_age: usize,
}

#[derive(Component, PartialEq, Clone)]
pub struct Citizen {
    pub name: String,
    pub birthday: NaiveDate,
}

#[derive(Component)]
struct Female;

#[derive(Component)]
struct Male;

#[derive(Component)]
pub struct Pregnancy {
    pub baby_due_date: NaiveDate,
}

#[derive(Component)]
pub struct Spouse {
    pub spouse: Entity,
}

fn population_info_windows(mut contexts: EguiContexts, populations: Query<&Population>) {
    for population in &populations {
        Window::new(format!("Population {}", population.count)).show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Population: {}", population.count));
            ui.label(format!("Average Age: {}", population.average_age));
        });
    }
}

fn citizen_births(
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
                            birthday: game_date.date,
                        };

                        match percentage_chance(50) {
                            true => commands.spawn((new_born, CitizenOf { colony }, Male)),
                            false => commands.spawn((new_born, CitizenOf { colony }, Female)),
                        };

                        event_writer.send(CitizenCreated { colony: colony });
                    }
                }
                commands.entity(entity).remove::<Pregnancy>();
            }
        }
    }
}

fn init_pregnancies(
    mut commands: Commands,
    game_date: Res<GameDate>,
    mut event_reader: EventReader<DateChanged>,
    mut citizens: Query<(Entity, &mut Citizen), (With<Female>, With<Spouse>)>,
) {
    for _ in event_reader.read() {
        for citizen in &mut citizens {
            if game_date.date.years_since(citizen.1.birthday).unwrap() >= 18
                && game_date.date.years_since(citizen.1.birthday).unwrap() <= 45
            {
                let pregnancy_chance = percentage_chance(42);
                let pregnancy_term = thread_rng().gen_range(270..=280);

                if pregnancy_chance {
                    commands.entity(citizen.0).insert(Pregnancy {
                        baby_due_date: game_date
                            .date
                            .checked_add_signed(chrono::Duration::days(pregnancy_term))
                            .unwrap(),
                    });
                }
            }
        }
    }
}

fn init_couples(
    mut commands: Commands,
    mut event_reader: EventReader<DateChanged>,
    game_date: Res<GameDate>,
    colonies: Query<Entity, With<WorldColony>>,
    men: Query<(Entity, &Citizen, &CitizenOf), (With<Male>, Without<Spouse>)>,
    women: Query<
        (Entity, &Citizen, &CitizenOf),
        (With<Female>, Without<Spouse>, Without<Pregnancy>),
    >,
) {
    for _ in event_reader.read() {
        for colony in &colonies {
            let mut colony_available_men: Vec<Entity> = men
                .iter()
                .filter_map(|(entity, _, m_citizen_of)| {
                    if m_citizen_of.colony == colony {
                        Some(entity)
                    } else {
                        None
                    }
                })
                .collect();
            let colony_available_women = women
                .iter()
                .filter_map(|(entity, w_citizen, w_citizen_of)| {
                    if w_citizen_of.colony == colony {
                        Some((entity, w_citizen.birthday))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            for (woman_entity, w_birthday) in colony_available_women {
                if game_date.date.years_since(w_birthday).unwrap() > 18 {
                    if let Some(man_entity) = colony_available_men.pop() {
                        commands
                            .entity(woman_entity)
                            .insert(Spouse { spouse: man_entity });
                        commands.entity(man_entity).insert(Spouse {
                            spouse: woman_entity,
                        });
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct CitizenOf {
    pub colony: Entity,
}

fn init_citizens(
    colonies: Query<Entity, With<WorldColony>>,
    mut commands: Commands,
    mut event_writer: EventWriter<CitizenCreated>,
    game_date: Res<GameDate>,
) {
    for colony in &colonies {
        let mut rng = thread_rng();
        let name_rng = RNG::try_from(&Language::Roman).unwrap();
        let skew_normal = SkewNormal::new(18.0, 6.0, 10.0).unwrap();
        let mut age_gen = skew_normal.sample_iter(&mut rng);
        let year = game_date.date.year_ce().1 as u32;

        for _ in 0..1000 {
            let age = age_gen.next().unwrap().floor() as u32;
            let birthday = NaiveDate::from_yo_opt(
                (year - age).try_into().unwrap(),
                thread_rng().gen_range(1..=365),
            )
            .unwrap();

            let citizen = Citizen {
                name: name_rng.generate_name(),
                birthday: birthday,
            };
            match percentage_chance(50) {
                true => commands.spawn((citizen, CitizenOf { colony }, Male)),
                false => commands.spawn((citizen, CitizenOf { colony }, Female)),
            };

            event_writer.send(CitizenCreated { colony: colony });
        }

        commands.entity(colony).insert(Population::default());
    }
}

#[derive(Event)]
pub struct CitizenCreated {
    pub colony: Entity,
}

fn update_population(
    mut event_reader: EventReader<CitizenCreated>,
    mut populations: Query<(&mut Population, &Parent)>,
    citizens: Query<(&Citizen, &CitizenOf)>,
    game_date: Res<GameDate>,
) {
    for event in event_reader.read() {
        for (mut population, colony) in &mut populations.iter_mut() {
            if colony.get() == event.colony {
                population.count += 1;

                let all_citizen_ages: Vec<usize> = citizens
                    .iter()
                    .filter_map(|(citizen, citizen_of)| {
                        if citizen_of.colony == colony.get() {
                            Some(game_date.date.years_since(citizen.birthday).unwrap() as usize)
                        } else {
                            None
                        }
                    })
                    .collect();

                population.average_age =
                    all_citizen_ages.iter().sum::<usize>() / all_citizen_ages.len();
            }
        }
    }
}
