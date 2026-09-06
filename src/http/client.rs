use std::{
    net::{IpAddr, SocketAddr},
    ptr,
};

use anyhow::{Result, anyhow, ensure};
use bytes::Bytes;
use http::{HeaderValue, Request, Response, uri::Scheme};
use http_body_util::{BodyExt, Full};
use rustls::pki_types::ServerName;
use tokio::{io, task::JoinSet};

use crate::dns::DnsClient;

use super::stream::HttpStream;

pub struct HttpClient;

impl HttpClient {
    pub async fn request(req: Request<Bytes>) -> Result<Response<Bytes>> {
        /// returns true if [s] contains only http conform characters
        fn is_valid_name(s: &str) -> Result<&str> {
            ensure!(
                s.bytes().any(|b| b >= 32 && b != 127 || b == b'\t'),
                "A host may only contain valid http characters"
            );

            Ok(s)
        }

        let host_box = Box::into_pin(
            req.uri()
                .host()
                .ok_or(anyhow!("Request uri is required to contain a host"))
                .and_then(is_valid_name)?
                .to_string()
                .into_boxed_str(),
        );

        let host = unsafe { &*ptr::from_ref(&*host_box) as &'static str };

        let is_https = req.uri().scheme() != Some(&Scheme::HTTP);

        let (port, is_custom) = req
            .uri()
            .port_u16()
            .map_or_else(|| (if is_https { 443 } else { 80 }, false), |p| (p, true));

        let pretty_host = {
            let schema = if is_https { "https://" } else { "http://" };
            if is_custom {
                format!("{schema}{host}:{port}")
            } else {
                format!("{schema}{host}")
            }
        };

        let server_name = ServerName::try_from(host)
            .map_err(|_| anyhow!("Failed to parse {host} as ip or domain name."))?;

        let mut stream = Self::connect(server_name, port, is_https, &pretty_host).await?;

        let (parts, body) = req.into_parts();
        let response = stream
            .send(Request::from_parts(parts, Full::new(body)), || {
                let s = if is_custom {
                    format!("{host}:{port}")
                } else {
                    host.to_string()
                };

                // character compatibility was already checked.
                unsafe { HeaderValue::from_maybe_shared_unchecked(s) }
            })
            .await?;

        let (parts, body) = response.into_parts();
        let body = body.collect().await?.to_bytes();
        Ok(Response::from_parts(parts, body))
    }

    async fn connect(
        server_name: ServerName<'static>,
        port: u16,
        is_https: bool,
        pretty_host: &str,
    ) -> Result<HttpStream> {
        let (ips, host_is_domain) = match &server_name {
            ServerName::IpAddress(ip) => (Box::new([(*ip).into()]) as Box<[IpAddr]>, false),
            ServerName::DnsName(host) => (DnsClient::global().await.resolve(&host).await?, true),
            _ => unreachable!(),
        };

        let mut set = JoinSet::new();

        for ip in ips {
            let server_name = server_name.clone();
            set.spawn(async move {
                HttpStream::connect(SocketAddr::new(ip, port), server_name, is_https)
                    .await
                    .map(|stream| (stream, ip))
            });
        }

        let mut errors = Vec::new();
        while let Some(res) = set.join_next().await {
            match res.map_err(anyhow::Error::from).and_then(|inner| inner) {
                Ok((stream, ip)) => {
                    set.abort_all();

                    if host_is_domain {
                        println!("Connected to `{}` via {ip}", pretty_host);
                    }

                    if !errors.is_empty() {
                        eprintln!(
                            "Received errors from connection attempts to `{}` {:?}",
                            pretty_host, errors
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
            Err(anyhow!("`{}` is unreachable", pretty_host))
        } else {
            Err(anyhow!(
                "Failed to establish connection with `{}` reason: {:?}",
                pretty_host,
                errors
            ))
        }
    }
}
