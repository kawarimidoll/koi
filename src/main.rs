mod editor;
use editor::Editor;

fn main() {
    println!("Hello, koi!");
    println!("Type something. Press 'q' to quit.");
    Editor::default().run().unwrap();
    println!("Goodbye, koi!");
}
