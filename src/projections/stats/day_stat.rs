use chrono::Weekday;
use serde::{ser::SerializeStruct, Serialize};

use super::play_count::PlayCount;

#[derive(Clone)]
pub struct DayStat {
    pub weekday: Weekday,
    pub plays: Vec<PlayCount>,
}

impl DayStat {
    fn to_serialize(&self) -> SerializedDayStruct {
        SerializedDayStruct {
            weekday: self.weekday.to_string(),
            plays: self
                .plays
                .iter()
                .map(|play| play.total_plays_display())
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct SerializedDayStruct {
    pub weekday: String,
    pub plays: Vec<String>,
}

impl Serialize for DayStat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serialized = self.to_serialize();
        let mut state = serializer.serialize_struct("day_play", 2)?;
        state.serialize_field("weekday", &serialized.weekday)?;
        state.serialize_field("plays", &serialized.plays)?;
        state.end()
    }
}
