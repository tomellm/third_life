use bevy::prelude::*;

#[derive(Event)]
pub struct WheatFarmNeedsWorker {
    pub colony: Entity,
    pub farm: Entity,
}

#[derive(Event)]
pub struct CowFarmNeedsWorker {
    pub colony: Entity,
    pub farm: Entity,
}

#[derive(Event)]
pub struct MeatConsumed {
    pub colony: Entity,
    pub amount: f32
}

#[derive(Event)]
pub struct CarbCreated {
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
