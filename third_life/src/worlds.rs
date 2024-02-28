mod config;
mod food;
mod population;

use std::{collections::HashMap, ops::Deref};

use bevy::{log::tracing_subscriber::fmt::format, prelude::*};
use bevy_egui::{
    egui::{Color32, Ui, Window},
    EguiContexts,
};
use chrono::NaiveDate;
use egui_plot::{Bar, BarChart, Legend, Plot};

use crate::{
    config::{ConfigReaderFinishedEvent, RegisterConfigReaderEvent},
    SimulationState,
};

use self::{
    config::{WorldsConfig, WorldsConfigPlugin},
    food::FoodPlugin,
    population::{Citizen, CitizenOf, PopulationPlugin},
};
use crate::time::GameDate;

pub struct WorldsPlugin;

impl Plugin for WorldsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SimulationState::Running), init_colonies)
            .add_plugins((WorldsConfigPlugin, PopulationPlugin, FoodPlugin))
            .add_systems(
                Update,
                display_colonies.run_if(in_state(SimulationState::Running)),
            );
    }
}

#[derive(Component, PartialEq)]
pub struct WorldColony;

#[derive(Component)]
pub struct WorldEntity {
    name: String,
}

impl WorldEntity {
    fn new(name: &str) -> Self {
        let name = name.to_string();
        WorldEntity { name }
    }
}

#[derive(Component)]
pub struct ResourceAmount(f64);

fn init_colonies(mut commands: Commands) {
    commands.spawn((WorldColony, WorldEntity::new("Earth")));
    commands.spawn((WorldColony, WorldEntity::new("Mars")));
    commands.spawn((WorldColony, WorldEntity::new("Saturn")));
}

fn display_colonies(
    mut contexts: EguiContexts,
    citizens: Query<(&Citizen, &CitizenOf)>,
    game_date: Res<GameDate>,
) {
    let populations_map = citizens.into_iter().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, HashMap<usize, usize>>, (citizen, citizen_of)| {
            *acc.entry(citizen_of.colony)
                .or_insert(HashMap::new())
                .entry(game_date.date.years_since(citizen.birthday).unwrap() as usize)
                .or_insert(0) += 1;
            acc
        },
    );
    for (parent, pop) in populations_map {
        Window::new(format!("Window of {parent:?}")).show(contexts.ctx_mut(), |ui| {
            ui.label(format!(
                "Years Elapsed:{:?}",
                game_date
                    .date
                    .years_since(NaiveDate::from_ymd_opt(2150, 01, 01).unwrap())
                    .unwrap()
            ));
            let bars = (0..100)
                .into_iter()
                .map(|index| {
                    let height = pop.get(&index).map(|u| *u).unwrap_or(0);
                    Bar::new(index as f64, height as f64).width(1.)
                })
                .collect::<Vec<_>>();
            let chart = BarChart::new(bars)
                .color(Color32::LIGHT_BLUE)
                .name(format!("Population chart of {parent:?}"));
            Plot::new(format!("Population {:?}", parent))
                .legend(Legend::default())
                .clamp_grid(true)
                .y_axis_width(3)
                .allow_zoom(false)
                .allow_drag(false)
                .show(ui, |plot_ui| plot_ui.bar_chart(chart))
                .response;
        });
    }
}
