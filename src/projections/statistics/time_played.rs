use serde::Serialize;

#[derive(Clone, Default, Debug, Serialize)]
pub struct TimePlayed {
    pub time_ms: u64,
    pub time_sec: f64,
    pub time_min: f64,
    pub time_hr: f64,
}

impl TimePlayed {
    pub fn add_ms(&mut self, additional_ms: u64) {
        self.time_ms += additional_ms;
        self.time_sec = self.calculate_time_sec(additional_ms);
        self.time_min = self.calculate_time_min(additional_ms);
        self.time_hr = self.calculate_time_hr(additional_ms);
    }

    fn calculate_time_sec(&self, additional_ms: u64) -> f64 {
        (self.time_ms + additional_ms) as f64 / 1000.0
    }

    fn calculate_time_min(&self, additional_ms: u64) -> f64 {
        (self.time_ms + additional_ms) as f64 / 60_000.0
    }

    fn calculate_time_hr(&self, additional_ms: u64) -> f64 {
        (self.time_ms + additional_ms) as f64 / (60_000.0 * 60.0)
    }
}
