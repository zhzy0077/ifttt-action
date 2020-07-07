use crate::{Feed, Indexable, State};
use anyhow::Result;
use async_trait::async_trait;
use rss::{Channel, Item};
use serde::Deserialize;

const RSS_LAST_LINK: &str = "rss_last_link";

pub struct RssFeed {
    pub config: RssConfig,
}

impl RssFeed {
    pub fn new(url: impl Into<String>, count: usize) -> Self {
        let config = RssConfig {
            url: url.into(),
            count,
        };
        RssFeed { config }
    }
}

#[async_trait]
impl Feed for RssFeed {
    async fn feed(&self, state: &mut State) -> Result<Vec<Box<dyn Indexable>>> {
        let res = reqwest::get(&self.config.url).await?;
        let content = res.bytes().await?;
        let channel = Channel::read_from(&content[..])?;

        let mut news: Vec<Box<dyn Indexable>> = Vec::new();
        let last_news_link = state.get(RSS_LAST_LINK).map(String::as_str);
        let mut latest_link: Option<String> = None;

        for item in channel.into_items().into_iter().take(self.config.count) {
            if item.link() == last_news_link {
                break;
            }

            if latest_link.is_none() {
                latest_link = item.link().map(str::to_string);
            }

            news.push(Box::new(RssOutput(item)));
        }

        if let Some(link) = latest_link {
            state.insert(RSS_LAST_LINK.to_string(), link);
        }

        Ok(news)
    }
}

#[derive(Deserialize)]
pub struct RssConfig {
    pub url: String,
    pub count: usize,
}

pub struct RssOutput(Item);

impl<'a> std::ops::Index<&'a str> for RssOutput {
    type Output = str;
    fn index(&self, field: &'a str) -> &Self::Output {
        match field {
            "description" => self.0.description(),
            "title" => self.0.title(),
            "link" => self.0.link(),
            _ => None,
        }
        .unwrap_or_default()
    }
}
