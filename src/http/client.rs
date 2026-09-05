use std::net::{IpAddr, SocketAddr};

use anyhow::{Result, anyhow};
use bytes::Bytes;
use http::{Request, Response, uri::Scheme};
use http_body_util::{BodyExt, Full};
use rustls::pki_types::ServerName;
use tokio::{io, task::JoinSet};

use crate::dns::DnsClient;

use super::stream::HttpStream;

pub struct HttpClient;

impl HttpClient {
    pub async fn request(req: Request<Bytes>) -> Result<Response<Bytes>> {
        let is_https = req.uri().scheme() != Some(&Scheme::HTTP);

        let host = req
            .uri()
            .host()
            .ok_or_else(|| anyhow::anyhow!("Request uri is required to contain a host"))?;

        let port = req
            .uri()
            .port_u16()
            .unwrap_or(if is_https { 443 } else { 80 });

        let mut stream = Self::connect(host, port, is_https).await?;

        let (parts, body) = req.into_parts();
        let response = stream
            .send(Request::from_parts(parts, Full::new(body)))
            .await?;

        let (parts, body) = response.into_parts();
        let body = body.collect().await?.to_bytes();
        Ok(Response::from_parts(parts, body))
    }

    async fn connect(host: &str, port: u16, use_tls: bool) -> Result<HttpStream> {
        let (ips, server, did_dns) = host
            .parse::<IpAddr>()
            .map(|ip| (vec![ip], ServerName::IpAddress(ip.into()), false))
            .unwrap_or((
                {
                    let resolved = DnsClient::global().await.resolve(&host).await?;
                    println!("Resolved `{host}` to {:?}", resolved);
                    resolved
                },
                ServerName::DnsName(host.to_string().try_into()?),
                true,
            ));

        let mut set = JoinSet::new();
        for ip in ips {
            let server = server.clone();
            set.spawn(async move {
                HttpStream::connect(SocketAddr::new(ip, port), server, use_tls)
                    .await
                    .map(|s| (s, ip))
            });
        }

        let mut errors = Vec::new();
        while let Some(res) = set.join_next().await {
            match res.map_err(anyhow::Error::from).and_then(|inner| inner) {
                Ok((stream, ip)) => {
                    set.abort_all();

                    if did_dns {
                        println!("Connected to `{host}` via {ip}:{port}");
                    }

                    if !errors.is_empty() {
                        eprintln!(
                            "Received errors from connection attempts to `{host}` {:?}",
                            errors
                        )
                    }

                    return Ok(stream);
                }
                Err(e) => {
                    if let Some(io_err) = e.downcast_ref::<io::Error>() {
                        match io_err.kind() {
                            io::ErrorKind::HostUnreachable | io::ErrorKind::NetworkUnreachable => {}
                            _ => errors.push(e),
                        }
                    } else {
                        errors.push(e);
                    }
                }
            }
        }

        if errors.is_empty() {
            Err(anyhow!("`{host}`:{port} is unreachable"))
        } else {
            Err(anyhow!(
                "Failed to establish connection with `{host}`:{port} reason: {:?}",
                errors
            ))
        }
    }
}
