use super::BarDataPoint;

#[derive(Clone, Debug)]
pub struct BarChart {
    pub title: String,
    pub data_points: Vec<BarDataPoint>,
}

#[derive(Clone, Debug)]
pub enum BarBreakdown {
    Month,
    Weekday,
}

impl Default for BarBreakdown {
    fn default() -> Self {
        Self::Month
    }
}

impl ToString for BarBreakdown {
    fn to_string(&self) -> String {
        match *self {
            BarBreakdown::Month => "month".to_string(),
            BarBreakdown::Weekday => "weekday".to_string(),
        }
    }
}

impl TryFrom<&str> for BarBreakdown {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "month" => Ok(Self::Month),
            "weekday" => Ok(Self::Weekday),
            _ => Err(()),
        }
    }
}
