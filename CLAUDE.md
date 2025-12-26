# Adaptable REST

A TUI application for mocking REST API endpoints on-the-fly during development.

## Architecture

- **TUI**: ratatui + crossterm with vim-style modal input (Normal/Insert modes)
- **Server**: actix-web running in separate thread with its own tokio runtime
- **Logging**: Custom `TuiLogger` implementing `log::Log` trait, routes all logs to TUI via channel
- **Commands**: clap-based parser with subcommands (Endpoint add/delete/list)

## Project Structure

```
src/
├── main.rs      # Entry point, spawns server thread, runs TUI
├── app.rs       # App struct, TUI widgets (CommandPane, LogPane), event handling
├── server.rs    # ServerState, actix-web handlers, endpoint management
├── command.rs   # Clap CLI/Command/EndpointAction definitions
├── logger.rs    # TuiLogger for routing log::* macros to TUI
└── util.rs      # InternalError enum, InternalResult type alias
```

## Key Patterns

- `Arc<ServerState>` shared between TUI and server threads
- `RwLock<HashMap<String, String>>` for thread-safe endpoint storage
- `tokio::sync::mpsc::unbounded_channel` for log messages (sender to logger, receiver in App)
- `event::poll()` with 100ms timeout for non-blocking TUI updates
- Server runs in `std::thread::spawn` with its own `tokio::runtime::Runtime` (actix futures aren't Send)

## Commands (typed in TUI)

```
endpoint add /path '{"json": "response"}'
endpoint delete /path
endpoint list
help
```

Note: JSON responses should be quoted with single quotes.

## Error Handling

- `InternalError` enum with thiserror derive
- `InternalResult<T>` type alias
- Implements `From<InternalError> for std::io::Error` for `?` compatibility in main

## TODOs (from code comments)

- Input parsers for more complex endpoint definitions
- Load endpoints from JSON files
- Handle different HTTP methods and response formats
- Tab completion for commands

## Future features
- Theming
- add endpoints from files
- UI, input improvements (history, arrow keys/vim keys)
- scrolling
- config file -> configure UI, port, etc
- more tests
- command history
- custom help text
- better layout for listing endpoints -> formatting with escape codes does not work
- return content on 404
- log warning if no commands can be parsed
- listing with no endpoints should return something


## Dependencies

Key crates: ratatui, crossterm, actix-web, tokio, clap (derive), log, thiserror, shlex
