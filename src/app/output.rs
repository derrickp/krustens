use super::{BarChart, MessageSet};

#[derive(Clone, Debug)]
pub enum Output {
    MessageSet(MessageSet),
    BarChart(BarChart),
}
