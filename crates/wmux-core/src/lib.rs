pub mod core;
pub mod error;
pub mod model;
pub mod socket;
pub mod terminal;

pub use crate::core::{WmuxCore, SurfaceId, WorkspaceId, FocusDirection};
pub use crate::error::WmuxError;

// Re-export vt100 for frontend convenience
pub use vt100;
