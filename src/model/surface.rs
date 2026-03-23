use uuid::Uuid;
use vt100::Parser;
use std::io::Write;
use crate::terminal::pty::PtyHandle;

#[allow(dead_code)]
pub struct Surface {
    pub id: Uuid,
    pub shell: String,
    pub size: (u16, u16),
    pub dirty: bool,
    pub exited: Option<i32>,
    pub parser: Parser,
    pub pty: Option<PtyHandle>,
}

impl Surface {
    pub fn new(id: Uuid, shell: String, cols: u16, rows: u16, pty: PtyHandle) -> Self {
        Self {
            id,
            shell,
            size: (cols, rows),
            dirty: true,
            exited: None,
            parser: Parser::new(rows, cols, 1000),
            pty: Some(pty),
        }
    }

    /// Feed raw PTY output bytes into the VT parser.
    pub fn process_output(&mut self, data: &[u8]) {
        self.parser.process(data);
        self.dirty = true;
    }

    /// Write text to the PTY (user input).
    pub fn send_text(&mut self, text: &str) -> Result<(), std::io::Error> {
        self.send_bytes(text.as_bytes())
    }

    /// Write raw bytes to the PTY.
    pub fn send_bytes(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        if let Some(ref mut pty) = self.pty {
            pty.writer.write_all(data)?;
            pty.writer.flush()?;
        }
        Ok(())
    }

    /// Send a named key to the PTY.
    pub fn send_key(&mut self, key: &str) -> Result<(), std::io::Error> {
        let bytes = match key {
            "Enter" => "\r",
            "Tab" => "\t",
            "Escape" => "\x1b",
            "Backspace" => "\x7f",
            "Up" => "\x1b[A",
            "Down" => "\x1b[B",
            "Right" => "\x1b[C",
            "Left" => "\x1b[D",
            "Home" => "\x1b[H",
            "End" => "\x1b[F",
            "Delete" => "\x1b[3~",
            "F1" => "\x1bOP",
            "F2" => "\x1bOQ",
            "F3" => "\x1bOR",
            "F4" => "\x1bOS",
            "F5" => "\x1b[15~",
            "F6" => "\x1b[17~",
            "F7" => "\x1b[18~",
            "F8" => "\x1b[19~",
            "F9" => "\x1b[20~",
            "F10" => "\x1b[21~",
            "F11" => "\x1b[23~",
            "F12" => "\x1b[24~",
            "Ctrl+C" => "\x03",
            "Ctrl+D" => "\x04",
            "Ctrl+Z" => "\x1a",
            "Ctrl+L" => "\x0c",
            "Ctrl+A" => "\x01",
            _ => return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown key: {}", key),
            )),
        };
        self.send_text(bytes)
    }

    /// Resize the surface.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.size = (cols, rows);
        self.parser.set_size(rows, cols);
        if let Some(ref mut pty) = self.pty {
            let _ = pty.master.resize(portable_pty::PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
        self.dirty = true;
    }

    /// Mark process as exited.
    pub fn mark_exited(&mut self, code: i32) {
        self.exited = Some(code);
        self.dirty = true;
    }

    /// Get the vt100 screen for rendering.
    pub fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }
}
