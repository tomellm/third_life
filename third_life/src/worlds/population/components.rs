use bevy::prelude::*;
use chrono::NaiveDate;

#[derive(Component, Default)]
pub struct Population {
    pub count: usize,
    pub working_pop: usize,
    pub younglings: usize,
    pub retirees: usize,
    pub average_age: usize,
    pub average_children_per_mother: f32
}

#[derive(Component, PartialEq, Clone)]
pub struct Citizen {
    pub name: String,
    pub birthday: NaiveDate,
}

#[derive(Component)]
pub struct CitizenOf {
    pub colony: Entity,
}

#[derive(Component)]
pub struct Starving {
    pub days_since_last_meal: usize
}

impl Starving {
    pub fn died(&self) -> bool { self.days_since_last_meal > 21 }
}

#[derive(Component)]
pub struct Female {
    pub children_had: usize
}

#[derive(Component)]
pub struct Male;

#[derive(Component)]
pub struct Ovulation {
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

#[derive(Component)]
pub struct Widowed;

#[derive(Component)]
pub struct Employed;

#[derive(Component)]
pub struct Employable;


#[derive(Component)]
pub struct Youngling;

#[derive(Component)]
pub struct Retiree;

