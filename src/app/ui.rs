use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

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
