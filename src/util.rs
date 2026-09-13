use std::time::{Duration, UNIX_EPOCH};

use anyhow::anyhow;
use aws_sdk_s3::presigning::PresigningConfig as AwsPresignedConfig;
use chrono::{DateTime, SecondsFormat, Utc};

use crate::result::Result;

pub trait GetTimestamp {
    fn get_timestamp(&self) -> Result<String>;
}

impl GetTimestamp for AwsPresignedConfig {
    fn get_timestamp(&self) -> Result<String> {
        let start = {
            let secs = self
                .start_time()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| anyhow!("start time must be after 1970-01-01 00:00:00 UTC"))?
                .as_secs();
            UNIX_EPOCH + Duration::from_secs(secs)
        };
        let duration = Duration::from_secs(self.expires().as_secs());

        let expires_at = start
            .checked_add(duration)
            .ok_or(anyhow!("expires_in is too small"))?;

        let string = DateTime::<Utc>::from(expires_at).to_rfc3339_opts(SecondsFormat::Secs, true);

        Ok(string)
    }
}

pub struct PresignedConfig {
    pub inner: AwsPresignedConfig,
    pub expires_at: String,
}

impl PresignedConfig {
    pub fn new(expires_in: surrealdb_types::Duration) -> Result<Self> {
        let inner = AwsPresignedConfig::expires_in(expires_in.into())?;
        let expires_at = inner.get_timestamp()?;

        Ok(Self { inner, expires_at })
    }
}
