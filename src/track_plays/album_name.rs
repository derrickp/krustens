use std::fmt::Display;

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AlbumName(pub String);

impl AlbumName {
    pub fn eq_ignore_ascii_case(&self, other: &AlbumName) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Display for AlbumName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for AlbumName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for AlbumName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_string(StringVisitor)
            .map(AlbumName)
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
    use super::AlbumName;

    #[derive(serde::Deserialize, serde::Serialize)]
    struct TestWrapper {
        album_name: AlbumName,
    }

    #[test]
    fn serialization() {
        let album_name = AlbumName("From Which Nightmares Crawl".to_string());
        let wrapper = TestWrapper { album_name };

        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(
            "{\"album_name\":\"From Which Nightmares Crawl\"}",
            &serialized
        );
    }

    #[test]
    fn deserialization() {
        let serialized = "{\"album_name\":\"From Which Nightmares Crawl\"}";

        let wrapper: TestWrapper = serde_json::from_str(serialized).unwrap();
        assert_eq!(
            AlbumName("From Which Nightmares Crawl".to_string()),
            wrapper.album_name
        );
    }
}
