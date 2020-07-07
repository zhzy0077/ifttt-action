use crate::Sink;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;
use std::str::FromStr;

pub struct WebSink {
    config: WebConfig,
}

impl WebSink {
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        WebSink {
            config: WebConfig {
                method: method.into(),
                url: url.into(),
            },
        }
    }
}

#[async_trait(?Send)]
impl Sink for WebSink {
    async fn sink(&self, input: String) -> Result<()> {
        let client = reqwest::Client::new();
        client
            .request(Method::from_str(&self.config.method)?, &self.config.url)
            .body(input)
            .send()
            .await?;

        Ok(())
    }
}

pub struct WebConfig {
    method: String,
    url: String,
}
