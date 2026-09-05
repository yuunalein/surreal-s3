use anyhow::Result;
use surrealism::surrealism;

use crate::http::{HttpClient, Request};

mod config;
mod dns;
mod http;

#[surrealism(comment = "greets everyone")]
async fn hello() -> Result<String> {
    let response = HttpClient::request(
        Request::post("https://httpbin.org/post")
            .header("target", "the whole world")
            .body((b"Hello world" as &[u8]).into())?,
    )
    .await?;

    Ok(String::from_utf8_lossy(response.body()).to_string())
}
