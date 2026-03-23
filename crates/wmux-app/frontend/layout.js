import * as tm from './terminal-manager.js';

const { invoke } = window.__TAURI__.core;

let resizeTimeout = null;
let refreshInFlight = null;
let refreshQueued = false;
let refreshGeneration = 0;
let metricsInterval = null;

// The "Living Focus" Engine
// To prevent resize jitter, we defer actual PTY resize until after CSS transitions
async function runRefresh(generation) {
  const paneArea = document.getElementById('pane-area');
  const rect = paneArea.getBoundingClientRect();

  if (!rect.width || !rect.height) return;

  const { cellWidth, cellHeight } = tm.getCellDimensions();
  const widthCells = Math.floor(rect.width / cellWidth) || 80;
  const heightCells = Math.floor(rect.height / cellHeight) || 24;

  const layout = await invoke('get_layout', { width: widthCells, height: heightCells });
  if (generation !== refreshGeneration) return;

  // Apply layout visually (CSS positions/sizes)
  await tm.applyLayout(layout.panes, widthCells, heightCells, layout.surface_ids);
  if (generation !== refreshGeneration) return;

  // Defer PTY resize to end of transition (200ms)
  setTimeout(async () => {
    if (generation !== refreshGeneration) return;
    await Promise.all(layout.panes.map((pane) => {
      const entry = tm.getTerminal(pane.surface_id);
      if (entry) {
        return invoke('resize_terminal', {
          surfaceId: pane.surface_id,
          cols: entry.term.cols,
          rows: entry.term.rows,
        });
      }
      return Promise.resolve();
    }));
  }, 210);

  // Update status bar
  const tabInfo = await invoke('get_tab_info');
  const statusShell = document.getElementById('status-shell');
  const statusWorkspace = document.getElementById('status-workspace');
  const statusPane = document.getElementById('status-pane');

  statusShell.textContent = layout.shell;
  statusWorkspace.textContent = tabInfo.tabs[tabInfo.active_index]?.name || '';
  const paneIdx = layout.panes.findIndex(p => p.is_focused);
  statusPane.textContent = layout.panes.length ? `pane ${paneIdx + 1}/${layout.panes.length}` : '0/0';
  
  startMetricsPolling();
}

function startMetricsPolling() {
  if (metricsInterval) clearInterval(metricsInterval);
  metricsInterval = setInterval(async () => {
    const focusedId = tm.getFocusedId();
    if (!focusedId) return;

    try {
      const metrics = await invoke('get_process_metrics', { surfaceId: focusedId });
      const cpuVal = document.getElementById('status-cpu');
      const memVal = document.getElementById('status-mem');
      if (cpuVal) cpuVal.textContent = `${metrics.cpu.toFixed(1)}%`;
      if (memVal) memVal.textContent = `${(metrics.memory / 1024 / 1024).toFixed(0)}MB`;
    } catch (e) {
      // Ignore if process not found
    }
  }, 2000);
}

export function refreshLayout() {
  refreshGeneration += 1;
  const generation = refreshGeneration;

  if (refreshInFlight) {
    refreshQueued = true;
    return refreshInFlight;
  }

  refreshInFlight = runRefresh(generation)
    .catch(console.error)
    .finally(() => {
      refreshInFlight = null;
      if (refreshQueued) {
        refreshQueued = false;
        refreshLayout();
      }
    });

  return refreshInFlight;
}

export function setupResizeHandler() {
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => refreshLayout(), 100);
  });

  setupGhostBorders();
}

function setupGhostBorders() {
  const area = document.getElementById('pane-area');
  const ghost = document.createElement('div');
  ghost.className = 'ghost-border';
  area.appendChild(ghost);

  let activeEdge = null;

  area.addEventListener('mousemove', (e) => {
    const rect = area.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    
    const edgeSize = 12;
    activeEdge = null;
    ghost.style.display = 'none';

    if (x < edgeSize) activeEdge = { dir: 'vertical', pos: 'left' };
    else if (x > rect.width - edgeSize) activeEdge = { dir: 'vertical', pos: 'right' };
    else if (y < edgeSize) activeEdge = { dir: 'horizontal', pos: 'top' };
    else if (y > rect.height - edgeSize) activeEdge = { dir: 'horizontal', pos: 'bottom' };

    if (activeEdge) {
      ghost.style.display = 'block';
      ghost.className = `ghost-border ${activeEdge.dir === 'vertical' ? 'v' : 'h'}`;
      if (activeEdge.pos === 'left') { ghost.style.left = '0'; ghost.style.top = '0'; }
      else if (activeEdge.pos === 'right') { ghost.style.left = `${rect.width - 3}px`; ghost.style.top = '0'; }
      else if (activeEdge.pos === 'top') { ghost.style.top = '0'; ghost.style.left = '0'; }
      else if (activeEdge.pos === 'bottom') { ghost.style.top = `${rect.height - 3}px`; ghost.style.left = '0'; }
    }
  });

  area.addEventListener('click', () => {
    if (activeEdge) {
      invoke('split_pane', { direction: activeEdge.dir });
    }
  });
}
