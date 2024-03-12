use bevy::prelude::*;
use chrono::NaiveDate;

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
    pub size: f32,
    pub harvested: f32,
}

#[derive(Component)]
pub struct WheatFarmOf {
    pub colony: Entity,
}

#[derive(Component)]
pub struct WheatFarmer {
    pub farm: Entity,
}

#[derive(Component)]
pub struct Cow {
    pub birthday: NaiveDate
}

#[derive(Component)]
pub struct CowOf {
    pub cow_farm: Entity
}

#[derive(Component)]
pub struct IsBull;

#[derive(Component)]
pub struct IsBreeder;

#[derive(Component)]
pub struct MeatResource {
    pub amount: f32,
}

#[derive(Component)]
pub struct CowFarm {
    pub size: f32,
}

#[derive(Component)]
pub struct CowFarmOf {
    pub colony: Entity,
}

#[derive(Component)]
pub struct CowFarmer {
    pub farm: Entity,
}
