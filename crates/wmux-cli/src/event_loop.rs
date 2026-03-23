use std::io;

use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind, MouseEventKind, MouseButton, EnableMouseCapture, DisableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use uuid::Uuid;

use wmux_core::{WmuxCore, FocusDirection};
use crate::input::{Action, InputHandler, key_event_to_bytes, mouse_event_to_sgr_bytes};
use crate::model::split_tree::Direction;
use crate::terminal::shell::detect_shell;
use crate::tui::render::{render_frame, RenderContext};

pub struct DragState {
    pub border_path: Vec<bool>,
    pub direction: Direction,
    pub region_x: u16,
    pub region_y: u16,
    pub region_w: u16,
    pub region_h: u16,
}

pub struct SocketRequest {
    pub request: crate::socket::protocol::Request,
    pub response_tx: tokio::sync::oneshot::Sender<crate::socket::protocol::Response>,
}

/// Restore terminal to normal state.
fn cleanup_terminal() {
    let _ = io::stdout().execute(DisableMouseCapture);
    let _ = terminal::disable_raw_mode();
    let _ = io::stdout().execute(LeaveAlternateScreen);
}

/// Main event loop.
pub async fn run(
    cli_shell: Option<String>,
    pipe_path: String,
    socket_rx: mpsc::UnboundedReceiver<SocketRequest>,
    _socket_cmd_tx: mpsc::UnboundedSender<SocketRequest>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let shell = detect_shell(cli_shell);
    let mut core = WmuxCore::new(shell, pipe_path);
    let mut input_handler = InputHandler::new();

    let (pty_tx, mut pty_rx) = mpsc::unbounded_channel::<(Uuid, Vec<u8>)>();
    let (exit_tx, mut exit_rx) = mpsc::unbounded_channel::<Uuid>();

    let size = terminal.size()?;
    let content_height = size.height.saturating_sub(2);
    core.set_terminal_size(size.width, content_height);
    if let Err(e) = core.create_workspace(None, &pty_tx, &exit_tx, size.width, content_height) {
        cleanup_terminal();
        return Err(format!("Failed to start shell: {}. Use --shell to specify a different shell.", e).into());
    }

    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Event>();
    std::thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(ev) = event::read() {
                    if input_tx.send(ev).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let mut socket_rx = socket_rx;
    let mut render_interval = tokio::time::interval(Duration::from_millis(33));
    let mut drag_state: Option<DragState> = None;

    loop {
        tokio::select! {
            Some(ev) = input_rx.recv() => {
                match ev {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        let action = input_handler.handle_key(key);
                        handle_action(&mut core, action, &pty_tx, &exit_tx, &terminal)?;
                        if core.should_quit() {
                            break;
                        }
                    }
                    Event::Resize(w, h) => {
                        let content_h = h.saturating_sub(2);
                        core.set_terminal_size(w, content_h);
                        if let Some(ws) = core.active_workspace_ref() {
                            let layouts = ws.split_tree.layout(0, 0, w, content_h);
                            for layout in &layouts {
                                if let Some(surface) = core.surfaces.get_mut(&layout.surface_id) {
                                    surface.resize(layout.width.saturating_sub(2), layout.height.saturating_sub(2));
                                }
                            }
                        }
                    }
                    Event::Mouse(mouse) => {
                        let content_y_offset = 1u16; // tab bar
                        let (tw, th) = core.terminal_size;

                        // Skip clicks on tab bar or status bar
                        if mouse.row < content_y_offset || mouse.row >= content_y_offset + th {
                            continue;
                        }
                        let mx = mouse.column;
                        let my = mouse.row.saturating_sub(content_y_offset);

                        // Zoom mode: all mouse goes to zoomed surface
                        if let Some(zoom_id) = core.zoom_surface {
                            match mouse.kind {
                                MouseEventKind::Down(_) | MouseEventKind::Up(_)
                                | MouseEventKind::Drag(_) | MouseEventKind::ScrollUp
                                | MouseEventKind::ScrollDown | MouseEventKind::Moved => {
                                    if let Some(surface) = core.surfaces.get_mut(&zoom_id) {
                                        if surface.screen().mouse_protocol_mode() != vt100::MouseProtocolMode::None {
                                            let rel_x = mx.saturating_sub(1);
                                            let rel_y = my.saturating_sub(1);
                                            if let Some(bytes) = mouse_event_to_sgr_bytes(&mouse, rel_x, rel_y) {
                                                let _ = surface.send_bytes(&bytes);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                            continue;
                        }

                        match mouse.kind {
                            MouseEventKind::Down(MouseButton::Left) => {
                                // Extract hit info without holding borrow on core
                                let hit = core.workspaces.get(core.active_workspace).and_then(|ws| {
                                    if let Some(border) = ws.split_tree.border_hit(mx, my, 0, 0, tw, th) {
                                        Some(Err(border))
                                    } else {
                                        ws.split_tree.surface_at(mx, my, 0, 0, tw, th).map(|sid| {
                                            let layouts = ws.split_tree.layout(0, 0, tw, th);
                                            let layout = layouts.into_iter().find(|l| l.surface_id == sid);
                                            Ok((sid, layout))
                                        })
                                    }
                                });

                                match hit {
                                    Some(Err((path, dir, rx, ry, rw, rh))) => {
                                        drag_state = Some(DragState {
                                            border_path: path,
                                            direction: dir,
                                            region_x: rx,
                                            region_y: ry,
                                            region_w: rw,
                                            region_h: rh,
                                        });
                                    }
                                    Some(Ok((surface_id, layout))) => {
                                        core.focused_surface = Some(surface_id);
                                        if let Some(surface) = core.surfaces.get_mut(&surface_id) {
                                            if surface.screen().mouse_protocol_mode() != vt100::MouseProtocolMode::None {
                                                if let Some(layout) = layout {
                                                    let rel_x = mx.saturating_sub(layout.x + 1);
                                                    let rel_y = my.saturating_sub(layout.y + 1);
                                                    if let Some(bytes) = mouse_event_to_sgr_bytes(&mouse, rel_x, rel_y) {
                                                        let _ = surface.send_bytes(&bytes);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            MouseEventKind::Drag(MouseButton::Left) => {
                                if let Some(ref drag) = drag_state {
                                    let new_ratio = match drag.direction {
                                        Direction::Vertical => (mx.saturating_sub(drag.region_x)) as f64 / drag.region_w as f64,
                                        Direction::Horizontal => (my.saturating_sub(drag.region_y)) as f64 / drag.region_h as f64,
                                    };
                                    let path = drag.border_path.clone();
                                    core.set_ratio_at(&path, new_ratio);
                                    core.resize_active_workspace();
                                    for surface in core.surfaces.values_mut() {
                                        surface.dirty = true;
                                    }
                                }
                            }
                            MouseEventKind::Up(MouseButton::Left) => {
                                drag_state = None;
                            }
                            // Mouse passthrough for scroll and other events
                            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                            | MouseEventKind::Up(_) | MouseEventKind::Down(_) => {
                                if drag_state.is_none() {
                                    if let Some(id) = core.focused_surface {
                                        if let Some(surface) = core.surfaces.get_mut(&id) {
                                            if surface.screen().mouse_protocol_mode() != vt100::MouseProtocolMode::None {
                                                if let Some(ws) = core.workspaces.get(core.active_workspace) {
                                                    let layouts = ws.split_tree.layout(0, 0, tw, th);
                                                    if let Some(layout) = layouts.iter().find(|l| l.surface_id == id) {
                                                        let rel_x = mx.saturating_sub(layout.x + 1);
                                                        let rel_y = my.saturating_sub(layout.y + 1);
                                                        if let Some(bytes) = mouse_event_to_sgr_bytes(&mouse, rel_x, rel_y) {
                                                            let _ = surface.send_bytes(&bytes);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            Some((surface_id, data)) = pty_rx.recv() => {
                core.process_pty_output(surface_id, &data);
            }

            Some(surface_id) = exit_rx.recv() => {
                core.handle_pty_exit(surface_id);
            }

            Some(req) = socket_rx.recv() => {
                let response = wmux_core::socket::commands::dispatch(&mut core, &req.request, &pty_tx, &exit_tx);
                let _ = req.response_tx.send(response);
            }

            _ = render_interval.tick() => {
                let has_dirty = core.surfaces.values().any(|s| s.dirty);
                if has_dirty {
                    terminal.draw(|f| {
                        let size = f.area();
                        let content_h = size.height.saturating_sub(2);

                        let layouts = core.active_workspace_ref()
                            .map(|ws| ws.split_tree.layout(0, 0, size.width, content_h))
                            .unwrap_or_default();

                        let shell_name = core.shell.rsplit(['\\', '/']).next().unwrap_or(&core.shell);

                        let ctx = RenderContext {
                            tabs: core.tab_info(),
                            layouts,
                            surfaces: &core.surfaces,
                            focused_surface: core.focused_surface,
                            workspace_index: core.active_workspace,
                            shell_name,
                            pipe_path: &core.pipe_path,
                            zoom_surface: core.zoom_surface,
                        };
                        render_frame(f, &ctx);
                    })?;

                    for surface in core.surfaces.values_mut() {
                        surface.dirty = false;
                    }
                }
            }
        }
    }

    cleanup_terminal();
    Ok(())
}

fn handle_action(
    core: &mut WmuxCore,
    action: Action,
    pty_tx: &mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
    exit_tx: &mpsc::UnboundedSender<Uuid>,
    terminal: &Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let content_h = size.height.saturating_sub(2);

    match action {
        Action::ForwardToSurface(key) => {
            if let Some(id) = core.focused_surface {
                if let Some(surface) = core.surfaces.get_mut(&id) {
                    if let Some(bytes) = key_event_to_bytes(&key) {
                        let _ = surface.send_bytes(&bytes);
                    }
                }
            }
        }
        Action::NewWorkspace => {
            core.create_workspace(None, pty_tx, exit_tx, size.width, content_h)?;
        }
        Action::NextWorkspace => {
            core.next_workspace();
        }
        Action::PrevWorkspace => {
            core.prev_workspace();
        }
        Action::SelectWorkspace(idx) => {
            core.switch_workspace(idx);
        }
        Action::SplitVertical => {
            core.split_surface(Direction::Vertical, pty_tx, exit_tx, size.width / 2, content_h)?;
        }
        Action::SplitHorizontal => {
            core.split_surface(Direction::Horizontal, pty_tx, exit_tx, size.width, content_h / 2)?;
        }
        Action::FocusRight => {
            core.focus_direction(FocusDirection::Right);
        }
        Action::FocusDown => {
            core.focus_direction(FocusDirection::Down);
        }
        Action::FocusLeft => {
            core.focus_direction(FocusDirection::Left);
        }
        Action::FocusUp => {
            core.focus_direction(FocusDirection::Up);
        }
        Action::CloseSurface => {
            if let Some(id) = core.focused_surface {
                if core.close_surface(id) {
                    core.request_quit();
                }
            }
        }
        Action::ToggleZoom => {
            core.toggle_zoom();
        }
        Action::Quit => {
            core.request_quit();
        }
        Action::None => {}
    }
    Ok(())
}
