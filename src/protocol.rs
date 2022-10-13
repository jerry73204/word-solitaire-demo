use anyhow::{bail, Result};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserCommand {
    Guess { guess: String },
    Exit,
}

impl FromStr for UserCommand {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> Result<UserCommand> {
        use UserCommand as C;

        let tokens: Vec<&str> = line.split_whitespace().collect();

        let cmd = match tokens.as_slice() {
            [] => {
                bail!("Error: empty command");
            }
            ["guess", guess] => C::Guess {
                guess: guess.to_string(),
            },
            ["exit"] => C::Exit,
            [command, ..] => {
                bail!("unknown command '{}'", command)
            }
        };

        Ok(cmd)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerMessage {
    Update { new_riddle: String },
    Accepted,
    Rejected,
    Close,
}

impl FromStr for ServerMessage {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> Result<Self> {
        use ServerMessage as M;

        let tokens: Vec<&str> = line.split_whitespace().collect();
        let msg = match tokens.as_slice() {
            [] => {
                bail!("Error: empty response");
            }
            ["update", new_riddle] => M::Update {
                new_riddle: new_riddle.to_string(),
            },
            ["accepted"] => M::Accepted,
            ["rejected"] => M::Rejected,
            ["close"] => M::Close,
            [command, ..] => {
                bail!("unknown command '{}'", command)
            }
        };

        Ok(msg)
    }
}
