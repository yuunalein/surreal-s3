use std::net::SocketAddr;

use anyhow::Result;
use surrealdb_types::SurrealValue;
use surrealism::{sql, surrealism};

use crate::dns::DnsClient;

#[derive(SurrealValue)]
struct Config {
    name_servers: Option<Vec<String>>,
    enable_default_name_servers: Option<bool>,
}

#[surrealism(init)]
fn init() -> Result<()> {
    let conf: Option<Config> = sql("RETURN $s3_module_config")?;

    if let Some(conf) = conf {
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
            println!(
                "No name-servers are being used, this will prevent any non-ip host resolution"
            );
            DnsClient::set_global(DnsClient::without_default_ns(&[])?)?
        }
    } else {
        println!("No config available");
    }

    Ok(())
}
