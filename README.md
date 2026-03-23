<div align="center">

# wmux

**The terminal multiplexer Windows deserves.**

[![Build](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Windows 10+](https://img.shields.io/badge/Windows-10%2B-0078D6?logo=windows)](https://www.microsoft.com/windows)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://www.rust-lang.org/)

Split panes. Tabbed workspaces. A socket API for AI agents.
A native desktop app for Windows. Built with Rust and Tauri.

</div>

---

```
 ┌──────────────────────────────────────────────────┐
 │ [ Project ] [ API ] [ Tests ]             [ + ]  │
 ├────────────────────────┬─────────────────────────┤
 │                        │                         │
 │  PS C:\project> _      │  PS C:\api> cargo test  │
 │                        │  running 75 tests...    │
 │                        │  test result: ok        │
 │                        │                         │
 ├────────────────────────┴─────────────────────────┤
 │ Shell: pwsh | WS: Project | Pane: 1 | CPU: 0.2%  │
 └──────────────────────────────────────────────────┘
```

## Why wmux?

Windows developers have been asking for a native terminal multiplexer for years. wmux is a high-performance desktop application that brings the power of `tmux` to Windows with a modern, graphical interface.

**wmux is the answer.** A native `.exe` that gives you:

- **Graphical Split Panes** — vertical and horizontal, infinitely nestable
- **Tabbed Workspaces** — switch contexts with a clean, modern UI
- **AI-First Socket API** — a JSON-RPC interface that lets AI coding agents (Claude Code, Cursor, Copilot) control your terminal sessions programmatically
- **Native Performance** — Built on ConPTY (Windows' native pseudo-terminal) for maximum compatibility and speed

## Quick Start

### Download
Grab the latest release from the [Releases](https://github.com/fernandomenuk/wmux/releases) page.

### Build from Source
```bash
# Requires Rust and Tauri CLI
git clone https://github.com/fernandomenuk/wmux.git
cd wmux/crates/wmux-app
cargo tauri build
```

The app will be available in `target/release/wmux-app.exe`.

## Key Features

### Modern UI
- **Sidebar Navigation**: Manage all your workspaces at a glance.
- **Command Palette**: Quick actions with `Ctrl+Shift+P`.
- **Customizable Layouts**: Drag and drop support coming soon.

### Socket API
wmux exposes a JSON-RPC API over a Windows named pipe at `\\.\pipe\wmux`. This is what makes wmux special — AI agents can control your terminal sessions programmatically.

#### Connect (PowerShell)
```powershell
$pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", "wmux", [System.IO.Pipes.PipeDirection]::InOut)
$pipe.Connect(5000)
```

#### Protocol
Newline-delimited JSON-RPC. Compatible with [cmux](https://github.com/anthropics/cmux) v2 format.

| Method | Description |
|--------|-------------|
| `workspace.create` | Create a new tabbed workspace |
| `surface.split` | Split the current terminal pane |
| `surface.send_text` | Send commands to a terminal |

## Architecture

wmux is built with a modular architecture:
- **wmux-core**: The engine handling PTYs, layouts, and state.
- **wmux-app**: The Tauri-based desktop interface.
- **wmux-cli**: A legacy terminal-based interface (archived).

## Roadmap

- [x] Graphical Split panes
- [x] Tabbed workspaces
- [x] JSON-RPC socket API
- [x] Command Palette
- [ ] Drag-and-drop pane resizing
- [ ] Theme support (Light/Dark/Custom)
- [ ] Session persistence (Detach/Reattach)

## License

[MIT](LICENSE)

---

<div align="center">

**Built for Windows developers who are tired of waiting.**

If wmux helps you, star the repo.

</div>
