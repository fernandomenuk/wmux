import { toggleCommandPalette } from './command-palette.js';

const { invoke } = window.__TAURI__.core;

let prefixMode = false;

export function attachKeybindings(term, getFocusedSurfaceId, getCurrentTabIndex) {
  term.attachCustomKeyEventHandler((event) => {
    if (event.type !== 'keydown') return true;

    // Ctrl+Shift+K → command palette
    if (event.ctrlKey && event.shiftKey && event.key === 'K') {
      toggleCommandPalette();
      return false;
    }

    if (!prefixMode && event.ctrlKey && event.key === 'a') {
      prefixMode = true;
      return false;
    }

    if (prefixMode) {
      prefixMode = false;

      if (event.ctrlKey && event.key === 'a') {
        const surfaceId = getFocusedSurfaceId();
        if (surfaceId) invoke('send_input', { surfaceId, data: '\x01' });
        return false;
      }

      const surfaceId = getFocusedSurfaceId();

      switch (event.key) {
        case '|':
        case '\\':
          invoke('split_pane', { direction: 'vertical' });
          break;
        case '-':
          invoke('split_pane', { direction: 'horizontal' });
          break;
        case 'x':
          if (surfaceId) {
            invoke('close_pane', { surfaceId }).then(r => {
              if (r?.should_quit) window.__TAURI__.window.getCurrentWindow().close();
            });
          }
          break;
        case 'z':
          invoke('toggle_zoom');
          break;
        case 'c':
          invoke('create_workspace', { name: null });
          break;
        case 'n':
          invoke('next_workspace');
          break;
        case 'p':
          invoke('prev_workspace');
          break;
        case 'q':
          window.__TAURI__.window.getCurrentWindow().close();
          break;
        case 'ArrowUp':
          invoke('focus_direction', { direction: 'up' });
          break;
        case 'ArrowDown':
          invoke('focus_direction', { direction: 'down' });
          break;
        case 'ArrowLeft':
          invoke('focus_direction', { direction: 'left' });
          break;
        case 'ArrowRight':
          invoke('focus_direction', { direction: 'right' });
          break;
        default:
          if (event.key >= '1' && event.key <= '9') {
            invoke('switch_workspace', { index: parseInt(event.key) - 1 });
          }
          break;
      }
      return false;
    }

    return true;
  });
}
