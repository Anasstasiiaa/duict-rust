use crate::uploader::Uploader;
use std::env;
use std::fs;

pub struct FsUploader;

impl Uploader for FsUploader {
    fn upload(&self, filename: &str, data: Vec<u8>) {
        let dir = env::var("MYME_FILES_PATH").expect("MYME_FILES_PATH not set");

        let path = format!("{}/{}", dir, filename);
        fs::write(path, data).unwrap();
    }
}
