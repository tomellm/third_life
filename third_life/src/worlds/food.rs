use std::usize;

use crate::{time::DateChanged, SimulationState};
use bevy::{prelude::*, reflect::List, transform::commands, utils::HashMap};
use bevy_egui::{
    egui::{Color32, Ui, Window},
    EguiContexts,
};

use super::{
    init_colonies,
    population::{self, Citizen, CitizenOf, Population},
    WorldColony,
};

pub struct FoodPlugin;
impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            init_food.after(init_colonies),
        )
        .add_systems(
            Update,
            (
                check_farm_workers,
                get_farm_workers,
                work_farm,
                check_cow_farm_workers,
                get_cow_farm_workers,
                work_cow_farm,
                cook_food,
                //display_food,
            )
                .run_if(in_state(SimulationState::Running)),
        )
        .add_event::<WheatFarmNeedsWorker>()
        .add_event::<CowFarmNeedsWorker>()
        .add_event::<MeatCreated>()
        .add_event::<CarbCreated>()
        .add_event::<MeatConsumed>()
        .add_event::<CarbConsumed>()
        .add_event::<FoodCreated>();
    }
}

#[derive(Component)]
pub struct Employed;

#[derive(Component, PartialEq, Eq, Hash)]
pub struct ResourceOf {
    pub colony: Entity,
}

#[derive(Component)]
pub struct FoodResource {
    pub amount: f32,
}

#[derive(Component)]
pub struct CarbResource {
    pub amount: f32,
}

#[derive(Component)]
pub struct WheatFarm {
    size: f32,
    harvested: f32,
}

#[derive(Component)]
pub struct WheatFarmOf {
    colony: Entity,
}

#[derive(Component)]
pub struct WheatFarmer {
    farm: Entity,
}

#[derive(Component)]
pub struct MeatResource {
    pub amount: f32,
}

#[derive(Component)]
pub struct CowFarm {
    size: f32,
    harvested: f32,
}

#[derive(Component)]
pub struct CowFarmOf {
    colony: Entity,
}

#[derive(Component)]
pub struct CowFarmer {
    farm: Entity,
}
fn init_food(mut commands: Commands, colonies: Query<Entity, With<WorldColony>>) {
    for colony in colonies.iter() {
        commands.spawn((
            WheatFarm {
                size: 17.4,
                harvested: 0.0,
            },
            WheatFarmOf { colony },
        ));
        commands.spawn((
            WheatFarm {
                size: 17.4,
                harvested: 0.0,
            },
            WheatFarmOf { colony },
        ));
        commands.spawn((
            WheatFarm {
                size: 17.4,
                harvested: 0.0,
            },
            WheatFarmOf { colony },
        ));

        commands.spawn((
            WheatFarm {
                size: 17.4,
                harvested: 0.0,
            },
            WheatFarmOf { colony },
        ));
        commands.spawn((
            WheatFarm {
                size: 17.4,
                harvested: 0.0,
            },
            WheatFarmOf { colony },
        ));
        commands.spawn((
            WheatFarm {
                size: 17.4,
                harvested: 0.0,
            },
            WheatFarmOf { colony },
        ));
        commands.spawn((
            CowFarm {
                size: 500.0,
                harvested: 0.0,
            },
            CowFarmOf { colony },
        ));
        commands.spawn((
            CowFarm {
                size: 500.0,
                harvested: 0.0,
            },
            CowFarmOf { colony },
        ));
        commands.spawn((
            CowFarm {
                size: 500.0,
                harvested: 0.0,
            },
            CowFarmOf { colony },
        ));
        commands.spawn((FoodResource { amount: 0.0 }, ResourceOf { colony }));
        commands.spawn((CarbResource { amount: 0.0 }, ResourceOf { colony }));
        commands.spawn((MeatResource { amount: 0.0 }, ResourceOf { colony }));
    }
}

fn display_food(
    mut contexts: EguiContexts,
    colonies: Query<Entity, With<WorldColony>>,
    food_resources: Query<(&FoodResource, &ResourceOf)>,
    carb_resources: Query<(&CarbResource, &ResourceOf)>,
    meat_resources: Query<(&MeatResource, &ResourceOf)>,
) {
    for (carb_resource, resource_of) in carb_resources.iter() {
        Window::new(format!("Carb Resources of {:?}", resource_of.colony)).show(
            contexts.ctx_mut(),
            |ui| {
                ui.label(format!("Carbs: {:?}kg", carb_resource.amount));
            },
        );
    }
    for (meat_resource, resource_of) in meat_resources.iter() {
        Window::new(format!("Meat Resources of {:?}", resource_of.colony)).show(
            contexts.ctx_mut(),
            |ui| {
                ui.label(format!("Meat: {:?}kg", meat_resource.amount));
            },
        );
    }
    for (food_resource, resource_of) in food_resources.iter() {
        Window::new(format!("Food Resources of {:?}", resource_of.colony)).show(
            contexts.ctx_mut(),
            |ui| {
                ui.label(format!("Food: {:?}kg", food_resource.amount));
            },
        );
    }
}

#[derive(Event)]
pub struct WheatFarmNeedsWorker {
    pub colony: Entity,
    pub farm: Entity,
}

fn check_farm_workers(
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

fn get_farm_workers(
    mut commands: Commands,
    mut event_reader: EventReader<WheatFarmNeedsWorker>,
    free_citizens: Query<(Entity, &CitizenOf), Without<Employed>>,
) {
    for needs_worker_event in event_reader.read() {
        for (citizen, citizen_of) in free_citizens.iter() {
            if citizen_of.colony == needs_worker_event.colony {
                commands.entity(citizen).insert((
                    WheatFarmer {
                        farm: needs_worker_event.farm,
                    },
                    Employed,
                ));
                break;
            }
        }
    }
}

#[derive(Event)]
pub struct CarbCreated {
    pub colony: Entity,
    pub amount: f32
}

fn work_farm(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut wheat_farms: Query<(Entity, &mut WheatFarm, &WheatFarmOf)>,
    farmers: Query<(&WheatFarmer, &CitizenOf)>,
    mut carb_resources: Query<(&mut CarbResource, &ResourceOf)>,
    mut carb_created: EventWriter<CarbCreated>
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
                            carb_created.send(CarbCreated{ colony, amount });
                        }
                    }
                }
            }
        }
    }
}

#[derive(Event)]
pub struct CowFarmNeedsWorker {
    pub colony: Entity,
    pub farm: Entity,
}

fn check_cow_farm_workers(
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

fn get_cow_farm_workers(
    mut commands: Commands,
    mut event_reader: EventReader<CowFarmNeedsWorker>,
    free_citizens: Query<(Entity, &CitizenOf), Without<Employed>>,
) {
    for needs_worker_event in event_reader.read() {
        for (citizen, citizen_of) in free_citizens.iter() {
            if citizen_of.colony == needs_worker_event.colony {
                commands.entity(citizen).insert((
                    CowFarmer {
                        farm: needs_worker_event.farm,
                    },
                    Employed,
                ));
                break;
            }
        }
    }
}

#[derive(Event)]
pub struct MeatCreated {
    pub colony: Entity,
    pub amount: f32
}

fn work_cow_farm(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut cow_farms: Query<(Entity, &mut CowFarm, &CowFarmOf)>,
    farmers: Query<(&CowFarmer, &CitizenOf)>,
    mut meat_resources: Query<(&mut MeatResource, &ResourceOf)>,
    mut meat_created: EventWriter<MeatCreated>
) {
    for _ in day_changed_event_reader.read() {
        let mut farms_map = cow_farms.iter_mut().fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, HashMap<Entity, usize>>, (farm_entity, _, wheat_farm_of)| {
                acc.entry(wheat_farm_of.colony)
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
            for (farm_entity, farmer_count) in farms {
                let (_, mut cow_farm, _) = cow_farms.get_mut(farm_entity).unwrap();
                // 1.0 signifies multiplier for 1 8 hour work day
                // harvested_amount is in ha
                let mut harvested_amount = 1.0 * (farmer_count as f32);
                if harvested_amount > (cow_farm.size / 2.0) - cow_farm.harvested {
                    harvested_amount = (cow_farm.size / 2.0) - cow_farm.harvested;
                }
                cow_farm.harvested += harvested_amount;
                if harvested_amount > 0.0 {
                    for (mut meat_resource, resource_of) in meat_resources.iter_mut() {
                        if resource_of.colony == colony {
                            //todo: need to figure out 1 day of work= how many kilos meat.
                            let amount = harvested_amount * 2000.0;
                            meat_resource.amount += amount;
                            meat_created.send(MeatCreated { colony, amount });
                        }
                    }
                }
            }
        }
    }
}

#[derive(Event)]
pub struct MeatConsumed {
    pub colony: Entity,
    pub amount: f32
}

#[derive(Event)]
pub struct CarbConsumed {
    pub colony: Entity,
    pub amount: f32
}

#[derive(Event)]
pub struct FoodCreated {
    pub colony: Entity,
    pub amount: f32
}

fn cook_food(
    mut food_resources: Query<(Entity, &mut FoodResource, &ResourceOf)>,
    mut carb_resources: Query<(Entity, &mut CarbResource, &ResourceOf)>,
    mut meat_resources: Query<(Entity, &mut MeatResource, &ResourceOf)>,
    mut food_created: EventWriter<FoodCreated>,
    mut carb_consumed: EventWriter<CarbConsumed>,
    mut meat_consumed: EventWriter<MeatConsumed>,
) {
    let mut colony_food_resources_map = food_resources.iter_mut().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, HashMap<&str, Entity>>, (entity, _, colony)| {
            acc.entry(colony.colony)
                .or_insert(HashMap::new())
                .entry("food")
                .or_insert(entity);
            acc
        },
    );

    for (entity, _, resource_of) in carb_resources.iter_mut() {
        colony_food_resources_map
            .get_mut(&resource_of.colony)
            .unwrap()
            .entry("carb")
            .or_insert(entity);
    }

    for (entity, _, resource_of) in meat_resources.iter_mut() {
        colony_food_resources_map
            .get_mut(&resource_of.colony)
            .unwrap()
            .entry("meat")
            .or_insert(entity);
    }

    for (colony, resource_entities) in colony_food_resources_map {
        let (_, mut food_resource, _) = food_resources
            .get_mut(*resource_entities.get("food").unwrap())
            .unwrap();
        let (_, mut carb_resource, _) = carb_resources
            .get_mut(*resource_entities.get("carb").unwrap())
            .unwrap();
        let (_, mut meat_resource, _) = meat_resources
            .get_mut(*resource_entities.get("meat").unwrap())
            .unwrap();

        let food_cook_multiplier = 5.0;

        if carb_resource.amount > 100.0  * food_cook_multiplier
            && meat_resource.amount > 100.0 * food_cook_multiplier {
            let carb_consumed_amount = 100.0 * food_cook_multiplier;
            let meat_consumed_amount = 100.0 * food_cook_multiplier;
            let food_created_amount = 100.0 * food_cook_multiplier;

            carb_resource.amount -= carb_consumed_amount;
            meat_resource.amount -= meat_consumed_amount;
            food_resource.amount += food_created_amount;

            carb_consumed.send(CarbConsumed { colony, amount: carb_consumed_amount });
            meat_consumed.send(MeatConsumed { colony, amount: meat_consumed_amount });
            food_created.send(FoodCreated { colony, amount: food_created_amount });
        }
    }
}
