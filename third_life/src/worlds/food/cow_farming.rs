use std::usize;

use bevy::{prelude::*, reflect::List, transform::commands, utils::hashbrown::HashMap};
use chrono::{Datelike, NaiveDate};
use rand_distr::num_traits::Float;

use crate::{
    common::utils::roll_chance,
    time::{DateChanged, GameDate},
    worlds::population::components::{CitizenOf, Employed, Retiree, Youngling},
};

use super::{
    Cow, CowFarm, CowFarmNeedsWorker, CowFarmOf, CowFarmer, CowOf, IsBreeder, IsBull,
    MeatResource, ResourceOf,
};

pub fn mark_breeders(
    mut commands: Commands,
    cow_farms: Query<Entity, With<CowFarm>>,
    breeding_bulls: Query<(Entity, &CowOf), With<IsBreeder>>,
    bulls: Query<(Entity, &CowOf), (With<IsBull>, Without<IsBreeder>)>,
) {
    let mut farms_map = cow_farms.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, HashMap<&str, Vec<Entity>>>, farm_entity| {
            acc.entry(farm_entity)
                .or_insert(HashMap::new())
                .entry("breeders")
                .or_insert(Vec::new());

            acc.entry(farm_entity)
                .or_insert(HashMap::new())
                .entry("bulls")
                .or_insert(Vec::new());
            acc
        },
    );

    for (cow_entity, cow_of) in breeding_bulls.iter() {
        farms_map
            .get_mut(&cow_of.cow_farm)
            .unwrap()
            .get_mut("breeders")
            .unwrap()
            .push(cow_entity);
    }
    for (cow_entity, cow_of) in bulls.iter() {
        farms_map
            .get_mut(&cow_of.cow_farm)
            .unwrap()
            .get_mut("bulls")
            .unwrap()
            .push(cow_entity);
    }

    for (_, animals) in farms_map {
        if animals.get("breeders").unwrap().len() < 2 {
            if animals.get("bulls").unwrap().len() > 0 {
                commands
                    .get_entity(animals.get("bulls").unwrap()[0])
                    .map(|mut b| {
                        b.try_insert(IsBreeder);
                    });
            }
        }
    }
}

pub fn breed_cows(
    mut commands: Commands,
    mut day_changed_event_reader: EventReader<DateChanged>,
    cows: Query<(&Cow, &CowOf), Without<IsBull>>,
) {
    for day in day_changed_event_reader.read() {
        if day.date.month() == 6 && day.date.day() == 1 {
            let mut cows_to_spawn: Vec<_> = Vec::new();
            let mut bulls_to_spawn: Vec<_> = Vec::new();
            for (_, cow_of) in cows.iter() {
                match roll_chance(50) {
                    true => cows_to_spawn.push((
                        Cow { birthday: day.date },
                        CowOf {
                            cow_farm: cow_of.cow_farm,
                        },
                    )),
                    false => bulls_to_spawn.push((
                        Cow { birthday: day.date },
                        CowOf {
                            cow_farm: cow_of.cow_farm,
                        },
                        IsBull,
                    )),
                }
            }
            commands.spawn_batch(cows_to_spawn);
            commands.spawn_batch(bulls_to_spawn);
        }
    }
}

pub fn check_cow_farm_workers(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut event_writer: EventWriter<CowFarmNeedsWorker>,
    cow_farms: Query<(Entity, &CowFarmOf), With<CowFarm>>,
    farmers: Query<(&CowFarmer, &CitizenOf)>,
) {
    for _ in day_changed_event_reader.read() {
        let mut farms_map = cow_farms.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, HashMap<Entity, usize>>, (farm_entity, cow_farm_of)| {
                acc.entry(cow_farm_of.colony)
                    .or_insert(HashMap::new())
                    .entry(farm_entity)
                    .or_insert(0);
                acc
            },
        );

        for (cow_farmer, colony_of) in farmers.iter() {
            farms_map
                .get_mut(&colony_of.colony)
                .unwrap()
                .entry(cow_farmer.farm)
                .and_modify(|count| *count += 1);
        }

        for (colony, farms) in farms_map {
            for (farm, farmer_count) in farms {
                if farmer_count < 4 {
                    for _ in 0..(4 - farmer_count) {
                        event_writer.send(CowFarmNeedsWorker { colony, farm });
                    }
                }
            }
        }
    }
}

pub fn get_cow_farm_workers(
    mut commands: Commands,
    mut event_reader: EventReader<CowFarmNeedsWorker>,
    free_citizens: Query<(Entity, &CitizenOf), (Without<Employed>, Without<Youngling>, Without<Retiree>)>,
) {
    for needs_worker_event in event_reader.read() {
        for (citizen, citizen_of) in free_citizens.iter() {
            if citizen_of.colony == needs_worker_event.colony {
                commands.get_entity(citizen).map(|mut c| {
                    c.try_insert((
                        CowFarmer {
                            farm: needs_worker_event.farm,
                        },
                        Employed,
                    ));
                });
                break;
            }
        }
    }
}

pub fn work_cow_farm(
    mut commands: Commands,
    game_date: Res<GameDate>,
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut cow_farms: Query<(Entity, &mut CowFarm, &CowFarmOf)>,
    cows: Query<(Entity, &Cow, &CowOf)>,
    bulls: Query<(Entity, &Cow, &CowOf), (With<IsBull>, Without<IsBreeder>)>,
    farmers: Query<(&CowFarmer, &CitizenOf)>,
    mut meat_resources: Query<(&mut MeatResource, &ResourceOf)>,
) {
    for _ in day_changed_event_reader.read() {
        let mut farms_map = cow_farms.iter_mut().fold(
            HashMap::new(),
            |mut acc: HashMap<
                Entity,
                HashMap<Entity, Vec<(Entity, &Cow)>>,
            >,
             (farm_entity, _, cow_farm_of)| {
                acc.entry(cow_farm_of.colony)
                    .or_insert(HashMap::new())
                    .entry(farm_entity)
                    .or_insert(Vec::new());
                acc
            },
        );
        
        for (_, farms) in farms_map.iter_mut() {
            for (bull_entity, cow, cow_of) in bulls.iter() {
                farms
                    .entry(cow_of.cow_farm)
                    .and_modify(|f| f.push((bull_entity, cow)));
            }
        }
        for (_, farms) in farms_map.iter_mut() {
            for (cow_entity, cow, cow_of) in cows.iter() {
                farms
                    .entry(cow_of.cow_farm)
                    .and_modify(|f| f.push((cow_entity, cow)));
            }
        }

        let farmers_map = farmers.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, usize>, (cow_farmer, _)| {
                *acc.entry(cow_farmer.farm).or_insert(0) += 1;
                acc
            },
        );

        for (colony, farms) in farms_map {
            let mut meat_harvested = 0;
            for (farm_entity, cows) in farms {
                let cows_count = cows.len();
                //47 is the min amount of cows we want to keep
                let min_to_keep = 47;
                if cows_count <= min_to_keep {
                    continue;
                }
                let mut to_harvest = cows_count - 47;
                if to_harvest as f32
                    > (farmers_map.get(&farm_entity).unwrap_or(&0) * 8) as f32 / 6.25
                {
                    to_harvest = ((farmers_map.get(&farm_entity).unwrap_or(&0) * 8) as f32 / 6.25)
                        .floor() as usize;
                }
                

                for cow in cows {
                    if to_harvest <= 0 {
                        break;
                    }
                    let months = get_age_in_months(game_date.date, cow.1.birthday);
                    //TODO: maybe better implement this to kill specific cows by age etc
                    if months > 18 {
                        commands.get_entity(cow.0).map(|mut e| e.despawn());
                        meat_harvested += 250;
                        to_harvest -= 1;
                    }
                }
            }

            for (mut meat_resource, resource_of) in meat_resources.iter_mut() {
                if resource_of.colony == colony {
                    meat_resource.amount += meat_harvested as f32;
                }
            }
        }
    }
}

fn get_age_in_months(game_date: NaiveDate, birthday: NaiveDate) -> i32 {
    let years = game_date.year() as i32 - birthday.year() as i32;
    let mut months = game_date.month() as i32 - birthday.month() as i32;
    months += years * 12;

    months
}
