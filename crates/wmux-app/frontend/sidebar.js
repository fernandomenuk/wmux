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

  newWsBtn.addEventListener('click', () => {
    invoke('create_workspace', { name: null });
  });

  newTabBtn.addEventListener('click', () => {
    invoke('create_workspace', { name: null });
  });
}

export async function refreshTabs() {
  const result = await invoke('get_tab_info');
  const tabsEl = document.getElementById('tabs');
  const wsList = document.getElementById('workspace-list');

  // Render tabs
  tabsEl.innerHTML = '';
  result.tabs.forEach((tab, idx) => {
    const el = document.createElement('div');
    el.className = 'tab' + (tab.is_active ? ' active' : '');
    el.textContent = `${idx + 1}:${tab.name}`;
    el.addEventListener('click', () => invoke('switch_workspace', { index: idx }));
    tabsEl.appendChild(el);
  });

  // Render sidebar workspace list
  wsList.innerHTML = '';
  result.tabs.forEach((tab, idx) => {
    const el = document.createElement('div');
    el.className = 'ws-item' + (tab.is_active ? ' active' : '');
    el.textContent = `${idx + 1}: ${tab.name}`;
    el.addEventListener('click', () => invoke('switch_workspace', { index: idx }));
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
