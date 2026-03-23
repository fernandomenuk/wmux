# Design Spec: Next Level UI Upgrade (wmux)

**Date**: 2026-03-23
**Status**: Draft
**Topic**: Adaptive, Contextual, and Intelligent UI for `wmux-app`

## 1. Vision
Transform `wmux-app` from a basic terminal multiplexer into a premium, "living" developer tool. The UI should be so intuitive that "anyone can easily work" without memorizing complex hotkeys, while providing "intelligence" that anticipates user needs through adaptive layouts and contextual controls.

## 2. Visual Language ("The Studio Look")
- **Color Palette**: 
  - Background: `#0a0a0c` (Deep Midnight)
  - Focused Accent: `#6366f1` (Electric Indigo)
  - Secondary/Muted: `#71717a` (Muted Slate)
  - Borders: `rgba(255,255,255,0.08)` with a 2px inner glow for focused panes.
- **Typography**:
  - UI: `Inter` (Variable)
  - Terminal: `Geist Mono`
- **Animations**: 200ms `cubic-bezier(0.4, 0, 0.2, 1)` transitions for all state changes (focus, split, resize, tab switch). 
  - *Technical Note*: To prevent PTY resize jitter during animations, actual terminal resizes will trigger only at the *end* of the transition, while the `xterm.js` container uses CSS transforms/scaling for visual continuity during the move.

## 3. The "Living Focus" Engine (Adaptive Sizing)
The UI automatically optimizes pane dimensions based on user focus:
- **Dynamic Weighting**: The focused pane automatically expands to claim **65-75%** of the available split axis. Unfocused panes shrink to a "Glance" state.
- **Manual Override**: If a user manually resizes a split using the center handle (see Section 4), the "Living Focus" engine is temporarily disabled for that workspace to honor user intent, until the workspace is reset or all panes are closed.
- **Visual Dimming**: Unfocused panes drop to **85% opacity** and slight grayscale to reduce visual noise.
- **Auto-Scroll Protection**: Background panes that receive high-velocity output (e.g., build errors) will "pulse" their border indigo to alert the user without stealing focus.

## 4. Assistive Visual Controls (Zero Learning Curve)
- **Ghost Borders**: Hovering within 10px of a pane edge reveals a thin `Electric Indigo` line. Clicking this line splits the pane in that direction.
- **Contextual Action Bar**: A floating "Glass" bar at the top-right of the focused pane with:
  - `Split Vertical` / `Split Horizontal`
  - `Toggle Zoom` (Fullscreen)
  - `Close Pane`
- **Interactive Resizing**: A center "Handle" on every split for manual dragging with a ghost-layout preview. Dragging a handle sets a `manual_layout` flag on the workspace.

## 5. Intelligent Command Palette (`Ctrl+K`)
A floating, fuzzy-searchable command center:
- **Fuzzy Discovery**: Suggestions like "New Workspace", "Split Vertical", "Select PowerShell".
- **Contextual Actions**: If a process is running, suggest "Clear Output", "Restart Shell", or "Open in New Pane".
- **Learning Mode**: Shows "Popular Actions" for new users to aid discoverability.

## 6. Adaptive Status Bar
- **Process Intelligence**: Shows CPU/Memory usage of the specific process in the focused pane (e.g., `node.exe - 4% CPU`).
- **Contextual Help**: Dynamic shortcut reminders (e.g., `ESC to exit zoom`).
- **Workspace Previews**: Hovering over workspace tabs shows a mini-thumbnail of the layout. 
  - *Performance Note*: Thumbnails are generated using a low-overhead `OffscreenCanvas` render of the terminal screens and cached in the frontend to avoid expensive re-renders.

## 7. Technical Implementation Strategy
- **Frontend**: Custom CSS variables for theme and transition control. CSS Grid/Flexbox for the dynamic layout engine.
- **Backend (Tauri/Rust)**: 
  - Enhance `WmuxCore` to support proportional split ratios (ratios between 0.0 and 1.0) instead of fixed cell counts.
  - Implement `sysinfo` integration in `wmux-core` to fetch per-PTY process metrics.
  - State Synchronization: Ensure `WmuxCore` state broadcasts proportional layout updates that the frontend can animate smoothly.

## 8. Edge Case Handling
- **Ultra-wide Monitors**: Limit auto-expansion to a maximum width to prevent overly wide terminals.
- **Multiple Panes (3+)**: The "Living Focus" engine will intelligently distribute the remaining space among unfocused panes.
- **High-Latency Shells**: Ensure UI interactions (splitting/focusing) are independent of PTY readiness to maintain a "snappy" feel.
