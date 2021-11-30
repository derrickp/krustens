use serde::Serialize;

#[derive(Default, Serialize, Clone, Debug, Hash, Eq)]
pub struct SongPlayCount(pub String, pub u64);

impl SongPlayCount {
    pub fn display(&self) -> String {
        format!("{} - {}", self.0.clone(), self.1)
    }
}

impl PartialEq for SongPlayCount {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
