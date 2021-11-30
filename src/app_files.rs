use crate::persistence::file_writer::FileWriter;

pub struct AppFiles {
    pub folder: &'static str,
}

impl AppFiles {
    pub fn streams_file(&self) -> String {
        format!("{}/streams.json", self.folder)
    }

    pub fn snapshot_file(&self) -> String {
        format!("{}/snapshot.json", self.folder)
    }

    pub fn streams_writer(&self) -> FileWriter {
        FileWriter::from(self.streams_file())
    }

    pub fn snapshot_writer(&self) -> FileWriter {
        FileWriter::from(self.snapshot_file())
    }
}
