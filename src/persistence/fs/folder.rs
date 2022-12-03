pub trait Folder {
    fn create_if_necessary(&self);
    fn path(&self) -> String;
}
