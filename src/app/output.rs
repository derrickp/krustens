use super::{MessageSet, BarChart};

#[derive(Clone, Debug)]
pub enum Output {
    MessageSet(MessageSet),
    BarChart(BarChart),
}
