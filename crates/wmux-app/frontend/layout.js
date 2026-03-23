import * as tm from './terminal-manager.js';

const { invoke } = window.__TAURI__.core;

let resizeTimeout = null;

export async function refreshLayout() {
  const paneArea = document.getElementById('pane-area');
  const rect = paneArea.getBoundingClientRect();

  // Get cell dimensions to convert pixels to cells
  const { cellWidth, cellHeight } = tm.getCellDimensions();
  const widthCells = Math.floor(rect.width / cellWidth) || 80;
  const heightCells = Math.floor(rect.height / cellHeight) || 24;

  const layout = await invoke('get_layout', { width: widthCells, height: heightCells });

  tm.applyLayout(layout.panes, widthCells, heightCells);

  // Resize PTYs to match fitted terminal dimensions
  for (const pane of layout.panes) {
    const entry = tm.getTerminal(pane.surface_id);
    if (entry) {
      invoke('resize_terminal', {
        surfaceId: pane.surface_id,
        cols: entry.term.cols,
        rows: entry.term.rows,
      });
    }
  }

  // Update status bar
  const tabInfo = await invoke('get_tab_info');
  const statusShell = document.getElementById('status-shell');
  const statusWorkspace = document.getElementById('status-workspace');
  const statusPane = document.getElementById('status-pane');

  statusShell.textContent = layout.shell;
  if (tabInfo.tabs[tabInfo.active_index]) {
    statusWorkspace.textContent = tabInfo.tabs[tabInfo.active_index].name;
  }
  const paneIdx = layout.panes.findIndex(p => p.is_focused);
  statusPane.textContent = `pane ${paneIdx + 1}/${layout.panes.length}`;
}

export function setupResizeHandler() {
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => refreshLayout(), 100);
  });
}
