#![surrealism]

use std::borrow::Cow;

use anyhow::anyhow;
use surrealdb_types::{Duration, SurrealValue};

use crate::{aws_sdk::public_aws_client, result::Result, util::PresignedConfig};

#[surrealism(comment = "Returns a URI for a PUT action on the specified object")]
async fn put(
    bucket: String,
    key: String,
    expires_in: Duration,
    content_size: Option<i64>,
) -> Result<PresignedResponse> {
    let config = PresignedConfig::new(expires_in)
        .map_err(|e| anyhow!("Failed to create presigned uri: {e}"))?;

    let uri = public_aws_client()
        .put_object()
        .bucket(bucket)
        .key(key)
        .set_content_length(content_size)
        .presigned(config.inner)
        .await?
        .uri()
        .to_string();

    Ok(PresignedResponse {
        method: Cow::Borrowed("PUT"),
        uri,
        expires_at: config.expires_at,
    })
}

#[surrealism(comment = "Returns a URI for a GET action on the specified object")]
async fn get(bucket: String, key: String, expires_in: Duration) -> Result<PresignedResponse> {
    let config = PresignedConfig::new(expires_in)
        .map_err(|e| anyhow!("Failed to create presigned uri: {e}"))?;

    let uri = public_aws_client()
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(config.inner)
        .await?
        .uri()
        .to_string();

    Ok(PresignedResponse {
        method: Cow::Borrowed("GET"),
        uri,
        expires_at: config.expires_at,
    })
}

#[surrealism(comment = "Returns a URI for a HEAD action on the specified object")]
async fn head(bucket: String, key: String, expires_in: Duration) -> Result<PresignedResponse> {
    let config = PresignedConfig::new(expires_in)
        .map_err(|e| anyhow!("Failed to create presigned uri: {e}"))?;

    let uri = public_aws_client()
        .head_object()
        .bucket(bucket)
        .key(key)
        .presigned(config.inner)
        .await?
        .uri()
        .to_string();

    Ok(PresignedResponse {
        method: Cow::Borrowed("HEAD"),
        uri,
        expires_at: config.expires_at,
    })
}

#[surrealism(comment = "Returns a URI for a DELETE action on the specified object")]
async fn delete(bucket: String, key: String, expires_in: Duration) -> Result<PresignedResponse> {
    let config = PresignedConfig::new(expires_in)
        .map_err(|e| anyhow!("Failed to create presigned uri: {e}"))?;

    let uri = public_aws_client()
        .delete_object()
        .bucket(bucket)
        .key(key)
        .presigned(config.inner)
        .await?
        .uri()
        .to_string();

    Ok(PresignedResponse {
        method: Cow::Borrowed("DELETE"),
        uri,
        expires_at: config.expires_at,
    })
}

#[derive(SurrealValue)]
struct PresignedResponse {
    method: Cow<'static, str>,
    uri: String,
    expires_at: String,
}
