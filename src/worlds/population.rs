use crate::time::{DateChanged, GameDate};
use bevy::prelude::*;
use bevy_egui::{egui::Window, EguiContexts};
use chrono::{Datelike, NaiveDate};
use rand::{thread_rng, Rng};
use rand_distr::{num_traits::Float, Distribution, SkewNormal};
use rnglib::{Language, RNG};

use super::{init_colonies, WorldColony};
use crate::common::utils::percentage_chance;

pub struct PopulationPlugin;
impl Plugin for PopulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (init_population, init_citizens)
                .chain()
                .after(init_colonies),
        )
        .add_systems(
            Update,
            (
                update_population,
                age_citizens,
                init_couples,
                init_pregnancies,
                citizen_births,
            ),
        )
        .add_systems(Update, population_info_windows)
        .add_event::<CitizenCreated>();
    }
}

#[derive(Component)]
pub struct Population {
    pub count: usize,
    pub average_age: usize,
}

#[derive(Component, PartialEq, Clone)]
pub struct Citizen {
    pub name: String,
    pub age: usize,
    pub birthday: NaiveDate,
}

#[derive(Component)]
struct Female;

#[derive(Component)]
struct Male;

#[derive(Component)]
pub struct Reproduction {
    pub baby_due_date: NaiveDate,
}

#[derive(Component)]
pub struct Spouse {
    pub spouse: Entity,
}

fn age_citizens(
    mut citizens: Query<&mut Citizen>,
    game_date: Res<GameDate>,
    mut event_reader: EventReader<DateChanged>,
) {
    for _ in event_reader.read() {
        for mut citizen in &mut citizens {
            if citizen.birthday.month() == game_date.date.month() && citizen.birthday.day() == game_date.date.day() {
                citizen.age += 1;
            }
        }
    }
}

fn population_info_windows(mut contexts: EguiContexts, populations: Query<&Population>) {
    for population in &populations {
        Window::new("Population Info").show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Population: {}", population.count));
            ui.label(format!("Average Age: {}", population.average_age));
        });
    }
}

fn citizen_births(
    mut commands: Commands,
    mut event_reader: EventReader<DateChanged>,
    mut event_writer: EventWriter<CitizenCreated>,
    mut pregnant_women: Query<
        (Entity, &mut Citizen, &mut Reproduction, &Parent),
        With<Reproduction>,
    >,
    game_date: Res<GameDate>,
) {
    for _ in event_reader.read() {
        println!("GameDate: {} ", game_date.date);
        for (entity, citizen, reproduction, parent) in &mut pregnant_women.iter_mut() {
            if reproduction.baby_due_date == game_date.date {
                println!("{} has given birth", citizen.name);
                commands.entity(parent.get()).with_children(|commands| {
                    let name_rng = RNG::try_from(&Language::Roman).unwrap();

                    let new_born = Citizen {
                        name: name_rng.generate_name(),
                        age: 0,
                        birthday: game_date.date,
                    };
                    event_writer.send(CitizenCreated {
                        citizen: new_born.clone(),
                        population: parent.get(),
                    });

                    let mut entity_commands = commands.spawn(new_born);

                    match percentage_chance(50) {
                        true => entity_commands.insert(Male),
                        false => entity_commands.insert(Female),
                    };
                });

                commands.entity(entity).remove::<Reproduction>();
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
            if citizen.1.age >= 18 && citizen.1.age <= 45 {
                let pregnancy_chance = percentage_chance(42);
                if pregnancy_chance {
                    commands.entity(citizen.0).insert(Reproduction {
                        baby_due_date: game_date
                            .date
                            .checked_add_signed(chrono::Duration::days(274))
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
    men: Query<(Entity, &Citizen), (With<Male>, Without<Spouse>)>,
    women: Query<(Entity, &Citizen), (With<Female>, Without<Spouse>, Without<Reproduction>)>,
) {
    let mut available_men: Vec<Entity> = men.iter().map(|(entity, _)| entity).collect();
    for _ in event_reader.read() {
        for (woman_entity, w_citizen) in women.iter() {
            if w_citizen.age > 18 {
                if let Some(man_entity) = available_men.pop() {
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

fn init_population(colonies: Query<Entity, With<WorldColony>>, mut commands: Commands) {
    for colony in &colonies {
        commands.entity(colony).with_children(|commands| {
            commands.spawn(Population {
                count: 0,
                average_age: 0,
            });
        });
    }
}

fn init_citizens(
    populations: Query<Entity, With<Population>>,
    mut commands: Commands,
    mut event_writer: EventWriter<CitizenCreated>,
    game_date: Res<GameDate>,
) {
    for population in &populations {
        commands.entity(population).with_children(|parent| {
            let mut rng = thread_rng();
            let name_rng = RNG::try_from(&Language::Roman).unwrap();
            let skew_normal = SkewNormal::new(18.0, 6.0, 10.0).unwrap();
            let mut age_gen = skew_normal.sample_iter(&mut rng);
            let year = game_date.date.year_ce().1 as u32;

            for _ in 0..1000 {
                let mut rng = thread_rng();
                let birthday = NaiveDate::from_yo_opt(
                    year.try_into().unwrap(),
                     rng.gen_range(1..=365))
                     .unwrap();
                let citizen = Citizen {
                    name: name_rng.generate_name(),
                    age: age_gen.next().unwrap().floor() as usize,
                    birthday: birthday,
                };
                event_writer.send(CitizenCreated {
                    population: population,
                    citizen: citizen.clone(),
                });

                let mut entity_commands = parent.spawn(citizen);

                match percentage_chance(50) {
                    true => entity_commands.insert(Male),
                    false => entity_commands.insert(Female),
                };
            }
        });
    }
}

#[derive(Event)]
pub struct CitizenCreated {
    population: Entity,
    citizen: Citizen,
}

fn update_population(
    mut event_reader: EventReader<CitizenCreated>,
    mut populations: Query<&mut Population>,
) {
    for event in event_reader.read() {
        let mut population = populations.get_mut(event.population).unwrap();
        population.count += 1;
        population.average_age = population.average_age + event.citizen.age / 2;
    }
}
