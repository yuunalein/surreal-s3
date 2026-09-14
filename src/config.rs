use std::net::SocketAddr;

use surrealdb_types::SurrealValue;
use surrealism::{sql, surrealism};

use crate::{
    aws_sdk::client::{AwsClientConfig, set_aws_client},
    dns::DnsClient,
    result::Result,
};

#[derive(SurrealValue)]
struct Config {
    name_servers: Option<Vec<String>>,
    enable_default_name_servers: Option<bool>,
    endpoint_url: String,
    public_endpoint_url: Option<String>,
    region: Option<String>,
    access_key_id: String,
    secret_access_key: String,
    force_path_style: Option<bool>,
    public_force_path_style: Option<bool>,
}

#[surrealism(init)]
fn init() -> Result<()> {
    let conf: Config = sql("RETURN $s3_module_config")?;

    if let Some(name_servers) = conf.name_servers {
        let name_servers: Vec<SocketAddr> = name_servers
            .iter()
            .filter_map(|s| match s.parse::<SocketAddr>() {
                Ok(a) => Some(a),
                Err(e) => {
                    eprintln!("Failed to parse `{s}` as name-server during init: {e}");
                    None
                }
            })
            .collect();

        let client = if conf.enable_default_name_servers.unwrap_or(true) {
            println!("Default name-servers are enabled");
            DnsClient::new(&name_servers)?
        } else {
            println!("Default name-servers are disabled");
            DnsClient::without_default_ns(&name_servers)?
        };
        println!("User defined name-servers: {:?}", name_servers);

        DnsClient::set_global(client)?;
    } else if !conf.enable_default_name_servers.unwrap_or(true) {
        println!("No name-servers are being used, this will prevent any non-ip host resolution");
        DnsClient::set_global(DnsClient::without_default_ns(&[])?)?
    }

    set_aws_client(AwsClientConfig {
        endpoint_url: conf.endpoint_url,
        public_endpoint_url: conf.public_endpoint_url,
        region: conf.region,
        access_key_id: conf.access_key_id,
        secret_access_key: conf.secret_access_key,
        force_path_style: conf.force_path_style,
        public_force_path_style: conf.public_force_path_style,
    })?;

    Ok(())
}
