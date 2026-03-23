use portable_pty::{native_pty_system, CommandBuilder, PtySize, MasterPty, Child};
use std::io::{Read, Write};
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct PtyHandle {
    pub master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn Write + Send>,
    pub child: Box<dyn Child + Send + Sync>,
}

/// Spawn a new PTY with the given shell and size.
pub fn spawn_pty(
    shell: &str,
    cols: u16,
    rows: u16,
    working_directory: Option<&str>,
) -> Result<PtyHandle, Box<dyn std::error::Error>> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new(shell);
    if let Some(dir) = working_directory {
        cmd.cwd(dir);
    }

    let child = pair.slave.spawn_command(cmd)?;
    let writer = pair.master.take_writer()?;

    Ok(PtyHandle {
        master: pair.master,
        writer,
        child,
    })
}

/// Start a thread that reads from the PTY and sends output to a channel.
/// When the reader ends (shell exits), sends an exit notification on `exit_tx`.
pub fn start_pty_reader(
    surface_id: Uuid,
    master: &dyn MasterPty,
    tx: mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
    exit_tx: mpsc::UnboundedSender<Uuid>,
) -> Result<std::thread::JoinHandle<()>, Box<dyn std::error::Error>> {
    let mut reader = master.try_clone_reader()?;
    let handle = std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send((surface_id, buf[..n].to_vec())).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        // Notify that this surface's PTY has exited
        let _ = exit_tx.send(surface_id);
    });
    Ok(handle)
}
