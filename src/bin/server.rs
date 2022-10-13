use anyhow::{ensure, Result};
use async_std::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
};
use clap::Parser;
use futures::{prelude::*, stream::FuturesUnordered, AsyncWriteExt};
use game_server::{SuffixMatcher, UserCommand};
use rand::prelude::*;
use std::{
    collections::HashSet,
    io::prelude::*,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::watch;

#[derive(Debug, Parser)]
struct Opts {
    #[clap(long, default_value = "0.0.0.0:7373")]
    pub addr: SocketAddr,
    #[clap(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/words.txt"))]
    pub dict_file: PathBuf,
}

#[async_std::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let opts = Opts::parse();

    let tcp_listener = TcpListener::bind(&opts.addr).await?;
    let mut client_futures = FuturesUnordered::new();
    let state = Arc::new(RwLock::new(ServerState::new(&opts.dict_file)?));
    let (bsender, breceiver) = watch::channel(());
    let bsender = Arc::new(Mutex::new(bsender));

    loop {
        futures::select! {
            result = tcp_listener.accept().fuse() => {
                let (stream, addr) = result?;
                eprintln!("Accepted a client from {}", addr);
                let future = serve_client(state.clone(), stream, bsender.clone(), breceiver.clone());
                client_futures.push(future);
            }
            result = client_futures.next() => {
                match result {
                    Some(Ok(())) => {}
                    Some(Err(err)) => {
                        eprintln!("Error: {:#}", err);
                    }
                    None => {}
                }
            }
        }
    }
}

async fn serve_client(
    state: Arc<RwLock<ServerState>>,
    stream: TcpStream,
    bsender: Arc<Mutex<watch::Sender<()>>>,
    mut breceiver: watch::Receiver<()>,
) -> Result<()> {
    use async_std::io::BufReader;

    let (reader, mut writer) = stream.split();
    let mut lines = BufReader::new(reader).lines().fuse();

    loop {
        futures::select! {
            line = lines.try_next() => {
                match line? {
                    Some(line) => {
                        let exit = on_client_command(&state, &mut writer, &bsender, line).await?;
                        if exit {
                            break;
                        }
                    }
                    None => break,
                }
            }
            result = breceiver.changed().fuse() => {
                match result {
                    Ok(()) =>  on_notified_update(&state, &mut writer).await?,
                    Err(_) => break
                }
            }
        }
    }

    Ok(())
}

async fn on_client_command<W>(
    state: &RwLock<ServerState>,
    mut writer: W,
    bsender: &Mutex<watch::Sender<()>>,
    line: String,
) -> Result<bool>
where
    W: AsyncWrite + Unpin,
{
    let cmd: UserCommand = line.parse()?;

    match cmd {
        UserCommand::Guess { guess } => {
            let accept = {
                // The lock is bounded in the scope.
                let mut guard = state.write().await;

                let not_same_word = guard.word != guess;
                let not_used = !guard.used_words.contains(&guess);
                let in_dictionary = guard.dictionary.contains(&guess);
                let matches_suffix = guard.matcher.try_match(&guess);
                let accept = not_same_word && not_used && in_dictionary && matches_suffix;

                // update the state if the guess is accepted
                // and notify other clients
                if accept {
                    guard.used_words.insert(guess.clone());
                    guard.matcher = SuffixMatcher::new(&guess);
                    guard.word = guess;

                    let result = bsender.lock().await.send(());
                    if result.is_err() {
                        return Ok(true);
                    }
                }

                accept
            };

            if accept {
                writer.write_all(b"accept\n").await?;
            } else {
                writer.write_all(b"reject\n").await?;
            }

            Ok(false)
        }
        UserCommand::Exit => Ok(true),
    }
}

async fn on_notified_update<W>(state: &RwLock<ServerState>, mut writer: W) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let guard = state.read().await;
    let word = &guard.word;
    writer
        .write_all(format!("update {}", word).as_bytes())
        .await?;
    Ok(())
}

struct ServerState {
    word: String,
    matcher: SuffixMatcher,
    dictionary: HashSet<String>,
    used_words: HashSet<String>,
}

impl ServerState {
    pub fn new<P>(dict_file: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        use std::{fs::File, io::BufReader};

        // Collect words with length >= 3 in the file
        let lines: Result<Vec<_>, _> = BufReader::new(File::open(dict_file.as_ref())?)
            .lines()
            .collect();
        let dict_vec: Vec<_> = lines?.into_iter().filter(|word| word.len() >= 3).collect();
        ensure!(!dict_vec.is_empty(), "No words with length >= 3 found");

        // Randomly choose an initial word
        let mut rng = rand::thread_rng();
        let initial_word = dict_vec.choose(&mut rng).unwrap().clone();

        // Convert a vec of words to a set of words
        let dict_set: HashSet<_> = dict_vec.into_iter().collect();

        let matcher = SuffixMatcher::new(&initial_word);

        Ok(Self {
            word: initial_word,
            matcher,
            dictionary: dict_set,
            used_words: HashSet::new(),
        })
    }
}
