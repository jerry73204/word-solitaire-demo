use anyhow::{Context, Result};
use async_std::{io::BufReader, net::TcpStream};
use clap::Parser;
use futures::prelude::*;
use std::net::SocketAddr;
use word_solitaire_demo::{ServerMessage, UserCommand};

#[derive(Debug, Parser)]
struct Opts {
    #[clap(long, default_value = "127.0.0.1:7373")]
    pub server: SocketAddr,
}

#[async_std::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let opts = Opts::parse();

    // Connect to server
    let result: Result<TcpStream, _> = TcpStream::connect(&opts.server).await;
    let tcp_stream = result
        .with_context(|| format!("Cannot connect to {}. Is the server started?", opts.server))?;
    let (tcp_reader, tcp_writer) = tcp_stream.split();

    // Start concurrent message receiver and user input handler workers
    let receiving_future = receiving_worker(tcp_reader);
    let user_input_future = user_input_worker(tcp_writer);

    // Wait for both workers to finish or one of them fails.
    futures::try_join!(receiving_future, user_input_future)?;

    Ok(())
}

async fn receiving_worker<R>(tcp_reader: R) -> Result<()>
where
    R: AsyncRead + Unpin,
{
    let reader = BufReader::new(tcp_reader);
    let mut lines = reader.lines();

    while let Some(line) = lines.try_next().await? {
        let msg: Result<ServerMessage> = line.parse();
        let msg = match msg {
            Ok(msg) => msg,
            Err(err) => {
                eprintln!("Error: unable to parse server message: {:#}", err);
                return Ok(());
            }
        };

        match msg {
            ServerMessage::Update { new_riddle } => {
                eprintln!("New riddle: {}", new_riddle);
            }
            ServerMessage::Accepted => {
                eprintln!("answer accepted");
            }
            ServerMessage::Rejected => {
                eprintln!("answer rejected");
            }
            ServerMessage::Close => {
                eprintln!("Connection closed by server");
                break;
            }
        }
    }

    Ok(())
}

async fn user_input_worker<W>(mut tcp_writer: W) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let stdin = async_std::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Some(line) = lines.try_next().await? {
        // parse command
        let cmd: Result<UserCommand> = line.parse();

        let cmd = match cmd {
            Ok(cmd) => cmd,
            Err(err) => {
                eprintln!("Error: cannot understand command: {:#}", err);
                continue;
            }
        };

        match cmd {
            UserCommand::Guess { guess } => {
                // send guess to server
                let request = format!("guess {}\n", guess);
                tcp_writer.write_all(request.as_bytes()).await?;
            }
            UserCommand::Exit => {
                // Tell the server that I'm going to exit
                tcp_writer.write_all("exit".as_bytes()).await?;
                break;
            }
        }
    }

    Ok(())
}
