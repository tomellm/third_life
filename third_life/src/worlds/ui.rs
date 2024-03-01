
use std::collections::HashMap;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui::{Color32, Window, Ui}};
use chrono::NaiveDate;
use egui_plot::{Plot, BarChart, Legend, Bar, PlotPoint, PlotPoints};
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
                    carbs_changed,
                    meats_changed,
                    food_changed,
                    update_ages,
                    update_general_pop
            ).run_if(in_state(SimulationState::Running)));
    }
}

#[derive(Component)]
struct WorldUi;

#[derive(Component)]
struct WorldUiName(String);

#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, Copy)]
struct WorldUiEntity(Entity);

#[derive(Component)]
struct PopulationHistorgram {
    count: usize,
    average_age: usize,
    ages: HashMap<usize, usize>,
    average_children_per_mother: f32, 
}

#[derive(Component)]
struct MeatStorage { pub graph: f32 }

impl MeatStorage {
    pub fn new() -> Self {
        Self { graph: 0. }
    }
}

#[derive(Component)]
struct CarbStorage { pub graph: f32 }

impl CarbStorage {
    pub fn new() -> Self {
        Self { graph: 0. }
    }
}

#[derive(Component)]
struct FoodStorage { pub graph: f32 }

impl FoodStorage {
    pub fn new() -> Self {
        Self { graph: 0. }
    }
}

#[derive(Bundle)]
struct WorldUiBundle {
    ui: WorldUi,
    name: WorldUiName,
    entity: WorldUiEntity,
    pop: PopulationHistorgram,
    meat_stor: MeatStorage,
    carb_stor: CarbStorage,
    food_stor: FoodStorage,
}

impl WorldUiBundle {
    pub fn new(name: String, entity: Entity) -> Self {
        Self { 
            ui: WorldUi,
            name: WorldUiName(name),
            entity: WorldUiEntity(entity),
            pop: PopulationHistorgram { count: 0, average_age: 0, ages: HashMap::new(), average_children_per_mother: 0.0},
            meat_stor: MeatStorage::new(),
            carb_stor: CarbStorage::new(),
            food_stor: FoodStorage::new() 
        }
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
    ui_data: Query<(&WorldUiName, &PopulationHistorgram, &MeatStorage, &CarbStorage, &FoodStorage)>,
) {
    for (world, pop, meat_stor, carb_stor, food_stor) in &ui_data {
        let name = &world.0;
        Window::new(format!("Window of {name}"))
            .show(contexts.ctx_mut(), |ui| {
                let start_date = NaiveDate::from_ymd_opt(config.starting_day().year(),config.starting_day().month(), config.starting_day().day()).unwrap();
                ui.label(format!("Years Elapsed:{}", game_date.date.years_since(start_date).unwrap()));
                ui.separator();
                meats_storage(ui, &meat_stor);
                carbs_storage(ui, &carb_stor);
                food_storage(ui, &food_stor);
                ui.separator();
                general_pop(ui, &pop.count, &pop.average_age, &pop.average_children_per_mother);
                ui.separator();
                age_histogram(name, ui, &pop.ages);
            });
    }
}

fn add_citizens_to_population_histogram(
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

fn update_ages(
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

fn update_general_pop(
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

fn general_pop(
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

fn age_histogram(
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

fn carbs_changed(
    time: Res<Time>,
    mut graph: Query<(&WorldUiEntity, &mut CarbStorage)>,
    carbs: Query<(&ResourceOf, &CarbResource)>
) {
    let mut map = graph.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
    for (ResourceOf { colony }, res) in &carbs {
        match map.get_mut(&colony) {
            Some(c) => c.graph = res.amount,
            None => ()
        }
    }
    
}

fn meats_changed(
    time: Res<Time>,
    mut graph: Query<(&WorldUiEntity, &mut MeatStorage)>,
    meats: Query<(&ResourceOf, &MeatResource)>
) {
    let mut map = graph.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
    for (ResourceOf { colony }, res) in &meats {
        match map.get_mut(&colony) {
            Some(c) => c.graph = res.amount,
            None => ()
        }
    }
}

fn food_changed(
    time: Res<Time>,
    mut graph: Query<(&WorldUiEntity, &mut FoodStorage)>,
    food: Query<(&ResourceOf, &FoodResource)>
) {
    let mut map = graph.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
    for (ResourceOf { colony }, res) in &food {
        match map.get_mut(&colony) {
            Some(c) => c.graph = res.amount,
            None => ()
        }
    }
}

fn meats_storage(
    ui: &mut Ui,
    meat_stor: &MeatStorage
) {
    ui.horizontal(|ui| {
        ui.label(format!("Meat in storage {:.2}", meat_stor.graph));
    });
}
fn carbs_storage(
    ui: &mut Ui,
    carb_stor: &CarbStorage
) {
    ui.horizontal(|ui| {
        ui.label(format!("Carb in storage {:.2}", carb_stor.graph));
    });
}

fn food_storage(
    ui: &mut Ui,
    food_stor: &FoodStorage
) {
    ui.horizontal(|ui| {
        ui.label(format!("Food in storage {:.2}", food_stor.graph));
    });
}

fn f32_to_plotpoints(
    vec: &Vec<f32>
) -> PlotPoints {
    let vec = vec.into_iter().enumerate()
        .map(|(i, n)| PlotPoint::new(i as f64, *n))
        .collect::<Vec<_>>();
    PlotPoints::Owned(vec)
}
