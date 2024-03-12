
use bevy::{prelude::*, utils::warn};

use crate::worlds::config::GovernmentConfig;

#[derive(Bundle)]
pub struct ColonyWealthBundle {
    gdp: WealthAndSpending,
}

impl ColonyWealthBundle {
    pub fn new(government: GovernmentConfig) -> Self {
        ColonyWealthBundle { 
            gdp: WealthAndSpending { 
                total_wealth: 0.,
                spending_available: 0.,
                citizen_payout: government.citizen_payout(),
                policy: SpendingPolicy { 
                    civil_spending: government.civil_spending(),
                    sanitation_spending: government.sanitation_spending(),
                    social_spending: government.social_spending(),
                    environmental_spending: government.environmental_spending()
                }
            } 
        }
    }
}

#[derive(Component, Default)]
pub struct WealthAndSpending {
    pub total_wealth: f32,
    /// 0 to 1 number that reppresents how many percent of total gdp are 
    /// available to be used for spending
    pub spending_available: f32,
    /// 0 to 1 value that each citizen gets payed
    pub citizen_payout: f32,
    pub policy: SpendingPolicy,
}

impl WealthAndSpending {
    /// Each citizen gets a payout according to [`Self::citizen_payout`] which
    /// leaves us with a share of the money made to use as gdp
    pub fn calc(&mut self, working: usize, population_count: usize) {
        let population_count = population_count as f32;
        self.total_wealth = working as f32;
        self.spending_available =  self.total_wealth - (population_count * self.citizen_payout);
    }
    pub fn total_civil_spending(&self) -> f32 {
        Self::to_01(self.policy.civil_spending) * self.spending_available
    }
    pub fn total_sanitation_spending(&self) -> f32 {
        Self::to_01(self.policy.sanitation_spending) * self.spending_available
    }
    pub fn total_social_spending(&self) -> f32 {
        Self::to_01(self.policy.social_spending) * self.spending_available
    }
    pub fn total_environmental_spending(&self) -> f32 {
        Self::to_01(self.policy.environmental_spending) * self.spending_available
    }
    fn to_01(num: usize) -> f32 {
        num as f32 / 100.
    }
}

/// Contains percentages of how the remaining gdp should be spent. The
/// numbers schould all be between 0 and 100 and in total come together
/// to be 100 when added up
#[derive(Default)]
pub struct SpendingPolicy {
    civil_spending: usize,
    sanitation_spending: usize,
    social_spending: usize,
    environmental_spending: usize,
}

impl SpendingPolicy {
    
}
