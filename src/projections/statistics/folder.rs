use std::{fs::create_dir_all, path::Path};

pub struct Folder {
    pub output_folder: String,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

#[derive(Default)]
pub struct FolderInfoBuilder {
    root_path: Option<String>,
    year: Option<i32>,
    month: Option<u32>,
}

impl FolderInfoBuilder {
    pub fn build(&self) -> Folder {
        Folder {
            output_folder: self
                .root_path
                .to_owned()
                .unwrap_or_else(|| "./output/".to_string()),
            year: self.year,
            month: self.month,
        }
    }

    pub fn root_path(&mut self, path: &str) -> &mut Self {
        self.root_path = Some(path.to_string());

        self
    }

    pub fn year(&mut self, year: i32) -> &mut Self {
        self.year = Some(year);

        self
    }

    pub fn month(&mut self, month: u32) -> &mut Self {
        self.month = Some(month);

        self
    }
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
    pub fn builder() -> FolderInfoBuilder {
        FolderInfoBuilder::default()
    }
    pub fn file_name(&self, file: &FileName) -> String {
        match (self.year, self.month) {
            (None, None) | (None, Some(_)) => {
                format!("{}/{}", self.output_folder, file.to_string())
            }
            (Some(year), None) => self.year_file(year, file),
            (Some(year), Some(month)) => self.year_month_file(year, month, file),
        }
    }

    pub fn folder_name(&self) -> String {
        match (self.year, self.month) {
            (None, None) | (None, Some(_)) => self.output_folder.to_string(),
            (Some(year), None) => self.year_folder(year),
            (Some(year), Some(month)) => self.year_month_folder(year, month),
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
