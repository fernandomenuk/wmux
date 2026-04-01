import { getActiveIndex } from './sidebar.js';
import { showInputModal } from './modal.js';

const { invoke } = window.__TAURI__.core;

const COMMANDS = [
  { id: 'new-ws', name: 'New Workspace', hint: 'Create a fresh workspace', action: async () => {
    const name = await showInputModal('Enter workspace name:');
    if (name !== null) invoke('create_workspace', { name: name || null });
  }},
  { id: 'rename-ws', name: 'Rename Workspace', hint: 'Change current workspace name', action: async () => {
    const idx = getActiveIndex();
    const currentName = document.querySelector('.tab.active')?.textContent.split(':')[1] || '';
    const name = await showInputModal('Rename workspace:', currentName);
    if (name !== null && name !== '') invoke('rename_workspace', { index: idx, name });
  }},
  { id: 'split-v', name: 'Split Vertical', hint: 'Add pane to the right', action: () => invoke('split_pane', { direction: 'vertical' }) },
  { id: 'split-h', name: 'Split Horizontal', hint: 'Add pane below', action: () => invoke('split_pane', { direction: 'horizontal' }) },
  { id: 'zoom', name: 'Toggle Zoom', hint: 'Fullscreen focused pane', action: () => invoke('toggle_zoom') },
  { id: 'close-pane', name: 'Close Pane', hint: 'Remove focused pane', action: () => invoke('close_pane', { surfaceId: getFocusedId() }) },
  { id: 'next-ws', name: 'Next Workspace', hint: 'Switch to next tab', action: () => invoke('next_workspace') },
  { id: 'prev-ws', name: 'Previous Workspace', hint: 'Switch to previous tab', action: () => invoke('prev_workspace') },
  { id: 'claude-code', name: 'Connect to Claude Code', hint: 'Set up MCP integration', action: async () => {
    try {
      const msg = await invoke('setup_claude_code');
      alert(msg + '\n\nRestart Claude Code to activate.');
    } catch (e) {
      alert('Error: ' + e);
    }
  }},
];

let isOpen = false;
let selectedIndex = 0;
let filteredCommands = [...COMMANDS];

function getFocusedId() {
  return document.querySelector('.pane.focused')?.dataset.surfaceId;
}

export function toggleCommandPalette() {
  if (isOpen) {
    closePalette();
  } else {
    openPalette();
  }
}

function openPalette() {
  const palette = document.getElementById('command-palette');
  const input = document.getElementById('cp-input');
  isOpen = true;
  filteredCommands = [...COMMANDS];
  selectedIndex = 0;
  palette.classList.add('open');
  input.value = '';
  input.focus();
  render();
}

function closePalette() {
  const palette = document.getElementById('command-palette');
  const input = document.getElementById('cp-input');
  isOpen = false;
  palette.classList.remove('open');
  input.value = '';
  document.querySelector('.pane.focused')?.focus();
}

function render() {
  const results = document.getElementById('cp-results');
  results.innerHTML = '';
  filteredCommands.forEach((cmd, i) => {
    const div = document.createElement('div');
    div.className = `cp-item ${i === selectedIndex ? 'selected' : ''}`;
    div.innerHTML = `<span>${cmd.name}</span><span class="cp-hint">${cmd.hint}</span>`;
    div.onclick = () => { cmd.action(); closePalette(); };
    results.appendChild(div);
  });
}

export function setupCommandPalette() {
  const input = document.getElementById('cp-input');

  // Handle keyboard navigation on the input itself (works even when xterm has focus)
  input.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      closePalette();
    }
    if (e.key === 'ArrowDown' || (e.key === 'Tab' && !e.shiftKey)) {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % filteredCommands.length;
      render();
    }
    if (e.key === 'ArrowUp' || (e.key === 'Tab' && e.shiftKey)) {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + filteredCommands.length) % filteredCommands.length;
      render();
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      filteredCommands[selectedIndex]?.action();
      closePalette();
    }
  });

  // Filter as user types
  input.addEventListener('input', (e) => {
    const val = e.target.value.toLowerCase();
    filteredCommands = COMMANDS.filter(c =>
      c.name.toLowerCase().includes(val) || c.hint.toLowerCase().includes(val)
    );
    selectedIndex = 0;
    render();
  });

  // Ctrl+K on window as fallback (for when no terminal is focused)
  window.addEventListener('keydown', (e) => {
    if ((e.key === 'k' || e.key === 'K') && e.ctrlKey && e.shiftKey) {
      e.preventDefault();
      toggleCommandPalette();
    }
  });
}
