mod components;
mod population_ui;
mod resources_ui;


use components::*;
use population_ui::*;
use resources_ui::*;

use core::panic;
use std::collections::HashMap;
use bevy::{prelude::*, reflect::List};
use bevy_egui::{EguiContexts, egui::{Color32, Window, Ui}};
use chrono::NaiveDate;
use egui_plot::{Plot, BarChart, Legend, Bar, PlotPoint, PlotPoints, Line};
use crate::{config::ThirdLifeConfig, time::GameDate, SimulationState};
use super::{population::{Citizen, CitizenOf, CitizenCreated, Population}, init_colonies, WorldEntity, food::{CarbResource, MeatResource, FoodResource, ResourceOf}};


pub struct WorldsUiPlugin;

impl Plugin for WorldsUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(SimulationState::Running), init_worlds_windows.after(init_colonies))
            .add_systems(Update, (
                    display_world_uis,
                    add_citizens_to_population_histogram,
                    resources_changed,
                    update_ages,
                    update_general_pop
            ).run_if(in_state(SimulationState::Running)));
    }
}





fn init_worlds_windows(
    mut commands: Commands,
    worlds: Query<(Entity, &WorldEntity)>,
) {
    for (entity, world) in &worlds {
        commands.spawn(WorldUiBundle::new(world.name.clone(), entity));
    }
}

fn display_world_uis(
    mut contexts: EguiContexts,
    config: Res<ThirdLifeConfig>,
    game_date: Res<GameDate>,
    ui_data: Query<(&WorldUiName, &PopulationHistorgram, &ResourceStorage)>,
) {
    for (world, pop, stor) in &ui_data {
        let name = &world.0;
        Window::new(format!("Window of {name}"))
            .show(contexts.ctx_mut(), |ui| {
                let start_date = NaiveDate::from_ymd_opt(config.starting_day().year(),config.starting_day().month(), config.starting_day().day()).unwrap();
                ui.label(format!("Years Elapsed:{}", game_date.date.years_since(start_date).unwrap()));
                ui.separator();
                resources_storage(name, ui, &stor);
                ui.separator();
                general_pop(ui, &pop.count, &pop.average_age, &pop.average_children_per_mother);
                ui.separator();
                age_histogram(name, ui, &pop.ages);
            });
    }
}




