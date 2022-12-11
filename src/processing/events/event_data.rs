use serde::{Deserialize, Serialize};

use super::{track_play_added::TrackPlayAdded, track_skipped::TrackSkipped};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum EventData {
    TrackPlayAdded(TrackPlayAdded),
    TrackPlayIgnored(TrackSkipped),
}
