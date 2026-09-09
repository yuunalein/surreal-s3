use std::fmt::{Debug, Display};

use aws_sdk_s3::{
    config::{
        HttpClient as AwsHttpClient, RuntimeComponents,
        http::{HttpRequest, HttpResponse},
    },
    error::ConnectorError,
};
use aws_smithy_runtime_api::client::http::{
    HttpConnector, HttpConnectorFuture, HttpConnectorSettings, SharedHttpConnector,
};
use aws_smithy_types::body::SdkBody;
use http_body_util::BodyExt;

use crate::http::{HttpClient, Request, Response};

#[derive(Debug, Clone, Copy)]
pub struct HttpClientAwsConnector;

impl AwsHttpClient for HttpClientAwsConnector {
    fn http_connector(
        &self,
        _settings: &HttpConnectorSettings,
        _components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        SharedHttpConnector::new(*self)
    }
}

impl HttpConnector for HttpClientAwsConnector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        HttpConnectorFuture::new(async move {
            let request = request
                .try_into_http1x()
                .map_err(|e| ConnectorError::user(e.into()))?;

            let (parts, body) = request.into_parts();
            let body = body
                .collect()
                .await
                .map_err(ConnectorError::user)?
                .to_bytes();

            let response = HttpClient::request(Request::from_parts(parts, body))
                .await
                .map_err(|e| ConnectorError::io(HttpCLientErrorWrapper(e).into()))?;

            let (parts, body) = response.into_parts();
            let response = Response::from_parts(parts, SdkBody::from(body));

            HttpResponse::try_from(response).map_err(|e| ConnectorError::other(e.into(), None))
        })
    }
}

pub struct HttpCLientErrorWrapper(pub anyhow::Error);

impl Display for HttpCLientErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for HttpCLientErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl std::error::Error for HttpCLientErrorWrapper {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}
