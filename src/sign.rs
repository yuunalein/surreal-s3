use std::{borrow::Cow, marker::PhantomData};

use anyhow::{Context, Result, ensure};
use aws_sdk_s3::presigning::PresigningConfig as AwsPresignedConfig;
use aws_sigv4::{
    http_request::{
        PercentEncodingMode, SignableBody, SignableRequest, SignatureLocation, SigningSettings,
        UriPathNormalizationMode, sign,
    },
    sign::v4,
};
use aws_smithy_runtime_api::client::identity::Identity;
use surrealdb_types::Duration;
use url::Url;

use crate::{client::AwsClient, util::PresignedConfig};

pub struct CustomPresignedBuilder<'a, A, B, C, D> {
    client: &'a mut AwsClient,
    bucket: Option<Cow<'a, str>>,
    key: Option<Cow<'a, str>>,
    config: Option<&'a PresignedConfig>,
    method: Option<Cow<'a, str>>,
    phantom: PhantomData<(A, B, C, D)>,
}

impl<'a> From<&'a mut AwsClient> for CustomPresignedBuilder<'a, Unset, Unset, Unset, Unset> {
    fn from(value: &'a mut AwsClient) -> Self {
        Self {
            client: value,
            bucket: None,
            key: None,
            config: None,
            method: None,
            phantom: PhantomData,
        }
    }
}

impl<'a, Bucket, Key, Expires, Method> CustomPresignedBuilder<'a, Bucket, Key, Expires, Method> {
    fn transform<A, B, C, D>(self) -> CustomPresignedBuilder<'a, A, B, C, D> {
        CustomPresignedBuilder {
            client: self.client,
            bucket: self.bucket,
            key: self.key,
            config: self.config,
            method: self.method,
            phantom: PhantomData,
        }
    }

    pub fn bucket<S: Into<Cow<'a, str>>>(
        mut self,
        input: S,
    ) -> CustomPresignedBuilder<'a, Set, Key, Expires, Method> {
        self.bucket = Some(input.into());
        self.transform()
    }

    pub fn key<S: Into<Cow<'a, str>>>(
        mut self,
        input: S,
    ) -> CustomPresignedBuilder<'a, Bucket, Set, Expires, Method> {
        self.key = Some(input.into());
        self.transform()
    }

    pub fn config(
        mut self,
        input: &'a PresignedConfig,
    ) -> CustomPresignedBuilder<'a, Set, Key, Set, Method> {
        self.config = Some(input);
        self.transform()
    }

    pub fn method<S: Into<Cow<'a, str>>>(
        mut self,
        input: S,
    ) -> CustomPresignedBuilder<'a, Bucket, Key, Expires, Set> {
        self.method = Some(input.into());
        self.transform()
    }
}

impl<'a> CustomPresignedBuilder<'a, Set, Set, Set, Set> {
    #[allow(private_bounds)]
    pub async fn build<R: IntoResult>(
        self,
        query_fn: impl AsyncFnOnce(&mut Url) -> R,
    ) -> Result<Url> {
        let mut url = self.base_url().await?;

        query_fn(&mut url).await.into_result()?;

        self.sign_url(url)
    }

    async fn base_url(&self) -> Result<Url> {
        let bucket = unsafe { self.bucket.as_deref().unwrap_unchecked() };
        let key = unsafe { self.key.as_deref().unwrap_unchecked() };

        ensure!(!bucket.is_empty(), "`bucket` must not be empty");
        ensure!(!key.is_empty(), "`key` must not be empty");

        let base = self
            .client
            .put_object()
            .bucket(bucket)
            .key(key)
            .presigned(AwsPresignedConfig::expires_in(
                Duration::from_secs(1).into(),
            )?)
            .await?;

        let mut url =
            Url::parse(base.uri()).context("Unable to parse url returned from AWS SDK")?;
        url.set_query(None);

        Ok(url)
    }

    fn sign_url(self, mut url: Url) -> Result<Url> {
        let aws_config = self.client.config();
        let region = aws_config
            .region()
            .context("client has no region")?
            .to_string();
        let identity: Identity = self.client.credentials().clone().into();
        let signing_config = unsafe { self.config.unwrap_unchecked() };

        let mut settings = SigningSettings::default();
        settings.signature_location = SignatureLocation::QueryParams;
        settings.expires_in = Some(signing_config.inner.expires());
        settings.percent_encoding_mode = PercentEncodingMode::Single;
        settings.uri_path_normalization_mode = UriPathNormalizationMode::Disabled;

        let params = v4::SigningParams::builder()
            .identity(&identity)
            .region(&region)
            .name("s3")
            .time(signing_config.inner.start_time())
            .settings(settings)
            .build()?
            .into();

        let method = unsafe { self.method.unwrap_unchecked() };
        let request = SignableRequest::new(
            &method,
            url.as_str(),
            std::iter::empty(),
            SignableBody::UnsignedPayload,
        )?;
        let (instructions, _signature) = sign(request, &params)?.into_parts();

        {
            let mut query = url.query_pairs_mut();
            for (name, value) in instructions.params() {
                query.append_pair(name, value);
            }
        }

        Ok(url)
    }
}

pub struct Unset;
pub struct Set;

trait IntoResult {
    fn into_result(self) -> Result<()>;
}

impl IntoResult for () {
    fn into_result(self) -> Result<()> {
        Ok(())
    }
}

impl IntoResult for Result<()> {
    fn into_result(self) -> Result<()> {
        self
    }
}
