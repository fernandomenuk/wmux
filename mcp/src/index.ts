#!/usr/bin/env node

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  ListToolsRequestSchema,
  CallToolRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { WmuxClient } from "./wmux-client.js";
import { tools, handleToolCall } from "./tools.js";

const client = new WmuxClient();
const server = new Server(
  { name: "wmux", version: "0.6.3" },
  { capabilities: { tools: {} } }
);

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: tools.map((t) => ({
    name: t.name,
    description: t.description,
    inputSchema: t.inputSchema,
  })),
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args = {} } = request.params;

  try {
    const result = await handleToolCall(
      client,
      name,
      args as Record<string, unknown>
    );
    return {
      content: [
        { type: "text" as const, text: JSON.stringify(result, null, 2) },
      ],
    };
  } catch (err) {
    const message = err instanceof Error ? err.message : "Unknown error";

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

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch((err) => {
  console.error("Fatal:", err);
  process.exit(1);
});
