use bevy::prelude::*;
use chrono::{prelude::*, Duration};

pub struct TimeDatePlugin;

impl Plugin for TimeDatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DayLength>()
            .init_resource::<GameDate>()
            .add_systems(Update, update_date)
            .add_event::<DateChanged>();
    }
}

#[derive(Resource)]
struct DayLength {
    timer: Timer,
}

impl Default for DayLength {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

#[derive(Resource, Debug)]
pub struct GameDate {
    pub date: NaiveDate,
}

impl Default for GameDate {
    fn default() -> Self {
        Self {
            date: NaiveDate::from_ymd_opt(2150, 01, 01).unwrap(),
        }
    }
}

fn update_date(
    time: Res<Time>,
    mut day_length: ResMut<DayLength>,
    mut game_date: ResMut<GameDate>,
    mut date_changed_writer: EventWriter<DateChanged>,
) {
    day_length.timer.tick(time.delta());

    if day_length.timer.finished() {
        game_date.date = game_date.date + Duration::days(1);
        date_changed_writer.send(DateChanged);
    }
}

impl std::ops::Deref for GameDate {
    type Target = NaiveDate;
    fn deref(&self) -> &Self::Target {
        &self.date
    }
}



#[derive(Event)]
pub struct DateChanged;
