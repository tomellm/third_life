use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiSettings};

pub struct ThridLifeUiPlugin;

impl Plugin for ThridLifeUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, configure_visuals_system);
    }
}

fn configure_visuals_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut egui_settings: ResMut<EguiSettings>,
) {
    commands.spawn(Camera2dBundle::default());
    let ctx = contexts.ctx_mut();
    ctx.set_visuals(bevy_egui::egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });

    egui_settings.scale_factor = 1.0;
}
