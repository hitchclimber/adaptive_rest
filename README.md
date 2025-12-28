# Adaptive Mock REST Server

A simple REST Server based on `actix-web` that lets you add endpoints to test during runtime. While this was mainly for my own educational purposes (learned a ton, btw), I do hope someone might stumble upon this repo and find it useful and/or leave some pointers.
I'm certain there are thousands of other implementations, but I didn't check; sometimes, while playing around on new projects, I just wanted to have a convenient way (preferably via TUI) to change the API on the go.

## Usage

You can configure server address, port and themes via `toml` or `json` files. The program tries to load your configuration file from command line options (`--config`), environment (`ADPTV_CONFIG`) and will finally default to some preset.

On startup, you'll be in `vim`-like `Normal mode`. Following the onscreen instructions you can use these commands to add endpoints:

| Command | Aliases | Description |
|---------|---------|-------------|
| `endpoint add <path> <response>` | `ep a`, `ep ad`, `ep update`, `ep u`, `ep up` | Add or update an endpoint |
| `endpoint delete <path>` | `ep d`, `ep del` | Delete an endpoint |
| `endpoint list` | `ep l` | List all endpoints |
| `endpoint allow <method> <path>` | `ep allow` | Allow an HTTP method on an endpoint |
| `endpoint deny <method> <path>` | `ep deny` | Deny an HTTP method on an endpoint |

Supported methods: `get`, `post`, `put`, `patch`, `delete` (case insensitive)

In a separate window you can start hitting those endpoints with your app/`curl`.

## TODOs

- [x] config file support
- [ ] `XDG` support for config
- [ ] `OPTIONS` method
- [ ] load endpoints from file during runtime


