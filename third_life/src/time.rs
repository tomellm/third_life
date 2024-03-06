use crate::{config::ThirdLifeConfig, SimulationState};
use bevy::prelude::*;
use chrono::{prelude::*, Duration};

pub struct TimeDatePlugin;

impl Plugin for TimeDatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            (init_day_length, init_start_date),
        )
        .add_systems(
            Update,
            update_date.run_if(in_state(SimulationState::Running)),
        )
        .add_event::<DateChanged>()
        .add_event::<MonthChanged>();
    }
}

#[derive(Resource)]
struct DayLength {
    timer: Timer,
}

fn init_day_length(mut commands: Commands, config: Res<ThirdLifeConfig>) {
    commands.insert_resource(DayLength {
        timer: Timer::from_seconds(config.real_time_day_length(), TimerMode::Repeating),
    });
}

#[derive(Resource, Debug)]
pub struct GameDate {
    pub date: NaiveDate,
}

fn init_start_date(mut commands: Commands, config: Res<ThirdLifeConfig>) {
    commands.insert_resource(GameDate {
        date: NaiveDate::from_ymd_opt(
            config.starting_day().year(),
            config.starting_day().month(),
            config.starting_day().day(),
        )
        .unwrap(),
    })
}

fn update_date(
    time: Res<Time>,
    mut day_length: ResMut<DayLength>,
    mut game_date: ResMut<GameDate>,
    mut date_changed_writer: EventWriter<DateChanged>,
    mut month_changed_writer: EventWriter<MonthChanged>,
) {
    day_length.timer.tick(time.delta());

    if day_length.timer.finished() {
        game_date.date = game_date.date + Duration::days(1);

        if game_date.date.month() != game_date.date.pred_opt().unwrap().month() {
            month_changed_writer.send(MonthChanged);
        }

        date_changed_writer.send(DateChanged::from_date(game_date.date));
    }
}

impl std::ops::Deref for GameDate {
    type Target = NaiveDate;
    fn deref(&self) -> &Self::Target {
        &self.date
    }
}

#[derive(Event)]
pub struct DateChanged {
    date: NaiveDate
}

impl DateChanged {
    pub fn from_date(date: NaiveDate) -> Self {
        Self { date }
    }
}

#[derive(Event)]
pub struct MonthChanged;
