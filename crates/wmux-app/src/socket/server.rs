use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::ServerOptions;
use tauri::Emitter;
use wmux_core::socket::protocol::{Request, Response};
use wmux_core::socket::commands::dispatch;
use crate::{AppState, FocusChangedPayload};

pub async fn start_pipe_server(
    app_handle: tauri::AppHandle,
    state: Arc<AppState>,
    pipe_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let server = ServerOptions::new()
            .first_pipe_instance(false)
            .create(&pipe_path)?;

        server.connect().await?;

        let state = state.clone();
        let handle = app_handle.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(handle, server, state).await {
                eprintln!("Socket connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(
    app_handle: tauri::AppHandle,
    pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = tokio::io::split(pipe);
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let request: Request = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let err_resp =
                    Response::error("".into(), "parse_error", &format!("Invalid JSON: {}", e));
                let mut json = serde_json::to_string(&err_resp)?;
                json.push('\n');
                writer.write_all(json.as_bytes()).await?;
                continue;
            }
        };

        // Dispatch command to core
        let response = {
            let mut core = state.core.lock().await;
            let res = dispatch(&mut core, &request, &state.pty_tx, &state.exit_tx);
            
            // Notify UI of changes
            if !request.method.starts_with("system.") && !request.method.ends_with(".list") {
                let _ = app_handle.emit("layout-changed", ());
                if let Some(focused) = core.focused_surface {
                    let _ = app_handle.emit("focus-changed", FocusChangedPayload {
                        surface_id: focused.to_string(),
                    });
                }
            }
            res
        };

        let mut json = serde_json::to_string(&response)?;
        json.push('\n');
        writer.write_all(json.as_bytes()).await?;
    }

    Ok(())
}
