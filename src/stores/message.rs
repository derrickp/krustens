use serde::{Deserialize, Serialize};

use crate::events::event_data::EventData;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub stream: String,
    pub position: u64,
    pub data: EventData,
}
