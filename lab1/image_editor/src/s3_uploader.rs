use aws_sdk_s3::{Client};
use crate::uploader::Uploader;
use tokio::runtime::Runtime;

pub struct S3Uploader {
    pub bucket: String,
}

impl Uploader for S3Uploader {
    fn upload(&self, filename: &str, data: Vec<u8>) {
        let bucket = self.bucket.clone();

        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let config = aws_config::load_from_env().await;
            let client = Client::new(&config);

            client.put_object()
                .bucket(bucket)
                .key(filename)
                .body(data.into())
                .send()
                .await
                .unwrap();
        });
    }
}