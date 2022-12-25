use serde::{Deserialize, Serialize};

use crate::app::HasId;

use super::BarDataPoint;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BarChart {
    id: String,
    title: String,
    data_points: Vec<BarDataPoint>,
}

impl HasId for BarChart {
    fn id(&self) -> &str {
        &self.id
    }
}

impl BarChart {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn data_points(&self) -> &Vec<BarDataPoint> {
        &self.data_points
    }

    pub fn with_data_points(title: &str, data_points: Vec<BarDataPoint>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            data_points,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
