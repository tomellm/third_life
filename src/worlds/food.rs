use bevy::prelude::*;

use super::*;

#[derive(Component)]
pub struct FoodResource;

#[derive(Component)]
pub struct MeatResource;
#[derive(Component)]
pub struct CarbResource;

#[derive(Component)]
pub struct CowFarm {
    available_resource: f32
}

/*

fn create_meat(
    farms: Query<(&CowFarm, &Parent)>>,
    worlds: Query<(Entity, &Children), With<Planet>>,
    meats: Query<(Entity, &ResourceAmount), With<MeatResource>>,
) {

}


fn work_on_cow_farm(
    farms: Query<(&CowFarm, &Parent)>,
    planet: Query<(&ColonyPopulation, &Children)>
) {

}

fn create_carb(
    query: Query<&ResourceAmount, With<CarbResource>>
) {

}

fn create_food(
    meats: Query<&ResourceAmount, With<MeatResource>>,
    carbs: Query<&ResourceAmount, With<CarbResource>>,
    food: Query<&ResourceAmount, With<FoodResource>>
) {

}









*/





