
use crate::{worlds::{ui::components::*, population::{events::{CitizenCreated, CitizenDied, DeathReason}, components::{Citizen, CitizenOf, Population}}}, time::DateChanged, SimulationState};
use std::collections::HashMap;
use bevy::{prelude::*, reflect::List};
use bevy_egui::{EguiContexts, egui::{Color32, Ui}};
use chrono::NaiveDate;
use egui_plot::{Plot, BarChart, Legend, Bar, Line};
use crate::time::GameDate;

use super::usize_to_plotpoints;

pub struct PopulationUiPlugin;

impl Plugin for PopulationUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                    add_citizens_to_population_histogram,
                    update_ages,
                    update_general_pop,
                    death_events_listener
            ).run_if(in_state(SimulationState::Running)));
    }
}

pub fn add_citizens_to_population_histogram(
    mut pop_histograms: Query<(&WorldUiEntity, &mut PopulationHistorgram)>,
    mut citizen_created: EventReader<CitizenCreated>,
) {
    let mut map = pop_histograms.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
    for created_event in citizen_created.read() {
        let Some(hist) = map.get_mut(&created_event.colony) else {
            continue;
        };
        *hist.ages
            .entry(created_event.age)
            .or_insert(0) += 1;
    }
}

pub fn update_ages(
    citizens: Query<(&Citizen, &CitizenOf)>,
    game_date: Res<GameDate>,
    mut populations: Query<(&WorldUiEntity, &mut PopulationHistorgram)>
) {
    let map = populations.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
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

    for (key, mut val) in map.into_iter() {
        match populations_map.get(&key) {
            Some(new_map) => val.ages = new_map.clone(),
            None => ()
        }
    }
}

pub fn update_general_pop(
    query: Query<(Entity, &Population)>,
    mut populations: Query<(&WorldUiEntity, &mut PopulationHistorgram)>
) {
    let mut map = populations.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
    for (col, population) in query.iter() {
        match map.get_mut(&col) {
            Some(p) => {
                p.count = population.count;
                p.working_pop = population.working_pop;
                p.younglings = population.younglings;
                p.retirees = population.retirees;
                p.average_age = population.average_age;
                p.average_children_per_mother = population.average_children_per_mother;
            }
            None => ()
        }
    }
}

pub fn general_pop(
    ui: &mut Ui,
    pop: &PopulationHistorgram,
) {
    ui.horizontal(|ui| {
        ui.label(format!("Pop count: {:?}",pop.count));
        ui.label(format!("Working Pop: {:?}",pop.working_pop));
        ui.label(format!("Younglings count: {:?}",pop.younglings));
        ui.label(format!("Retirees count: {:?}",pop.retirees));
        ui.label(format!("Average Age: {:?}", pop.average_age));
        ui.label(format!("Average Children per Mother: {:?}", pop.average_children_per_mother));
    });
}

pub fn age_histogram(
    planet_name: &str,
    ui: &mut Ui,
    ages: &HashMap<usize, usize>
) {
    let bars = (0..100)
        .into_iter()
        .map(|index| {
            let height = ages.get(&index).map(|u| *u).unwrap_or(0);
            Bar::new(index as f64, height as f64).width(1.)
        })
    .collect::<Vec<_>>();
    let chart = BarChart::new(bars)
        .color(Color32::LIGHT_BLUE)
        .name(format!("Population chart of {planet_name}"));
    Plot::new(format!("Population {planet_name}"))
        .legend(Legend::default())
        .height(200.)
        .y_axis_width(3)
        .allow_zoom(false)
        .allow_drag(false)
        .show(ui, |plot_ui| plot_ui.bar_chart(chart))
        .response;
}

pub fn death_lines(
    planet_name: &str,
    ui: &mut Ui,
    deaths: &PopulationDeathLines
) {
    Plot::new(format!("Deaths on planet {planet_name}"))
        .height(150.).width(400.)
        .legend(Legend::default())
        .allow_zoom(false).allow_scroll(false).allow_drag(false)
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(usize_to_plotpoints(&deaths.old_age_deaths))
                    .color(Color32::from_rgb(0, 0, 255))
                    .name("old age")
            );
            plot_ui.line(
                Line::new(usize_to_plotpoints(&deaths.starvation_deaths))
                    .color(Color32::from_rgb(255, 0, 0))
                    .name("starvation")
            );
            plot_ui.line(
                Line::new(usize_to_plotpoints(&deaths.infant_deaths))
                    .color(Color32::from_rgb(0, 255, 0))
                    .name("infant death")
            );
        });
}

pub fn death_events_listener(
    time: Res<Time>,
    mut events: EventReader<CitizenDied>,
    mut uis: Query<(&WorldUiEntity, &mut PopulationDeathLines)>
) {
    let mapped_events = events.read().into_iter()
        .fold(HashMap::new(), |mut acc: HashMap<Entity, (usize, usize, usize)>, e| {
            let col_map = acc.entry(e.colony).or_insert((0, 0, 0));
            match e.reason {
                DeathReason::OldAge => col_map.0 += 1,
                DeathReason::Starvation => col_map.1 += 1,
                DeathReason::InfantDeath => col_map.2 += 1,
            };
            acc
        });

    for (WorldUiEntity(colony), mut lines) in uis.iter_mut() {
        lines.new_step(time.delta());
        let (old_age, starvation, infant) = mapped_events.get(&colony)
            .map(|(old_age, starvation, infant)|(*old_age, *starvation, *infant))
            .unwrap_or((0, 0, 0));
        *lines.old_age_deaths.last_mut().unwrap() += old_age;
        *lines.starvation_deaths.last_mut().unwrap() += starvation;
        *lines.infant_deaths.last_mut().unwrap() += infant;
    }
}
