use clap::Parser;
use tokio::sync::mpsc;

mod app;
mod input;
mod model;
mod socket;
mod terminal;
mod tui;

#[derive(Parser, Debug)]
#[command(name = "wmux", version, about = "Windows terminal multiplexer")]
struct Args {
    /// Shell to use (default: auto-detect)
    #[arg(long)]
    shell: Option<String>,

    /// Named pipe path
    #[arg(long, default_value = r"\\.\pipe\wmux")]
    pipe: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let (socket_tx, socket_rx) = mpsc::unbounded_channel::<app::SocketRequest>();

    let pipe_path = args.pipe.clone();
    let socket_tx_clone = socket_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = socket::server::start_pipe_server(pipe_path, socket_tx_clone).await {
            eprintln!("Socket server error: {}", e);
        }
    });

    app::run(args.shell, args.pipe, socket_rx, socket_tx).await?;

    Ok(())
}
