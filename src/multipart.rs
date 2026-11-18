use surrealism::surrealism;

#[allow(clippy::module_inception)]
#[surrealism]
mod multipart {
    use anyhow::{Result, anyhow, ensure};
    use surrealdb_types::{Duration, SurrealValue};

    use crate::{
        client::AwsClient, root::PresignedResponse, sign::CustomPresignedBuilder,
        util::PresignedConfig,
    };

    #[surrealism(comment = "Returns a URI to create a multipart upload for the specified object")]
    async fn create(
        bucket: String,
        key: String,
        expires_in: Duration,
    ) -> Result<PresignedResponse> {
        async {
            let config = PresignedConfig::new(expires_in)?;
            let method = "POST".to_string();

            let uri = CustomPresignedBuilder::from(&mut AwsClient::new().await?)
                .bucket(bucket)
                .key(key)
                .config(&config)
                .method(&method)
                .build(async |url| {
                    url.set_query(Some("uploads"));
                })
                .await?
                .to_string();

            anyhow::Ok(PresignedResponse {
                method,
                uri,
                expires_at: config.expires_at,
            })
        }
        .await
        .map_err(|e| anyhow!("Failed to create presigned multipart create URI: {e:#}"))
    }

    #[surrealism(
        comment = "Returns URIs for PUT actions to upload a range of parts for the specified multipart upload"
    )]
    async fn upload(
        bucket: String,
        key: String,
        upload_id: String,
        start: i32,
        count: i32,
        expires_in: Duration,
        part_size: Option<i64>,
    ) -> Result<UploadResponse> {
        const MIN_PART_NUMBER: i32 = 1;
        const MAX_PART_NUMBER: i32 = 10000;

        async {
            let config = PresignedConfig::new(expires_in)?;

            ensure!(
                (MIN_PART_NUMBER..=MAX_PART_NUMBER).contains(&start),
                "`start` must be between {MIN_PART_NUMBER} and {MAX_PART_NUMBER}, got {start}"
            );
            ensure!(count >= 0, "`count` must not be negative, got {count}");

            let end = start
                .checked_add(count)
                .filter(|end| (end - 1) <= MAX_PART_NUMBER)
                .ok_or_else(|| anyhow!(
                    "start ({start}) + count ({count}) exceeds the maximum part number {MAX_PART_NUMBER}"
                ))?;

            let mut parts = Vec::with_capacity(count as usize);
            for i in start..end {
                let uri = AwsClient::new()
                    .await?
                    .upload_part()
                    .bucket(bucket.clone())
                    .key(key.clone())
                    .part_number(i)
                    .upload_id(upload_id.clone())
                    .set_content_length(part_size)
                    .presigned(config.inner.clone())
                    .await?
                    .uri()
                    .to_string();

                parts.push(SignedPart {
                    part_number: i,
                    uri,
                });
            }

            anyhow::Ok(UploadResponse {
                parts,
                expires_at: config.expires_at,
            })
        }
        .await
        .map_err(|e| anyhow!("Failed to create presigned multipart upload URIs: {e:#}"))
    }

    #[surrealism(comment = "Returns a URI to complete the specified multipart upload")]
    async fn complete(
        bucket: String,
        key: String,
        upload_id: String,
        expires_in: Duration,
    ) -> Result<PresignedResponse> {
        async {
            let config = PresignedConfig::new(expires_in)?;
            let method = "POST".to_string();

            let uri = CustomPresignedBuilder::from(&mut AwsClient::new().await?)
                .bucket(bucket)
                .key(key)
                .config(&config)
                .method(&method)
                .build(async |url| {
                    url.query_pairs_mut().append_pair("uploadId", &upload_id);
                })
                .await?
                .to_string();

            anyhow::Ok(PresignedResponse {
                method,
                uri,
                expires_at: config.expires_at,
            })
        }
        .await
        .map_err(|e| anyhow!("Failed to create presigned multipart complete URI: {e:#}"))
    }

    #[surrealism(comment = "Returns a URI to abort the specified multipart upload")]
    async fn abort(
        bucket: String,
        key: String,
        upload_id: String,
        expires_in: Duration,
    ) -> Result<PresignedResponse> {
        async {
            let config = PresignedConfig::new(expires_in)?;
            let method = "DELETE".to_string();

            let uri = CustomPresignedBuilder::from(&mut AwsClient::new().await?)
                .bucket(bucket)
                .key(key)
                .config(&config)
                .method(&method)
                .build(async |url| {
                    url.query_pairs_mut().append_pair("uploadId", &upload_id);
                })
                .await?
                .to_string();

            anyhow::Ok(PresignedResponse {
                method,
                uri,
                expires_at: config.expires_at,
            })
        }
        .await
        .map_err(|e| anyhow!("Failed to create presigned multipart abort URI: {e:#}"))
    }

    #[derive(SurrealValue)]
    struct UploadResponse {
        parts: Vec<SignedPart>,
        expires_at: String,
    }

    #[derive(SurrealValue)]
    struct SignedPart {
        part_number: i32,
        uri: String,
    }
}
