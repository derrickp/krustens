use serde::Serialize;

#[derive(Clone, Default, Debug, Serialize)]
pub struct TimePlayed {
    pub time_ms: u64,
    pub time_sec: f32,
    pub time_min: f32,
    pub time_hr: f32,
}

impl TimePlayed {
    pub fn add_ms(&mut self, time: u64) {
        self.time_ms += time;
        self.time_sec += time as f32 / 1000.0;
        self.time_min += time as f32 / 60000.0;
        self.time_hr += time as f32 / (60000.0 * 60.0);
    }
}
