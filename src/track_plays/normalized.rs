use chrono::NaiveDateTime;

pub struct Normalized {
    pub end_time: NaiveDateTime,
    pub artist_name: String,
    pub track_name: String,
    pub service_hint: String,
    pub ms_played: Option<u64>,
    pub track_ms: Option<u64>,
    pub skipped: Option<bool>,
}

impl Normalized {
    pub fn formatted_end_time(&self) -> String {
        format!("{}", self.end_time.format("%Y-%m-%d %H:%M:%S"))
    }

    pub fn play_time(&self) -> u64 {
        self.ms_played.unwrap_or_default()
    }

    pub fn is_skipped(&self) -> bool {
        self.skipped.unwrap_or_default() || self.is_skipped_by_percent()
    }

    pub fn is_skipped_by_percent(&self) -> bool {
        self.ms_played
            .zip(self.track_ms)
            .map(|(ms_played, track_ms)| (ms_played as f64 / track_ms as f64) < 0.1)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use super::Normalized;

    #[test]
    fn is_skipped_by_percent() {
        let normalized = Normalized {
            end_time: NaiveDateTime::default(),
            artist_name: "artist".to_string(),
            track_name: "track".to_string(),
            service_hint: "apple_music".to_string(),
            ms_played: Some(330994),
            track_ms: Some(357000),
            skipped: Some(false),
        };

        assert!(!normalized.is_skipped_by_percent());
    }
}
