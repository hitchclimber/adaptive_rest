use crossterm::cursor::SetCursorStyle;
use ratatui::{
    layout::Constraint,
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Widget},
};

use crate::{
    config::{ColorValue, ThemeConfig},
    server::endpoint::EndpointEntry,
};

#[derive(Debug, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

pub struct ColorTheme {
    border: Color,
    title: Color,
    text: Color,
    bg: Color,
    emph: Color,
    warn: Color,
    error: Color,
    log: Color,
}

#[derive(Debug)]
pub struct AppStyle {
    pub border_style: Style,
    pub title_style: Style,
    pub accent_style: Style,
    pub bg: Color,
    pub cursor_style: SetCursorStyle,
    pub text_style: Style,
    pub header_style: Style,
    pub warn_style: Style,
    pub error_style: Style,
    pub log_style: Style,
}
impl From<&ColorTheme> for AppStyle {
    fn from(value: &ColorTheme) -> Self {
        Self {
            border_style: Style::default().fg(value.border),
            title_style: Style::default().bold().fg(value.title),
            accent_style: Style::default().italic().fg(value.emph),
            text_style: Style::default().fg(value.text),
            cursor_style: SetCursorStyle::BlinkingUnderScore,
            header_style: Style::default().fg(value.title).bg(value.emph).bold(),
            bg: value.bg,
            warn_style: Style::default().fg(value.warn).italic(),
            error_style: Style::default().fg(value.error).bold(),
            log_style: Style::default().fg(value.log),
        }
    }
}
impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            border: Color::Rgb(255, 237, 0),
            text: Color::Rgb(255, 0, 128),
            title: Color::Rgb(255, 0, 128),
            bg: Color::Rgb(53, 28, 34),
            emph: Color::Rgb(0, 180, 180),
            warn: Color::Rgb(255, 200, 0),
            error: Color::Rgb(255, 60, 60),
            log: Color::Rgb(0, 180, 180),
        }
    }
}

impl From<&ColorValue> for Color {
    fn from(value: &ColorValue) -> Self {
        match value {
            ColorValue::Rgb([r, g, b]) => Color::Rgb(*r, *g, *b),
            ColorValue::Hex(s) => {
                let s = s.trim_start_matches('#');
                let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
                Color::Rgb(r, g, b)
            }
        }
    }
}

impl From<&ThemeConfig> for ColorTheme {
    fn from(value: &ThemeConfig) -> Self {
        Self {
            border: (&value.border).into(),
            title: (&value.title).into(),
            text: (&value.text).into(),
            bg: (&value.bg).into(),
            emph: (&value.emph).into(),
            warn: (&value.warn).into(),
            error: (&value.error).into(),
            log: (&value.log).into(),
        }
    }
}

impl ColorTheme {
    pub fn or_default(config: Option<ThemeConfig>) -> Self {
        if let Some(c) = config {
            ColorTheme::from(&c)
        } else {
            ColorTheme::default()
        }
    }
}

pub struct CommandPane<'a> {
    pub input: &'a str,
    pub mode: &'a InputMode,
    pub theme: &'a AppStyle,
}

impl<'a> Widget for &CommandPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let instruction = Line::from(vec![
            Span::styled("Q", self.theme.border_style),
            Span::styled(" to quit ", self.theme.border_style),
            Span::styled("I", self.theme.border_style),
            Span::styled(" for insert mode ", self.theme.border_style),
            Span::styled("ESC", self.theme.border_style),
            Span::styled(" for normal mode ", self.theme.border_style),
        ]);

        let title = Line::from(
            match self.mode {
                InputMode::Normal => "Press I to enter commands",
                InputMode::Insert => "Enter commands",
            }
            .bold(),
        );
        let block = Block::default()
            .bg(self.theme.bg)
            .title(title.left_aligned())
            .title_style(self.theme.title_style)
            .title_bottom(instruction)
            .borders(Borders::ALL)
            .border_style(self.theme.border_style);
        Paragraph::new(self.input).block(block).render(area, buf);
    }
}

pub struct LogPane<'a> {
    pub messages: &'a [String],
    pub theme: &'a AppStyle,
}

impl<'a> Widget for &LogPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Server Logs")
            .title_style(self.theme.title_style)
            .borders(Borders::ALL)
            .bg(self.theme.bg)
            .border_style(self.theme.border_style);
        let text: Vec<Line> = self
            .messages
            .iter()
            .flat_map(|m| {
                let style = if m.starts_with("[ERROR]") {
                    self.theme.error_style
                } else if m.starts_with("[WARN]") {
                    self.theme.warn_style
                } else if m.starts_with("[INFO]") {
                    self.theme.log_style
                } else if m.starts_with("[DEBUG]") {
                    self.theme.accent_style
                } else {
                    self.theme.text_style
                };
                m.lines().enumerate().map(move |(i, line)| {
                    if i == 0 {
                        Line::from(Span::styled(line, style))
                    } else {
                        // Indent continuation lines to align with message content after "[LEVEL] "
                        Line::from(Span::styled(format!("        {}", line), style))
                    }
                })
            })
            .collect();
        Paragraph::new(text).block(block).render(area, buf);
    }
}

pub struct ListPane<'a> {
    pub endpoints: &'a [EndpointEntry],
    pub theme: &'a AppStyle,
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }
    if text.len() <= width {
        return vec![text.to_string()];
    }
    text.chars()
        .collect::<Vec<_>>()
        .chunks(width)
        .map(|c| c.iter().collect())
        .collect()
}

impl<'a> Widget for &ListPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header = ["Path", "Data", "Allowed Methods"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(self.theme.header_style)
            .height(1);

        // Calculate column widths (Path: Min 15, Data: 30, Methods: 30%)
        let methods_col_width = (area.width as usize * 30) / 100;
        let path_col_width = 15.max(area.width as usize - 30 - methods_col_width - 4); // 4 for borders/padding

        let rows = self
            .endpoints
            .iter()
            .map(|data| {
                // Wrap path to fit column width
                let path_lines = wrap_text(&data.path, path_col_width);

                // Methods: comma-joined if fits, otherwise one per line
                let methods_str = data.methods.iter().map(|m| m.as_str()).collect::<Vec<_>>();
                let comma_joined = methods_str.join(", ");
                let methods_lines = if comma_joined.len() <= methods_col_width {
                    vec![comma_joined]
                } else {
                    methods_str.iter().map(|s| s.to_string()).collect()
                };

                // Truncate data (middle column) - no multiline
                let data_display = if data.data.len() > 27 {
                    format!("{}...", &data.data[..27])
                } else {
                    data.data.clone()
                };

                // Row height is max of path and methods line counts
                let height = path_lines.len().max(methods_lines.len()).max(1);

                Row::new([
                    path_lines.join("\n"),
                    data_display,
                    methods_lines.join("\n"),
                ])
                .height(height as u16)
            })
            .chain(std::iter::once(Row::new([
                "/api/health",
                r#"{"status": "ok"}"#,
                "HEAD, GET",
            ])));
        let t = Table::new(
            rows,
            [
                Constraint::Length(path_col_width as u16),
                Constraint::Length(30),
                Constraint::Length(methods_col_width as u16),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title("Endpoints list")
                .title_style(self.theme.title_style)
                .borders(Borders::ALL)
                .bg(self.theme.bg)
                .border_style(self.theme.border_style),
        );
        t.render(area, buf);
    }
}
