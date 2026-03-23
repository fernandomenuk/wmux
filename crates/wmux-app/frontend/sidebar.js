import { showInputModal } from './modal.js';
const { invoke } = window.__TAURI__.core;

let sidebarVisible = true;

export function setupSidebar() {
  const toggle = document.getElementById('sidebar-toggle');
  const sidebar = document.getElementById('sidebar');
  const newWsBtn = document.getElementById('sidebar-new-ws');
  const newTabBtn = document.getElementById('new-tab');

  toggle.addEventListener('click', () => {
    sidebarVisible = !sidebarVisible;
    sidebar.classList.toggle('hidden', !sidebarVisible);
  });

  newWsBtn.addEventListener('click', async () => {
    const name = await showInputModal('Enter workspace name:');
    if (name !== null) {
      invoke('create_workspace', { name: name || null });
    }
  });

  newTabBtn.addEventListener('click', async () => {
    const name = await showInputModal('Enter workspace name:');
    if (name !== null) {
      invoke('create_workspace', { name: name || null });
    }
  });
}

export async function refreshTabs() {
  const result = await invoke('get_tab_info');
  const tabsEl = document.getElementById('tabs');
  const wsList = document.getElementById('workspace-list');

  const onRename = async (idx, currentName) => {
    const newName = await showInputModal('Rename workspace:', currentName);
    if (newName !== null && newName !== '') {
      await invoke('rename_workspace', { index: idx, name: newName });
    }
  };

  const onClose = async (idx) => {
    const result = await invoke('close_workspace', { index: idx });
    if (result.should_quit) {
      window.close();
    }
  };

  // Render tabs
  tabsEl.innerHTML = '';
  result.tabs.forEach((tab, idx) => {
    const el = document.createElement('div');
    el.className = 'tab' + (tab.is_active ? ' active' : '');
    
    const nameSpan = document.createElement('span');
    nameSpan.textContent = `${idx + 1}:${tab.name}`;
    el.appendChild(nameSpan);

    const closeBtn = document.createElement('button');
    closeBtn.className = 'tab-close';
    closeBtn.innerHTML = '&times;';
    closeBtn.onclick = (e) => {
      e.stopPropagation();
      onClose(idx);
    };
    el.appendChild(closeBtn);

    el.addEventListener('click', () => invoke('switch_workspace', { index: idx }));
    el.addEventListener('auxclick', (e) => {
      if (e.button === 1) { // Middle click
        e.preventDefault();
        onClose(idx);
      }
    });
    el.addEventListener('dblclick', (e) => {
      e.stopPropagation();
      onRename(idx, tab.name);
    });
    tabsEl.appendChild(el);
  });

  // Render sidebar workspace list
  wsList.innerHTML = '';
  result.tabs.forEach((tab, idx) => {
    const el = document.createElement('div');
    el.className = 'ws-item' + (tab.is_active ? ' active' : '');
    
    const nameSpan = document.createElement('span');
    nameSpan.textContent = `${idx + 1}: ${tab.name}`;
    el.appendChild(nameSpan);

    const closeBtn = document.createElement('button');
    closeBtn.className = 'ws-close';
    closeBtn.innerHTML = '&times;';
    closeBtn.onclick = (e) => {
      e.stopPropagation();
      onClose(idx);
    };
    el.appendChild(closeBtn);

    el.addEventListener('click', () => invoke('switch_workspace', { index: idx }));
    el.addEventListener('auxclick', (e) => {
      if (e.button === 1) {
        e.preventDefault();
        onClose(idx);
      }
    });
    el.addEventListener('dblclick', (e) => {
      e.stopPropagation();
      onRename(idx, tab.name);
    });
    wsList.appendChild(el);
  });

  return result;
}

export function getActiveIndex() {
  // Read from DOM
  const active = document.querySelector('.tab.active');
  if (!active) return 0;
  const tabs = [...document.querySelectorAll('.tab')];
  return tabs.indexOf(active);
}
