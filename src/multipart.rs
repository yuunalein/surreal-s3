#![surrealism]

use std::fmt::Debug;

use anyhow::anyhow;
use aws_sdk_s3::{
    error::{ProvideErrorMetadata, SdkError},
    types::{CompletedMultipartUpload, CompletedPart},
};
use surrealdb_types::SurrealValue;
use tokio::task::JoinSet;

use crate::{
    aws_sdk::aws_client,
    result::{Error, Result},
    util::PresignedConfig,
};

#[surrealism]
async fn create(bucket: String, key: String) -> Result<String> {
    let upload_id = aws_client()
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .send()
        .await?
        .upload_id()
        .ok_or(anyhow!("Failed to get multipart upload id"))?
        .to_string();

    Ok(upload_id)
}

#[surrealism]
async fn uri(
    bucket: String,
    key: String,
    upload_id: String,
    start: i32,
    count: i32,
    expires_in: surrealdb_types::Duration,
) -> Result<UriResponse> {
    let config = PresignedConfig::new(expires_in)
        .map_err(|e| anyhow!("Failed to create presigned uri: {e}"))?;

    let mut set = JoinSet::new();
    for i in 0..count {
        let i = start + i;
        let bucket = bucket.clone();
        let key = key.clone();
        let upload_id = upload_id.clone();
        let presigning_config = config.inner.clone();
        set.spawn(async move {
            let uri = aws_client()
                .upload_part()
                .bucket(bucket)
                .key(key)
                .part_number(i)
                .upload_id(upload_id)
                .presigned(presigning_config)
                .await?
                .uri()
                .to_string();

            Ok(SignedPart {
                part_number: i,
                uri,
            })
        });
    }

    let parts = set
        .join_all()
        .await
        .into_iter()
        .collect::<Result<Vec<SignedPart>>>()?;

    Ok(UriResponse {
        parts,
        expires_at: config.expires_at,
    })
}

#[surrealism]
async fn complete(
    bucket: String,
    key: String,
    upload_id: String,
    parts: Vec<CompletePart>,
) -> Result<()> {
    let completed_parts = parts
        .into_iter()
        .map(|p| {
            CompletedPart::builder()
                .part_number(p.part_number)
                .e_tag(p.e_tag)
                .build()
        })
        .collect();

    aws_client()
        .complete_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .multipart_upload(
            CompletedMultipartUpload::builder()
                .set_parts(Some(completed_parts))
                .build(),
        )
        .send()
        .await
        .map_err(multipart_exists)?;

    Ok(())
}

#[surrealism]
async fn abort(bucket: String, key: String, upload_id: String) -> Result<()> {
    aws_client()
        .abort_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .send()
        .await
        .map_err(multipart_exists)?;

    Ok(())
}

fn multipart_exists<E, R>(e: SdkError<E, R>) -> Error
where
    E: std::error::Error + ProvideErrorMetadata + Send + Sync + 'static,
    R: Debug + Send + Sync + 'static,
{
    if let Some(se) = e.as_service_error()
        && let Some(code) = se.code()
        && code == "NoSuchUpload"
    {
        anyhow!("The specified multipart upload does not exist").into()
    } else {
        e.into()
    }
}

#[derive(SurrealValue)]
struct CompletePart {
    part_number: i32,
    e_tag: String,
}

#[derive(SurrealValue)]
struct UriResponse {
    parts: Vec<SignedPart>,
    expires_at: String,
}

#[derive(SurrealValue)]
struct SignedPart {
    part_number: i32,
    uri: String,
}
