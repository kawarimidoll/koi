#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]

// ref: https://github.com/crossterm-rs/crossterm/blob/master/examples/event-stream-tokio.rs
// nix run .#cargo -- run --bin key_checker

// keycode checker
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use futures::{future::FutureExt, StreamExt};
use futures_timer::Delay;
use std::time::Duration;

async fn print_events() {
    let mut reader = EventStream::new();
    let mut three_second_timer = Box::pin(Delay::new(Duration::from_secs(0)).fuse());
    let mut timer_active = false;

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();

        futures::select! {
            () = delay => { println!(".\r"); },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(Event::Key(KeyEvent { code: KeyCode::Char(c), .. }))) => {
                        println!("Key pressed: {c}\r");
                        three_second_timer = Box::pin(Delay::new(Duration::from_secs(3)).fuse());
                        timer_active = true;
                    },
                    Some(Ok(Event::Key(KeyEvent { code: KeyCode::Esc, .. }))) => break,
                    Some(Err(e)) => println!("Error: {e:?}\r"),
                    _ => {}
                }
            }
            () = three_second_timer => {
                if timer_active {
                    println!("Three seconds have elapsed since the key input!\r");
                    timer_active = false;
                }
            }
        };
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Prints '.' every second if there's no event");
    println!("Prints a message 3 seconds after a key press");
    println!("Use Esc to quit");
    enable_raw_mode()?;
    print_events().await;
    disable_raw_mode()
}
