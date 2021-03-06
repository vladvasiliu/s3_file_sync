extern crate base64;
extern crate md5;
extern crate rusoto_core;
extern crate rusoto_s3;

use std::fs::File as FSFile;
use std::io::Read;
use std::str::FromStr;

use crossbeam_channel::{Receiver, Sender};

use log::{debug, info, warn};
use rusoto_core::Region;
use rusoto_s3::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompletedMultipartUpload,
    CompletedPart, CreateMultipartUploadRequest, S3Client, UploadPartRequest, S3,
};

pub mod error;

use crate::controller::file::File;
use crate::uploader::error::{Error, Result};

pub struct Uploader {
    bucket_name: String,
    s3_client: S3Client,
    request_payer: Option<String>,
    part_size: usize,
    controller_rx: Receiver<File>,
    controller_tx: Sender<(File, Result<()>)>,
}

impl Uploader {
    pub fn new(
        bucket_name: &str,
        region_name: &str,
        controller_rx: Receiver<File>,
        controller_tx: Sender<(File, Result<()>)>,
    ) -> Uploader {
        let region = Region::from_str(region_name).unwrap();
        let s3_client = S3Client::new(region);
        let bucket_name: String = bucket_name.into();

        Uploader {
            bucket_name,
            s3_client,
            request_payer: None,
            part_size: 1024 * 1024 * 100, // 100 MB
            controller_rx,
            controller_tx,
        }
    }

    pub fn run(&self) {
        loop {
            match self.controller_rx.recv() {
                Err(err) => {
                    info!("Channel disconnected, shutting down.");
                    debug!("{}", err);
                    break;
                }
                Ok(file) => {
                    let upload_result = self.upload_file(&file);
                    self.controller_tx
                        .send((file, upload_result))
                        .unwrap_or_else(|err| warn!("Failed to send file to controller: {}", err));
                }
            }
        }
    }

    fn upload_file(&self, file: &File) -> Result<()> {
        let upload_id = self.create_multipart_upload(&file)?;

        self.upload_file_parts(&file, &upload_id)
            .and_then(|multipart_upload| {
                self.complete_multipart_upload(&file, multipart_upload, &upload_id)
            })
            .or_else(|err| {
                self.abort_multipart_upload(&file, &upload_id);
                Err(err)
            })
    }

    fn upload_file_parts(&self, file: &File, upload_id: &str) -> Result<CompletedMultipartUpload> {
        let mut fs_file = FSFile::open(&file.full_path)?;
        let mut part_number = 0;
        let mut completed_parts: Vec<CompletedPart> = Vec::new();

        loop {
            let mut buffer = vec![0; self.part_size];
            part_number += 1;

            match fs_file.read(&mut buffer) {
                Ok(0) => break,
                Ok(len) => {
                    buffer.truncate(len);
                    completed_parts.push(self.upload_part(
                        buffer,
                        file.key.to_str().unwrap().into(),
                        part_number,
                        upload_id,
                    )?);
                }
                Err(err) => {
                    return Err(Error::Read(err));
                }
            }
        }

        Ok(CompletedMultipartUpload {
            parts: Some(completed_parts),
        })
    }

    fn upload_part(
        &self,
        body: Vec<u8>,
        key: &str,
        part_number: i64,
        upload_id: &str,
    ) -> Result<CompletedPart> {
        let content_length = body.len() as i64;
        let digest = md5::compute(&body);
        let content_md5 = base64::encode(digest.as_ref());
        match self
            .s3_client
            .upload_part(UploadPartRequest {
                part_number,
                body: Some(body.into()),
                content_length: Some(content_length),
                content_md5: Some(content_md5),
                bucket: self.bucket_name.to_owned(),
                key: key.to_owned(),
                upload_id: upload_id.to_owned(),
                request_payer: self.request_payer.to_owned(),
                ..Default::default()
            })
            .sync()
        {
            Ok(res) => {
                let e_tag = res.e_tag.unwrap();
                debug!("Uploaded part {} - etag: {}", part_number, e_tag);
                Ok(CompletedPart {
                    part_number: Some(part_number),
                    e_tag: Some(e_tag),
                })
            }
            Err(error) => Err(Error::UploadPart { part_number, error }),
        }
    }

    fn complete_multipart_upload(
        &self,
        file: &File,
        multipart_upload: CompletedMultipartUpload,
        upload_id: &str,
    ) -> Result<()> {
        self.s3_client
            .complete_multipart_upload(CompleteMultipartUploadRequest {
                bucket: self.bucket_name.to_owned(),
                key: file.key.to_str().unwrap().into(),
                multipart_upload: Some(multipart_upload),
                upload_id: upload_id.to_owned(),
                request_payer: self.request_payer.to_owned(),
            })
            .sync()?;
        debug!("Completed upload");
        Ok(())
    }

    fn create_multipart_upload(&self, file: &File) -> Result<String> {
        match self
            .s3_client
            .create_multipart_upload(CreateMultipartUploadRequest {
                bucket: self.bucket_name.clone(),
                key: file.key.to_str().unwrap().into(),
                ..Default::default()
            })
            .sync()
        {
            Ok(result) => match result.upload_id {
                Some(upload_id) => Ok(upload_id),
                None => Err("Didn't get an upload_id".into()),
            },
            Err(err) => Err(Error::CreateMultipartUpload(err)),
        }
    }

    fn abort_multipart_upload(&self, file: &File, upload_id: &str) {
        let key = file.key.to_str().unwrap().to_owned();
        match self
            .s3_client
            .abort_multipart_upload(AbortMultipartUploadRequest {
                bucket: self.bucket_name.to_owned(),
                key: key.to_owned(),
                upload_id: upload_id.into(),
                request_payer: self.request_payer.to_owned(),
            })
            .sync()
        {
            Ok(_) => warn!("Aborted upload of {} (upload id: {})", key, upload_id),
            Err(err) => warn!(
                "Failed to abort upload of {} (upload id: {}): {}",
                key, upload_id, err
            ),
        }
    }
}
