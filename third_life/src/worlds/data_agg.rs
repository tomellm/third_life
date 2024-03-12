use std::str::FromStr;

use crate::time::GameDate;
use crate::worlds::food::{
    components::CarbResource, components::FoodResource, components::MeatResource,
    components::ResourceOf,
};
use crate::worlds::population::components::Population;
use crate::worlds::{init_colonies, WorldEntity};
use crate::SimulationState;
use bevy::prelude::*;
use bevy::tasks::futures_lite::stream;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use influxdb2::api::buckets::ListBucketsRequest;
use influxdb2::api::organization::ListOrganizationRequest;
use influxdb2::models::{DataPoint, PostBucketRequest};
use influxdb2::Client;

use bevy_async_task::{AsyncTask, AsyncTaskPool};

pub struct DataAggPlugin;

impl Plugin for DataAggPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            (init_data_agg).chain().after(init_colonies),
        )
        .add_systems(
            Update,
            (food_queue, population_queue).run_if(in_state(SimulationState::Running)),
        )
        .init_resource::<InfluxDB>();
    }
}

#[derive(Resource)]
struct InfluxDB {
    client: Client,
    bucket: String,
}

impl Default for InfluxDB {
    fn default() -> Self {
        let client = Client::new("http://localhost:8086", "third-life-team", "admin-token");
        let bucket = "third-life-1".to_string();
        InfluxDB { client, bucket }
    }
}

fn food_queue(
    mut task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    game_date: Res<GameDate>,
    food_resources: Query<(&FoodResource, &ResourceOf)>,
    carb_resources: Query<(&CarbResource, &ResourceOf)>,
    meat_resources: Query<(&MeatResource, &ResourceOf)>,
    worlds: Query<(Entity, &WorldEntity)>,
) {
    let date_time = Utc::from_local_datetime(
        &Utc,
        &NaiveDateTime::new(
            NaiveDate::from_ymd_opt(
                game_date.date.year(),
                game_date.date.month(),
                game_date.date.day(),
            )
            .unwrap(),
            NaiveTime::from_hms_nano_opt(0, 0, 0, 0).unwrap(),
        ),
    )
    .unwrap()
    .timestamp_nanos_opt()
    .unwrap();

    for (entity, world) in worlds.iter() {
        let filtered_resources = food_resources
            .iter()
            .zip(carb_resources.iter())
            .zip(meat_resources.iter())
            .filter_map(
                |(
                    ((food, food_resource_of), (carb, carb_resource_of)),
                    (meat, meat_resource_of),
                )| {
                    if food_resource_of.colony == entity
                        && carb_resource_of.colony == entity
                        && meat_resource_of.colony == entity
                    {
                        Some((food_resource_of, carb, meat, food))
                    } else {
                        None
                    }
                },
            )
            .collect::<Vec<_>>();

        for (_, carb, meat, food) in filtered_resources {
            let data = vec![DataPoint::builder("resources")
                .tag("world", world.name.clone())
                .field("carb", carb.amount as f64)
                .field("meat", meat.amount as f64)
                .field("food", food.amount as f64)
                //.timestamp(date_time)
                .field("game_date", date_time)
                .build()
                .unwrap()];

            let client = influxdb.client.clone();
            let bucket = influxdb.bucket.clone();
            let task = AsyncTask::new(async move {
                let result = write_data(client, bucket.as_str(), data.clone()).await;
                if !result.is_ok() {
                    println!("Error writing data to InfluxDB");
                    println!("{:?}", result.err());
                }
            });

            let (fut, _) = task.into_parts();
            task_pool.spawn(async move {
                fut.await;
            });
        }
    }
}

fn population_queue(
    mut task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    game_date: Res<GameDate>,
    populations: Query<(&WorldEntity, &Population)>,
) {
    let date_time = Utc::from_local_datetime(
        &Utc,
        &NaiveDateTime::new(
            game_date.date,
            NaiveTime::from_hms_nano_opt(1, 1, 1, 1).unwrap(),
        ),
    )
    .unwrap()
    .to_string();

    for (world, population) in &populations {
        let data = vec![DataPoint::builder("population")
            .tag("world", world.name.clone())
            .field("population_count", population.count as i64)
            .field("average_age", population.average_age as f64)
            .field(
                "average_children_per_mother",
                population.average_children_per_mother as f64,
            )
            .field("game_date", date_time.clone())
            //.timestamp(date_time)
            .build()
            .unwrap()];

        let client = influxdb.client.clone();
        let bucket = influxdb.bucket.clone();
        let task = AsyncTask::new(async move {
            let result = write_data(client, bucket.as_str(), data).await;
            if !result.is_ok() {
                println!("Error writing population data to InfluxDB");
                println!("{:?}", result.err());
            }
        });
        let (fut, _) = task.into_parts();
        task_pool.spawn(async {
            fut.await;
        })
    }
}

fn init_data_agg(mut task_pool: AsyncTaskPool<()>, influxdb: Res<InfluxDB>) {
    println!("Initializing DataAgg");

    let client = influxdb.client.clone();
    let bucket = influxdb.bucket.clone();

    let init_infra_task = AsyncTask::new(async move {
        println!("Task running: Initializing InfluxDB client");
        let organization_id = client
            .list_organizations(ListOrganizationRequest {
                ..Default::default()
            })
            .await
            .unwrap()
            .orgs
            .iter()
            .filter(|org| org.name == "third-life-team")
            .map(|org| org.id.clone())
            .next()
            .unwrap()
            .unwrap();

        let buckets = client
            .list_buckets(Some(ListBucketsRequest {
                org_id: Option::Some(organization_id.clone()),
                ..Default::default()
            }))
            .await
            .unwrap();
        // check all the buckets names. the first bucket is "third-life-1"
        // if it exists create third-life-2 and so on till third-life-5
        //  at which point delete third-life-5 and create a new third-life-5

        let bucket_names = buckets
            .buckets
            .iter()
            .map(|bucket| bucket.name.clone())
            .collect::<Vec<String>>();

        let mut new_bucket_name = bucket;
        let mut i = 1;
        while bucket_names.contains(&new_bucket_name) {
            i += 1;
            new_bucket_name = format!("third-life-{}", i);
        }

        if i >= 5 {
            let last_bucket_id = buckets
                .buckets
                .iter()
                .filter(|bucket| bucket.name == "third-life-5")
                .map(|bucket| bucket.id.clone())
                .next()
                .unwrap()
                .unwrap();
            client.delete_bucket(last_bucket_id.as_str()).await.unwrap();
        }

        let result = client
            .create_bucket(Some(PostBucketRequest::new(
                organization_id,
                new_bucket_name.clone(),
            )))
            .await;

        println!(
            "Task finished: InfluxDb client Initialization was successful?: {}",
            result.is_ok()
        );
    });

    let (fut, _) = init_infra_task.into_parts();

    task_pool.spawn(async {
        fut.await;
    });
}

async fn write_data(
    client: Client,
    bucket: &str,
    data: Vec<DataPoint>,
) -> Result<(), Box<dyn std::error::Error>> {
    // client
    //     .write_with_precision(bucket, stream::iter(data), TimestampPrecision::Seconds)
    //     .await?;
    client.write(bucket, stream::iter(data)).await?;
    Ok(())
}
