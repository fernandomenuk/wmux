import { Terminal } from './vendor/xterm.mjs';
import { FitAddon } from './vendor/addon-fit.mjs';
import { WebglAddon } from './vendor/addon-webgl.mjs';

const THEME = {
  background: '#09090b',
  foreground: '#fafafa',
  cursor: '#a78bfa',
  selectionBackground: 'rgba(167, 139, 250, 0.3)',
  black: '#09090b',
  red: '#f87171',
  green: '#4ade80',
  yellow: '#fbbf24',
  blue: '#60a5fa',
  magenta: '#a78bfa',
  cyan: '#22d3ee',
  white: '#fafafa',
  brightBlack: '#71717a',
  brightRed: '#fca5a5',
  brightGreen: '#86efac',
  brightYellow: '#fde68a',
  brightBlue: '#93c5fd',
  brightMagenta: '#c4b5fd',
  brightCyan: '#67e8f9',
  brightWhite: '#ffffff',
};

// Map of surfaceId → { term, fitAddon, container, onDataDispose }
const terminals = new Map();

// The currently focused surface ID
let focusedId = null;

// Callback set by main.js for input forwarding
let onInputCallback = null;

let onNewTerminalCallback = null;

export function setOnInput(callback) {
  onInputCallback = callback;
}

export function setOnNewTerminal(callback) {
  onNewTerminalCallback = callback;
}

export function createTerminal(surfaceId) {
  if (terminals.has(surfaceId)) return terminals.get(surfaceId);

  const term = new Terminal({
    cursorBlink: true,
    fontFamily: "'Cascadia Code', 'Consolas', 'Courier New', monospace",
    fontSize: 14,
    theme: THEME,
  });

  const fitAddon = new FitAddon();
  term.loadAddon(fitAddon);

  try {
    const webglAddon = new WebglAddon();
    term.loadAddon(webglAddon);
  } catch (e) {
    // WebGL not available or context limit hit — canvas fallback
  }

  const container = document.createElement('div');
  container.className = 'pane';
  container.dataset.surfaceId = surfaceId;
  document.getElementById('pane-area').appendChild(container);

  term.open(container);

  if (onNewTerminalCallback) {
    onNewTerminalCallback(surfaceId, term);
  }

  // Forward input to backend
  const onDataDispose = term.onData((data) => {
    if (surfaceId === focusedId && onInputCallback) {
      onInputCallback(surfaceId, data);
    }
  });

  // Click to focus
  container.addEventListener('mousedown', () => {
    if (onInputCallback) {
      window.__TAURI__.core.invoke('focus_pane', { surfaceId });
    }
  });

  const entry = { term, fitAddon, container, onDataDispose };
  terminals.set(surfaceId, entry);
  return entry;
}

export function destroyTerminal(surfaceId) {
  const entry = terminals.get(surfaceId);
  if (!entry) return;
  entry.onDataDispose.dispose();
  entry.term.dispose();
  entry.container.remove();
  terminals.delete(surfaceId);
}

export function writeOutput(surfaceId, data) {
  const entry = terminals.get(surfaceId);
  if (entry) {
    entry.term.write(data);
  }
}

export function setFocused(surfaceId) {
  focusedId = surfaceId;
  for (const [id, entry] of terminals) {
    entry.container.classList.toggle('focused', id === surfaceId);
    if (id === surfaceId) {
      entry.term.focus();
    }
  }
}

export function getFocusedId() {
  return focusedId;
}

export function getTerminal(surfaceId) {
  return terminals.get(surfaceId);
}

export function getAllSurfaceIds() {
  return new Set(terminals.keys());
}

// Position panes based on layout data and fit them.
// totalWidthCells/totalHeightCells are the cell dimensions passed to get_layout.
export function applyLayout(panes, totalWidthCells, totalHeightCells) {
  const newIds = new Set(panes.map(p => p.surface_id));

  // Hide terminals not in this layout (workspace switch — keep alive for scrollback)
  for (const [id, entry] of terminals) {
    if (!newIds.has(id)) {
      entry.container.style.display = 'none';
    }
  }

  // Create new terminals and position all visible ones
  for (const pane of panes) {
    let entry = terminals.get(pane.surface_id);
    if (!entry) {
      entry = createTerminal(pane.surface_id);
    }

    // Convert cell coordinates to percentages of the pane area
    const left = (pane.x / totalWidthCells) * 100;
    const top = (pane.y / totalHeightCells) * 100;
    const width = (pane.width / totalWidthCells) * 100;
    const height = (pane.height / totalHeightCells) * 100;

    entry.container.style.left = `${left}%`;
    entry.container.style.top = `${top}%`;
    entry.container.style.width = `${width}%`;
    entry.container.style.height = `${height}%`;
    entry.container.style.display = '';

    if (pane.is_focused) {
      setFocused(pane.surface_id);
    }
  }

  // Fit all terminals after positioning (need a frame for CSS to settle)
  requestAnimationFrame(() => {
    for (const pane of panes) {
      const entry = terminals.get(pane.surface_id);
      if (entry) {
        entry.fitAddon.fit();
      }
    }
  });
}

// Hide all terminals (used when switching workspaces — terminals for other workspaces stay alive)
export function hideAll() {
  for (const [, entry] of terminals) {
    entry.container.style.display = 'none';
  }
}

// Get xterm.js cell dimensions (for converting pixel area to cell counts)
export function getCellDimensions() {
  // Use the first terminal's dimensions as reference
  for (const [, entry] of terminals) {
    const dims = entry.term._core._renderService?.dimensions;
    if (dims) {
      return { cellWidth: dims.css.cell.width, cellHeight: dims.css.cell.height };
    }
  }
  // Fallback
  return { cellWidth: 8, cellHeight: 16 };
}
