pub mod config;
mod crypto;
mod mapper;
mod rss;
mod web;

use crate::mapper::TextMapper;
use crate::rss::RssFeed;
use crate::web::WebSink;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use enum_dispatch::enum_dispatch;
use std::collections::HashMap;
use std::ops::Index;
use std::str::FromStr;

const ACTION_NEXT_EXEC: &'static str = "action_next_exec";

pub type States = HashMap<ActionKey, State>;

pub type State = HashMap<String, String>;

pub type ActionKey = String;

pub trait Indexable: for<'a> Index<&'a str, Output = str> {}

impl<T> Indexable for T where T: for<'a> Index<&'a str, Output = str> {}

pub struct ActionRun<F, M, S>
where
    F: Feed,
    M: Mapper,
    S: Sink,
{
    pub key: String,
    pub feed: F,
    pub sink: S,
    pub mapper: M,
    pub state: State,
    config: ActionConfigs,
}

#[derive(Default, Debug)]
pub struct ActionConfigs {
    schedule: Option<String>,
}

impl ActionConfigs {
    pub fn new(schedule: String) -> Self {
        ActionConfigs {
            schedule: Some(schedule),
        }
    }
}

impl<F, M, S> ActionRun<F, M, S>
where
    F: Feed,
    M: Mapper,
    S: Sink,
{
    pub fn new(
        key: impl Into<String>,
        feed: F,
        mapper: M,
        sink: S,
        state: State,
        config: ActionConfigs,
    ) -> Self {
        ActionRun {
            key: key.into(),
            feed,
            mapper,
            sink,
            state,
            config,
        }
    }
}

#[async_trait(?Send)]
impl<F, M, S> Action for ActionRun<F, M, S>
where
    F: Feed,
    M: Mapper,
    S: Sink,
{
    async fn execute(&mut self) -> Result<()> {
        if !self.should_run()? {
            return Ok(());
        }
        let output = self.feed.feed(&mut self.state).await?;
        for params in output.iter() {
            let input = self.mapper.map(params.as_ref())?;
            self.sink.sink(input).await?;
        }

        Ok(())
    }

    fn key(&self) -> ActionKey {
        self.key.clone()
    }
}

impl<F, M, S> ActionRun<F, M, S>
where
    F: Feed,
    M: Mapper,
    S: Sink,
{
    pub fn should_run(&mut self) -> Result<bool> {
        let should_run = match self.state.get(ACTION_NEXT_EXEC) {
            None => true,
            Some(t) => {
                let next_exec = chrono::DateTime::parse_from_rfc3339(t)?;
                Utc::now() > next_exec
            }
        };

        match self.config.schedule {
            Some(ref schedule) if should_run => {
                let schedule =
                    Schedule::from_str(schedule).expect("Unable to parse the cron expression.");
                let next_exec_time = schedule.upcoming(Utc).next().unwrap();
                self.state
                    .insert(ACTION_NEXT_EXEC.to_string(), next_exec_time.to_rfc3339());
            }
            _ => (),
        }

        Ok(should_run)
    }
}

#[async_trait(?Send)]
pub trait Action {
    async fn execute(&mut self) -> Result<()>;

    // The unique name within a config.
    fn key(&self) -> ActionKey;
}

#[async_trait]
#[enum_dispatch(Feeds)]
pub trait Feed {
    async fn feed(&self, state: &mut State) -> Result<Vec<Box<dyn Indexable>>>;
}

#[enum_dispatch(Mappers)]
pub trait Mapper {
    fn map(&self, input: &dyn Indexable) -> Result<String>;
}

#[async_trait(?Send)]
#[enum_dispatch(Sinks)]
pub trait Sink {
    async fn sink(&self, input: String) -> Result<()>;
}

#[enum_dispatch]
pub enum Feeds {
    RssFeed,
}

#[enum_dispatch]
pub enum Mappers {
    TextMapper,
}

#[enum_dispatch]
pub enum Sinks {
    WebSink,
}
