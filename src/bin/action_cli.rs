use anyhow::Result;
use async_std::task;
use futures_util::future;
use ifttt_action::{config::Config, Action, ActionRun, States};
use std::{convert::TryInto, io, io::Read};

fn main() -> Result<()> {
    let mut config_input = String::new();
    io::stdin().read_to_string(&mut config_input)?;

    let config: Config = serde_json::from_str(&config_input).unwrap();

    let parameters = config.parameters.clone();
    let mut actions: Vec<ActionRun<_, _, _>> = config.try_into()?;

    let combined_futures = future::join_all(actions.iter_mut().map(|action| action.execute()));

    task::block_on(combined_futures);

    let states: States = actions
        .into_iter()
        .map(|action| (action.key, action.state))
        .collect();

    parameters.write_states(&states)?;

    Ok(())
}
