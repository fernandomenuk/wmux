use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use crate::model::split_tree::SurfaceLayout;
use crate::model::surface::Surface;
use crate::tui::surface_view::SurfaceView;
use crate::tui::tabs::TabBar;
use crate::tui::status::StatusBar;
use std::collections::HashMap;
use uuid::Uuid;

pub struct RenderContext<'a> {
    pub tabs: Vec<(String, bool)>,
    pub layouts: Vec<SurfaceLayout>,
    pub surfaces: &'a HashMap<Uuid, Surface>,
    pub focused_surface: Option<Uuid>,
    pub zoom_surface: Option<Uuid>,
    pub workspace_index: usize,
    pub shell_name: &'a str,
    pub pipe_path: &'a str,
}

pub fn render_frame(f: &mut Frame, ctx: &RenderContext) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(size);

    // Tab bar
    let tab_bar = TabBar { tabs: &ctx.tabs };
    f.render_widget(tab_bar, chunks[0]);

    let content = chunks[1];

    // If zoomed, render only the zoomed surface fullscreen
    if let Some(zoom_id) = ctx.zoom_surface {
        let is_focused = true;
        let border_style = Style::default().fg(Color::Yellow);
        let block = Block::default().borders(Borders::ALL).border_style(border_style);
        let inner = block.inner(content);
        f.render_widget(block, content);

        if let Some(surface) = ctx.surfaces.get(&zoom_id) {
            let view = SurfaceView::new(surface.screen(), is_focused, surface.exited);
            f.render_widget(view, inner);
        }
    } else {
        // Normal split pane rendering
        for layout in &ctx.layouts {
            let area = Rect {
                x: content.x + layout.x,
                y: content.y + layout.y,
                width: layout.width.min(content.width.saturating_sub(layout.x)),
                height: layout.height.min(content.height.saturating_sub(layout.y)),
            };

            if area.width == 0 || area.height == 0 {
                continue;
            }

            let is_focused = ctx.focused_surface == Some(layout.surface_id);

            let border_style = if is_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let block = Block::default().borders(Borders::ALL).border_style(border_style);
            let inner = block.inner(area);
            f.render_widget(block, area);

            if let Some(surface) = ctx.surfaces.get(&layout.surface_id) {
                let view = SurfaceView::new(surface.screen(), is_focused, surface.exited);
                f.render_widget(view, inner);
            }
        }
    }

    // Status bar
    let surface_index = ctx.focused_surface
        .and_then(|id| ctx.layouts.iter().position(|l| l.surface_id == id))
        .unwrap_or(0);

    let status = StatusBar {
        workspace_index: ctx.workspace_index,
        surface_index,
        shell_name: ctx.shell_name,
        pipe_path: ctx.pipe_path,
    };
    f.render_widget(status, chunks[2]);
}
