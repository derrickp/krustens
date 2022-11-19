pub trait Folder {
    fn create_if_necessary(&self);
    fn full_path(&self, file_name: &str) -> String;
    fn path(&self) -> String;
}
