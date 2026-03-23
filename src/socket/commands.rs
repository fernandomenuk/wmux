use serde_json::{json, Value};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::app::App;
use crate::model::split_tree::Direction;
use crate::socket::protocol::{Request, Response};

pub fn dispatch(
    app: &mut App,
    req: &Request,
    pty_tx: &mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
    exit_tx: &mpsc::UnboundedSender<Uuid>,
) -> Response {
    match req.method.as_str() {
        "system.ping" => Response::success(req.id.clone(), json!({"pong": true})),

        "system.capabilities" => Response::success(req.id.clone(), json!({
            "version": "0.1.0",
            "commands": [
                "system.ping", "system.capabilities",
                "workspace.list", "workspace.create", "workspace.select",
                "workspace.current", "workspace.close",
                "surface.list", "surface.split", "surface.focus",
                "surface.close", "surface.send_text", "surface.send_key"
            ]
        })),

        "workspace.list" => {
            let workspaces: Vec<Value> = app.workspaces.iter().enumerate().map(|(i, ws)| {
                json!({"id": ws.id.to_string(), "name": ws.name, "index": i})
            }).collect();
            Response::success(req.id.clone(), json!({"workspaces": workspaces}))
        }

        "workspace.create" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let name = params.get("name").and_then(|v| v.as_str()).map(String::from);
            let (cols, rows) = app.terminal_size;
            match app.create_workspace(name, pty_tx, exit_tx, cols, rows) {
                Ok(ws_id) => Response::success(req.id.clone(), json!({"id": ws_id.to_string()})),
                Err(e) => Response::error(req.id.clone(), "create_failed", &e.to_string()),
            }
        }

        "workspace.select" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let id_str = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
            match Uuid::parse_str(id_str) {
                Ok(id) => {
                    if let Some(idx) = app.workspaces.iter().position(|ws| ws.id == id) {
                        app.active_workspace = idx;
                        app.focused_surface = Some(
                            app.workspaces[idx].split_tree.first_surface(),
                        );
                        Response::success(req.id.clone(), json!({}))
                    } else {
                        Response::error(req.id.clone(), "not_found", "Workspace not found")
                    }
                }
                Err(_) => Response::error(req.id.clone(), "invalid_id", "Invalid UUID"),
            }
        }

        "workspace.current" => {
            if let Some(ws) = app.active_workspace_ref() {
                Response::success(req.id.clone(), json!({
                    "id": ws.id.to_string(),
                    "name": ws.name,
                    "index": app.active_workspace
                }))
            } else {
                Response::error(req.id.clone(), "no_workspace", "No active workspace")
            }
        }

        "workspace.close" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let id_str = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
            match Uuid::parse_str(id_str) {
                Ok(id) => {
                    if let Some(idx) = app.workspaces.iter().position(|ws| ws.id == id) {
                        let surface_ids = app.workspaces[idx].split_tree.surface_ids();
                        for sid in &surface_ids {
                            app.surfaces.remove(sid);
                        }
                        app.workspaces.remove(idx);
                        if app.workspaces.is_empty() {
                            app.should_quit = true;
                        } else {
                            app.active_workspace = app.active_workspace.min(app.workspaces.len() - 1);
                            app.focused_surface = Some(
                                app.workspaces[app.active_workspace].split_tree.first_surface(),
                            );
                        }
                        Response::success(req.id.clone(), json!({}))
                    } else {
                        Response::error(req.id.clone(), "not_found", "Workspace not found")
                    }
                }
                Err(_) => Response::error(req.id.clone(), "invalid_id", "Invalid UUID"),
            }
        }

        "surface.list" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let ws_id_str = params.get("workspace_id").and_then(|v| v.as_str());

            let ws = if let Some(id_str) = ws_id_str {
                Uuid::parse_str(id_str).ok().and_then(|id| {
                    app.workspaces.iter().find(|ws| ws.id == id)
                })
            } else {
                app.active_workspace_ref()
            };

            if let Some(ws) = ws {
                let surfaces: Vec<Value> = ws.split_tree.surface_ids().iter().map(|id| {
                    json!({
                        "id": id.to_string(),
                        "focused": app.focused_surface == Some(*id)
                    })
                }).collect();
                Response::success(req.id.clone(), json!({"surfaces": surfaces}))
            } else {
                Response::error(req.id.clone(), "not_found", "Workspace not found")
            }
        }

        "surface.split" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let dir_str = params.get("direction").and_then(|v| v.as_str()).unwrap_or("vertical");
            let direction = match dir_str {
                "horizontal" => Direction::Horizontal,
                _ => Direction::Vertical,
            };
            let (cols, rows) = app.terminal_size;
            match app.split_surface(direction, pty_tx, exit_tx, cols, rows) {
                Ok(Some(id)) => Response::success(req.id.clone(), json!({"id": id.to_string()})),
                Ok(None) => Response::error(req.id.clone(), "no_focus", "No focused surface to split"),
                Err(e) => Response::error(req.id.clone(), "split_failed", &e.to_string()),
            }
        }

        "surface.focus" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let id_str = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
            match Uuid::parse_str(id_str) {
                Ok(id) => {
                    if app.surfaces.contains_key(&id) {
                        app.focused_surface = Some(id);
                        Response::success(req.id.clone(), json!({}))
                    } else {
                        Response::error(req.id.clone(), "not_found", "Surface not found")
                    }
                }
                Err(_) => Response::error(req.id.clone(), "invalid_id", "Invalid UUID"),
            }
        }

        "surface.close" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let id_str = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
            match Uuid::parse_str(id_str) {
                Ok(id) => {
                    if app.surfaces.contains_key(&id) {
                        app.close_surface(id);
                        Response::success(req.id.clone(), json!({}))
                    } else {
                        Response::error(req.id.clone(), "not_found", "Surface not found")
                    }
                }
                Err(_) => Response::error(req.id.clone(), "invalid_id", "Invalid UUID"),
            }
        }

        "surface.send_text" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let id_str = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");
            match Uuid::parse_str(id_str) {
                Ok(id) => {
                    if let Some(surface) = app.surfaces.get_mut(&id) {
                        match surface.send_text(text) {
                            Ok(_) => Response::success(req.id.clone(), json!({})),
                            Err(e) => Response::error(req.id.clone(), "send_failed", &e.to_string()),
                        }
                    } else {
                        Response::error(req.id.clone(), "not_found", "Surface not found")
                    }
                }
                Err(_) => Response::error(req.id.clone(), "invalid_id", "Invalid UUID"),
            }
        }

        "surface.send_key" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let id_str = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let key = params.get("key").and_then(|v| v.as_str()).unwrap_or("");
            match Uuid::parse_str(id_str) {
                Ok(id) => {
                    if let Some(surface) = app.surfaces.get_mut(&id) {
                        match surface.send_key(key) {
                            Ok(_) => Response::success(req.id.clone(), json!({})),
                            Err(e) => Response::error(req.id.clone(), "send_failed", &e.to_string()),
                        }
                    } else {
                        Response::error(req.id.clone(), "not_found", "Surface not found")
                    }
                }
                Err(_) => Response::error(req.id.clone(), "invalid_id", "Invalid UUID"),
            }
        }

        _ => Response::error(req.id.clone(), "unknown_method", &format!("Unknown method: {}", req.method)),
    }
}
