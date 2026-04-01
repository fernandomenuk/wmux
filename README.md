<div align="center">

<a href="https://github.com/fernandomenuk/wmux">
  <img alt="wmux — terminal multiplexer for Windows" src="docs/banner.png" width="100%">
</a>

<br>

[![Release](https://img.shields.io/github/v/release/fernandomenuk/wmux?style=for-the-badge&color=7c3aed&labelColor=18181b)](https://github.com/fernandomenuk/wmux/releases)
[![Stars](https://img.shields.io/github/stars/fernandomenuk/wmux?style=for-the-badge&color=7c3aed&labelColor=18181b)](https://github.com/fernandomenuk/wmux/stargazers)
[![License](https://img.shields.io/badge/License-MIT-7c3aed?style=for-the-badge&labelColor=18181b)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Rust-18181b?style=for-the-badge&logo=rust&logoColor=a78bfa)](https://www.rust-lang.org/)

<br>

Split panes. Tabbed workspaces. A socket API for AI agents.<br>
One native `.exe`. Zero dependencies. Built with Rust and Tauri.

<br>

[**Website**](https://fernandomenuk.github.io/wmux) · [**Download**](https://github.com/fernandomenuk/wmux/releases/latest/download/wmux.msi) · [**Releases**](https://github.com/fernandomenuk/wmux/releases) · [**Issues**](https://github.com/fernandomenuk/wmux/issues)

<br>

</div>

## What is wmux?

**A native terminal multiplexer for Windows.** Split your terminal into panes, organize work into tabbed workspaces, and let AI agents control everything over a JSON-RPC socket — all from a single desktop app built in Rust.

> [!NOTE]
> wmux is the Windows counterpart to [cmux](https://github.com/anthropics/cmux). Same protocol, native experience.

## Get Started

**Download the installer:**

```
https://github.com/fernandomenuk/wmux/releases/latest/download/wmux.msi
```

**Or build from source:**

```bash
cargo install tauri-cli
git clone https://github.com/fernandomenuk/wmux.git
cd wmux/crates/wmux-app
cargo tauri build
```

## Features

**Graphical Split Panes** — vertical and horizontal, infinitely nestable, with high-DPI support

**Tabbed Workspaces** — create, rename, and switch between workspaces from the sidebar

**AI-First Socket API** — JSON-RPC over Windows named pipes at `\\.\pipe\wmux`. Let Claude Code, Cursor, or Copilot control your terminals programmatically

**Any Shell** — PowerShell, CMD, WSL, nushell — anything that runs on Windows

**Command Palette** — quick actions with `Ctrl+Shift+P`

**Blazing Fast** — Rust + ConPTY, near-zero overhead, no Electron

## Socket API

The API is what makes wmux different. AI agents connect over a named pipe and control your terminal sessions with JSON-RPC.

```powershell
$pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", "wmux", [System.IO.Pipes.PipeDirection]::InOut)
$pipe.Connect(5000)
```

| Method | Description |
|---|---|
| `workspace.create` | Create a new tabbed workspace |
| `workspace.close` | Close a workspace by ID |
| `surface.split` | Split the current pane |
| `surface.send_text` | Send commands to a terminal |

Compatible with [cmux](https://github.com/anthropics/cmux) v2 protocol.

## Architecture

```
wmux/
├── wmux-core     # PTY management, layouts, state machine
├── wmux-app      # Tauri desktop app with WebView UI
└── wmux-cli      # Legacy CLI (archived)
```

## Roadmap

- [x] Graphical split panes
- [x] Tabbed workspaces
- [x] JSON-RPC socket API
- [x] Command palette
- [x] Workspace sidebar
- [ ] Drag-and-drop pane resizing
- [ ] Theme customization
- [ ] Session persistence
- [ ] Plugin system

## Contributing

```bash
cargo tauri dev     # Dev mode with hot reload
cargo test          # Run the test suite
```

## License

[MIT](LICENSE)

---

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/logo.png" width="32">
  <source media="(prefers-color-scheme: light)" srcset="docs/logo.png" width="32">
  <img alt="wmux" src="docs/logo.png" width="32">
</picture>

<br><br>

**Built for Windows developers who are tired of waiting.**

[Star this repo](https://github.com/fernandomenuk/wmux) if wmux helps you — it helps others find it.

<br>

<a href="https://star-history.com/#fernandomenuk/wmux&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=fernandomenuk/wmux&type=Date&theme=dark">
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=fernandomenuk/wmux&type=Date">
    <img alt="Star History" src="https://api.star-history.com/svg?repos=fernandomenuk/wmux&type=Date" width="600">
  </picture>
</a>

</div>
