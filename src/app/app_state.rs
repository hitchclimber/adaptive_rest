use std::{io, sync::Arc, time::Duration};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{
    command::{Cli, Command, EndpointAction},
    server::ServerState,
    util::result::InternalResult,
};

use super::ui::{CommandPane, InputMode, LogPane};

#[derive(Debug)]
pub struct App {
    pub input: String,
    pub messages: Vec<String>,
    history: Vec<String>,
    history_index: Option<usize>,
    mode: InputMode,
    exit: bool,
    log_rx: UnboundedReceiver<String>,
    server_state: Arc<ServerState>,
}

impl App {
    pub fn new(log_rx: mpsc::UnboundedReceiver<String>, server_state: Arc<ServerState>) -> Self {
        Self {
            input: String::new(),
            messages: Vec::new(),
            mode: InputMode::default(),
            exit: false,
            log_rx,
            server_state,
            history: Vec::new(),
            history_index: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            while let Ok(msg) = self.log_rx.try_recv() {
                self.messages.push(msg)
            }
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(frame.area());
        let input_widget = CommandPane {
            input: &self.input,
            mode: &self.mode,
        };
        frame.render_widget(&input_widget, chunks[0]);

        let log_widget = LogPane {
            messages: &self.messages,
        };
        frame.render_widget(&log_widget, chunks[1]);
    }

    fn exit(&mut self) {
        self.exit = true
    }

    fn handle_events(&mut self) -> InternalResult<()> {
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key_event) = event::read()?
            && key_event.kind == KeyEventKind::Press
        {
            self.handle_key_event(key_event)?
        }
        Ok(())
    }

    fn history_backward(&mut self) {
        self.history_index = match self.history_index {
            Some(i) if i < self.history.len() => Some(i + 1),
            None if !self.history.is_empty() => Some(1),
            other => other,
        }
    }

    fn history_forward(&mut self) {
        self.history_index = match self.history_index {
            Some(i) if i > 1 => Some(i - 1),
            _ => None,
        }
    }

    fn history(&self) -> &str {
        match self.history_index {
            None => "",
            Some(i) => &self.history[self.history.len() - i],
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> InternalResult<()> {
        match self.mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('i') => self.mode = InputMode::Insert,
                _ => {}
            },
            InputMode::Insert => match key_event.code {
                KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.input.clear()
                }
                KeyCode::Char('w') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(pos) = self.input.trim_end().rfind(' ') {
                        self.input.truncate(pos + 1);
                    } else {
                        self.input.clear();
                    }
                }
                KeyCode::Esc => self.mode = InputMode::Normal,
                KeyCode::Enter => {
                    self.execute_command()?;
                }
                KeyCode::Char(c) => self.input.push(c),
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Up => {
                    self.history_backward();
                    self.input = self.history().to_string();
                }
                KeyCode::Down => {
                    self.history_forward();
                    self.input = self.history().to_string();
                }
                _ => {}
            },
        }
        Ok(())
    }

    fn execute_command(&mut self) -> InternalResult<()> {
        if self.input.trim().is_empty() {
            return Ok(());
        }
        log::debug!("> {}", self.input);
        let args = shlex::split(&self.input).unwrap_or_default();
        match Cli::try_parse_from(std::iter::once("").chain(args.iter().map(|s| s.as_str()))) {
            Ok(cli) => match cli.command {
                Command::Endpoint { action } => match action {
                    EndpointAction::Add {
                        method,
                        path,
                        response,
                    } => {
                        self.server_state
                            .add_endpoint(method.into(), &path, response)?;
                    }
                    EndpointAction::List { method } => {
                        let method = method.map(Into::into);
                        self.server_state.list_endpoints(method.as_ref())?;
                    }
                    EndpointAction::Delete { method, path } => {
                        self.server_state.delete_endpoint(&method.into(), &path)?;
                    }
                },
            },
            Err(e) => {
                if e.kind() == clap::error::ErrorKind::DisplayHelp
                    || e.kind() == clap::error::ErrorKind::DisplayVersion
                {
                    log::info!("{}", e);
                } else {
                    log::warn!("{}", e);
                }
            }
        }
        self.history.push(self.input.clone());
        self.history_index = None;
        self.input.clear();
        Ok(())
    }
}
