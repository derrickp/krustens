use std::fmt::Display;

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Clone, Debug, Default, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct TrackName(pub String);

impl Display for TrackName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl TrackName {
    pub fn eq_ignore_ascii_case(&self, other: &TrackName) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Serialize for TrackName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for TrackName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_string(StringVisitor)
            .map(TrackName)
    }
}

struct StringVisitor;

impl<'de> Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any string value")
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::TrackName;

    #[derive(serde::Deserialize, serde::Serialize)]
    struct TestWrapper {
        song_name: TrackName,
    }

    #[test]
    fn serialization() {
        let song_name = TrackName("A Terrible Pilgrimage to Seek the Nighted Throne".to_string());
        let wrapper = TestWrapper { song_name };

        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(
            "{\"song_name\":\"A Terrible Pilgrimage to Seek the Nighted Throne\"}",
            &serialized
        );
    }

    #[test]
    fn deserialization() {
        let serialized = "{\"song_name\":\"A Terrible Pilgrimage to Seek the Nighted Throne\"}";

        let wrapper: TestWrapper = serde_json::from_str(serialized).unwrap();
        assert_eq!(
            TrackName("A Terrible Pilgrimage to Seek the Nighted Throne".to_string()),
            wrapper.song_name
        );
    }
}
