use clap::Parser;
use tokio::sync::mpsc;

mod event_loop;
mod input;
mod model;
mod socket;
mod terminal;
mod tui;

#[derive(Parser, Debug)]
#[command(
    name = "wmux",
    version,
    about = "tmux for Windows \u{2014} split panes, tabbed workspaces, and a socket API for AI agents",
    long_about = "wmux is a terminal multiplexer for Windows. It provides split panes, \
tabbed workspaces, and a JSON-RPC socket API that lets AI coding agents \
control terminal sessions programmatically.\n\n\
Prefix key: Ctrl+A (then press action key)\n\
  |  Split vertical       -  Split horizontal\n\
  c  New workspace        n/p  Next/prev workspace\n\
  x  Close pane           z  Toggle zoom\n\
  q  Quit"
)]
struct Args {
    /// Shell executable to use (default: auto-detect from WMUX_SHELL, COMSPEC, or powershell.exe)
    #[arg(long, value_name = "PATH")]
    shell: Option<String>,

    /// Named pipe path for the JSON-RPC socket API
    #[arg(long, value_name = "PIPE", default_value = r"\\.\pipe\wmux")]
    pipe: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    eprintln!("\u{26A0}\u{FE0F}  Notice: The terminal-based wmux is deprecated in favor of the new desktop application.");
    eprintln!("    Download the modern wmux for Windows at: https://github.com/fernandomenuk/wmux");
    eprintln!();

    // Validate shell exists if explicitly specified
    if let Some(ref shell) = args.shell {
        if !std::path::Path::new(shell).exists() {
            // Check if it's on PATH
            if which(shell).is_none() {
                eprintln!("error: shell not found: {}", shell);
                eprintln!("  Specify a full path or ensure it's on your PATH.");
                std::process::exit(1);
            }
        }
    }

    let (socket_tx, socket_rx) = mpsc::unbounded_channel::<event_loop::SocketRequest>();

    let pipe_path = args.pipe.clone();
    let socket_tx_clone = socket_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = socket::server::start_pipe_server(pipe_path, socket_tx_clone).await {
            // Don't crash — pipe errors are non-fatal (e.g. pipe already in use)
            eprintln!("warning: socket server failed: {}", e);
            eprintln!("  Another wmux instance may be using the same pipe.");
            eprintln!("  Use --pipe to specify a different path.");
        }
    });

    if let Err(e) = event_loop::run(args.shell, args.pipe, socket_rx, socket_tx).await {
        // Terminal cleanup already happened inside event_loop::run
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

/// Simple which-like lookup: check if a command exists on PATH.
fn which(cmd: &str) -> Option<std::path::PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    let exts = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT".into());
    let extensions: Vec<&str> = exts.split(';').collect();

    for dir in path_var.split(';') {
        let base = std::path::Path::new(dir).join(cmd);
        // Check exact name
        if base.exists() {
            return Some(base);
        }
        // Check with extensions
        for ext in &extensions {
            let with_ext = std::path::Path::new(dir).join(format!("{}{}", cmd, ext));
            if with_ext.exists() {
                return Some(with_ext);
            }
        }
    }
    None
}
