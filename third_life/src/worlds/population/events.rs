use bevy::prelude::*;
use chrono::NaiveDate;

#[derive(Event)]
pub struct CitizenCreated {
    pub age: usize,
    pub colony: Entity,
}

pub enum DeathReason {
    OldAge, Starvation, InfantDeath
}

#[derive(Event)]
pub struct CitizenDied {
    pub colony: Entity,
    pub citizen: Entity,
    pub reason: DeathReason,
}

impl CitizenDied {
    pub fn old_age(colony: Entity, citizen: Entity) -> Self {
        Self { colony, citizen, reason: DeathReason::OldAge }
    }
    pub fn starved(colony: Entity, citizen: Entity) -> Self {
        Self { colony, citizen, reason: DeathReason::Starvation }
    }
    pub fn infant_death(colony: Entity, citizen: Entity) -> Self {
        Self { colony, citizen, reason: DeathReason::InfantDeath }
    }
}

#[derive(Event)]
pub struct CitizenBirthday {
    pub entity: Entity,
    pub age: usize
}

