use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::Widget;

pub struct TabBar<'a> {
    pub tabs: &'a [(String, bool)],
}

impl<'a> Widget for TabBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::default().bg(Color::DarkGray));

        let mut x = area.x + 1;
        for (i, (name, active)) in self.tabs.iter().enumerate() {
            let label = format!(" {}: {} ", i + 1, name);
            if x + label.len() as u16 > area.right() {
                break;
            }
            let style = if *active {
                Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray).bg(Color::DarkGray)
            };
            buf.set_string(x, area.y, &label, style);
            x += label.len() as u16 + 1;
        }

        let brand = " wmux ";
        let bx = area.right().saturating_sub(brand.len() as u16);
        buf.set_string(bx, area.y, brand, Style::default().fg(Color::Cyan).bg(Color::DarkGray));
    }
}
