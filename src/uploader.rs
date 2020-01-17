extern crate rusoto_core;
extern crate rusoto_s3;

use std::error::Error as StdError;
use std::io::{Read, Error as IOError};
use std::fmt;
use std::fs::File as FSFile;
use std::result::Result as StdResult;
use std::str::FromStr;

use crossbeam_channel::Receiver;
use log::{info, warn, error};
use rusoto_core::{ByteStream, Region, RusotoError};
use rusoto_s3::{
    S3Client,
    S3,
    PutObjectRequest,
    UploadPartRequest,
    AbortMultipartUploadRequest,
    CreateMultipartUploadRequest,
    CreateMultipartUploadError,
    CompletedMultipartUpload,
    CompleteMultipartUploadRequest,
};

use crate::database::{Database, File};
use self::rusoto_s3::CompletedPart;


pub struct Uploader {
    bucket_name: String,
    upload_rx: Receiver<File>,
    db: Database,
    s3_client: S3Client,
    request_payer: Option<String>,
    part_size: usize,
}


impl Uploader {
    pub fn new(bucket_name: &str, region_name: &str, upload_rx: Receiver<File>, db: Database) -> Uploader {
        let region = Region::from_str(region_name).unwrap();
        let s3_client = S3Client::new(region);
        let bucket_name: String = bucket_name.into();

        Uploader {
            bucket_name,
            upload_rx,
            db,
            s3_client,
            request_payer: None,
            part_size: 1024*1024*100,
        }
    }

    pub fn upload_file(&self, filename: &str) -> Result<()> {
        let upload_id = self.create_multipart_upload(filename)?;

        if let Ok(c_mp_u) = self.upload_file_parts(filename, &upload_id) {
            if let Ok(()) = self.complete_multipart_upload(filename, c_mp_u, &upload_id) {
                info!("Completed upload of {}", filename);
                return Ok(())
            }
        }
        self.abort_multipart_upload(filename, &upload_id);
        Err(Error::CompleteMultipartUpload)
    }

    fn upload_file_parts(&self, filename: &str, upload_id: &str) -> Result<CompletedMultipartUpload> {
        let mut file = FSFile::open(filename).unwrap();

        let mut part_number = 1;

        let mut completed_parts: Vec<CompletedPart> = Vec::new();

        loop {
            let mut buffer = vec![0; self.part_size];

            match file.read(&mut buffer) {
                Ok(len) => {
                    if len == 0 {
                        info!("Done!");
                        break;
                    }
                    buffer.truncate(len);
                    match self.upload_part(buffer, filename, part_number, upload_id) {
                        Ok(completed_part) => {
                            completed_parts.push(completed_part);
                            part_number += 1;
                        },
                        Err(err) => {
                            return Err(err);
                        }
                    }
                },
                Err(err) => {
                    error!("Error reading file: {}", err);
                    return Err(Error::Read(err));
                }
            }
        }

        Ok(CompletedMultipartUpload{parts: Some(completed_parts)})
    }

    fn upload_part(&self,
                   body: Vec<u8>,
                   filename: &str,
                   part_number: i64,
                   upload_id: &str) -> Result<CompletedPart> {
        return match self.s3_client.upload_part(
            UploadPartRequest {
                body: Some(body.into()),
                bucket: self.bucket_name.to_owned(),
                key: filename.to_owned(),
                part_number,
                upload_id: upload_id.to_owned(),
                ..Default::default()
            }).sync() {
            Ok(res) => {
                let e_tag = res.e_tag.unwrap();
                info!("Uploaded part {} - etag: {}", part_number, e_tag);
                Ok(CompletedPart {
                    part_number: Some(part_number),
                    e_tag: Some(e_tag)
                })
            },
            Err(err) => {
                error!("Failed to upload part {}: {}", part_number, err);
                Err(Error::UploadPart)
            }
        }
    }

    fn complete_multipart_upload(&self,
                                 filename: &str,
                                 multipart_upload: CompletedMultipartUpload,
                                 upload_id: &str) -> Result<()> {
            match self.s3_client.complete_multipart_upload(
                CompleteMultipartUploadRequest {
                    bucket: self.bucket_name.to_owned(),
                    key: filename.to_owned(),
                    multipart_upload: Some(multipart_upload),
                    upload_id: upload_id.to_owned(),
                    request_payer: self.request_payer.to_owned(),
                }
            ).sync() {
                Ok(_) => {
                    info!("Completed upload");
                    Ok(())
                },
                Err(err) => {
                    error!("Failed to complete upload: {}", err);
                    Err(Error::CompleteMultipartUpload)
                }
            }
    }

    fn create_multipart_upload(&self, filename: &str) -> Result<String> {
        match self.s3_client.create_multipart_upload(
            CreateMultipartUploadRequest {
                bucket: self.bucket_name.clone(),
                key: filename.to_owned(),
                ..Default::default()
            }).sync()
            {
                Ok(result) => {
                    match result.upload_id {
                        Some(upload_id) => Ok(upload_id),
                        None => Err(Error::CreateMultipartUpload)
                    }
                },
                Err(err) => {
                    error!("Failed to create multipart upload for {}: {}", filename, err);
                    Err(Error::CreateMultipartUpload)
                }
            }
    }

    fn abort_multipart_upload(&self, filename: &str, upload_id: &str) {
        match self.s3_client.abort_multipart_upload(AbortMultipartUploadRequest {
            bucket: self.bucket_name.to_owned(),
            key: filename.to_owned(),
            upload_id: upload_id.into(),
            request_payer: self.request_payer.to_owned(),
        }).sync() {
            Ok(_) => warn!("Aborted upload of {} (upload id: {})", filename, upload_id),
            Err(err) => warn!("Failed to abort upload of {} (upload id: {}): {}", filename, upload_id, err),
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    CreateMultipartUpload,
    UploadPart,
    CompleteMultipartUpload,
    Read(IOError),
}

