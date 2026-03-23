use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ForwardToSurface(KeyEvent),
    NewWorkspace,
    NextWorkspace,
    PrevWorkspace,
    SelectWorkspace(usize),
    SplitVertical,
    SplitHorizontal,
    FocusUp,
    FocusDown,
    FocusLeft,
    FocusRight,
    CloseSurface,
    ToggleZoom,
    Quit,
    None,
}

pub struct InputHandler {
    prefix_mode: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self { prefix_mode: false }
    }

    pub fn is_prefix_mode(&self) -> bool {
        self.prefix_mode
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Action {
        if self.prefix_mode {
            self.prefix_mode = false;
            return self.dispatch_prefix(key);
        }

        if key.modifiers.contains(KeyModifiers::CONTROL)
            && key.code == KeyCode::Char('a')
        {
            self.prefix_mode = true;
            return Action::None;
        }

        Action::ForwardToSurface(key)
    }

    fn dispatch_prefix(&self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('c') => Action::NewWorkspace,
            KeyCode::Char('n') => Action::NextWorkspace,
            KeyCode::Char('p') => Action::PrevWorkspace,
            KeyCode::Char('|') | KeyCode::Char('\\') => Action::SplitVertical,
            KeyCode::Char('-') => Action::SplitHorizontal,
            KeyCode::Char('x') => Action::CloseSurface,
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('z') => Action::ToggleZoom,
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                Action::SelectWorkspace((c as usize) - ('1' as usize))
            }
            KeyCode::Up => Action::FocusUp,
            KeyCode::Down => Action::FocusDown,
            KeyCode::Left => Action::FocusLeft,
            KeyCode::Right => Action::FocusRight,
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::ForwardToSurface(KeyEvent::new(KeyCode::Char('\x01'), KeyModifiers::NONE))
            }
            _ => Action::None,
        }
    }
}

/// Convert a crossterm KeyEvent to bytes suitable for writing to a PTY.
pub fn key_event_to_bytes(key: &KeyEvent) -> Option<Vec<u8>> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Char(c) => {
            if ctrl {
                let code = (c.to_ascii_lowercase() as u8).wrapping_sub(b'a').wrapping_add(1);
                Some(vec![code])
            } else {
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                Some(s.as_bytes().to_vec())
            }
        }
        KeyCode::Enter => Some(b"\r".to_vec()),
        KeyCode::Tab => Some(b"\t".to_vec()),
        KeyCode::Backspace => Some(b"\x7f".to_vec()),
        KeyCode::Esc => Some(b"\x1b".to_vec()),
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
        KeyCode::F(1) => Some(b"\x1bOP".to_vec()),
        KeyCode::F(2) => Some(b"\x1bOQ".to_vec()),
        KeyCode::F(3) => Some(b"\x1bOR".to_vec()),
        KeyCode::F(4) => Some(b"\x1bOS".to_vec()),
        KeyCode::F(n @ 5..=12) => {
            let codes = [15, 17, 18, 19, 20, 21, 23, 24];
            let idx = (n - 5) as usize;
            if idx < codes.len() {
                Some(format!("\x1b[{}~", codes[idx]).into_bytes())
            } else {
                None
            }
        }
        _ => None,
    }
}
