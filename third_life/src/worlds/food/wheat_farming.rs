use bevy::{prelude::*, utils::HashMap};
use chrono::{Datelike, NaiveDate};

use crate::{time::{DateChanged, GameDate}, worlds::population::components::{CitizenOf, Employed}};

use super::{
    CarbCreated, CarbResource, ResourceOf, WheatFarm, WheatFarmNeedsWorker, WheatFarmOf,
    WheatFarmer,
};

pub fn season_check_wheat(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut wheat_farms: Query<&mut WheatFarm>,
    game_date: Res<GameDate>
) {
    for _ in day_changed_event_reader.read() {
        if game_date.date.month() == 6 && game_date.date.day() == 1 {
            warn!("Harvest season has begun {:?}", game_date.date);
            for mut wheat_farm in wheat_farms.iter_mut() {
                wheat_farm.harvested = 0.0;
            }
        }
    }
}

pub fn check_farm_workers(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut event_writer: EventWriter<WheatFarmNeedsWorker>,
    wheat_farms: Query<(Entity, &WheatFarmOf), With<WheatFarm>>,
    farmers: Query<(&WheatFarmer, &CitizenOf)>,
) {
    for _ in day_changed_event_reader.read() {
        let mut farms_map = wheat_farms.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, HashMap<Entity, usize>>, (farm_entity, wheat_farm_of)| {
                acc.entry(wheat_farm_of.colony)
                    .or_insert(HashMap::new())
                    .entry(farm_entity)
                    .or_insert(0);
                acc
            },
        );

        for (wheat_farmer, colony_of) in farmers.iter() {
            farms_map
                .get_mut(&colony_of.colony)
                .unwrap()
                .entry(wheat_farmer.farm)
                .and_modify(|count| *count += 1);
        }

        for (colony, farms) in farms_map {
            for (farm, farmer_count) in farms {
                if farmer_count < 4 {
                    for _ in 0..(4 - farmer_count) {
                        event_writer.send(WheatFarmNeedsWorker { colony, farm });
                    }
                }
            }
        }
    }
}

pub fn get_farm_workers(
    mut commands: Commands,
    mut event_reader: EventReader<WheatFarmNeedsWorker>,
    free_citizens: Query<(Entity, &CitizenOf), Without<Employed>>,
) {
    for needs_worker_event in event_reader.read() {
        for (citizen, citizen_of) in free_citizens.iter() {
            if citizen_of.colony == needs_worker_event.colony {
                commands.get_entity(citizen).map(|mut c| {
                    c.try_insert((
                        WheatFarmer {
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

pub fn work_farm(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut wheat_farms: Query<(Entity, &mut WheatFarm, &WheatFarmOf)>,
    farmers: Query<(&WheatFarmer, &CitizenOf)>,
    mut carb_resources: Query<(&mut CarbResource, &ResourceOf)>,
    mut carb_created: EventWriter<CarbCreated>,
) {
    for _ in day_changed_event_reader.read() {
        let mut farms_map = wheat_farms.iter_mut().fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, HashMap<Entity, usize>>, (farm_entity, _, wheat_farm_of)| {
                acc.entry(wheat_farm_of.colony)
                    .or_insert(HashMap::new())
                    .entry(farm_entity)
                    .or_insert(0);
                acc
            },
        );

        for (wheat_farmer, colony_of) in farmers.iter() {
            farms_map
                .get_mut(&colony_of.colony)
                .unwrap()
                .entry(wheat_farmer.farm)
                .and_modify(|count| *count += 1);
        }

        for (colony, farms) in farms_map {
            for (farm_entity, farmer_count) in farms {
                let (_, mut wheat_farm, _) = wheat_farms.get_mut(farm_entity).unwrap();
                // 1.0 signifies multiplier for 1 8 hour work day
                // harvested_amount is in ha
                let mut harvested_amount = 1.0 * (farmer_count as f32);
                if harvested_amount > wheat_farm.size - wheat_farm.harvested {
                    harvested_amount = wheat_farm.size - wheat_farm.harvested;
                }
                wheat_farm.harvested += harvested_amount;
                if harvested_amount > 0.0 {
                    for (mut carb_resource, resource_of) in carb_resources.iter_mut() {
                        if resource_of.colony == colony {
                            let amount = harvested_amount * 2670.0;
                            carb_resource.amount += amount;
                            carb_created.send(CarbCreated { colony, amount });
                        }
                    }
                }
            }
        }
    }
}
