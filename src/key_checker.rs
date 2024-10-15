#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]

// ref: https://github.com/crossterm-rs/crossterm/blob/master/examples/event-stream-tokio.rs
// nix run .#cargo -- run --bin key_checker

// keycode checker
use crossterm::event::{Event, EventStream, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;
use std::time::Duration;

async fn print_events() {
    let mut reader = EventStream::new();

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();

        select! {
            _ = delay => { println!(".\r"); },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        println!("Event::{event:?}\r");
                        if event == Event::Key(KeyCode::Esc.into()) {
                            break;
                        }
                    }
                    Some(Err(e)) => println!("Error: {e:?}\r"),
                    None => break,
                }
            },
        };
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Prints '.' every second if there's no event");
    println!("Use Esc to quit");

    enable_raw_mode()?;

    print_events().await;

    disable_raw_mode()
}
