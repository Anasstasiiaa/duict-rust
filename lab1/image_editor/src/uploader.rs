pub trait Uploader {
    fn upload(&self, filename: &str, data: Vec<u8>);
}
