# wmux MCP Server — Design Spec

## Overview

A TypeScript MCP server that bridges Claude Code (and other MCP-compatible AI agents) to wmux's JSON-RPC socket API. Lives at `mcp/` in the wmux repo. Connects to the wmux named pipe (`\\.\pipe\wmux`) and exposes tools for controlling and reading terminal sessions.

## Architecture

```
Claude Code ←(stdio)→ MCP Server (Node.js) ←(named pipe)→ wmux app (Rust)
```

The MCP server is a stateless bridge — it translates MCP tool calls into wmux JSON-RPC requests over the named pipe and returns responses. No state is held in the MCP server itself; wmux is the source of truth.

## Tools

### Raw Tools (1:1 mapping to wmux API)

| MCP Tool | wmux Method | Params | Returns |
|----------|------------|--------|---------|
| `wmux_ping` | `system.ping` | none | `{pong: true}` |
| `wmux_workspace_list` | `workspace.list` | none | `{workspaces: [{id, name, index}]}` |
| `wmux_workspace_create` | `workspace.create` | `{name?: string}` | `{id: string}` |
| `wmux_workspace_select` | `workspace.select` | `{id: string}` | `{}` |
| `wmux_workspace_close` | `workspace.close` | `{id: string}` | `{}` |
| `wmux_surface_list` | `surface.list` | `{workspace_id?: string}` | `{surfaces: [{id, focused}]}` |
| `wmux_surface_split` | `surface.split` | `{direction: "vertical"\|"horizontal"}` | `{id: string}` |
| `wmux_surface_focus` | `surface.focus` | `{id: string}` | `{}` |
| `wmux_surface_close` | `surface.close` | `{id: string}` | `{}` |
| `wmux_surface_send_text` | `surface.send_text` | `{id: string, text: string}` | `{}` |
| `wmux_surface_send_key` | `surface.send_key` | `{id: string, key: string}` | `{}` |
| `wmux_surface_read_output` | `surface.read_output` | `{id: string, rows?: number}` | `{output: string}` |

### Convenience Tools

| MCP Tool | Description | Params | Behavior |
|----------|------------|--------|----------|
| `wmux_run_command` | Run a command and return output | `{id: string, command: string, wait_ms?: number}` | Sends `command + \r` via `surface.send_text`, waits `wait_ms` (default 1000ms), reads output via `surface.read_output`, returns `{output: string}` |

## New wmux-core Method: `surface.read_output`

### Rationale
The existing API is write-only. AI agents need to read terminal output to be useful (e.g. check if tests passed). The vt100 parser in each `Surface` already maintains the full screen buffer — this method just exposes it.

### Implementation
- **File:** `crates/wmux-core/src/socket/commands.rs`
- **Method name:** `surface.read_output`
- **Params:** `{id: string, rows?: number}`
  - `id` — surface UUID
  - `rows` — optional, limits output to last N rows (default: all visible rows)
- **Returns:** `{output: string}` — screen content as text, one line per row, trailing whitespace trimmed
- **Implementation:** Call `surface.parser.screen().contents()` from the vt100 crate, which returns the full screen text. If `rows` is specified, take only the last N lines.

## Project Structure

```
mcp/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts          # MCP server entry point, stdio transport
│   ├── wmux-client.ts    # Named pipe JSON-RPC client
│   └── tools.ts          # Tool definitions and handlers
└── README.md
```

## Dependencies

- `@modelcontextprotocol/sdk` — MCP server SDK (stdio transport)
- `typescript` — build toolchain
- Node.js `net` module — connects to `\\.\pipe\wmux` (built-in, no extra dep)

## Named Pipe Client (`wmux-client.ts`)

Responsibilities:
- Connect to `\\.\pipe\wmux` using `net.connect()`
- Send newline-delimited JSON requests
- Read newline-delimited JSON responses
- Match responses to requests by `id`
- Handle connection errors (wmux not running)
- Auto-reconnect on disconnect

Key interface:
```typescript
class WmuxClient {
  connect(): Promise<void>
  call(method: string, params?: object): Promise<any>
  disconnect(): void
}
```

## Error Handling

- If wmux is not running (pipe doesn't exist): return clear error message "wmux is not running. Start the wmux desktop app first."
- If wmux returns an error response (`ok: false`): forward the error code and message to the agent
- If the pipe disconnects mid-call: attempt one reconnect, then fail with error

## Installation & Configuration

After building, users add to their Claude Code MCP config:
```json
{
  "mcpServers": {
    "wmux": {
      "command": "node",
      "args": ["path/to/wmux/mcp/dist/index.js"]
    }
  }
}
```

## Out of Scope

- SSE/HTTP transport (stdio only for v1)
- Terminal output streaming/subscriptions (poll with `read_output` for now)
- Authentication (local named pipe, same security model as wmux itself)
