use serde::{Deserialize, Serialize};

use super::event_data::EventData;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub version: u64,
    pub data: EventData,
}
