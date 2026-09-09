#![surrealism]

use anyhow::anyhow;
use aws_sdk_s3::presigning::PresigningConfig;
use surrealdb_types::Duration;

use crate::{aws_sdk::aws_client, result::Result};

#[surrealism(comment = "Returns a URI for a PUT action on the specified object")]
async fn put(bucket: String, key: String, expires_in: Duration) -> Result<String> {
    let uri = aws_client()
        .put_object()
        .bucket(bucket)
        .key(key)
        .presigned(presigning_config(expires_in)?)
        .await?
        .uri()
        .to_string();

    Ok(uri)
}

#[surrealism(comment = "Returns a URI for a GET action on the specified object")]
async fn get(bucket: String, key: String, expires_in: Duration) -> Result<String> {
    let uri = aws_client()
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(presigning_config(expires_in)?)
        .await?
        .uri()
        .to_string();

    Ok(uri)
}

#[surrealism(comment = "Returns a URI for a HEAD action on the specified object")]
async fn head(bucket: String, key: String, expires_in: Duration) -> Result<String> {
    let uri = aws_client()
        .head_object()
        .bucket(bucket)
        .key(key)
        .presigned(presigning_config(expires_in)?)
        .await?
        .uri()
        .to_string();

    Ok(uri)
}

#[surrealism(comment = "Returns a URI for a DELETE action on the specified object")]
async fn delete(bucket: String, key: String, expires_in: Duration) -> Result<String> {
    let uri = aws_client()
        .delete_object()
        .bucket(bucket)
        .key(key)
        .presigned(presigning_config(expires_in)?)
        .await?
        .uri()
        .to_string();

    Ok(uri)
}

fn presigning_config(expires_in: Duration) -> Result<PresigningConfig> {
    let config = PresigningConfig::expires_in(expires_in.into())
        .map_err(|e| anyhow!("Failed to create presigned uri: {e}"))?;

    Ok(config)
}
