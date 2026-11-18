use anyhow::{Result, anyhow};
use surrealdb_types::{Duration, SurrealValue};
use surrealism::surrealism;

use crate::{client::AwsClient, util::PresignedConfig};

#[surrealism(comment = "Returns a URI for a PUT action on the specified object")]
async fn put(
    bucket: String,
    key: String,
    expires_in: Duration,
    content_size: Option<i64>,
) -> Result<PresignedResponse> {
    async {
        let config = PresignedConfig::new(expires_in)?;

        let uri = AwsClient::new()
            .await?
            .put_object()
            .bucket(bucket)
            .key(key)
            .set_content_length(content_size)
            .presigned(config.inner)
            .await?
            .uri()
            .to_string();

        anyhow::Ok(PresignedResponse {
            method: "PUT".to_string(),
            uri,
            expires_at: config.expires_at,
        })
    }
    .await
    .map_err(|e| anyhow!("Failed to create presigned PUT URI: {e:#}"))
}

#[surrealism(comment = "Returns a URI for a GET action on the specified object")]
async fn get(bucket: String, key: String, expires_in: Duration) -> Result<PresignedResponse> {
    async {
        let config = PresignedConfig::new(expires_in)?;

        let uri = AwsClient::new()
            .await?
            .get_object()
            .bucket(bucket)
            .key(key)
            .presigned(config.inner)
            .await?
            .uri()
            .to_string();

        anyhow::Ok(PresignedResponse {
            method: "GET".to_string(),
            uri,
            expires_at: config.expires_at,
        })
    }
    .await
    .map_err(|e| anyhow!("Failed to create presigned GET URI: {e:#}"))
}

#[surrealism(comment = "Returns a URI for a HEAD action on the specified object")]
async fn head(bucket: String, key: String, expires_in: Duration) -> Result<PresignedResponse> {
    async {
        let config = PresignedConfig::new(expires_in)?;

        let uri = AwsClient::new()
            .await?
            .head_object()
            .bucket(bucket)
            .key(key)
            .presigned(config.inner)
            .await?
            .uri()
            .to_string();

        anyhow::Ok(PresignedResponse {
            method: "HEAD".to_string(),
            uri,
            expires_at: config.expires_at,
        })
    }
    .await
    .map_err(|e| anyhow!("Failed to create presigned HEAD URI: {e:#}"))
}

#[surrealism(comment = "Returns a URI for a DELETE action on the specified object")]
async fn delete(bucket: String, key: String, expires_in: Duration) -> Result<PresignedResponse> {
    async {
        let config = PresignedConfig::new(expires_in)?;

        let uri = AwsClient::new()
            .await?
            .delete_object()
            .bucket(bucket)
            .key(key)
            .presigned(config.inner)
            .await?
            .uri()
            .to_string();

        anyhow::Ok(PresignedResponse {
            method: "DELETE".to_string(),
            uri,
            expires_at: config.expires_at,
        })
    }
    .await
    .map_err(|e| anyhow!("Failed to create presigned DELETE URI: {e:#}"))
}

#[derive(SurrealValue)]
pub struct PresignedResponse {
    pub method: String,
    pub uri: String,
    pub expires_at: String,
}
