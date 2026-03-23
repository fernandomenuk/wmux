import * as tm from './terminal-manager.js';
import { attachKeybindings } from './keybindings.js';
import { refreshLayout, setupResizeHandler } from './layout.js';
import { setupSidebar, refreshTabs, getActiveIndex } from './sidebar.js';
import { setupCommandPalette } from './command-palette.js';

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

async function init() {
  // Forward terminal input to backend
  tm.setOnInput((surfaceId, data) => {
    invoke('send_input', { surfaceId, data });
  });

  // Attach keybindings when new terminals are created
  tm.setOnNewTerminal((surfaceId, term) => {
    attachKeybindings(term, () => tm.getFocusedId(), getActiveIndex);
  });

  // Set up UI components
  setupSidebar();
  setupResizeHandler();
  setupCommandPalette();

  // PTY output → route to correct terminal
  listen('pty-output', (event) => {
    const { surface_id, data } = event.payload;
    const bytes = Uint8Array.from(atob(data), c => c.charCodeAt(0));
    tm.writeOutput(surface_id, bytes);
  });

  // PTY exit
  listen('pty-exit', (event) => {
    const { surface_id } = event.payload;
    tm.writeOutput(surface_id, new TextEncoder().encode('\r\n\x1b[90m[Process exited]\x1b[0m\r\n'));
  });

  // Layout/focus changes → refresh UI
  listen('layout-changed', async () => {
    await refreshTabs();
    await refreshLayout();
  });

  listen('focus-changed', (event) => {
    tm.setFocused(event.payload.surface_id);
  });

  // Initial load
  await refreshTabs();
  await refreshLayout();
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
