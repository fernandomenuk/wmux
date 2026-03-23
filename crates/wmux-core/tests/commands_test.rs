use serde_json::{json, Value};
use tokio::sync::mpsc;
use uuid::Uuid;

use wmux_core::WmuxCore;
use wmux_core::socket::commands::dispatch;
use wmux_core::socket::protocol::Request;

fn make_request(id: &str, method: &str, params: Value) -> Request {
    serde_json::from_value(json!({
        "id": id,
        "method": method,
        "params": params
    }))
    .unwrap()
}

fn setup_core_with_workspace() -> (WmuxCore, mpsc::UnboundedSender<(Uuid, Vec<u8>)>, mpsc::UnboundedSender<Uuid>) {
    let (pty_tx, _pty_rx) = mpsc::unbounded_channel();
    let (exit_tx, _exit_rx) = mpsc::unbounded_channel();
    let mut core = WmuxCore::new("cmd.exe".into(), r"\\.\pipe\wmux-test".into());
    core.create_workspace(Some("test".into()), &pty_tx, &exit_tx, 80, 24)
        .expect("Failed to create workspace");
    (core, pty_tx, exit_tx)
}

// === system.ping ===

#[test]
fn system_ping() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("1", "system.ping", json!({}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    assert_eq!(resp.result.unwrap()["pong"], true);
}

// === system.capabilities ===

#[test]
fn system_capabilities_returns_version_and_commands() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("1", "system.capabilities", json!({}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    let result = resp.result.unwrap();
    assert_eq!(result["version"], "0.1.0");
    let commands = result["commands"].as_array().unwrap();
    assert!(commands.len() >= 13);
    assert!(commands.contains(&json!("system.ping")));
    assert!(commands.contains(&json!("surface.send_text")));
}

// === workspace.list ===

#[test]
fn workspace_list_returns_workspaces() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("1", "workspace.list", json!({}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    let workspaces = resp.result.unwrap()["workspaces"].as_array().unwrap().clone();
    assert_eq!(workspaces.len(), 1);
    assert_eq!(workspaces[0]["name"], "test");
    assert_eq!(workspaces[0]["index"], 0);
}

// === workspace.create ===

#[test]
fn workspace_create_adds_workspace() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("1", "workspace.create", json!({"name": "new-ws"}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    assert!(resp.result.unwrap()["id"].as_str().is_some());
    assert_eq!(core.workspaces.len(), 2);
    assert_eq!(core.workspaces[1].name, "new-ws");
}

// === workspace.select ===

#[test]
fn workspace_select_switches_active() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    // Create a second workspace
    core.create_workspace(Some("second".into()), &pty_tx, &exit_tx, 80, 24).unwrap();
    let ws_id = core.workspaces[0].id.to_string();

    let req = make_request("1", "workspace.select", json!({"id": ws_id}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    assert_eq!(core.active_workspace, 0);
}

#[test]
fn workspace_select_invalid_id_errors() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("1", "workspace.select", json!({"id": "not-a-uuid"}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(!resp.ok);
    assert_eq!(resp.error.unwrap().code, "invalid_id");
}

#[test]
fn workspace_select_nonexistent_errors() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let fake_id = Uuid::new_v4().to_string();
    let req = make_request("1", "workspace.select", json!({"id": fake_id}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(!resp.ok);
    assert_eq!(resp.error.unwrap().code, "not_found");
}

// === workspace.current ===

#[test]
fn workspace_current_returns_active() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("1", "workspace.current", json!({}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    let result = resp.result.unwrap();
    assert_eq!(result["name"], "test");
    assert_eq!(result["index"], 0);
}

// === workspace.close ===

#[test]
fn workspace_close_removes_workspace() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    // Need at least 2 workspaces so closing one doesn't quit
    core.create_workspace(Some("second".into()), &pty_tx, &exit_tx, 80, 24).unwrap();
    let ws_id = core.workspaces[0].id.to_string();

    let req = make_request("1", "workspace.close", json!({"id": ws_id}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    assert_eq!(core.workspaces.len(), 1);
    assert_eq!(core.workspaces[0].name, "second");
}

#[test]
fn workspace_close_last_sets_should_quit() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let ws_id = core.workspaces[0].id.to_string();

    let req = make_request("1", "workspace.close", json!({"id": ws_id}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    assert!(core.should_quit());
}

// === surface.list ===

#[test]
fn surface_list_returns_surfaces() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("1", "surface.list", json!({}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    let surfaces = resp.result.unwrap()["surfaces"].as_array().unwrap().clone();
    assert_eq!(surfaces.len(), 1);
    assert_eq!(surfaces[0]["focused"], true);
}

// === surface.split ===

#[test]
fn surface_split_creates_new_surface() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("1", "surface.split", json!({"direction": "vertical"}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    assert!(resp.result.unwrap()["id"].as_str().is_some());
    assert_eq!(core.surfaces.len(), 2);
}

#[test]
fn surface_split_horizontal() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("1", "surface.split", json!({"direction": "horizontal"}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    assert_eq!(core.surfaces.len(), 2);
}

// === surface.focus ===

#[test]
fn surface_focus_changes_focused() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    // Split to get a second surface
    let split_req = make_request("1", "surface.split", json!({"direction": "vertical"}));
    dispatch(&mut core, &split_req, &pty_tx, &exit_tx);

    let surface_ids: Vec<Uuid> = core.surfaces.keys().copied().collect();
    let unfocused = surface_ids
        .iter()
        .find(|id| Some(**id) != core.focused_surface)
        .unwrap();

    let req = make_request("1", "surface.focus", json!({"id": unfocused.to_string()}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    assert_eq!(core.focused_surface, Some(*unfocused));
}

#[test]
fn surface_focus_nonexistent_errors() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let fake_id = Uuid::new_v4().to_string();
    let req = make_request("1", "surface.focus", json!({"id": fake_id}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(!resp.ok);
    assert_eq!(resp.error.unwrap().code, "not_found");
}

// === surface.close ===

#[test]
fn surface_close_removes_surface() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    // Split first so we have 2 surfaces
    let split_req = make_request("1", "surface.split", json!({"direction": "vertical"}));
    dispatch(&mut core, &split_req, &pty_tx, &exit_tx);
    assert_eq!(core.surfaces.len(), 2);

    let id = core.focused_surface.unwrap().to_string();
    let req = make_request("1", "surface.close", json!({"id": id}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    assert_eq!(core.surfaces.len(), 1);
}

// === surface.send_text ===

#[test]
fn surface_send_text_to_valid_surface() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let id = core.focused_surface.unwrap().to_string();
    let req = make_request("1", "surface.send_text", json!({"id": id, "text": "echo hello\r"}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
}

#[test]
fn surface_send_text_nonexistent_errors() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let fake_id = Uuid::new_v4().to_string();
    let req = make_request("1", "surface.send_text", json!({"id": fake_id, "text": "hello"}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(!resp.ok);
    assert_eq!(resp.error.unwrap().code, "not_found");
}

// === surface.send_key ===

#[test]
fn surface_send_key_enter() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let id = core.focused_surface.unwrap().to_string();
    let req = make_request("1", "surface.send_key", json!({"id": id, "key": "Enter"}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
}

#[test]
fn surface_send_key_unknown_errors() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let id = core.focused_surface.unwrap().to_string();
    let req = make_request("1", "surface.send_key", json!({"id": id, "key": "FakeKey"}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(!resp.ok);
    assert_eq!(resp.error.unwrap().code, "send_failed");
}

// === unknown method ===

#[test]
fn unknown_method_errors() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("1", "bogus.method", json!({}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(!resp.ok);
    assert_eq!(resp.error.unwrap().code, "unknown_method");
}

// === response ID passthrough ===

#[test]
fn response_preserves_request_id() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let req = make_request("custom-id-42", "system.ping", json!({}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert_eq!(resp.id, "custom-id-42");
}
