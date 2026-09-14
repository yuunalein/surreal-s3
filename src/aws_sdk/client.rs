use std::sync::OnceLock;

use anyhow::{Result, anyhow};
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client, config::Credentials};

use super::http::HttpClientAwsConnector;

static AWS_CLIENT: OnceLock<(Client, Option<Client>)> = OnceLock::new();

pub fn aws_client() -> Client {
    AWS_CLIENT
        .get()
        .expect("AwsClient must first be initialized before it can be used")
        .0
        .clone()
}

pub fn public_aws_client() -> Client {
    let (client, public_client) = AWS_CLIENT
        .get()
        .expect("AwsClient must first be initialized before it can be used");

    public_client.clone().unwrap_or(client.clone())
}

pub fn set_aws_client(config: AwsClientConfig) -> Result<()> {
    fn build_client<S: Into<String>>(
        url: S,
        force_path_style: Option<bool>,
        config: &AwsClientConfig,
    ) -> Client {
        Client::from_conf(
            aws_sdk_s3::Config::builder()
                .http_client(HttpClientAwsConnector)
                .endpoint_url(url)
                .region(Region::new(
                    config.region.clone().unwrap_or("us-east-1".to_string()),
                ))
                .credentials_provider(Credentials::new(
                    config.access_key_id.clone(),
                    config.secret_access_key.clone(),
                    None,
                    None,
                    "config",
                ))
                .force_path_style(force_path_style.unwrap_or(false))
                .behavior_version(BehaviorVersion::v2026_01_12())
                .build(),
        )
    }

    AWS_CLIENT
        .set((
            build_client(&config.endpoint_url, config.force_path_style, &config),
            config.public_endpoint_url.clone().map(|url| {
                build_client(
                    url,
                    config.public_force_path_style.or(config.force_path_style),
                    &config,
                )
            }),
        ))
        .map_err(|_| anyhow!("AwsClient was already set"))
}

pub struct AwsClientConfig {
    pub endpoint_url: String,
    pub public_endpoint_url: Option<String>,
    pub region: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub force_path_style: Option<bool>,
    pub public_force_path_style: Option<bool>,
}
