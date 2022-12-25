use serde::{Deserialize, Serialize};

use super::{BarChart, HasId, MessageSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Output {
    MessageSet(MessageSet),
    BarChart(BarChart),
}

impl HasId for Output {
    fn id(&self) -> &str {
        match self {
            Output::MessageSet(it) => it.id(),
            Output::BarChart(it) => it.id(),
        }
    }
}
