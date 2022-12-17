use std::{fmt::Display, fs::create_dir_all, path::Path};

use crate::persistence::fs::Folder;

pub struct OutputFolder {
    pub root: String,
}

impl Folder for OutputFolder {
    fn create_if_necessary(&self) {
        if !Path::new(&self.path()).exists() {
            create_dir_all(self.path()).unwrap()
        }
    }

    fn path(&self) -> String {
        self.root.to_string()
    }
}

impl Display for OutputFolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path())
    }
}
