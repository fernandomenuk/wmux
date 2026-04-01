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

  const method = methodMap[name];
  if (!method) {
    throw new Error(`Unknown tool: ${name}`);
  }
  return await client.call(method, args);
}
