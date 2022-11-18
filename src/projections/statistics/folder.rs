use std::{fs::create_dir_all, path::Path};

pub struct Folder {
    pub output_folder: String,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

pub enum FileName {
    Complete,
    General,
    Top50,
    Top100,
}

impl ToString for FileName {
    fn to_string(&self) -> String {
        match *self {
            FileName::Complete => "complete.json".to_string(),
            FileName::General => "general.yaml".to_string(),
            FileName::Top50 => "top_50.json".to_string(),
            FileName::Top100 => "top_100.json".to_string(),
        }
    }
}

impl Folder {
    pub fn file_name(&self, file: &FileName) -> String {
        if self.year.is_some() && self.month.is_some() {
            self.year_month_file(self.year.unwrap(), self.month.unwrap(), file)
        } else if self.year.is_some() {
            self.year_file(self.year.unwrap(), file)
        } else {
            format!("{}/{}", self.output_folder, file.to_string())
        }
    }

    pub fn folder_name(&self) -> String {
        if self.year.is_some() && self.month.is_some() {
            self.year_month_folder(self.year.unwrap(), self.month.unwrap())
        } else if self.year.is_some() {
            self.year_folder(self.year.unwrap())
        } else {
            self.output_folder.to_string()
        }
    }

    pub fn create_if_necessary(&self) {
        if !Path::new(&self.folder_name()).exists() {
            create_dir_all(self.folder_name()).unwrap()
        }
    }

    fn year_file(&self, year: i32, file: &FileName) -> String {
        format!("{}/{}", self.year_folder(year), file.to_string())
    }

    fn year_folder(&self, year: i32) -> String {
        format!("{}/{}", self.output_folder, year)
    }

    fn year_month_folder(&self, year: i32, month: u32) -> String {
        format!("{}/{}_{}", self.output_folder, year, month)
    }

    fn year_month_file(&self, year: i32, month: u32, file: &FileName) -> String {
        format!(
            "{}/{}",
            self.year_month_folder(year, month),
            file.to_string()
        )
    }
}
