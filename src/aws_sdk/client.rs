use std::sync::OnceLock;

use anyhow::{Result, anyhow};
use aws_config::Region;
use aws_sdk_s3::{Client, config::Credentials};

use super::http::HttpClientAwsConnector;

static AWS_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn aws_client() -> Client {
    AWS_CLIENT
        .get()
        .expect("AwsClient must first be initialized before it can be used")
        .clone()
}

pub fn set_aws_client(config: AwsClientConfig) -> Result<()> {
    AWS_CLIENT
        .set(Client::from_conf(
            aws_sdk_s3::Config::builder()
                .http_client(HttpClientAwsConnector)
                .endpoint_url(config.endpoint_url)
                .region(Region::new(config.region))
                .credentials_provider(Credentials::new(
                    config.access_key_id,
                    config.secret_access_key,
                    None,
                    None,
                    "config",
                ))
                .behavior_version_latest()
                .build(),
        ))
        .map_err(|_| anyhow!("AwsClient was already set"))
}

pub struct AwsClientConfig {
    pub endpoint_url: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}
