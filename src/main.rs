#![deny(warnings)]
#![deny(clippy::redundant_clone)]
use clap::Parser;
use std::{io, path::PathBuf, sync::Arc, thread};
use tokio::sync::mpsc;

use crate::{
    app::{App, AppStyle, ColorTheme},
    server::{ServerState, run_server},
};

mod app;
mod command;
mod config;
mod logger;
mod server;
mod util;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
}
fn main() -> io::Result<()> {
    let (log_tx, log_rx) = mpsc::unbounded_channel();
    logger::TuiLogger::init(log_tx, log::Level::Debug)?;
    log::info!("Application starting");
    let args = Args::parse();
    let config = config::load(args.config);
    let color_theme = config.theme;

    let server = config.server;

    let style = AppStyle::from(&ColorTheme::or_default(color_theme));
    let server_state = Arc::new(ServerState::new());
    let server_state_clone = server_state.clone();
    log::debug!("endpoints file is {:?}", &config.endpoints_file);
    if let Some(path) = &config.endpoints_file
        && let Some(endpoints) = config::load_endpoints(path.clone())
    {
        log::debug!("Loaded {} endpoints", endpoints.len());
        for ep in endpoints {
            let methods = ep
                .methods
                .map(|ms| ms.iter().filter_map(|s| s.parse().ok()).collect());
            server_state.add_endpoint(&ep.path, ep.data.to_string(), methods)?;
        }
    }
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(run_server(server_state_clone, &server.address, server.port))
    });
    let mut terminal = ratatui::init();
    crossterm::execute!(std::io::stdout(), style.cursor_style)?;
    let app_result = App::new(log_rx, server_state, style).run(&mut terminal);
    ratatui::restore();
    app_result
}
