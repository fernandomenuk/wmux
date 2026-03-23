#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;
use wmux_core::terminal::shell::detect_shell;
use wmux_core::WmuxCore;

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

#[derive(Clone, Serialize)]
struct PaneInfo {
    surface_id: String,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    is_focused: bool,
}

#[derive(Clone, Serialize)]
struct LayoutResult {
    panes: Vec<PaneInfo>,
    surface_ids: Vec<String>,
    is_zoomed: bool,
    shell: String,
}

#[derive(Clone, Serialize)]
struct TabInfo {
    name: String,
    is_active: bool,
}

#[derive(Clone, Serialize)]
struct TabInfoResult {
    tabs: Vec<TabInfo>,
    active_index: usize,
}

#[derive(Clone, Serialize)]
struct SplitResult {
    surface_id: String,
}

#[derive(Clone, Serialize)]
struct MetricsResult {
    cpu: f32,
    memory: u64,
}

#[derive(Clone, Serialize)]
struct CreateResult {
    workspace_id: String,
}

#[derive(Clone, Serialize)]
struct CloseResult {
    should_quit: bool,
}

#[derive(Clone, Serialize)]
struct FocusChangedPayload {
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
        surface
            .send_bytes(data.as_bytes())
            .map_err(|e| e.to_string())?;
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

#[tauri::command]
async fn split_pane(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    direction: String,
) -> Result<SplitResult, String> {
    let mut core = state.core.lock().await;
    let dir = match direction.as_str() {
        "horizontal" => wmux_core::model::split_tree::Direction::Horizontal,
        _ => wmux_core::model::split_tree::Direction::Vertical,
    };
    let (cols, rows) = core.terminal_size;
    let result = core
        .split_surface(dir, &state.pty_tx, &state.exit_tx, cols / 2, rows / 2)
        .map_err(|e| e.to_string())?;
    match result {
        Some(id) => {
            let _ = app_handle.emit("layout-changed", ());
            let _ = app_handle.emit(
                "focus-changed",
                FocusChangedPayload {
                    surface_id: id.to_string(),
                },
            );
            Ok(SplitResult {
                surface_id: id.to_string(),
            })
        }
        None => Err("No focused surface to split".to_string()),
    }
}

#[tauri::command]
async fn close_pane(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    surface_id: String,
) -> Result<CloseResult, String> {
    let mut core = state.core.lock().await;
    let id = Uuid::parse_str(&surface_id).map_err(|e| e.to_string())?;
    let should_quit = core.close_surface(id);
    let _ = app_handle.emit("layout-changed", ());
    if let Some(focused) = core.focused_surface {
        let _ = app_handle.emit(
            "focus-changed",
            FocusChangedPayload {
                surface_id: focused.to_string(),
            },
        );
    }
    Ok(CloseResult { should_quit })
}

#[tauri::command]
async fn focus_pane(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    surface_id: String,
) -> Result<(), String> {
    let mut core = state.core.lock().await;
    let id = Uuid::parse_str(&surface_id).map_err(|e| e.to_string())?;
    core.focus_surface(id);
    if let Some(focused) = core.focused_surface {
        let _ = app_handle.emit(
            "focus-changed",
            FocusChangedPayload {
                surface_id: focused.to_string(),
            },
        );
    }
    Ok(())
}

#[tauri::command]
async fn focus_direction(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    direction: String,
) -> Result<(), String> {
    let mut core = state.core.lock().await;
    let dir = match direction.as_str() {
        "up" => wmux_core::FocusDirection::Up,
        "down" => wmux_core::FocusDirection::Down,
        "left" => wmux_core::FocusDirection::Left,
        "right" => wmux_core::FocusDirection::Right,
        _ => return Err(format!("Invalid direction: {}", direction)),
    };
    core.focus_direction(dir);
    if let Some(focused) = core.focused_surface {
        let _ = app_handle.emit(
            "focus-changed",
            FocusChangedPayload {
                surface_id: focused.to_string(),
            },
        );
    }
    Ok(())
}

#[tauri::command]
async fn create_workspace(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    name: Option<String>,
) -> Result<CreateResult, String> {
    let mut core = state.core.lock().await;
    let (cols, rows) = core.terminal_size;
    let ws_id = core
        .create_workspace(name, &state.pty_tx, &state.exit_tx, cols, rows)
        .map_err(|e| e.to_string())?;
    let _ = app_handle.emit("layout-changed", ());
    Ok(CreateResult {
        workspace_id: ws_id.to_string(),
    })
}

#[tauri::command]
async fn switch_workspace(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    index: usize,
) -> Result<(), String> {
    let mut core = state.core.lock().await;
    core.switch_workspace(index);
    let _ = app_handle.emit("layout-changed", ());
    if let Some(focused) = core.focused_surface {
        let _ = app_handle.emit(
            "focus-changed",
            FocusChangedPayload {
                surface_id: focused.to_string(),
            },
        );
    }
    Ok(())
}

#[tauri::command]
async fn next_workspace(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut core = state.core.lock().await;
    core.next_workspace();
    let _ = app_handle.emit("layout-changed", ());
    if let Some(focused) = core.focused_surface {
        let _ = app_handle.emit(
            "focus-changed",
            FocusChangedPayload {
                surface_id: focused.to_string(),
            },
        );
    }
    Ok(())
}

#[tauri::command]
async fn prev_workspace(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut core = state.core.lock().await;
    core.prev_workspace();
    let _ = app_handle.emit("layout-changed", ());
    if let Some(focused) = core.focused_surface {
        let _ = app_handle.emit(
            "focus-changed",
            FocusChangedPayload {
                surface_id: focused.to_string(),
            },
        );
    }
    Ok(())
}

#[tauri::command]
async fn get_layout(
    state: tauri::State<'_, Arc<AppState>>,
    width: u16,
    height: u16,
) -> Result<LayoutResult, String> {
    let mut core = state.core.lock().await;

    // Update terminal_size so new splits use correct dimensions
    core.set_terminal_size(width, height);

    let panes = if let Some(zoom_id) = core.zoom_surface {
        // Zoomed: single pane fills entire area
        vec![PaneInfo {
            surface_id: zoom_id.to_string(),
            x: 0,
            y: 0,
            width,
            height,
            is_focused: true,
        }]
    } else if let Some(ws) = core.active_workspace_ref() {
        ws.split_tree
            .layout(0, 0, width, height)
            .iter()
            .map(|l| PaneInfo {
                surface_id: l.surface_id.to_string(),
                x: l.x,
                y: l.y,
                width: l.width,
                height: l.height,
                is_focused: core.focused_surface == Some(l.surface_id),
            })
            .collect()
    } else {
        vec![]
    };

    let shell_name = core
        .shell
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(&core.shell)
        .to_string();
    let surface_ids = core.surfaces.keys().map(|id| id.to_string()).collect();

    Ok(LayoutResult {
        panes,
        surface_ids,
        is_zoomed: core.zoom_surface.is_some(),
        shell: shell_name,
    })
}

#[tauri::command]
async fn get_tab_info(state: tauri::State<'_, Arc<AppState>>) -> Result<TabInfoResult, String> {
    let core = state.core.lock().await;
    let tabs: Vec<TabInfo> = core
        .tab_info()
        .into_iter()
        .map(|(name, is_active)| TabInfo { name, is_active })
        .collect();
    let active_index = core.active_workspace;
    Ok(TabInfoResult { tabs, active_index })
}

#[tauri::command]
async fn toggle_zoom(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut core = state.core.lock().await;
    core.toggle_zoom();
    let _ = app_handle.emit("layout-changed", ());
    Ok(())
}

#[tauri::command]
async fn get_process_metrics(
    state: tauri::State<'_, Arc<AppState>>,
    surface_id: String,
) -> Result<MetricsResult, String> {
    let mut core = state.core.lock().await;
    let id = Uuid::parse_str(&surface_id).map_err(|e| e.to_string())?;
    match core.get_process_metrics(id) {
        Some((cpu, memory)) => Ok(MetricsResult { cpu, memory }),
        None => Err("Process not found or PID 0".to_string()),
    }
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
            split_pane,
            close_pane,
            focus_pane,
            focus_direction,
            create_workspace,
            switch_workspace,
            next_workspace,
            prev_workspace,
            get_layout,
            get_tab_info,
            toggle_zoom,
            get_process_metrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
