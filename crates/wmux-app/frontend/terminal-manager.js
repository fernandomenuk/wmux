import { Terminal } from './vendor/xterm.mjs';
import { FitAddon } from './vendor/addon-fit.mjs';
import { WebglAddon } from './vendor/addon-webgl.mjs';

const THEME = {
  background: '#0a0a0c',
  foreground: '#fafafa',
  cursor: '#6366f1',
  selectionBackground: 'rgba(99, 102, 241, 0.3)',
  black: '#0a0a0c',
  red: '#f87171',
  green: '#4ade80',
  yellow: '#fbbf24',
  blue: '#6366f1',
  magenta: '#a78bfa',
  cyan: '#22d3ee',
  white: '#fafafa',
  brightBlack: '#71717a',
  brightRed: '#fca5a5',
  brightGreen: '#86efac',
  brightYellow: '#fde68a',
  brightBlue: '#818cf8',
  brightMagenta: '#c4b5fd',
  brightCyan: '#67e8f9',
  brightWhite: '#ffffff',
};

const terminals = new Map();
let focusedId = null;
let onInputCallback = null;
let onNewTerminalCallback = null;

export function setOnInput(callback) { onInputCallback = callback; }
export function setOnNewTerminal(callback) { onNewTerminalCallback = callback; }

function createActionBar(surfaceId) {
  const bar = document.createElement('div');
  bar.className = 'action-bar';
  
  const createBtn = (icon, title, cmd, args = {}) => {
    const btn = document.createElement('button');
    btn.className = 'action-btn' + (title === 'Close' ? ' close' : '');
    btn.innerHTML = icon;
    btn.title = title;
    btn.onclick = (e) => {
      e.stopPropagation();
      window.__TAURI__.core.invoke(cmd, { surfaceId, ...args });
    };
    return btn;
  };

  bar.appendChild(createBtn('V', 'Split Vertical', 'split_pane', { direction: 'vertical' }));
  bar.appendChild(createBtn('H', 'Split Horizontal', 'split_pane', { direction: 'horizontal' }));
  bar.appendChild(createBtn('Z', 'Zoom', 'toggle_zoom'));
  bar.appendChild(createBtn('×', 'Close', 'close_pane'));

  return bar;
}

export function createTerminal(surfaceId) {
  if (terminals.has(surfaceId)) return terminals.get(surfaceId);

  const term = new Terminal({
    cursorBlink: true,
    fontFamily: "'Geist Mono', 'Cascadia Code', monospace",
    fontSize: 13,
    theme: THEME,
    allowTransparency: true,
  });

  const fitAddon = new FitAddon();
  term.loadAddon(fitAddon);

  try {
    term.loadAddon(new WebglAddon());
  } catch (e) {}

  const container = document.createElement('div');
  container.className = 'pane';
  container.dataset.surfaceId = surfaceId;

  const actionBar = createActionBar(surfaceId);
  container.appendChild(actionBar);

  document.getElementById('pane-area').appendChild(container);
  term.open(container);

  if (onNewTerminalCallback) onNewTerminalCallback(surfaceId, term);

  term.onData((data) => {
    if (surfaceId === focusedId && onInputCallback) onInputCallback(surfaceId, data);
  });

  container.addEventListener('mousedown', () => {
    window.__TAURI__.core.invoke('focus_pane', { surfaceId });
  });

  const entry = { term, fitAddon, container, actionBar };
  terminals.set(surfaceId, entry);
  return entry;
}

export function destroyTerminal(surfaceId) {
  const entry = terminals.get(surfaceId);
  if (!entry) return;
  entry.term.dispose();
  entry.container.remove();
  terminals.delete(surfaceId);
}

export function writeOutput(surfaceId, data) {
  const entry = terminals.get(surfaceId);
  if (entry) entry.term.write(data);
}

export function setFocused(surfaceId) {
  focusedId = surfaceId;
  for (const [id, entry] of terminals) {
    entry.container.classList.toggle('focused', id === surfaceId);
    if (id === surfaceId) entry.term.focus();
  }
}

export function getFocusedId() { return focusedId; }
export function getTerminal(surfaceId) { return terminals.get(surfaceId); }

export async function applyLayout(panes, totalWidthCells, totalHeightCells, knownSurfaceIds = []) {
  const newIds = new Set(panes.map(p => p.surface_id));
  const knownIds = new Set(knownSurfaceIds);

  for (const id of [...terminals.keys()]) {
    if (knownIds.size > 0 && !knownIds.has(id)) destroyTerminal(id);
  }

  for (const [id, entry] of terminals) {
    if (!newIds.has(id)) entry.container.style.display = 'none';
  }

  for (const pane of panes) {
    let entry = terminals.get(pane.surface_id) || createTerminal(pane.surface_id);
    const left = (pane.x / totalWidthCells) * 100;
    const top = (pane.y / totalHeightCells) * 100;
    const width = (pane.width / totalWidthCells) * 100;
    const height = (pane.height / totalHeightCells) * 100;

    Object.assign(entry.container.style, {
      left: `${left}%`, top: `${top}%`, width: `${width}%`, height: `${height}%`,
      display: '', right: 'auto', bottom: 'auto'
    });

    if (pane.is_focused) setFocused(pane.surface_id);
  }

  await new Promise(r => requestAnimationFrame(r));
  for (const pane of panes) {
    const entry = terminals.get(pane.surface_id);
    if (entry) entry.fitAddon.fit();
  }
}

export function getCellDimensions() {
  for (const [, entry] of terminals) {
    const dims = entry.term._core._renderService?.dimensions;
    if (dims) return { cellWidth: dims.css.cell.width, cellHeight: dims.css.cell.height };
  }
  return { cellWidth: 9, cellHeight: 17 };
}
