use std::fmt;
use uuid::Uuid;

#[derive(Debug)]
pub enum WmuxError {
    PtySpawn(String),
    ShellNotFound(String),
    Io(std::io::Error),
    SurfaceNotFound(Uuid),
}

impl fmt::Display for WmuxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WmuxError::PtySpawn(msg) => write!(f, "Failed to spawn PTY: {}", msg),
            WmuxError::ShellNotFound(shell) => write!(f, "Shell not found: {}", shell),
            WmuxError::Io(e) => write!(f, "IO error: {}", e),
            WmuxError::SurfaceNotFound(id) => write!(f, "Surface not found: {}", id),
        }
    }
}

impl std::error::Error for WmuxError {}

impl From<std::io::Error> for WmuxError {
    fn from(e: std::io::Error) -> Self {
        WmuxError::Io(e)
    }
}

impl From<Box<dyn std::error::Error>> for WmuxError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        WmuxError::PtySpawn(e.to_string())
    }
}
