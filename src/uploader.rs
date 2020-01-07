extern crate rusoto_core;
extern crate rusoto_s3;
extern crate tokio;

use std::str::FromStr;

use crossbeam_channel::Receiver;
use log::{info};
use rusoto_core::{ByteStream, Region};
use rusoto_s3::{S3Client, S3, PutObjectRequest};

use crate::database::{Database, File};
use crate::error::{Result};


pub struct Uploader {
    bucket_name: String,
    upload_rx: Receiver<File>,
    db: Database,
    s3_client: S3Client,
}


impl Uploader {
    pub fn new(bucket_name: &str, region_name: &str, upload_rx: Receiver<File>, db: Database) -> Result<Uploader> {
        let region = Region::from_str(region_name).unwrap();
        let s3_client = S3Client::new(region);
        let bucket_name: String = bucket_name.into();

        Ok(Uploader {
            bucket_name,
            upload_rx,
            db,
            s3_client,
        })
    }

    pub fn test(&self) {


        let p_o_r = PutObjectRequest {
            body: Some(byte_stream.into()),
            bucket: self.bucket_name.clone(),
            key: "copying".to_owned(),
            ..Default::default()
        };

        let put_result = self.s3_client.put_object(p_o_r).sync();
        match put_result {
            Ok(p_o_o) => info!("PutObjectOutput: {:?}", p_o_o),
            Err(err) => info!("PutObjectError: {:?}", err),
        }
    }
}
