use std::fmt::Display;

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArtistName(pub String);

impl ArtistName {
    pub fn starts_with(&self, name: &str) -> bool {
        self.0.to_ascii_lowercase().starts_with(name)
    }
}

impl Display for ArtistName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for ArtistName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ArtistName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_string(StringVisitor)
            .map(ArtistName)
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
    use super::ArtistName;

    #[derive(serde::Deserialize, serde::Serialize)]
    struct TestWrapper {
        artist_name: ArtistName,
    }

    #[test]
    fn serialization() {
        let artist_name = ArtistName("Strigoi".to_string());
        let wrapper = TestWrapper { artist_name };

        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!("{\"artist_name\":\"Strigoi\"}", &serialized);
    }

    #[test]
    fn deserialization() {
        let serialized = "{\"artist_name\":\"Strigoi\"}";

        let wrapper: TestWrapper = serde_json::from_str(serialized).unwrap();
        assert_eq!(ArtistName("Strigoi".to_string()), wrapper.artist_name);
    }
}
