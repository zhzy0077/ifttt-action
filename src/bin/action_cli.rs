use anyhow::{Error, Result};
use async_std::task;
use futures_util::{future, AsyncReadExt};
use ifttt_action::{config::Config, Action, ActionRun, States};
use std::fs::File;
use std::path::Path;
use std::{convert::TryInto, io, io::Read};

fn main() -> Result<()> {
    let config = match std::env::args().nth(1) {
        Some(uri) if uri.trim() != "-" => {
            if Path::new(&uri).exists() {
                let mut config_input = String::new();
                File::open(&uri)?.read_to_string(&mut config_input)?;
                config_input
            } else {
                task::block_on(async {
                    Ok::<String, Error>(reqwest::get(&uri).await?.text().await?)
                })?
            }
        }
        _ => {
            let mut config_input = String::new();
            io::stdin().read_to_string(&mut config_input)?;
            config_input
        }
    };

    let config: Config = serde_json::from_str(&config).unwrap();

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
