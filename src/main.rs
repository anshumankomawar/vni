use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::terminal::ClearType;

use crossterm::{cursor, event, execute, queue, terminal};
use std::io::{self, stdout, Write};
use std::time::Duration;
use std::usize;

struct Buffer {
    rows: usize,
    columns: usize,
    content: String,
}

impl Buffer {
    fn new(rows: usize, columns: usize) -> Buffer {
        Buffer {
            rows,
            columns,
            content: "".to_owned(),
        }
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> io::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}

impl io::Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(io::ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

struct Cursor {
    x: usize,
    y: usize,
    rows: usize,
    columns: usize,
}

impl Cursor {
    fn new(rows: usize, columns: usize) -> Cursor {
        Cursor {
            x: 0,
            y: 0,
            rows,
            columns,
        }
    }

    fn move_cursor(&mut self, code: char) {
        match code {
            'k' => {
                if self.y > 0 {
                    self.y -= 1;
                }
            }
            'j' => {
                if self.y < self.rows - 1 {
                    self.y += 1;
                }
            }
            'h' => {
                if self.x > 0 {
                    self.x -= 1;
                }
            }
            'l' => {
                if self.x < self.columns - 1 {
                    self.x += 1;
                }
            }
            _ => {}
        }
    }
}

struct Editor {
    rows: usize,
    columns: usize,
    cursor: Cursor,
    reader: Reader,
    buffer: Buffer,
}

impl Editor {
    fn new(rows: usize, columns: usize) -> Editor {
        Editor {
            rows,
            columns,
            cursor: Cursor::new(rows, columns),
            reader: Reader,
            buffer: Buffer::new(rows, columns),
        }
    }

    fn draw_rows(&mut self) {
        /* modify */
        let screen_rows = self.rows;
        for i in 0..screen_rows {
            self.buffer.content.push('~'); /* modify */
            queue!(self.buffer, terminal::Clear(ClearType::UntilNewLine)).unwrap();
            if i < screen_rows - 1 {
                self.buffer.content.push_str("\r\n"); /* modify */
            }
        }
    }

    fn clear_screen(&self) -> io::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))?;
        Ok(())
    }

    fn process_keypress(&mut self) -> io::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                kind,
                state,
            } => return Ok(false),
            KeyEvent {
                code: KeyCode::Char(val @ ('k' | 'j' | 'h' | 'l')),
                modifiers: event::KeyModifiers::NONE,
                kind,
                state,
            } => self.cursor.move_cursor(val),
            _ => {}
        }
        Ok(true)
    }

    fn run(&mut self) -> io::Result<bool> {
        queue!(self.buffer, cursor::Hide, cursor::MoveTo(0, 0))?;
        self.draw_rows();
        queue!(
            self.buffer,
            cursor::MoveTo(self.cursor.x as u16, self.cursor.y as u16),
            cursor::Show
        )?;
        self.buffer.flush()?;
        self.process_keypress()
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode");
        execute!(stdout(), terminal::Clear(ClearType::All)).expect("could not clear terminal");
        execute!(stdout(), cursor::MoveTo(0, 0)).unwrap();
    }
}

fn main() -> io::Result<()> {
    let (rows, columns) = terminal::size()?;
    let mut editor = Editor::new(rows as usize, columns as usize);
    terminal::enable_raw_mode()?;

    while editor.run()? {}

    Ok(())
}
