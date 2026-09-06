use std::sync::{Arc, LazyLock};

use anyhow::Result;
use bytes::Bytes;
use http::{HeaderValue, Request, header};
use http_body_util::Full;
use hyper::{
    body::Incoming,
    client::conn::{http1, http2},
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::{ClientConfig, pki_types::ServerName};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_rustls::{TlsConnector, client::TlsStream};
use tokio_util::either::Either;

pub enum HttpStream {
    H2(http2::SendRequest<Full<Bytes>>),
    H1(http1::SendRequest<Full<Bytes>>),
}

impl HttpStream {
    pub async fn connect<A: ToSocketAddrs>(
        addr: A,
        name: ServerName<'static>,
        use_tls: bool,
    ) -> Result<Self> {
        let (io, use_h2) = Self::create_io(addr, name, use_tls).await?;

        if use_h2 {
            let (send_request, conn) = http2::handshake(TokioExecutor::new(), io).await?;
            tokio::spawn(conn);

            Ok(Self::H2(send_request))
        } else {
            let (send_request, conn) = http1::handshake(io).await?;
            tokio::spawn(conn);

            Ok(Self::H1(send_request))
        }
    }

    async fn create_io<A: ToSocketAddrs>(
        addr: A,
        name: ServerName<'static>,
        use_tls: bool,
    ) -> Result<(TokioIo<Either<TlsStream<TcpStream>, TcpStream>>, bool)> {
        let tcp_stream = TcpStream::connect(addr).await?;

        Ok(match use_tls {
            true => {
                let tls_stream = TlsConnector::from(CLIENT_CONFIG.clone())
                    .connect(name, tcp_stream)
                    .await?;

                let use_h2 = tls_stream.get_ref().1.alpn_protocol() == Some(b"h2");
                let io = TokioIo::new(Either::Left(tls_stream));

                (io, use_h2)
            }
            false => {
                let io = TokioIo::new(Either::Right(tcp_stream));

                (io, false)
            }
        })
    }

    pub async fn send(
        &mut self,
        mut req: Request<Full<Bytes>>,
        host_value_fn: impl FnOnce() -> HeaderValue,
    ) -> hyper::Result<http::Response<Incoming>> {
        match self {
            Self::H2(sr) => sr.send_request(req).await,
            Self::H1(sr) => {
                // http/1.1 requires a HOST header in every request
                req.headers_mut()
                    .entry(header::HOST)
                    .or_insert_with(host_value_fn);

                sr.send_request(req).await
            }
        }
    }
}

static CLIENT_CONFIG: LazyLock<Arc<ClientConfig>> = LazyLock::new(|| {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("only one provider should be installed");

    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let mut config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Arc::new(config)
});
