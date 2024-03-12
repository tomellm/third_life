use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct ColonyInfraAndEnvBundle {
    civil: CivilInfrastructure,
    sanitation: SanitationInfrastructure,
    social: SocialInfrastructure,
    env_healt: EnvironmentalHealth,
    ecosystem_vitality: EcosystemVitality,
}

/// Power, Water, Seweage and so on
#[derive(Component, Default)]
pub struct CivilInfrastructure {
}

/// Anthing that has to do with sanitaion like hospitals but also 
/// things like bathrooms.
///
/// According to resaeartch and the correlation math we did
/// this relates in the following way to percentage of GDP spent
/// on healthcare: 
/// ```math
/// 5 * log(x) + 68.5
/// ```
#[derive(Component, Default)]
pub struct SanitationInfrastructure {
    pub health_index_score: f32,
    pub live_birth_mortality_rate: f32
}

impl SanitationInfrastructure {
    pub fn update(&mut self, spending: f32) {
        self.health_index_score_fn(spending);
        self.live_birth_mortality_rate_fn(spending);
    }
    fn health_index_score_fn(&mut self, spending: f32) {
        self.health_index_score = corr_ln(5.00186, 68.60778, spending);
    }
    fn live_birth_mortality_rate_fn(&mut self, spending: f32) {
        self.live_birth_mortality_rate = corr_ln(-0.00260, 0.00923, spending);
    }
}


/// Explains itself
#[derive(Component, Default)]
pub struct SocialInfrastructure {
}


/// Contains things that are directly affected by humans like:
/// - indoor air pollution
/// - drinking water
/// - urban particulates
#[derive(Component, Default)]
pub struct EnvironmentalHealth {
}

/// Things that are indirectly affected by humans only when the expand to 
/// - air quality
/// - water resources
/// - productive natural resources
/// - biodiversity
/// - sustainable energy
#[derive(Component, Default)]
pub struct EcosystemVitality {
}



fn corr_ln(a: f32, b: f32, x: f32) -> f32 {
    a * x.ln() + b
}
