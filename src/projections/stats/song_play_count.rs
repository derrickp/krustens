use serde::Serialize;

#[derive(Default, Serialize, Clone, Debug)]
pub struct SongPlayCount(pub String, pub u64);

impl SongPlayCount {
    pub fn display(&self) -> String {
        format!("{} - {}", self.0.clone(), self.1)
    }
}
