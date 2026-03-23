use std::collections::HashMap;
use std::io;

use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::input::{Action, InputHandler, key_event_to_bytes};
use crate::model::split_tree::Direction;
use crate::model::surface::Surface;
use crate::model::workspace::Workspace;
use crate::terminal::pty::{spawn_pty, start_pty_reader};
use crate::terminal::shell::detect_shell;
use crate::tui::render::{render_frame, RenderContext};

pub struct App {
    pub workspaces: Vec<Workspace>,
    pub surfaces: HashMap<Uuid, Surface>,
    pub active_workspace: usize,
    pub focused_surface: Option<Uuid>,
    pub shell: String,
    pub pipe_path: String,
    pub zoom_surface: Option<Uuid>,
    pub should_quit: bool,
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
        }
    }

    pub fn create_workspace(
        &mut self,
        name: Option<String>,
        pty_tx: &mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
        cols: u16,
        rows: u16,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let surface_id = Uuid::new_v4();
        let pty = spawn_pty(&self.shell, cols, rows, None)?;
        start_pty_reader(surface_id, pty.master.as_ref(), pty_tx.clone())?;

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
        cols: u16,
        rows: u16,
    ) -> Result<Option<Uuid>, Box<dyn std::error::Error>> {
        let focused = match self.focused_surface {
            Some(id) => id,
            None => return Ok(None),
        };

        let new_id = Uuid::new_v4();
        let pty = spawn_pty(&self.shell, cols, rows, None)?;
        start_pty_reader(new_id, pty.master.as_ref(), pty_tx.clone())?;

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
}

pub struct SocketRequest {
    pub request: crate::socket::protocol::Request,
    pub response_tx: tokio::sync::oneshot::Sender<crate::socket::protocol::Response>,
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
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let shell = detect_shell(cli_shell);
    let mut app = App::new(shell, pipe_path);
    let mut input_handler = InputHandler::new();

    let (pty_tx, mut pty_rx) = mpsc::unbounded_channel::<(Uuid, Vec<u8>)>();

    let size = terminal.size()?;
    let content_height = size.height.saturating_sub(2);
    app.create_workspace(None, &pty_tx, size.width, content_height)?;

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
                        handle_action(&mut app, action, &pty_tx, &terminal)?;
                        if app.should_quit {
                            break;
                        }
                    }
                    Event::Resize(w, h) => {
                        let content_h = h.saturating_sub(2);
                        if let Some(ws) = app.active_workspace_ref() {
                            let layouts = ws.split_tree.layout(0, 0, w, content_h);
                            for layout in &layouts {
                                if let Some(surface) = app.surfaces.get_mut(&layout.surface_id) {
                                    surface.resize(layout.width.saturating_sub(2), layout.height.saturating_sub(2));
                                }
                            }
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

            Some(req) = socket_rx.recv() => {
                let response = crate::socket::commands::dispatch(&mut app, &req.request, &pty_tx);
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
                            content_area: ratatui::layout::Rect::default(),
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

    terminal::disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_action(
    app: &mut App,
    action: Action,
    pty_tx: &mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
    terminal: &Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let content_h = size.height.saturating_sub(2);

    match action {
        Action::ForwardToSurface(key) => {
            if let Some(id) = app.focused_surface {
                if let Some(surface) = app.surfaces.get_mut(&id) {
                    if let Some(bytes) = key_event_to_bytes(&key) {
                        let _ = surface.send_text(std::str::from_utf8(&bytes).unwrap_or(""));
                    }
                }
            }
        }
        Action::NewWorkspace => {
            app.create_workspace(None, pty_tx, size.width, content_h)?;
        }
        Action::NextWorkspace => {
            if !app.workspaces.is_empty() {
                app.active_workspace = (app.active_workspace + 1) % app.workspaces.len();
                app.focused_surface = Some(
                    app.workspaces[app.active_workspace].split_tree.first_surface(),
                );
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
            }
        }
        Action::SelectWorkspace(idx) => {
            if idx < app.workspaces.len() {
                app.active_workspace = idx;
                app.focused_surface = Some(
                    app.workspaces[app.active_workspace].split_tree.first_surface(),
                );
            }
        }
        Action::SplitVertical => {
            app.split_surface(Direction::Vertical, pty_tx, size.width / 2, content_h)?;
        }
        Action::SplitHorizontal => {
            app.split_surface(Direction::Horizontal, pty_tx, size.width, content_h / 2)?;
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
