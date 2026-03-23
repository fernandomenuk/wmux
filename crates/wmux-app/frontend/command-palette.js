const { invoke } = window.__TAURI__.core;

const COMMANDS = [
  { id: 'new-ws', name: 'New Workspace', hint: 'Create a fresh workspace', action: () => invoke('create_workspace') },
  { id: 'split-v', name: 'Split Vertical', hint: 'Add pane to the right', action: () => invoke('split_pane', { direction: 'vertical' }) },
  { id: 'split-h', name: 'Split Horizontal', hint: 'Add pane below', action: () => invoke('split_pane', { direction: 'horizontal' }) },
  { id: 'zoom', name: 'Toggle Zoom', hint: 'Fullscreen focused pane', action: () => invoke('toggle_zoom') },
  { id: 'close-pane', name: 'Close Pane', hint: 'Remove focused pane', action: () => invoke('close_pane', { surfaceId: getFocusedId() }) },
  { id: 'next-ws', name: 'Next Workspace', hint: 'Switch to next tab', action: () => invoke('next_workspace') },
  { id: 'prev-ws', name: 'Previous Workspace', hint: 'Switch to previous tab', action: () => invoke('prev_workspace') },
];

let isOpen = false;
let selectedIndex = 0;
let filteredCommands = [...COMMANDS];

function getFocusedId() {
  // Simple accessor since we can't easily import from tm without circularity
  return document.querySelector('.pane.focused')?.dataset.surfaceId;
}

export function setupCommandPalette() {
  const palette = document.getElementById('command-palette');
  const input = document.getElementById('cp-input');
  const results = document.getElementById('cp-results');

  const close = () => {
    isOpen = false;
    palette.classList.remove('open');
    input.value = '';
    // Return focus to terminal
    document.querySelector('.pane.focused')?.focus();
  };

  const open = () => {
    isOpen = true;
    palette.classList.add('open');
    input.focus();
    render();
  };

  const render = () => {
    results.innerHTML = '';
    filteredCommands.forEach((cmd, i) => {
      const div = document.createElement('div');
      div.className = `cp-item ${i === selectedIndex ? 'selected' : ''}`;
      div.innerHTML = `<span>${cmd.name}</span><span class="cp-hint">${cmd.hint}</span>`;
      div.onclick = () => { cmd.action(); close(); };
      results.appendChild(div);
    });
  };

  window.addEventListener('keydown', (e) => {
    if (e.key === 'k' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      isOpen ? close() : open();
    }
    
    if (!isOpen) return;

    if (e.key === 'Escape') close();
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % filteredCommands.length;
      render();
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + filteredCommands.length) % filteredCommands.length;
      render();
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      filteredCommands[selectedIndex]?.action();
      close();
    }
  });

  input.addEventListener('input', (e) => {
    const val = e.target.value.toLowerCase();
    filteredCommands = COMMANDS.filter(c => 
      c.name.toLowerCase().includes(val) || c.hint.toLowerCase().includes(val)
    );
    selectedIndex = 0;
    render();
  });
}
