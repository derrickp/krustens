use super::BarDataPoint;

#[derive(Clone, Debug)]
pub struct BarChart {
    pub title: String,
    pub data_points: Vec<BarDataPoint>,
}
