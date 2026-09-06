use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::OnceLock,
};

use anyhow::{Result, anyhow};
use hickory_resolver::{
    Resolver,
    config::{
        ConnectionConfig, NameServerConfig, ResolverConfig, ResolverOpts, ServerOrderingStrategy,
    },
    net::runtime::TokioRuntimeProvider,
};
use tokio::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone)]
pub struct DnsClient {
    resolver: Resolver<TokioRuntimeProvider>,
}

impl DnsClient {
    const DEFAULT_NAME_SERVERS: &[SocketAddr] = &[
        // Docker network
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 11), 53)),
        // systemd-resolved
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 53), 53)),
    ];

    pub fn default() -> Result<Self> {
        Self::without_default_ns(Self::DEFAULT_NAME_SERVERS)
    }

    pub fn new(name_servers: &[SocketAddr]) -> Result<Self> {
        let mut name_servers = name_servers.to_vec();
        name_servers.extend_from_slice(Self::DEFAULT_NAME_SERVERS);

        Self::without_default_ns(&name_servers)
    }

    pub fn without_default_ns(name_servers: &[SocketAddr]) -> Result<Self> {
        let name_servers: Vec<NameServerConfig> = name_servers
            .iter()
            .map(|name_server| {
                let mut conn_config = ConnectionConfig::tcp();
                if name_server.port() != 0 {
                    conn_config.port = name_server.port();
                }

                NameServerConfig::new(name_server.ip(), true, vec![conn_config])
            })
            .collect();

        let mut options = ResolverOpts::default();
        options.server_ordering_strategy = ServerOrderingStrategy::UserProvidedOrder;
        // check all configured ns in parallel so a reachable server is always in round 1
        // a smaller batch can miss it entirely, giving partial or no results
        options.num_concurrent_reqs = name_servers.len();

        let resolver = Resolver::builder_with_config(
            ResolverConfig::from_name_servers(name_servers),
            TokioRuntimeProvider::default(),
        )
        .with_options(options)
        .build()?;

        Ok(Self { resolver })
    }

    pub async fn resolve<S: AsRef<str>>(&mut self, host: S) -> Result<Box<[IpAddr]>> {
        let addrs: Box<[IpAddr]> = self
            .resolver
            .lookup_ip(host.as_ref())
            .await
            .into_iter()
            .flatten()
            .collect();

        anyhow::ensure!(
            !addrs.is_empty(),
            "Could not resolve host `{}`",
            host.as_ref()
        );

        Ok(addrs)
    }

    pub async fn global() -> MutexGuard<'static, Self> {
        let cell = GLOBAL_CLIENT.get_or_init(|| {
            DnsClient::default()
                .expect("Failed to create default DnsClient")
                .into()
        });

        cell.lock().await
    }

    pub fn set_global(client: Self) -> Result<()> {
        GLOBAL_CLIENT
            .set(Mutex::new(client))
            .map_err(|_| anyhow!("DnsClient was already set"))
    }
}

static GLOBAL_CLIENT: OnceLock<Mutex<DnsClient>> = OnceLock::new();
