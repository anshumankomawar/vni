use crossterm::event::{Event, KeyCode, KeyEvent}; /* modify */
use crossterm::{event, terminal};
use std::io;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not disable raw mode")
    }
}

fn main() -> io::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;

    loop {
        if let Event::Key(event) = event::read().expect("Failed to read line") {
            match event {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: event::KeyModifiers::CONTROL,
                    kind,
                    state,
                } => break,
                _ => {}
            }
            println!("{:?}\r", event);
        };
    }

    Ok(())
}
