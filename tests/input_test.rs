use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind, KeyEventState, MouseEvent, MouseEventKind, MouseButton};
use wmux::input::{Action, InputHandler, key_event_to_bytes, mouse_event_to_sgr_bytes};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

// === Prefix mode state machine ===

#[test]
fn normal_key_forwards_to_surface() {
    let mut handler = InputHandler::new();
    let action = handler.handle_key(key(KeyCode::Char('a')));
    assert!(matches!(action, Action::ForwardToSurface(_)));
}

#[test]
fn ctrl_a_enters_prefix_mode() {
    let mut handler = InputHandler::new();
    let action = handler.handle_key(ctrl('a'));
    assert_eq!(action, Action::None);
    assert!(handler.is_prefix_mode());
}

#[test]
fn prefix_mode_exits_after_one_key() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a')); // enter prefix
    handler.handle_key(key(KeyCode::Char('c'))); // dispatch
    assert!(!handler.is_prefix_mode());
}

#[test]
fn unrecognized_prefix_key_returns_none() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    let action = handler.handle_key(key(KeyCode::Char('!')));
    assert_eq!(action, Action::None);
}

// === Keybinding dispatch ===

#[test]
fn prefix_c_creates_workspace() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('c'))), Action::NewWorkspace);
}

#[test]
fn prefix_n_next_workspace() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('n'))), Action::NextWorkspace);
}

#[test]
fn prefix_p_prev_workspace() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('p'))), Action::PrevWorkspace);
}

#[test]
fn prefix_pipe_splits_vertical() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('|'))), Action::SplitVertical);
}

#[test]
fn prefix_backslash_splits_vertical() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('\\'))), Action::SplitVertical);
}

#[test]
fn prefix_dash_splits_horizontal() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('-'))), Action::SplitHorizontal);
}

#[test]
fn prefix_x_closes_surface() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('x'))), Action::CloseSurface);
}

#[test]
fn prefix_q_quits() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('q'))), Action::Quit);
}

#[test]
fn prefix_z_toggles_zoom() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('z'))), Action::ToggleZoom);
}

#[test]
fn prefix_1_selects_workspace_0() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('1'))), Action::SelectWorkspace(0));
}

#[test]
fn prefix_9_selects_workspace_8() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Char('9'))), Action::SelectWorkspace(8));
}

#[test]
fn prefix_arrows_focus() {
    let mut handler = InputHandler::new();

    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Up)), Action::FocusUp);

    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Down)), Action::FocusDown);

    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Left)), Action::FocusLeft);

    handler.handle_key(ctrl('a'));
    assert_eq!(handler.handle_key(key(KeyCode::Right)), Action::FocusRight);
}

#[test]
fn prefix_ctrl_a_forwards_literal_ctrl_a() {
    let mut handler = InputHandler::new();
    handler.handle_key(ctrl('a'));
    let action = handler.handle_key(ctrl('a'));
    assert!(matches!(action, Action::ForwardToSurface(_)));
}

// === key_event_to_bytes ===

#[test]
fn regular_char_to_bytes() {
    let bytes = key_event_to_bytes(&key(KeyCode::Char('x'))).unwrap();
    assert_eq!(bytes, b"x");
}

#[test]
fn ctrl_c_to_bytes() {
    let bytes = key_event_to_bytes(&ctrl('c')).unwrap();
    assert_eq!(bytes, vec![3]); // 0x03
}

#[test]
fn ctrl_d_to_bytes() {
    let bytes = key_event_to_bytes(&ctrl('d')).unwrap();
    assert_eq!(bytes, vec![4]); // 0x04
}

#[test]
fn enter_to_bytes() {
    let bytes = key_event_to_bytes(&key(KeyCode::Enter)).unwrap();
    assert_eq!(bytes, b"\r");
}

#[test]
fn tab_to_bytes() {
    let bytes = key_event_to_bytes(&key(KeyCode::Tab)).unwrap();
    assert_eq!(bytes, b"\t");
}

#[test]
fn backspace_to_bytes() {
    let bytes = key_event_to_bytes(&key(KeyCode::Backspace)).unwrap();
    assert_eq!(bytes, b"\x7f");
}

#[test]
fn escape_to_bytes() {
    let bytes = key_event_to_bytes(&key(KeyCode::Esc)).unwrap();
    assert_eq!(bytes, b"\x1b");
}

#[test]
fn arrow_keys_to_bytes() {
    assert_eq!(key_event_to_bytes(&key(KeyCode::Up)).unwrap(), b"\x1b[A");
    assert_eq!(key_event_to_bytes(&key(KeyCode::Down)).unwrap(), b"\x1b[B");
    assert_eq!(key_event_to_bytes(&key(KeyCode::Right)).unwrap(), b"\x1b[C");
    assert_eq!(key_event_to_bytes(&key(KeyCode::Left)).unwrap(), b"\x1b[D");
}

#[test]
fn home_end_delete_to_bytes() {
    assert_eq!(key_event_to_bytes(&key(KeyCode::Home)).unwrap(), b"\x1b[H");
    assert_eq!(key_event_to_bytes(&key(KeyCode::End)).unwrap(), b"\x1b[F");
    assert_eq!(key_event_to_bytes(&key(KeyCode::Delete)).unwrap(), b"\x1b[3~");
}

#[test]
fn function_keys_to_bytes() {
    assert_eq!(key_event_to_bytes(&key(KeyCode::F(1))).unwrap(), b"\x1bOP");
    assert_eq!(key_event_to_bytes(&key(KeyCode::F(4))).unwrap(), b"\x1bOS");
    assert_eq!(key_event_to_bytes(&key(KeyCode::F(5))).unwrap(), b"\x1b[15~");
    assert_eq!(key_event_to_bytes(&key(KeyCode::F(12))).unwrap(), b"\x1b[24~");
}

#[test]
fn unicode_char_to_bytes() {
    let bytes = key_event_to_bytes(&key(KeyCode::Char('ñ'))).unwrap();
    assert_eq!(bytes, "ñ".as_bytes());
}

#[test]
fn unknown_key_returns_none() {
    let bytes = key_event_to_bytes(&key(KeyCode::PrintScreen));
    assert!(bytes.is_none());
}

// === Mouse SGR encoding ===

fn mouse(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
    MouseEvent { kind, column: col, row, modifiers: KeyModifiers::NONE }
}

#[test]
fn mouse_sgr_left_click() {
    let ev = mouse(MouseEventKind::Down(MouseButton::Left), 5, 10);
    let bytes = mouse_event_to_sgr_bytes(&ev, 3, 8).unwrap();
    // SGR: \x1b[<0;4;9M  (0=left, 1-indexed coords)
    assert_eq!(bytes, b"\x1b[<0;4;9M");
}

#[test]
fn mouse_sgr_release() {
    let ev = mouse(MouseEventKind::Up(MouseButton::Left), 5, 10);
    let bytes = mouse_event_to_sgr_bytes(&ev, 3, 8).unwrap();
    // Release uses lowercase 'm'
    assert_eq!(bytes, b"\x1b[<0;4;9m");
}

#[test]
fn mouse_sgr_right_click() {
    let ev = mouse(MouseEventKind::Down(MouseButton::Right), 0, 0);
    let bytes = mouse_event_to_sgr_bytes(&ev, 0, 0).unwrap();
    assert_eq!(bytes, b"\x1b[<2;1;1M");
}

#[test]
fn mouse_sgr_scroll_up() {
    let ev = mouse(MouseEventKind::ScrollUp, 5, 5);
    let bytes = mouse_event_to_sgr_bytes(&ev, 5, 5).unwrap();
    assert_eq!(bytes, b"\x1b[<64;6;6M");
}

#[test]
fn mouse_sgr_scroll_down() {
    let ev = mouse(MouseEventKind::ScrollDown, 5, 5);
    let bytes = mouse_event_to_sgr_bytes(&ev, 5, 5).unwrap();
    assert_eq!(bytes, b"\x1b[<65;6;6M");
}

#[test]
fn mouse_sgr_drag() {
    let ev = mouse(MouseEventKind::Drag(MouseButton::Left), 10, 10);
    let bytes = mouse_event_to_sgr_bytes(&ev, 10, 10).unwrap();
    assert_eq!(bytes, b"\x1b[<32;11;11M");
}

#[test]
fn mouse_sgr_coordinates_1_indexed() {
    let ev = mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    let bytes = mouse_event_to_sgr_bytes(&ev, 0, 0).unwrap();
    // 0,0 relative → 1,1 in SGR
    assert_eq!(bytes, b"\x1b[<0;1;1M");
}
