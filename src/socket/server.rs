use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::ServerOptions;
use crate::app::SocketRequest;
use crate::socket::protocol::{Request, Response};

pub async fn start_pipe_server(
    pipe_path: String,
    cmd_tx: mpsc::UnboundedSender<SocketRequest>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let server = ServerOptions::new()
            .first_pipe_instance(false)
            .create(&pipe_path)?;

        server.connect().await?;

        let cmd_tx = cmd_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(server, cmd_tx).await {
                eprintln!("Socket connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(
    pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    cmd_tx: mpsc::UnboundedSender<SocketRequest>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = tokio::io::split(pipe);
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let request: Request = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let err_resp = Response::error(
                    "".into(),
                    "parse_error",
                    &format!("Invalid JSON: {}", e),
                );
                let mut json = serde_json::to_string(&err_resp)?;
                json.push('\n');
                writer.write_all(json.as_bytes()).await?;
                continue;
            }
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let socket_req = SocketRequest {
            request,
            response_tx: resp_tx,
        };

        if cmd_tx.send(socket_req).is_err() {
            break;
        }

        match resp_rx.await {
            Ok(response) => {
                let mut json = serde_json::to_string(&response)?;
                json.push('\n');
                writer.write_all(json.as_bytes()).await?;
            }
            Err(_) => break,
        }
    }

    Ok(())
}
