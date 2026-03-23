use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

pub struct StatusBar<'a> {
    pub workspace_index: usize,
    pub surface_index: usize,
    pub shell_name: &'a str,
    pub pipe_path: &'a str,
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::default().bg(Color::DarkGray));
        let text = format!(
            " ws:{} surface:{} | {} | {} ",
            self.workspace_index + 1,
            self.surface_index + 1,
            self.shell_name,
            self.pipe_path
        );
        buf.set_string(area.x, area.y, &text, Style::default().fg(Color::White).bg(Color::DarkGray));
    }
}
