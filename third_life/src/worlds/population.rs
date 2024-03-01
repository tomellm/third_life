use crate::{
    common::utils::roll_chance,
    time::{DateChanged, GameDate, MonthChanged},
    SimulationState,
};
use bevy::prelude::*;
use bevy_egui::{egui::Window, EguiContexts};
use chrono::{Datelike, NaiveDate};
use rand::{thread_rng, Rng};
use rand_distr::{num_traits::Float, Distribution, SkewNormal};
use rnglib::{Language, RNG};

use super::{init_colonies, WorldColony, config::WorldsConfig, WorldEntity};

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
                init_ovulation,
                end_ovulation,
                init_miscarriage,
                init_pregnancies,
                citizen_births,
                update_population,
                //population_info_windows,
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
    //TODO: Optimize this and yell at thomas for not letting me earlier.
    pub days_since_meal: usize,
}

#[derive(Component)]
pub struct CitizenOf {
    pub colony: Entity,
}

#[derive(Component)]
struct Female;

#[derive(Component)]
struct Male;

#[derive(Component)]
struct Ovulation {
    pub ovulation_start_date: NaiveDate,
}

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
                            days_since_meal: 0
                        };

                        match roll_chance(50) {
                            true => commands.spawn((new_born, CitizenOf { colony }, Male)),
                            false => commands.spawn((new_born, CitizenOf { colony }, Female)),
                        };

                        event_writer.send(CitizenCreated { age: 0, colony });
                    }
                }
                commands.entity(entity).remove::<Pregnancy>();
            }
        }
    }
}

fn init_miscarriage(
    mut commands: Commands,
    mut event_reader: EventReader<MonthChanged>,
    mut pregnant_women: Query<(Entity, &Citizen), With<Pregnancy>>,
    game_date: Res<GameDate>,
) {
    for _ in event_reader.read() {
        for (entity, w_citizen) in &mut pregnant_women {
            if miscarriage_chance(game_date.date.years_since(w_citizen.birthday).unwrap() as u8) {
                commands.entity(entity).remove::<Pregnancy>();
            }
        }
    }
}

fn miscarriage_chance(age: u8) -> bool {
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

fn init_ovulation(
    mut commands: Commands,
    mut event_reader: EventReader<MonthChanged>,
    game_date: Res<GameDate>,
    women: Query<(Entity, &mut Citizen), (With<Female>, Without<Pregnancy>, Without<Ovulation>)>,
) {
    for _ in event_reader.read() {
        for (entity, _) in &women {
            let ovulation_start_date =
                game_date.date + chrono::Duration::days(thread_rng().gen_range(5..=20) as i64);

            commands.entity(entity).insert(Ovulation {
                ovulation_start_date: ovulation_start_date,
            });
        }
    }
}

fn end_ovulation(
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
                commands.entity(entity).remove::<Ovulation>();
            }
        }
    }
}

fn init_pregnancies(
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
                    commands.entity(w_entity).insert(Pregnancy {
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

fn pregnancy_chance(age: u8) -> bool {
    let age_f32 = age as f32;
    let pregnancy_chance = -0.0005893368566 * age_f32.powf(4.0)
        + 0.0730945581099 * age_f32.powf(3.0)
        - 3.3813849411076 * age_f32.powf(2.0)
        + 66.904528373158 * age_f32
        - 390.6749280259455;
    roll_chance(pregnancy_chance as u8)
}

fn pregnancy_desire() -> bool {
    let economy: f32 = thread_rng().gen_range(0.0..=1.0);
    let urbanization: f32 = thread_rng().gen_range(0.0..=1.0);
    let demand: f32 = thread_rng().gen_range(0.0..=1.0);
    let survivability: f32 = thread_rng().gen_range(0.0..=1.0);

    let mut preg_chance =
        2.1 * urbanization * (economy / economy) * demand * (1.0 - urbanization) * survivability;

    preg_chance = preg_chance * 100.0;

    roll_chance(preg_chance as u8)
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

fn init_citizens(
    colonies: Query<(Entity, &WorldEntity), With<WorldColony>>,
    mut commands: Commands,
    mut event_writer: EventWriter<CitizenCreated>,
    game_date: Res<GameDate>,
    worlds_config: Res<WorldsConfig>
) {
    for (colony, WorldEntity { name }) in colonies.iter() {
        let pop_config = worlds_config.worlds().iter().filter(|e| e.name().eq(name)).collect::<Vec<_>>().first().unwrap().population();
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
                birthday,
                days_since_meal: 0
            };
            match roll_chance(50) {
                true => commands.spawn((citizen, CitizenOf { colony }, Male)),
                false => commands.spawn((citizen, CitizenOf { colony }, Female)),
            };

            event_writer.send(CitizenCreated { age, colony });
        }

        commands.entity(colony).insert(Population::default());
    }
}

#[derive(Event)]
pub struct CitizenCreated {
    pub age: usize,
    pub colony: Entity,
}

fn update_population(
    mut event_reader: EventReader<CitizenCreated>,
    mut populations: Query<(Entity, &mut Population)>,
    citizens: Query<(&Citizen, &CitizenOf)>,
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

                population.average_age =
                    all_citizen_ages.iter().sum::<usize>() / all_citizen_ages.len();
            }
        }
    }
}
