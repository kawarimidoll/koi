#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]

mod editor;
use editor::Editor;

fn main() {
    Editor::new().unwrap().run();
}

// // keycode checker
// use crossterm::event::{read, Event, KeyCode};
// use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
// fn main() {
//     enable_raw_mode().unwrap();
//     loop {
//         match read() {
//             Ok(Event::Key(key_event)) => {
//                 println!("{:?}\r", key_event);
//                 if key_event.code == KeyCode::Char('q') {
//                     break;
//                 }
//             }
//             _ => {}
//         }
//     }
//     disable_raw_mode().unwrap();
// }
