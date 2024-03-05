use crate::worlds::{ui::components::*, food::components::{ResourceOf, CarbResource, MeatResource, FoodResource}};

use core::panic;
use std::collections::HashMap;
use bevy::{prelude::*, reflect::List};
use bevy_egui::{EguiContexts, egui::{Color32, Window, Ui}};
use chrono::NaiveDate;
use egui_plot::{Plot, BarChart, Legend, Bar, PlotPoint, PlotPoints, Line};
use crate::{config::ThirdLifeConfig, time::GameDate, SimulationState};

use super::f32_to_plotpoints;

pub struct ResourcesUiPlugin;

impl Plugin for ResourcesUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                    resources_changed,
            ).run_if(in_state(SimulationState::Running)));
    }
}


pub fn resources_changed(
    time: Res<Time>,
    mut graph: Query<(&WorldUiEntity, &mut ResourceStorage)>,
    resources: Query<(&ResourceOf, Option<&CarbResource>, Option<&MeatResource>, Option<&FoodResource>)>
) {
    let mut map = graph.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
    let resources_map = resources.iter()
        .map(|r| (
                r.0.colony,
                (r.1.map(|e|e.amount),
                r.2.map(|e|e.amount),
                r.3.map(|e|e.amount))
        ))
        .fold(HashMap::new(), |mut acc: HashMap<Entity, _>, (col, vals)| {
            let entry = acc.entry(col).or_insert((0., 0., 0.));
            match vals {
                (Some(carbs), None, None) => entry.0 = carbs,
                (None, Some(meat), None) => entry.1 = meat,
                (None, None, Some(food)) => entry.2 = food,
                _ => panic!("Should not be possible there should not be a single entity with two resource types")
            };
            acc
        });

    for (colony, (carb, meat, food)) in resources_map.iter() {
        let Some(storage) = map.get_mut(&colony) else {
            continue;
        };

        let push = storage.timer.tick(time.delta()).just_finished();
        
        let set_or_push = |vec: &mut Vec<f32>, amount: &f32| {
            if push {
                vec.push(0.);
                if vec.len() > 50 {
                    vec.remove(0);
                }
            }
            *vec.last_mut().unwrap() = *amount;
        };

        set_or_push(&mut storage.carb, carb);
        set_or_push(&mut storage.meat, meat);
        set_or_push(&mut storage.food, food);
    }
    
}

pub fn resources_storage(
    name: &str,
    ui: &mut Ui,
    stor: &ResourceStorage
) {
    ui.heading("Resources");
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(format!("Meat in storage {:.2}", stor.meat.last().unwrap()));
            plot_resource_line(format!("Meat Line {name}"), ui, &stor.meat);
            ui.label(format!("Carbs in storage {:.2}", stor.carb.last().unwrap()));
            plot_resource_line(format!("Carb Line {name}"), ui, &stor.carb);
        });
        ui.vertical(|ui| {
            ui.label(format!("Food in storage {:.2}", stor.food.last().unwrap()));
            plot_resource_line(format!("Food Line {name}"), ui, &stor.food);
        });
    });
}

pub fn plot_resource_line(
    label: String,
    ui: &mut Ui,
    vec: &Vec<f32>,
) {
    Plot::new(label)
        .height(75.).width(200.)
        .allow_zoom(false).allow_scroll(false).allow_drag(false)
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(f32_to_plotpoints(vec))
                    .color(Color32::from_rgb(100, 200, 100))
                    .name("line")
            );
        });
}


