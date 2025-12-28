use ratatui::{
    layout::Constraint,
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Widget},
};

use crate::server::endpoint::EndpointEntry;

#[derive(Debug, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

pub struct CommandPane<'a> {
    pub input: &'a str,
    pub mode: &'a InputMode,
}

impl<'a> Widget for &CommandPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let tips_style = Style::default().bold().fg(Color::Red);
        let instruction = Line::from(vec![
            Span::styled("Q", tips_style),
            Span::styled(" to quit ", Style::default().fg(Color::Red)),
            Span::styled("I", tips_style),
            Span::styled(" for insert mode ", Style::default().fg(Color::Red)),
            Span::styled("ESC", tips_style),
            Span::styled(" for normal mode ", Style::default().fg(Color::Red)),
        ]);

        let title = Line::from(
            match self.mode {
                InputMode::Normal => "Press I to enter commands",
                InputMode::Insert => "Enter commands",
            }
            .bold(),
        );
        let block = Block::default()
            .title(title.centered())
            .title_bottom(instruction)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        Paragraph::new(self.input).block(block).render(area, buf);
    }
}

pub struct LogPane<'a> {
    pub messages: &'a [String],
}

impl<'a> Widget for &LogPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Server Logs")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        let text: Vec<Line> = self
            .messages
            .iter()
            .flat_map(|m| {
                m.lines().enumerate().map(|(i, line)| {
                    if i == 0 {
                        Line::from(line)
                    } else {
                        // Indent continuation lines to align with message content after "[LEVEL] "
                        Line::from(format!("        {}", line))
                    }
                })
            })
            .collect();
        Paragraph::new(text).block(block).render(area, buf);
    }
}

pub struct ListPane<'a> {
    pub endpoints: &'a [EndpointEntry],
}

impl<'a> Widget for &ListPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header = ["Path", "Data", "Allowed Methods"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .height(1);
        let methods_col_width = (area.width as usize * 25) / 100;

        let rows = self.endpoints.iter().map(|data| {
            let methods_str = data.methods.iter().map(|m| m.as_str()).collect::<Vec<_>>();
            let comma_joined = methods_str.join(", ");
            let (me_str, height) = if comma_joined.len() <= methods_col_width {
                (comma_joined, 1)
            } else {
                (methods_str.join("\n"), methods_str.len().max(1))
            };
            let data_display = if data.data.len() > 27 {
                format!("{}...", &data.data[..27])
            } else {
                data.data.clone()
            };
            Row::new([data.path.clone(), data_display, me_str]).height(height as u16)
        });
        let t = Table::new(
            rows,
            [
                Constraint::Min(15),
                Constraint::Length(30),
                Constraint::Percentage(30),
            ],
        )
        .header(header);
        t.render(area, buf);
    }
}
