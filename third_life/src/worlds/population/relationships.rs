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

use super::{components::*, events::CitizenDied};

pub struct RelationshipsPlugin;

impl Plugin for RelationshipsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (
                    init_couples,
                    create_widowed
                ).run_if(in_state(SimulationState::Running))
            );
    }
}

pub fn init_couples(
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
                        commands.get_entity(woman_entity).map(|mut e| {
                            e.try_insert(Spouse { spouse: man_entity });
                        });
                        commands.get_entity(man_entity).map(|mut e| {
                            e.try_insert(Spouse { spouse: woman_entity });
                        });
                    }
                }
            }
        }
    }
}

pub fn create_widowed(
    mut commands: Commands,
    mut deaths: EventReader<CitizenDied>,
    spouses: Query<(Entity, &Spouse)>
) {
    let dead_people = deaths.read().map(|e|e.citizen).collect::<Vec<_>>();
    for (entity, Spouse { spouse }) in spouses.iter() {
        if dead_people.contains(spouse) {
            let _ = commands.get_entity(entity)
                .map(|mut e| {
                    e.remove::<Spouse>();
                    e.try_insert(Widowed);
                });
        }
    }
}


