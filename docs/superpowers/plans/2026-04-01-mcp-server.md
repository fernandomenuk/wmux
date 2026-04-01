# wmux MCP Server Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a TypeScript MCP server that bridges Claude Code to wmux's JSON-RPC socket API over Windows named pipes, including a new `surface.read_output` method in wmux-core.

**Architecture:** A Node.js process connects to `\\.\pipe\wmux` and translates MCP tool calls into wmux JSON-RPC requests. The MCP server uses stdio transport. A new `surface.read_output` Rust command exposes the vt100 screen buffer.

**Tech Stack:** TypeScript, `@modelcontextprotocol/sdk`, Node.js `net` module, Rust (wmux-core)

---

### Task 1: Add `surface.read_output` to wmux-core

**Files:**
- Modify: `crates/wmux-core/src/model/surface.rs:113-117`
- Modify: `crates/wmux-core/src/socket/commands.rs:9-217`
- Modify: `crates/wmux-core/tests/commands_test.rs`

- [ ] **Step 1: Add `read_output` method to Surface**

In `crates/wmux-core/src/model/surface.rs`, add after the `screen()` method (line 116):

```rust
    /// Read the terminal screen content as text.
    pub fn read_output(&self, max_rows: Option<usize>) -> String {
        let screen = self.parser.screen();
        let contents = screen.contents();
        match max_rows {
            Some(n) => {
                let lines: Vec<&str> = contents.lines().collect();
                let start = lines.len().saturating_sub(n);
                lines[start..].join("\n")
            }
            None => contents,
        }
    }
```

- [ ] **Step 2: Write failing test for `surface.read_output` command**

In `crates/wmux-core/tests/commands_test.rs`, add at the end:

```rust
// === surface.read_output ===

#[test]
fn surface_read_output_returns_screen_content() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let id = core.focused_surface.unwrap().to_string();
    let req = make_request("1", "surface.read_output", json!({"id": id}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    assert!(resp.result.unwrap()["output"].as_str().is_some());
}

#[test]
fn surface_read_output_with_rows_limit() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let id = core.focused_surface.unwrap().to_string();
    let req = make_request("1", "surface.read_output", json!({"id": id, "rows": 5}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(resp.ok);
    let output = resp.result.unwrap()["output"].as_str().unwrap().to_string();
    assert!(output.lines().count() <= 5);
}

#[test]
fn surface_read_output_nonexistent_errors() {
    let (mut core, pty_tx, exit_tx) = setup_core_with_workspace();
    let fake_id = Uuid::new_v4().to_string();
    let req = make_request("1", "surface.read_output", json!({"id": fake_id}));
    let resp = dispatch(&mut core, &req, &pty_tx, &exit_tx);
    assert!(!resp.ok);
    assert_eq!(resp.error.unwrap().code, "not_found");
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --manifest-path crates/wmux-core/Cargo.toml -- surface_read_output`
Expected: FAIL — `unknown_method` since the command handler doesn't exist yet.

- [ ] **Step 4: Add `surface.read_output` handler to commands.rs**

In `crates/wmux-core/src/socket/commands.rs`, add this arm before the `_ =>` catch-all (line 215):

```rust
        "surface.read_output" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let id_str = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let max_rows = params.get("rows").and_then(|v| v.as_u64()).map(|n| n as usize);
            match Uuid::parse_str(id_str) {
                Ok(id) => {
                    if let Some(surface) = core.surfaces.get(&id) {
                        let output = surface.read_output(max_rows);
                        Response::success(req.id.clone(), json!({"output": output}))
                    } else {
                        Response::error(req.id.clone(), "not_found", "Surface not found")
                    }
                }
                Err(_) => Response::error(req.id.clone(), "invalid_id", "Invalid UUID"),
            }
        }
```

Also update the `system.capabilities` command list (line 25) to include `"surface.read_output"`:

```rust
                "surface.close", "surface.send_text", "surface.send_key",
                "surface.read_output"
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --manifest-path crates/wmux-core/Cargo.toml`
Expected: ALL PASS

- [ ] **Step 6: Commit**

```bash
git add crates/wmux-core/src/model/surface.rs crates/wmux-core/src/socket/commands.rs crates/wmux-core/tests/commands_test.rs
git commit -m "feat: add surface.read_output command to wmux-core"
```

---

### Task 2: Scaffold MCP server project

**Files:**
- Create: `mcp/package.json`
- Create: `mcp/tsconfig.json`

- [ ] **Step 1: Create package.json**

```json
{
  "name": "wmux-mcp",
  "version": "0.6.3",
  "description": "MCP server for wmux terminal multiplexer",
  "type": "module",
  "main": "dist/index.js",
  "bin": {
    "wmux-mcp": "dist/index.js"
  },
  "scripts": {
    "build": "tsc",
    "dev": "tsc --watch"
  },
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.12.1"
  },
  "devDependencies": {
    "typescript": "^5.7.0",
    "@types/node": "^22.0.0"
  }
}
```

- [ ] **Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "Node16",
    "moduleResolution": "Node16",
    "outDir": "dist",
    "rootDir": "src",
    "strict": true,
    "esModuleInterop": true,
    "declaration": true,
    "sourceMap": true
  },
  "include": ["src"]
}
```

- [ ] **Step 3: Install dependencies**

Run: `cd mcp && npm install`
Expected: `node_modules/` created, `package-lock.json` generated.

- [ ] **Step 4: Commit**

```bash
git add mcp/package.json mcp/tsconfig.json mcp/package-lock.json
git commit -m "feat: scaffold wmux MCP server project"
```

---

### Task 3: Build named pipe client

**Files:**
- Create: `mcp/src/wmux-client.ts`

- [ ] **Step 1: Create the wmux client**

```typescript
import net from "net";

const PIPE_PATH = "\\\\.\\pipe\\wmux";

interface WmuxResponse {
  id: string;
  ok: boolean;
  result?: unknown;
  error?: { code: string; message: string };
}

export class WmuxClient {
  private socket: net.Socket | null = null;
  private buffer = "";
  private pending = new Map<
    string,
    { resolve: (value: WmuxResponse) => void; reject: (err: Error) => void }
  >();
  private nextId = 1;

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.socket = net.connect(PIPE_PATH, () => resolve());
      this.socket.on("error", (err) => {
        // Reject all pending requests
        for (const [, { reject: rej }] of this.pending) {
          rej(err);
        }
        this.pending.clear();
        reject(err);
      });
      this.socket.on("data", (data) => this.onData(data));
      this.socket.on("close", () => {
        this.socket = null;
      });
    });
  }

  private onData(data: Buffer): void {
    this.buffer += data.toString();
    const lines = this.buffer.split("\n");
    this.buffer = lines.pop() ?? "";

    for (const line of lines) {
      if (!line.trim()) continue;
      try {
        const response: WmuxResponse = JSON.parse(line);
        const pending = this.pending.get(response.id);
        if (pending) {
          this.pending.delete(response.id);
          pending.resolve(response);
        }
      } catch {
        // Ignore malformed responses
      }
    }
  }

  async call(method: string, params?: Record<string, unknown>): Promise<unknown> {
    if (!this.socket) {
      await this.connect();
    }

    const id = String(this.nextId++);
    const request = JSON.stringify({ id, method, params: params ?? {} });

    return new Promise<unknown>((resolve, reject) => {
      this.pending.set(id, {
        resolve: (resp) => {
          if (resp.ok) {
            resolve(resp.result);
          } else {
            reject(
              new Error(
                `wmux error [${resp.error?.code}]: ${resp.error?.message}`
              )
            );
          }
        },
        reject,
      });

      this.socket!.write(request + "\n", (err) => {
        if (err) {
          this.pending.delete(id);
          reject(err);
        }
      });

      // Timeout after 10s
      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error("wmux request timed out"));
        }
      }, 10000);
    });
  }

  disconnect(): void {
    this.socket?.destroy();
    this.socket = null;
  }

  get connected(): boolean {
    return this.socket !== null && !this.socket.destroyed;
  }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd mcp && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add mcp/src/wmux-client.ts
git commit -m "feat: add named pipe client for wmux JSON-RPC"
```

---

### Task 4: Define MCP tools

**Files:**
- Create: `mcp/src/tools.ts`

- [ ] **Step 1: Create tool definitions and handlers**

```typescript
import { WmuxClient } from "./wmux-client.js";

export interface Tool {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
}

const idParam = {
  id: { type: "string", description: "Surface UUID" },
};

export const tools: Tool[] = [
  {
    name: "wmux_ping",
    description: "Check if wmux is running",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "wmux_workspace_list",
    description: "List all workspaces (tabs)",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "wmux_workspace_create",
    description: "Create a new workspace",
    inputSchema: {
      type: "object",
      properties: { name: { type: "string", description: "Workspace name" } },
    },
  },
  {
    name: "wmux_workspace_select",
    description: "Switch to a workspace by ID",
    inputSchema: {
      type: "object",
      properties: { id: { type: "string", description: "Workspace UUID" } },
      required: ["id"],
    },
  },
  {
    name: "wmux_workspace_close",
    description: "Close a workspace by ID",
    inputSchema: {
      type: "object",
      properties: { id: { type: "string", description: "Workspace UUID" } },
      required: ["id"],
    },
  },
  {
    name: "wmux_surface_list",
    description: "List all surfaces (panes) in a workspace",
    inputSchema: {
      type: "object",
      properties: {
        workspace_id: {
          type: "string",
          description: "Workspace UUID (defaults to active)",
        },
      },
    },
  },
  {
    name: "wmux_surface_split",
    description: "Split the focused pane",
    inputSchema: {
      type: "object",
      properties: {
        direction: {
          type: "string",
          enum: ["vertical", "horizontal"],
          description: "Split direction",
        },
      },
      required: ["direction"],
    },
  },
  {
    name: "wmux_surface_focus",
    description: "Focus a specific pane",
    inputSchema: {
      type: "object",
      properties: idParam,
      required: ["id"],
    },
  },
  {
    name: "wmux_surface_close",
    description: "Close a pane",
    inputSchema: {
      type: "object",
      properties: idParam,
      required: ["id"],
    },
  },
  {
    name: "wmux_surface_send_text",
    description: "Send raw text to a terminal pane",
    inputSchema: {
      type: "object",
      properties: {
        ...idParam,
        text: { type: "string", description: "Text to send" },
      },
      required: ["id", "text"],
    },
  },
  {
    name: "wmux_surface_send_key",
    description:
      "Send a named key to a terminal pane (Enter, Tab, Escape, Ctrl+C, etc.)",
    inputSchema: {
      type: "object",
      properties: {
        ...idParam,
        key: { type: "string", description: "Key name (Enter, Ctrl+C, F1, etc.)" },
      },
      required: ["id", "key"],
    },
  },
  {
    name: "wmux_surface_read_output",
    description: "Read the current terminal screen content from a pane",
    inputSchema: {
      type: "object",
      properties: {
        ...idParam,
        rows: {
          type: "number",
          description: "Limit to last N rows (default: all visible)",
        },
      },
      required: ["id"],
    },
  },
  {
    name: "wmux_run_command",
    description:
      "Run a command in a terminal pane and return the output. Sends the command, waits, then reads the screen.",
    inputSchema: {
      type: "object",
      properties: {
        ...idParam,
        command: { type: "string", description: "Command to run" },
        wait_ms: {
          type: "number",
          description: "Milliseconds to wait for output (default: 1000)",
        },
      },
      required: ["id", "command"],
    },
  },
];

// Map MCP tool name to wmux JSON-RPC method
const methodMap: Record<string, string> = {
  wmux_ping: "system.ping",
  wmux_workspace_list: "workspace.list",
  wmux_workspace_create: "workspace.create",
  wmux_workspace_select: "workspace.select",
  wmux_workspace_close: "workspace.close",
  wmux_surface_list: "surface.list",
  wmux_surface_split: "surface.split",
  wmux_surface_focus: "surface.focus",
  wmux_surface_close: "surface.close",
  wmux_surface_send_text: "surface.send_text",
  wmux_surface_send_key: "surface.send_key",
  wmux_surface_read_output: "surface.read_output",
};

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function handleToolCall(
  client: WmuxClient,
  name: string,
  args: Record<string, unknown>
): Promise<unknown> {
  // Convenience tool: run_command
  if (name === "wmux_run_command") {
    const { id, command, wait_ms = 1000 } = args as {
      id: string;
      command: string;
      wait_ms?: number;
    };
    await client.call("surface.send_text", { id, text: command + "\r" });
    await sleep(wait_ms as number);
    return await client.call("surface.read_output", { id });
  }

  // Raw tools: direct 1:1 mapping
  const method = methodMap[name];
  if (!method) {
    throw new Error(`Unknown tool: ${name}`);
  }
  return await client.call(method, args);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd mcp && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add mcp/src/tools.ts
git commit -m "feat: define MCP tool definitions and handlers"
```

---

### Task 5: Create MCP server entry point

**Files:**
- Create: `mcp/src/index.ts`

- [ ] **Step 1: Create the MCP server**

```typescript
#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { WmuxClient } from "./wmux-client.js";
import { tools, handleToolCall } from "./tools.js";

const client = new WmuxClient();
const server = new McpServer({
  name: "wmux",
  version: "0.6.3",
});

// Register all tools
for (const tool of tools) {
  server.tool(tool.name, tool.description, tool.inputSchema, async (args) => {
    try {
      const result = await handleToolCall(
        client,
        tool.name,
        args as Record<string, unknown>
      );
      return {
        content: [
          { type: "text" as const, text: JSON.stringify(result, null, 2) },
        ],
      };
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Unknown error";

      // Friendly message if wmux isn't running
      if (message.includes("ENOENT") || message.includes("connect")) {
        return {
          content: [
            {
              type: "text" as const,
              text: "wmux is not running. Start the wmux desktop app first.",
            },
          ],
          isError: true,
        };
      }

      return {
        content: [{ type: "text" as const, text: message }],
        isError: true,
      };
    }
  });
}

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch((err) => {
  console.error("Fatal:", err);
  process.exit(1);
});
```

- [ ] **Step 2: Build the project**

Run: `cd mcp && npm run build`
Expected: `dist/` directory created with `index.js`, `wmux-client.js`, `tools.js`.

- [ ] **Step 3: Commit**

```bash
git add mcp/src/index.ts
git commit -m "feat: MCP server entry point with stdio transport"
```

---

### Task 6: Build, test end-to-end, and add to .gitignore

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: Add mcp build artifacts to .gitignore**

Append to `.gitignore`:

```
# MCP server
mcp/node_modules/
mcp/dist/
```

- [ ] **Step 2: Build the MCP server**

Run: `cd mcp && npm run build`
Expected: Clean build, `dist/index.js` exists.

- [ ] **Step 3: Test manually with wmux running**

Start wmux, then run:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | node mcp/dist/index.js
```

Expected: JSON response listing all 13 tools.

- [ ] **Step 4: Commit**

```bash
git add .gitignore mcp/
git commit -m "feat: wmux MCP server complete"
```

---

### Task 7: Add MCP config and documentation

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add MCP setup section to README.md**

Add after the "Socket API" section in `README.md`:

```markdown
## MCP Server (Claude Code Integration)

wmux ships with an MCP server so AI agents can control your terminals directly.

**Setup:**

```bash
cd mcp && npm install && npm run build
```

Add to your Claude Code MCP config (`~/.claude/settings.json`):

```json
{
  "mcpServers": {
    "wmux": {
      "command": "node",
      "args": ["C:/path/to/wmux/mcp/dist/index.js"]
    }
  }
}
```

Now Claude Code can create workspaces, split panes, run commands, and read terminal output — all through wmux.
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add MCP server setup instructions"
```
