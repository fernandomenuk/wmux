#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Serialize;
use tauri::{Emitter, Manager};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;
use wmux_core::WmuxCore;
use wmux_core::terminal::shell::detect_shell;

/// Shared application state accessible from Tauri commands
///
/// `pty_tx` and `exit_tx` are held here to keep the channel senders alive;
/// dropping them would close the channels and terminate the PTY reader loops.
#[allow(dead_code)]
struct AppState {
    core: Mutex<WmuxCore>,
    pty_tx: mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
    exit_tx: mpsc::UnboundedSender<Uuid>,
}

#[derive(Clone, Serialize)]
struct PtyOutputPayload {
    surface_id: String,
    data: String, // base64-encoded
}

#[derive(Clone, Serialize)]
struct PtyExitPayload {
    surface_id: String,
}

// ── Tauri Commands ──

#[tauri::command]
async fn get_surface_id(state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    let core = state.core.lock().await;
    core.focused_surface
        .map(|id| id.to_string())
        .ok_or_else(|| "No focused surface".to_string())
}

#[tauri::command]
async fn send_input(
    state: tauri::State<'_, Arc<AppState>>,
    surface_id: String,
    data: String,
) -> Result<(), String> {
    let mut core = state.core.lock().await;
    let id = Uuid::parse_str(&surface_id).map_err(|e| e.to_string())?;
    if let Some(surface) = core.surfaces.get_mut(&id) {
        surface.send_bytes(data.as_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn resize_terminal(
    state: tauri::State<'_, Arc<AppState>>,
    surface_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut core = state.core.lock().await;
    let id = Uuid::parse_str(&surface_id).map_err(|e| e.to_string())?;
    if let Some(surface) = core.surfaces.get_mut(&id) {
        surface.resize(cols, rows);
    }
    Ok(())
}

// ── App Setup ──

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Create PTY channels
            let (pty_tx, mut pty_rx) = mpsc::unbounded_channel::<(Uuid, Vec<u8>)>();
            let (exit_tx, mut exit_rx) = mpsc::unbounded_channel::<Uuid>();

            // Detect shell and create core
            let shell = detect_shell(None);
            let mut core = WmuxCore::new(shell, String::new());

            // Create initial workspace with default size (frontend will resize)
            if let Err(e) = core.create_workspace(None, &pty_tx, &exit_tx, 80, 24) {
                eprintln!("Failed to create initial workspace: {}", e);
            }

            // Store state
            let state = Arc::new(AppState {
                core: Mutex::new(core),
                pty_tx,
                exit_tx,
            });
            app.manage(state.clone());

            // Spawn channel-to-event bridge
            let bridge_state = state.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::select! {
                        Some((id, data)) = pty_rx.recv() => {
                            let payload = PtyOutputPayload {
                                surface_id: id.to_string(),
                                data: BASE64.encode(&data),
                            };
                            let _ = app_handle.emit("pty-output", payload);
                        }
                        Some(id) = exit_rx.recv() => {
                            // Mark surface as exited in core
                            let mut core = bridge_state.core.lock().await;
                            core.handle_pty_exit(id);
                            drop(core);

                            let payload = PtyExitPayload {
                                surface_id: id.to_string(),
                            };
                            let _ = app_handle.emit("pty-exit", payload);
                        }
                        else => break,
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_surface_id,
            send_input,
            resize_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
