use serde::{Deserialize, Serialize};

use super::listen_added::ListenAdded;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum EventData {
    ListenAdded(ListenAdded),
}
