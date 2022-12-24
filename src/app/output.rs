use serde::{Deserialize, Serialize};

use super::{BarChart, MessageSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Output {
    MessageSet(MessageSet),
    BarChart(BarChart),
}
