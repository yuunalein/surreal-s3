use std::ops::{Deref, DerefMut};

use anyhow::Result;
use aws_sdk_s3::{
    Client, Config,
    config::{Credentials, IdentityCache, Region, StalledStreamProtectionConfig},
};
use surrealism::binding;

pub struct AwsClient {
    client: Client,
    credentials: Credentials,
}

impl AwsClient {
    pub async fn new() -> Result<Self> {
        let region: String = binding("region").await?;
        let endpoint: Option<String> = binding("endpoint").await?;
        let force_path_style: Option<bool> = binding("force_path_style").await?;
        let access_key: String = binding("access_key").await?;
        let secret_key: String = binding("secret_key").await?;
        let session_token: Option<String> = binding("session_token").await?;

        let credentials = Credentials::new(access_key, secret_key, session_token, None, "bindings");

        let mut conf = Config::builder()
            .region(Region::new(region))
            .credentials_provider(credentials.clone())
            .identity_cache(IdentityCache::no_cache())
            .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
            .behavior_version_latest();

        conf.set_endpoint_url(endpoint);
        conf.set_force_path_style(force_path_style);

        Ok(Self {
            client: Client::from_conf(conf.build()),
            credentials,
        })
    }

    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}

impl Deref for AwsClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for AwsClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
