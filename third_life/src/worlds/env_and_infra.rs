pub mod components;
mod sanitation_infra;

use bevy::prelude::*;

use self::sanitation_infra::SanitationInfrastructurePlugin;

pub struct InfrastructurePlugin;

impl Plugin for InfrastructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SanitationInfrastructurePlugin));
    }
}

