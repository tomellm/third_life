
use bevy::prelude::*;
use rand::thread_rng;
use rand_distr::{Normal, Distribution, num_traits::Float};

use super::{WorldColony, init_colonies};

pub struct PopulationPlugin;
impl Plugin for PopulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (init_population, init_citizens)
                        .chain()
                        .after(init_colonies)
        );
    }
}

#[derive(Component)]
pub struct Population {

}

#[derive(Component)]
pub struct Citizen {
    pub name: String,
    pub age: usize
}

fn init_population(
    colonies: Query<Entity, With<WorldColony>>,
    mut commands: Commands
) {
    for colony in &colonies {
        commands.entity(colony).with_children(|commands| {
            commands.spawn(
                Population{}
            );
        });
    }
}


fn init_citizens(
    populations: Query<Entity, With<Population>>,
    mut commands: Commands
) {
    println!("hello my friends this should be here");
    for population in &populations {
        commands.entity(population).with_children(|parent| {
            let mut rng = thread_rng();
            let normal = Normal::new(30.0, 4.0).unwrap();
            let mut age_gen = normal.sample_iter(rng);

            for _ in 0..1000 {
                parent.spawn(
                    Citizen {
                        name: String::from("Geoff"),
                        age: age_gen.next().unwrap().floor() as usize
                    }
                );
            }
        });
    }
}
