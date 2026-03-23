use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;
use vt100::Screen;

/// Widget that renders a vt100 Screen into a ratatui Buffer.
pub struct SurfaceView<'a> {
    screen: &'a Screen,
    focused: bool,
    exited: Option<i32>,
}

impl<'a> SurfaceView<'a> {
    pub fn new(screen: &'a Screen, focused: bool, exited: Option<i32>) -> Self {
        Self { screen, focused, exited }
    }
}

impl<'a> Widget for SurfaceView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(code) = self.exited {
            let msg = format!("[Process exited (code {})]", code);
            let x = area.x + area.width.saturating_sub(msg.len() as u16) / 2;
            let y = area.y + area.height / 2;
            buf.set_string(x, y, &msg, Style::default().fg(Color::DarkGray));
            return;
        }

        let (screen_rows, screen_cols) = self.screen.size();
        let rows = area.height.min(screen_rows);
        let cols = area.width.min(screen_cols);

        for row in 0..rows {
            for col in 0..cols {
                let cell = self.screen.cell(row, col);
                if let Some(cell) = cell {
                    let x = area.x + col;
                    let y = area.y + row;
                    if x < area.right() && y < area.bottom() {
                        let style = vt100_to_ratatui_style(cell);
                        let ch = if cell.contents().is_empty() {
                            " ".to_string()
                        } else {
                            cell.contents()
                        };
                        buf.set_string(x, y, &ch, style);
                    }
                }
            }
        }

        // Draw cursor if focused
        if self.focused {
            let cursor = self.screen.cursor_position();
            let cx = area.x + cursor.1;
            let cy = area.y + cursor.0;
            if cx < area.right() && cy < area.bottom() {
                if let Some(buf_cell) = buf.cell_mut((cx, cy)) {
                    buf_cell.set_style(Style::default().add_modifier(Modifier::REVERSED));
                }
            }
        }
    }
}

fn vt100_to_ratatui_style(cell: &vt100::Cell) -> Style {
    let mut style = Style::default();

    style = style.fg(vt100_color_to_ratatui(cell.fgcolor()));
    style = style.bg(vt100_color_to_ratatui(cell.bgcolor()));

    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if cell.inverse() {
        style = style.add_modifier(Modifier::REVERSED);
    }

    style
}

fn vt100_color_to_ratatui(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
