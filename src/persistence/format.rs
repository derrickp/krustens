#[derive(Clone)]
pub enum Format {
    Json,
    Yaml,
}

impl Format {
    pub fn extension_display(&self) -> &str {
        match *self {
            Format::Json => "json",
            Format::Yaml => "yaml",
        }
    }
}

impl TryFrom<String> for Format {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lowered = value.to_lowercase();
        if lowered.eq("json") {
            Ok(Format::Json)
        } else if lowered.eq("yaml") || lowered.eq("yml") {
            Ok(Format::Yaml)
        } else {
            Err(())
        }
    }
}
