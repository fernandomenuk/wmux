use std::collections::HashMap;
use std::io;

use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind, MouseEventKind, MouseButton, EnableMouseCapture, DisableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::input::{Action, InputHandler, key_event_to_bytes, mouse_event_to_sgr_bytes};
use crate::model::split_tree::Direction;
use crate::model::surface::Surface;
use crate::model::workspace::Workspace;
use crate::terminal::pty::{spawn_pty, start_pty_reader};
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

pub struct App {
    pub workspaces: Vec<Workspace>,
    pub surfaces: HashMap<Uuid, Surface>,
    pub active_workspace: usize,
    pub focused_surface: Option<Uuid>,
    pub shell: String,
    pub pipe_path: String,
    pub zoom_surface: Option<Uuid>,
    pub should_quit: bool,
    pub terminal_size: (u16, u16),
    pub drag_state: Option<DragState>,
}

impl App {
    pub fn new(shell: String, pipe_path: String) -> Self {
        Self {
            workspaces: Vec::new(),
            surfaces: HashMap::new(),
            active_workspace: 0,
            focused_surface: None,
            shell,
            pipe_path,
            zoom_surface: None,
            should_quit: false,
            terminal_size: (80, 24),
            drag_state: None,
        }
    }

    pub fn create_workspace(
        &mut self,
        name: Option<String>,
        pty_tx: &mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
        exit_tx: &mpsc::UnboundedSender<Uuid>,
        cols: u16,
        rows: u16,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let surface_id = Uuid::new_v4();
        let pty = spawn_pty(&self.shell, cols, rows, None)?;
        start_pty_reader(surface_id, pty.master.as_ref(), pty_tx.clone(), exit_tx.clone())?;

        let surface = Surface::new(surface_id, self.shell.clone(), cols, rows, pty);
        self.surfaces.insert(surface_id, surface);

        let ws_name = name.unwrap_or_else(|| format!("workspace {}", self.workspaces.len() + 1));
        let workspace = Workspace::new(ws_name, surface_id);
        let ws_id = workspace.id;
        self.workspaces.push(workspace);
        self.active_workspace = self.workspaces.len() - 1;
        self.focused_surface = Some(surface_id);

        Ok(ws_id)
    }

    pub fn split_surface(
        &mut self,
        direction: Direction,
        pty_tx: &mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
        exit_tx: &mpsc::UnboundedSender<Uuid>,
        cols: u16,
        rows: u16,
    ) -> Result<Option<Uuid>, Box<dyn std::error::Error>> {
        let focused = match self.focused_surface {
            Some(id) => id,
            None => return Ok(None),
        };

        let new_id = Uuid::new_v4();
        let pty = spawn_pty(&self.shell, cols, rows, None)?;
        start_pty_reader(new_id, pty.master.as_ref(), pty_tx.clone(), exit_tx.clone())?;

        let surface = Surface::new(new_id, self.shell.clone(), cols, rows, pty);
        self.surfaces.insert(new_id, surface);

        if let Some(ws) = self.workspaces.get_mut(self.active_workspace) {
            ws.split_tree.split_at(focused, new_id, direction);
        }

        self.focused_surface = Some(new_id);
        Ok(Some(new_id))
    }

    pub fn close_surface(&mut self, surface_id: Uuid) -> bool {
        self.surfaces.remove(&surface_id);

        if let Some(ws) = self.workspaces.get_mut(self.active_workspace) {
            if ws.split_tree.surface_ids().len() == 1
                && ws.split_tree.surface_ids()[0] == surface_id
            {
                self.workspaces.remove(self.active_workspace);
                if self.workspaces.is_empty() {
                    return true;
                }
                self.active_workspace = self.active_workspace.min(self.workspaces.len() - 1);
                self.focused_surface = Some(
                    self.workspaces[self.active_workspace].split_tree.first_surface(),
                );
            } else {
                ws.split_tree.remove(surface_id);
                self.focused_surface = Some(ws.split_tree.first_surface());
            }
        }

        false
    }

    pub fn tab_info(&self) -> Vec<(String, bool)> {
        self.workspaces
            .iter()
            .enumerate()
            .map(|(i, ws)| (ws.name.clone(), i == self.active_workspace))
            .collect()
    }

    pub fn active_workspace_ref(&self) -> Option<&Workspace> {
        self.workspaces.get(self.active_workspace)
    }

    /// Resize all surfaces in the active workspace to match current terminal size.
    pub fn resize_active_workspace(&mut self) {
        let (w, h) = self.terminal_size;
        if let Some(ws) = self.workspaces.get(self.active_workspace) {
            let layouts = ws.split_tree.layout(0, 0, w, h);
            for layout in &layouts {
                if let Some(surface) = self.surfaces.get_mut(&layout.surface_id) {
                    surface.resize(layout.width.saturating_sub(2), layout.height.saturating_sub(2));
                }
            }
        }
    }
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
    let mut app = App::new(shell, pipe_path);
    let mut input_handler = InputHandler::new();

    let (pty_tx, mut pty_rx) = mpsc::unbounded_channel::<(Uuid, Vec<u8>)>();
    let (exit_tx, mut exit_rx) = mpsc::unbounded_channel::<Uuid>();

    let size = terminal.size()?;
    let content_height = size.height.saturating_sub(2);
    app.terminal_size = (size.width, content_height);
    if let Err(e) = app.create_workspace(None, &pty_tx, &exit_tx, size.width, content_height) {
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

    loop {
        tokio::select! {
            Some(ev) = input_rx.recv() => {
                match ev {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        let action = input_handler.handle_key(key);
                        handle_action(&mut app, action, &pty_tx, &exit_tx, &terminal)?;
                        if app.should_quit {
                            break;
                        }
                    }
                    Event::Resize(w, h) => {
                        let content_h = h.saturating_sub(2);
                        app.terminal_size = (w, content_h);
                        if let Some(ws) = app.active_workspace_ref() {
                            let layouts = ws.split_tree.layout(0, 0, w, content_h);
                            for layout in &layouts {
                                if let Some(surface) = app.surfaces.get_mut(&layout.surface_id) {
                                    surface.resize(layout.width.saturating_sub(2), layout.height.saturating_sub(2));
                                }
                            }
                        }
                    }
                    Event::Mouse(mouse) => {
                        let content_y_offset = 1u16; // tab bar
                        let (tw, th) = app.terminal_size;

                        // Skip clicks on tab bar or status bar
                        if mouse.row < content_y_offset || mouse.row >= content_y_offset + th {
                            continue;
                        }
                        let mx = mouse.column;
                        let my = mouse.row.saturating_sub(content_y_offset);

                        // Zoom mode: all mouse goes to zoomed surface
                        if let Some(zoom_id) = app.zoom_surface {
                            match mouse.kind {
                                MouseEventKind::Down(_) | MouseEventKind::Up(_)
                                | MouseEventKind::Drag(_) | MouseEventKind::ScrollUp
                                | MouseEventKind::ScrollDown | MouseEventKind::Moved => {
                                    if let Some(surface) = app.surfaces.get_mut(&zoom_id) {
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
                                // Extract hit info without holding borrow on app
                                let hit = app.workspaces.get(app.active_workspace).and_then(|ws| {
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
                                        app.drag_state = Some(DragState {
                                            border_path: path,
                                            direction: dir,
                                            region_x: rx,
                                            region_y: ry,
                                            region_w: rw,
                                            region_h: rh,
                                        });
                                    }
                                    Some(Ok((surface_id, layout))) => {
                                        app.focused_surface = Some(surface_id);
                                        if let Some(surface) = app.surfaces.get_mut(&surface_id) {
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
                                if let Some(ref drag) = app.drag_state {
                                    let new_ratio = match drag.direction {
                                        Direction::Vertical => (mx.saturating_sub(drag.region_x)) as f64 / drag.region_w as f64,
                                        Direction::Horizontal => (my.saturating_sub(drag.region_y)) as f64 / drag.region_h as f64,
                                    };
                                    let path = drag.border_path.clone();
                                    if let Some(ws) = app.workspaces.get_mut(app.active_workspace) {
                                        ws.split_tree.set_ratio_at(&path, new_ratio);
                                    }
                                    app.resize_active_workspace();
                                    for surface in app.surfaces.values_mut() {
                                        surface.dirty = true;
                                    }
                                }
                            }
                            MouseEventKind::Up(MouseButton::Left) => {
                                app.drag_state = None;
                            }
                            // Mouse passthrough for scroll and other events
                            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                            | MouseEventKind::Up(_) | MouseEventKind::Down(_) => {
                                if app.drag_state.is_none() {
                                    if let Some(id) = app.focused_surface {
                                        if let Some(surface) = app.surfaces.get_mut(&id) {
                                            if surface.screen().mouse_protocol_mode() != vt100::MouseProtocolMode::None {
                                                if let Some(ws) = app.workspaces.get(app.active_workspace) {
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
                if let Some(surface) = app.surfaces.get_mut(&surface_id) {
                    surface.process_output(&data);
                }
            }

            Some(surface_id) = exit_rx.recv() => {
                if let Some(surface) = app.surfaces.get_mut(&surface_id) {
                    // Try to get exit code from child process
                    let code = surface.pty.as_mut()
                        .and_then(|pty| pty.child.try_wait().ok().flatten())
                        .map(|status| status.exit_code() as i32)
                        .unwrap_or(0);
                    surface.mark_exited(code);
                }
            }

            Some(req) = socket_rx.recv() => {
                let response = crate::socket::commands::dispatch(&mut app, &req.request, &pty_tx, &exit_tx);
                let _ = req.response_tx.send(response);
            }

            _ = render_interval.tick() => {
                let has_dirty = app.surfaces.values().any(|s| s.dirty);
                if has_dirty {
                    terminal.draw(|f| {
                        let size = f.area();
                        let content_h = size.height.saturating_sub(2);

                        let layouts = app.active_workspace_ref()
                            .map(|ws| ws.split_tree.layout(0, 0, size.width, content_h))
                            .unwrap_or_default();

                        let shell_name = app.shell.rsplit(['\\', '/']).next().unwrap_or(&app.shell);

                        let ctx = RenderContext {
                            tabs: app.tab_info(),
                            layouts,
                            surfaces: &app.surfaces,
                            focused_surface: app.focused_surface,
                            workspace_index: app.active_workspace,
                            shell_name,
                            pipe_path: &app.pipe_path,
                            zoom_surface: app.zoom_surface,
                        };
                        render_frame(f, &ctx);
                    })?;

                    for surface in app.surfaces.values_mut() {
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
    app: &mut App,
    action: Action,
    pty_tx: &mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
    exit_tx: &mpsc::UnboundedSender<Uuid>,
    terminal: &Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let content_h = size.height.saturating_sub(2);

    match action {
        Action::ForwardToSurface(key) => {
            if let Some(id) = app.focused_surface {
                if let Some(surface) = app.surfaces.get_mut(&id) {
                    if let Some(bytes) = key_event_to_bytes(&key) {
                        let _ = surface.send_bytes(&bytes);
                    }
                }
            }
        }
        Action::NewWorkspace => {
            app.create_workspace(None, pty_tx, exit_tx, size.width, content_h)?;
        }
        Action::NextWorkspace => {
            if !app.workspaces.is_empty() {
                app.active_workspace = (app.active_workspace + 1) % app.workspaces.len();
                app.focused_surface = Some(
                    app.workspaces[app.active_workspace].split_tree.first_surface(),
                );
                app.resize_active_workspace();
            }
        }
        Action::PrevWorkspace => {
            if !app.workspaces.is_empty() {
                app.active_workspace = if app.active_workspace == 0 {
                    app.workspaces.len() - 1
                } else {
                    app.active_workspace - 1
                };
                app.focused_surface = Some(
                    app.workspaces[app.active_workspace].split_tree.first_surface(),
                );
                app.resize_active_workspace();
            }
        }
        Action::SelectWorkspace(idx) => {
            if idx < app.workspaces.len() {
                app.active_workspace = idx;
                app.focused_surface = Some(
                    app.workspaces[app.active_workspace].split_tree.first_surface(),
                );
                app.resize_active_workspace();
            }
        }
        Action::SplitVertical => {
            app.split_surface(Direction::Vertical, pty_tx, exit_tx, size.width / 2, content_h)?;
        }
        Action::SplitHorizontal => {
            app.split_surface(Direction::Horizontal, pty_tx, exit_tx, size.width, content_h / 2)?;
        }
        Action::FocusRight | Action::FocusDown => {
            if let (Some(focused), Some(ws)) = (app.focused_surface, app.active_workspace_ref()) {
                let ids = ws.split_tree.surface_ids();
                if let Some(pos) = ids.iter().position(|id| *id == focused) {
                    if pos + 1 < ids.len() {
                        app.focused_surface = Some(ids[pos + 1]);
                    }
                }
            }
        }
        Action::FocusLeft | Action::FocusUp => {
            if let (Some(focused), Some(ws)) = (app.focused_surface, app.active_workspace_ref()) {
                let ids = ws.split_tree.surface_ids();
                if let Some(pos) = ids.iter().position(|id| *id == focused) {
                    if pos > 0 {
                        app.focused_surface = Some(ids[pos - 1]);
                    }
                }
            }
        }
        Action::CloseSurface => {
            if let Some(id) = app.focused_surface {
                app.should_quit = app.close_surface(id);
            }
        }
        Action::ToggleZoom => {
            if app.zoom_surface.is_some() {
                app.zoom_surface = None;
            } else {
                app.zoom_surface = app.focused_surface;
            }
        }
        Action::Quit => {
            app.should_quit = true;
        }
        Action::None => {}
    }
    Ok(())
}
