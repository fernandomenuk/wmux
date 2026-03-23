use std::collections::HashMap;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::model::split_tree::Direction;
use crate::model::surface::Surface;
use crate::model::workspace::Workspace;
use crate::terminal::pty::{spawn_pty, start_pty_reader};

pub type SurfaceId = Uuid;
pub type WorkspaceId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Up,
    Down,
    Left,
    Right,
}

use sysinfo::{System, Pid};

pub struct WmuxCore {
    pub workspaces: Vec<Workspace>,
    pub surfaces: HashMap<Uuid, Surface>,
    pub active_workspace: usize,
    pub focused_surface: Option<Uuid>,
    pub shell: String,
    pub pipe_path: String,
    pub zoom_surface: Option<Uuid>,
    should_quit: bool,
    pub terminal_size: (u16, u16),
    pub sys: System,
}

impl WmuxCore {
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
            sys: System::new_all(),
        }
    }

    pub fn get_process_metrics(&mut self, surface_id: Uuid) -> Option<(f32, u64)> {
        let pid = self.surfaces.get(&surface_id)?.pid;
        if pid == 0 {
            return None;
        }

        self.sys.refresh_process(Pid::from(pid as usize));
        let process = self.sys.process(Pid::from(pid as usize))?;
        Some((process.cpu_usage(), process.memory()))
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn request_quit(&mut self) {
        self.should_quit = true;
    }

    pub fn set_terminal_size(&mut self, w: u16, h: u16) {
        self.terminal_size = (w, h);
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
        start_pty_reader(
            surface_id,
            pty.master.as_ref(),
            pty_tx.clone(),
            exit_tx.clone(),
        )?;

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
                    self.workspaces[self.active_workspace]
                        .split_tree
                        .first_surface(),
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
                    surface.resize(
                        layout.width.saturating_sub(2),
                        layout.height.saturating_sub(2),
                    );
                }
            }
        }
    }

    pub fn next_workspace(&mut self) {
        if !self.workspaces.is_empty() {
            self.active_workspace = (self.active_workspace + 1) % self.workspaces.len();
            self.focused_surface = Some(
                self.workspaces[self.active_workspace]
                    .split_tree
                    .first_surface(),
            );
            self.resize_active_workspace();
        }
    }

    pub fn prev_workspace(&mut self) {
        if !self.workspaces.is_empty() {
            self.active_workspace = if self.active_workspace == 0 {
                self.workspaces.len() - 1
            } else {
                self.active_workspace - 1
            };
            self.focused_surface = Some(
                self.workspaces[self.active_workspace]
                    .split_tree
                    .first_surface(),
            );
            self.resize_active_workspace();
        }
    }

    pub fn switch_workspace(&mut self, index: usize) {
        if index < self.workspaces.len() {
            self.active_workspace = index;
            self.focused_surface = Some(
                self.workspaces[self.active_workspace]
                    .split_tree
                    .first_surface(),
            );
            self.resize_active_workspace();
        }
    }

    pub fn focus_direction(&mut self, dir: FocusDirection) {
        match dir {
            FocusDirection::Right | FocusDirection::Down => {
                if let (Some(focused), Some(ws)) =
                    (self.focused_surface, self.active_workspace_ref())
                {
                    let ids = ws.split_tree.surface_ids();
                    if let Some(pos) = ids.iter().position(|id| *id == focused) {
                        if pos + 1 < ids.len() {
                            self.focused_surface = Some(ids[pos + 1]);
                        }
                    }
                }
            }
            FocusDirection::Left | FocusDirection::Up => {
                if let (Some(focused), Some(ws)) =
                    (self.focused_surface, self.active_workspace_ref())
                {
                    let ids = ws.split_tree.surface_ids();
                    if let Some(pos) = ids.iter().position(|id| *id == focused) {
                        if pos > 0 {
                            self.focused_surface = Some(ids[pos - 1]);
                        }
                    }
                }
            }
        }
    }

    /// Set focus to a specific surface by ID (validates it exists)
    pub fn focus_surface(&mut self, surface_id: Uuid) {
        if self.surfaces.contains_key(&surface_id) {
            self.focused_surface = Some(surface_id);
        }
    }

    pub fn toggle_zoom(&mut self) {
        if self.zoom_surface.is_some() {
            self.zoom_surface = None;
        } else {
            self.zoom_surface = self.focused_surface;
        }
    }

    pub fn set_ratio_at(&mut self, path: &[bool], ratio: f64) {
        if let Some(ws) = self.workspaces.get_mut(self.active_workspace) {
            ws.split_tree.set_ratio_at(path, ratio);
        }
    }

    pub fn process_pty_output(&mut self, surface_id: Uuid, data: &[u8]) {
        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            surface.process_output(data);
        }
    }

    pub fn handle_pty_exit(&mut self, surface_id: Uuid) {
        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            let code = surface
                .pty
                .as_mut()
                .and_then(|pty| pty.child.try_wait().ok().flatten())
                .map(|status| status.exit_code() as i32)
                .unwrap_or(0);
            surface.mark_exited(code);
        }
    }
}
