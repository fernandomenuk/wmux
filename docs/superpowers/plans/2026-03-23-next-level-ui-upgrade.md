# Next Level UI Upgrade Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal**: Transform `wmux-app` into a premium, adaptive developer tool with intelligent layouts and contextual controls.

**Architecture**: 
- **Core**: Update `wmux-core` to use `f64` proportional split ratios and `sysinfo` for process metrics.
- **Frontend**: Implement a "Living Focus" engine in `layout.js` using CSS Grid/Flex and transitions.
- **UI**: Add floating contextual bars, a command palette, and ghost borders for intuitive splitting.

**Tech Stack**: Rust (wmux-core, Tauri), JavaScript (xterm.js), CSS (Variables, Transitions, Grid), `sysinfo` crate.

---

### Task 1: Core - Proportional Split Ratios
**Files**:
- Modify: `crates/wmux-core/src/model/split_tree.rs`
- Modify: `crates/wmux-core/src/core.rs`
- Test: `crates/wmux-core/tests/split_tree_test.rs`

- [ ] **Step 1: Update `SplitNode` to store proportional ratios**
  Change `ratio` from fixed cell count to `f64` (0.0 to 1.0).
- [ ] **Step 2: Update layout calculation logic**
  Update `SplitNode::layout` to multiply total width/height by the proportional ratio.
- [ ] **Step 3: Update `split_at` and `set_ratio_at`**
  Ensure new splits default to `0.5` and manual overrides use `f64`.
- [ ] **Step 4: Verify with tests**
  Run: `cargo test -p wmux-core --test split_tree_test`
- [ ] **Step 5: Commit**
  `git add . && git commit -m "core: use proportional split ratios (f64)"`

### Task 2: Core - Process Monitoring
**Files**:
- Modify: `crates/wmux-core/Cargo.toml`
- Modify: `crates/wmux-core/src/terminal/pty.rs`
- Modify: `crates/wmux-core/src/core.rs`

- [ ] **Step 1: Add `sysinfo` dependency**
  Add `sysinfo = "0.30"` to `crates/wmux-core/Cargo.toml`.
- [ ] **Step 2: Capture Child PID in `Surface`**
  Update `Surface` struct and `spawn_pty` to store the OS PID of the shell process.
- [ ] **Step 3: Implement `get_process_metrics` in `WmuxCore`**
  Add a method to fetch CPU/Memory for a given surface ID using `sysinfo`.
- [ ] **Step 4: Verify metrics extraction**
  Run: `cargo run -p wmux-cli` and check if metrics can be queried (via temporary debug print).
- [ ] **Step 5: Commit**
  `git add . && git commit -m "core: add process monitoring via sysinfo"`

### Task 3: Tauri - Command API Updates
**Files**:
- Modify: `crates/wmux-app/src/main.rs`

- [ ] **Step 1: Update `get_layout` command**
  Ensure it returns proportional ratio data to the frontend.
- [ ] **Step 2: Add `get_process_metrics` command**
  Expose core's metrics to the frontend.
- [ ] **Step 3: Commit**
  `git add . && git commit -m "app: expose new core APIs to tauri frontend"`

### Task 4: Frontend - "Studio" Visual Language & Polish
**Files**:
- Modify: `crates/wmux-app/frontend/style.css`

- [ ] **Step 1: Define CSS Variables**
  Add `--bg-midnight`, `--accent-indigo`, `--text-muted`, and `--transition-smooth`.
- [ ] **Step 2: Implement Pane Transitions**
  Add `transition: all var(--transition-smooth)` to `.pane` and `.pane-border`.
- [ ] **Step 3: Add "Glass" styling and Indigo focus glow**
- [ ] **Step 4: Implement Visual Dimming**
  Set `opacity: 0.85; filter: grayscale(20%)` for unfocused panes.
- [ ] **Step 5: Add Indigo Pulse Animation**
  Define `@keyframes pulse-indigo` for background activity alerts.
- [ ] **Step 6: Commit**
  `git add . && git commit -m "ui: implement Studio visual language, dimming, and pulsing"`

### Task 5: Frontend - "Living Focus" Engine
**Files**:
- Modify: `crates/wmux-app/frontend/layout.js`
- Modify: `crates/wmux-app/frontend/terminal-manager.js`

- [ ] **Step 1: Implement Proportional Layout Logic**
  Update `refreshLayout` to use ratios from the backend instead of fixed cells.
- [ ] **Step 2: Add Auto-Expansion on Focus**
  When a pane is focused, update its split's ratio to `0.7` (if not in `manual_mode`).
- [ ] **Step 3: Implement Resize Deferral**
  Trigger `tm.resize()` ONLY after the 200ms transition ends.
- [ ] **Step 4: Commit**
  `git add . && git commit -m "ui: implement Living Focus engine with adaptive sizing"`

### Task 6: Frontend - Ghost Borders
**Files**:
- Modify: `crates/wmux-app/frontend/style.css`
- Modify: `crates/wmux-app/frontend/layout.js`

- [ ] **Step 1: Implement Ghost Border detection**
  Add mousemove listeners to `pane-area` to detect when the cursor is within 10px of a pane edge.
- [ ] **Step 2: Render Ghost Border Line**
  Inject a temporary absolute-positioned `div.ghost-border` when detected.
- [ ] **Step 3: Wire up "Split" on click**
  Trigger `invoke('split_surface')` based on the edge clicked.
- [ ] **Step 4: Commit**
  `git add . && git commit -m "ui: add ghost borders for intuitive splitting"`

### Task 7: Frontend - Contextual Action Bar
**Files**:
- Modify: `crates/wmux-app/frontend/index.html`
- Modify: `crates/wmux-app/frontend/style.css`
- Modify: `crates/wmux-app/frontend/layout.js`

- [ ] **Step 1: Create Action Bar UI**
  Absolute-positioned floating bar at the top-right of each pane.
- [ ] **Step 2: Add Split, Zoom, and Close icons**
- [ ] **Step 3: Bind actions to Tauri commands**
- [ ] **Step 4: Commit**
  `git add . && git commit -m "ui: add contextual action bars to panes"`

### Task 8: Frontend - Interactive Resizing & Previews
**Files**:
- Modify: `crates/wmux-app/frontend/layout.js`
- Modify: `crates/wmux-app/frontend/sidebar.js`

- [ ] **Step 1: Implement Resize Handles**
  Add a draggable `div.resize-handle` at split junctions.
- [ ] **Step 2: Implement Workspace Previews**
  Use `OffscreenCanvas` to capture terminal screenshots and show them on tab hover in `sidebar.js`.
- [ ] **Step 3: Commit**
  `git add . && git commit -m "ui: add interactive resize handles and workspace previews"`

### Task 9: Frontend - Intelligent Command Palette
**Files**:
- Create: `crates/wmux-app/frontend/command-palette.js`
- Modify: `crates/wmux-app/frontend/index.html`
- Modify: `crates/wmux-app/frontend/main.js`

- [ ] **Step 1: Create Palette UI**
  Floating centered modal with fuzzy search input.
- [ ] **Step 2: Implement Command Registry**
  List common actions (New Tab, Split, etc.).
- [ ] **Step 3: Wire up `Ctrl+K` listener**
- [ ] **Step 4: Commit**
  `git add . && git commit -m "ui: add intelligent command palette (Ctrl+K)"`

### Task 10: Frontend - Adaptive Status Bar
**Files**:
- Modify: `crates/wmux-app/frontend/layout.js`
- Modify: `crates/wmux-app/frontend/style.css`

- [ ] **Step 1: Implement Metrics Polling**
  Call `get_process_metrics` every 2 seconds for the focused pane.
- [ ] **Step 2: Update Status Bar UI**
  Show CPU/MEM chips in the status bar.
- [ ] **Step 3: Add Contextual Help Hints**
- [ ] **Step 4: Commit**
  `git add . && git commit -m "ui: update status bar with live process metrics"`

### Task 11: Final Polish & Verification
- [ ] **Step 1: Verify all interactions**
  Test splitting, zooming, focusing, and the command palette.
- [ ] **Step 2: Check for resize jitter**
- [ ] **Step 3: Run full project tests**
  `cargo test`
- [ ] **Step 4: Final Commit**
  `git commit --allow-empty -m "chore: finalize Next Level UI upgrade"`
