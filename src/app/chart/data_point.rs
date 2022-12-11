#[derive(Clone, Debug)]
pub struct BarDataPoint {
    x: String,
    y: u64,
}

impl BarDataPoint {
    pub fn new(x: String, y: u64) -> BarDataPoint {
        BarDataPoint { x, y }
    }

    pub fn x(&self) -> &str {
        &self.x
    }

    pub fn y(&self) -> u64 {
        self.y
    }
}
