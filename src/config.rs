use crate::mapper::TextMapper;
use crate::rss::RssFeed;
use crate::weather::WeatherFeed;
use crate::web::WebSink;
use crate::{ActionConfigs, ActionRun, Feeds, Mappers, Sinks, State};
use anyhow::{Error, Result};
use serde::{export::Formatter, export::TryFrom, Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::str::FromStr;

type CustomConfig<'a> = HashMap<&'a str, String>;

#[derive(Debug)]
pub enum ConfigError {
    NoConfigKey(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for ConfigError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config<'a> {
    #[serde(borrow)]
    actions: Vec<ActionConfig<'a>>,
    pub parameters: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionConfig<'a> {
    key: &'a str,
    #[serde(borrow)]
    feed: KindAndConfig<'a>,
    #[serde(borrow)]
    mapper: KindAndConfig<'a>,
    #[serde(borrow)]
    sink: KindAndConfig<'a>,
    #[serde(borrow, default)]
    config: CustomConfig<'a>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct KindAndConfig<'a> {
    kind: &'a str,
    config: CustomConfig<'a>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Parameters {
    #[serde(default)]
    pub(crate) state_key: Option<String>,
    #[serde(default)]
    pub(crate) state_file: String,
}

impl TryFrom<Config<'_>> for Vec<ActionRun<Feeds, Mappers, Sinks>> {
    type Error = Error;
    fn try_from(config: Config<'_>) -> Result<Self> {
        let mut actions = Vec::with_capacity(config.actions.len());
        let mut states = match config.parameters.read_states() {
            Ok(states) => states,
            Err(_) => {
                println!("No states is found. Use empty.");
                HashMap::new()
            }
        };

        for action_config in config.actions {
            let state = states.remove(action_config.key).unwrap_or_default();
            actions.push(action_config.into_action(state)?);
        }

        Ok(actions)
    }
}

impl ActionConfig<'_> {
    pub fn into_action(self, state: State) -> Result<ActionRun<Feeds, Mappers, Sinks>> {
        let feed: Feeds = self.feed.try_into()?;
        let mapper: Mappers = self.mapper.try_into()?;
        let sink: Sinks = self.sink.try_into()?;

        let config = if let Some(schedule) = self.config.get("schedule") {
            ActionConfigs::new(schedule.clone())
        } else {
            ActionConfigs::default()
        };

        Ok(ActionRun::new(self.key, feed, mapper, sink, state, config))
    }
}

impl TryFrom<KindAndConfig<'_>> for Feeds {
    type Error = Error;
    fn try_from(config: KindAndConfig<'_>) -> Result<Self> {
        let res = match config.kind {
            "rss" => RssFeed::new(
                config.read_val::<String, _>("url")?,
                config.read_val("count")?,
            )
            .into(),
            "weather" => WeatherFeed::new(
                config.read_val::<String, _>("key")?,
                config.read_val::<String, _>("location")?,
            )
            .into(),
            _ => unimplemented!(),
        };

        Ok(res)
    }
}

impl TryFrom<KindAndConfig<'_>> for Mappers {
    type Error = Error;
    fn try_from(config: KindAndConfig<'_>) -> Result<Self> {
        let res = match config.kind {
            "text" => TextMapper::new(config.read_val::<String, _>("text")?).into(),
            _ => unimplemented!(),
        };

        Ok(res)
    }
}

impl TryFrom<KindAndConfig<'_>> for Sinks {
    type Error = Error;
    fn try_from(config: KindAndConfig<'_>) -> Result<Self> {
        let res = match config.kind {
            "web" => WebSink::new(
                config.read_val::<String, _>("method")?,
                config.read_val::<String, _>("url")?,
            )
            .into(),
            _ => unimplemented!(),
        };

        Ok(res)
    }
}

impl KindAndConfig<'_> {
    pub fn read_val<T, TE>(&self, key: &'static str) -> Result<T>
    where
        T: FromStr<Err = TE>,
        TE: std::error::Error + Send + Sync + 'static,
    {
        let value = self
            .config
            .get(key)
            .ok_or_else(|| ConfigError::NoConfigKey(key))?;

        Ok(T::from_str(value)?)
    }
}

#[cfg(test)]
mod test_config {
    use super::*;

    #[test]
    fn test_deserialize() {
        let config = r###"
        {
            "actions": [
                {
                    "key": "index1",
                    "feed": {
                        "kind": "rss",
                        "config": {
                            "link": "rss link"
                        }
                    },
                    "mapper": {
                        "kind": "text",
                        "config": {
                            "text": "{title}\n{link}"
                        }
                    },
                    "sink": {
                        "kind": "web",
                        "config": {
                            "link": "web link"
                        }
                    }
                },
                {
                    "key": "Index2",
                    "feed": {
                        "kind": "rss",
                        "config": {
                            "link": "rss link"
                        }
                    },
                    "mapper": {
                        "kind": "text",
                        "config": {
                            "text": "{title}{link}"
                        }
                    },
                    "sink": {
                        "kind": "web",
                        "config": {
                            "link": "web link"
                        }
                    }
                }
            ],
            "parameters": {
                "state_key": "state_key",
                "state_file": "ifttt_state"
            }
        }
        "###;

        let config: Config = serde_json::from_str(config).unwrap();
        dbg!(config);
    }
}
