use std::fmt::Debug;

use anyhow::anyhow;
use aws_sdk_s3::{
    error::{ProvideErrorMetadata, SdkError},
    primitives::ByteStream,
};
use surrealdb_types::Bytes;
use surrealism::surrealism;

use crate::{
    aws_sdk::aws_client,
    result::{Error, Result},
};

#[surrealism(comment = "Returns the contents of the specified object")]
async fn get(bucket: String, key: String) -> Result<Bytes> {
    let body = aws_client()
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(object_exists)?
        .body;

    Ok(Bytes::from(body.collect().await?.into_bytes()))
}

#[surrealism(comment = "Uploads content to the specified object")]
async fn put(bucket: String, key: String, content: Bytes) -> Result<()> {
    aws_client()
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(content.into_inner()))
        .send()
        .await
        .map_err(object_exists)?;

    Ok(())
}

#[surrealism(comment = "Deletes the specified object")]
async fn delete(bucket: String, key: String) -> Result<()> {
    aws_client()
        .delete_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(object_exists)?;

    Ok(())
}

fn object_exists<E, R>(e: SdkError<E, R>) -> Error
where
    E: std::error::Error + ProvideErrorMetadata + Send + Sync + 'static,
    R: Debug + Send + Sync + 'static,
{
    if let Some(se) = e.as_service_error()
        && let Some(code) = se.code()
    {
        match code {
            "NoSuchBucket" => return anyhow!("The specified bucket does not exist").into(),
            "NoSuchKey" => return anyhow!("The specified key does not exist").into(),
            _ => (),
        }
    }

    e.into()
}
