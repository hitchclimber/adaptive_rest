#![deny(warnings)]
#![deny(clippy::redundant_clone)]
use std::{io, sync::Arc, thread};
use tokio::sync::mpsc;

use crate::{
    app::App,
    server::{ServerState, run_server},
};

mod app;
mod command;
mod logger;
mod server;
mod util;

fn main() -> io::Result<()> {
    let (log_tx, log_rx) = mpsc::unbounded_channel();
    logger::TuiLogger::init(log_tx, log::Level::Info)?;
    log::info!("Application starting");
    let server_state = Arc::new(ServerState::new());
    let server_state_clone = server_state.clone();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(run_server(server_state_clone, "127.0.0.1:3000"))
    });
    let mut terminal = ratatui::init();
    let app_result = App::new(log_rx, server_state).run(&mut terminal);
    ratatui::restore();
    app_result
}
