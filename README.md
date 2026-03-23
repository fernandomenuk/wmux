<div align="center">

# wmux

**tmux for Windows. Finally.**

[![Build](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Windows 10+](https://img.shields.io/badge/Windows-10%2B-0078D6?logo=windows)](https://www.microsoft.com/windows)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://www.rust-lang.org/)

Split panes. Tabbed workspaces. A socket API for AI agents.
One binary. Zero dependencies. Pure Windows.

</div>

---

```
 ┌──────────────────────────────────────────────────┐
 │ [1: project] [2: api] [3: tests]           wmux  │
 ├────────────────────────┬─────────────────────────┤
 │                        │                         │
 │  PS C:\project> _      │  PS C:\api> cargo test  │
 │                        │  running 75 tests...    │
 │                        │  test result: ok        │
 │                        │                         │
 ├────────────────────────┴─────────────────────────┤
 │ ws:1 surface:1 | powershell | \\.\pipe\wmux      │
 └──────────────────────────────────────────────────┘
```

## Why wmux?

Windows developers have been asking for a native terminal multiplexer for years. WSL workarounds, ConEmu configs, and "just use tmux in WSL" aren't real answers.

**wmux is the answer.** A single `.exe` that gives you:

- **Split panes** — vertical and horizontal, infinitely nestable
- **Tabbed workspaces** — switch contexts without losing your place
- **A JSON-RPC socket API** — so AI coding agents (Claude Code, Cursor, Copilot) can spawn terminals, run commands, and read output programmatically
- **Works with any shell** — PowerShell, cmd, WSL bash, nushell, whatever

Built on ConPTY (Windows' native pseudo-terminal), so it works in Windows Terminal, conhost, or any terminal emulator.

## Quick Start

```bash
# Build from source
git clone https://github.com/fernandomenuk/wmux.git
cd wmux
cargo build --release

# Run it
./target/release/wmux.exe
```

That's it. You're in.

## Keybindings

Prefix key: <kbd>Ctrl</kbd>+<kbd>A</kbd> (press prefix first, then the action key)

### Panes

| Keys | Action |
|------|--------|
| <kbd>Ctrl+A</kbd> <kbd>\|</kbd> | Split vertical |
| <kbd>Ctrl+A</kbd> <kbd>-</kbd> | Split horizontal |
| <kbd>Ctrl+A</kbd> <kbd>Arrow</kbd> | Move focus between panes |
| <kbd>Ctrl+A</kbd> <kbd>x</kbd> | Close current pane |
| <kbd>Ctrl+A</kbd> <kbd>z</kbd> | Toggle zoom (fullscreen current pane) |

### Workspaces

| Keys | Action |
|------|--------|
| <kbd>Ctrl+A</kbd> <kbd>c</kbd> | New workspace |
| <kbd>Ctrl+A</kbd> <kbd>n</kbd> | Next workspace |
| <kbd>Ctrl+A</kbd> <kbd>p</kbd> | Previous workspace |
| <kbd>Ctrl+A</kbd> <kbd>1-9</kbd> | Jump to workspace |

### Other

| Keys | Action |
|------|--------|
| <kbd>Ctrl+A</kbd> <kbd>q</kbd> | Quit wmux |
| <kbd>Ctrl+A</kbd> <kbd>Ctrl+A</kbd> | Send literal Ctrl+A to shell |

## Socket API

wmux exposes a JSON-RPC API over a Windows named pipe at `\\.\pipe\wmux`. This is what makes wmux special — AI agents can control your terminal sessions programmatically.

### Connect

```powershell
# PowerShell
$pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", "wmux", [System.IO.Pipes.PipeDirection]::InOut)
$pipe.Connect(5000)
$writer = New-Object System.IO.StreamWriter($pipe)
$reader = New-Object System.IO.StreamReader($pipe)
$writer.AutoFlush = $true
```

### Protocol

Newline-delimited JSON-RPC. Compatible with [cmux](https://github.com/anthropics/cmux) v2 format.

**Request:**
```json
{"id": "1", "method": "system.ping", "params": {}}
```

**Response:**
```json
{"id": "1", "ok": true, "result": {"pong": true}}
```

### Commands

#### System
| Method | Params | Returns |
|--------|--------|---------|
| `system.ping` | `{}` | `{"pong": true}` |
| `system.capabilities` | `{}` | `{"version": "0.1.0", "commands": [...]}` |

#### Workspaces
| Method | Params | Returns |
|--------|--------|---------|
| `workspace.list` | `{}` | `{"workspaces": [{"id", "name", "index"}]}` |
| `workspace.create` | `{name?}` | `{"id": "uuid"}` |
| `workspace.select` | `{id}` | `{}` |
| `workspace.current` | `{}` | `{"id", "name", "index"}` |
| `workspace.close` | `{id}` | `{}` |

#### Surfaces (Panes)
| Method | Params | Returns |
|--------|--------|---------|
| `surface.list` | `{workspace_id?}` | `{"surfaces": [{"id", "focused"}]}` |
| `surface.split` | `{direction}` | `{"id": "uuid"}` |
| `surface.focus` | `{id}` | `{}` |
| `surface.close` | `{id}` | `{}` |
| `surface.send_text` | `{id, text}` | `{}` |
| `surface.send_key` | `{id, key}` | `{}` |

**Supported keys for `surface.send_key`:** `Enter`, `Tab`, `Escape`, `Backspace`, `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `Delete`, `F1`-`F12`, `Ctrl+C`, `Ctrl+D`, `Ctrl+Z`, `Ctrl+L`, `Ctrl+A`

### Example: AI Agent Workflow

```python
import json, socket

# Connect to wmux pipe and create a workspace for running tests
send({"method": "workspace.create", "params": {"name": "tests"}})

# Get the surface ID
surfaces = send({"method": "surface.list", "params": {}})
surface_id = surfaces["surfaces"][0]["id"]

# Run tests
send({"method": "surface.send_text", "params": {"id": surface_id, "text": "cargo test\r"}})
```

## CLI Options

```
wmux                              # Launch with auto-detected shell
wmux --shell pwsh.exe             # Use specific shell
wmux --pipe \\.\pipe\my-wmux      # Custom pipe path
```

| Flag | Default | Description |
|------|---------|-------------|
| `--shell <path>` | Auto-detect | Shell executable to use |
| `--pipe <path>` | `\\.\pipe\wmux` | Named pipe path for socket API |
| `--help` | | Show help |
| `--version` | | Show version |

### Shell Detection

wmux picks your shell in this order:
1. `--shell` flag
2. `WMUX_SHELL` environment variable
3. `COMSPEC` environment variable (usually `cmd.exe`)
4. Fallback: `powershell.exe`

## Requirements

- **Windows 10 1809+** (ConPTY support required)
- **Recommended:** Windows Terminal for full true-color and Unicode support
- **Works in:** conhost (legacy), any terminal emulator with VT support

## Building from Source

```bash
# Requires Rust stable
cargo build --release

# Run tests
cargo test

# Install to PATH
cargo install --path .
```

## Architecture

Single-process Rust binary. No daemon, no background service.

```
wmux.exe
├── TUI Layer (ratatui + crossterm)
├── Socket Server (Tokio named pipe, JSON-RPC)
├── Shell Manager (ConPTY via portable-pty)
├── Terminal Emulation (vt100 crate)
└── App State (workspaces, split trees, surfaces)
```

The event loop multiplexes keyboard input, PTY output, socket commands, and rendering at 30fps via `tokio::select!`.

## Roadmap

- [x] Split panes (vertical + horizontal)
- [x] Tabbed workspaces
- [x] JSON-RPC socket API (cmux-compatible)
- [x] Zoom mode
- [x] Shell exit detection
- [x] Mouse support (click to focus, drag to resize, passthrough to apps)
- [ ] Configuration file
- [ ] CLI subcommands (`wmux list`, `wmux send-text`)
- [ ] Session persistence (detach/reattach)
- [ ] Scrollback navigation
- [ ] Copy/paste support

## License

[MIT](LICENSE)

---

<div align="center">

**Built for Windows developers who are tired of waiting.**

If wmux helps you, star the repo.

</div>
