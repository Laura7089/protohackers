use clap::Parser;
use protohackers::Server;
use tracing::{event, instrument, Level};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    /// Which challenge to run, by number
    #[arg(short, long, value_parser = challenge)]
    challenge: u8,
    /// Which port to listen on
    #[arg(short, long, default_value_t = 10_000)]
    port: u16,
}

fn challenge(input: &str) -> Result<u8, String> {
    let chal: u8 = input.parse().unwrap();
    if chal <= protohackers::MAX_CHALLENGE {
        Ok(chal)
    } else {
        Err(format!("No challenge known for {input}"))
    }
}

#[tokio::main]
#[instrument]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args.challenge {
        0 => {
            event!(Level::INFO, "running challenge 0 (smoke test)");
            protohackers::problem0::SmokeTest
                .run(args.port)
                .await
                .unwrap();
        }
        1 => {
            event!(Level::INFO, "running challenge 1 (prime time)");
            protohackers::problem1::PrimeTime::default()
                .run(args.port)
                .await
                .unwrap();
        }
        _ => panic!("Challenge number {} not known", args.challenge),
    }
}
