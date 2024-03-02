
use crate::worlds::ui::components::*;
use std::collections::HashMap;
use bevy::{prelude::*, reflect::List};
use bevy_egui::{EguiContexts, egui::{Color32, Ui}};
use egui_plot::{Plot, BarChart, Legend, Bar};
use crate::time::GameDate;
use super::super::{population::{Citizen, CitizenOf, CitizenCreated, Population}, init_colonies, WorldEntity, food::{CarbResource, MeatResource, FoodResource, ResourceOf}};

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
    mut contexts: EguiContexts,
    citizens: Query<(&Citizen, &CitizenOf)>,
    game_date: Res<GameDate>,
    mut populations: Query<(&WorldUiEntity, &mut PopulationHistorgram)>
) {
    let mut map = populations.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
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
                p.average_age = population.average_age;
                p.average_children_per_mother = population.average_children_per_mother;
            }
            None => ()
        }
    }
}

pub fn general_pop(
    ui: &mut Ui,
    count: &usize,
    average_age: &usize,
    average_children_per_mother: &f32,
) {
    ui.horizontal(|ui| {
        ui.label(format!("Pop count: {count}"));
        ui.label(format!("Average Age: {average_age}"));
        ui.label(format!("Average Children per Mother: {average_children_per_mother}"));
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
