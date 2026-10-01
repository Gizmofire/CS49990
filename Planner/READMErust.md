# Trade Planner (Rust Edition) — Technical Manual & Architecture

> **High-Performance Terminal Portfolio Rebalancing Engine**  
> Built with Rust, Ratatui, Crossterm, and Monotone Bloomberg-inspired Aesthetics.

---

## 1. Overview & Core Purpose

`trade_planner` is a terminal-based financial portfolio rebalancing utility written in Rust. It solves the discrete asset allocation problem by searching through combinations of buy and sell transactions to transition an existing portfolio of stocks/assets into a target goal band while minimizing transaction fees and respecting cash constraints.

The application features:
- **Zero-lag Asynchronous Search**: Searches run on background worker threads with lock-free message passing, keeping the UI thread 100% interactive.
- **7 Classical & Modern Search Algorithms**: `A*`, `GBFS`, `BFS`, `UCS`, `Beam Search`, `DLS`, and `IDDFS`.
- **Bloomberg Terminal Aesthetics**: Monotone blocky geometry (`0px` border radius, ASCII/Unicode box-drawing, zero rounded edges).
- **Interactive Multi-Entry Management**: Full modal input forms for adding/editing portfolio holdings, target goal bands, cash, and fees.
- **Dynamic Modular Layout**: Move and rearrange modules across columns or switch between 2-column split and single-stream stacked layouts.
- **Pre-configured Scenario Presets**: 5 market scenarios across tech mega-caps, diversified ETFs, high-volatility crypto/beta, and dividend equities.

---

## 2. Directory & Component Breakdown

```
CS49990/
├── Cargo.toml            # Project dependencies and package metadata
├── Design.md             # Visual design choices and terminal architecture notes
├── READMErust.md         # This technical reference manual
└── src/
    ├── main.rs           # Terminal lifecycle, raw mode setup, and event loop
    ├── app.rs            # Application state machine, forms, presets, and async engine
    ├── planner.rs        # Algorithmic search engine (7 algorithms) and state evaluation
    └── ui.rs             # Ratatui rendering engine, monotone theming, and animations
```

---

## 3. Detailed Component Reference

### A. `src/main.rs` — Terminal Lifecycle & Event Dispatcher
- **Terminal Setup & Teardown**:
  - Initializes `crossterm` in raw mode (`enable_raw_mode()`).
  - Enters the alternate screen buffer (`EnterAlternateScreen`) and provisions the `Ratatui` backend.
  - Ensures clean exit restoration (`disable_raw_mode()`, `LeaveAlternateScreen`, `show_cursor()`) on `[Q]` or `[Ctrl+C]`.
- **Interactive Event Loop (`run_app`)**:
  - Runs at a 100ms poll interval (`event::poll(Duration::from_millis(100))`).
  - Calls `app.on_tick()` on every cycle to advance animations and poll async worker threads.
  - Dispatches keyboard events between **Modal Form Input Mode** (when a dialog is open) and **Dashboard Mode**.

---

### B. `src/app.rs` — State Machine & Async Orchestration
- **Data Models**:
  - `ModuleId`: Enumeration of the 6 workspace panels (`Params`, `Portfolio`, `Goals`, `Telemetry`, `Engine`, `Audit`).
  - `LayoutMode`: `Split` (2-column 50/50 layout) or `Stacked` (full-width single column).
  - `ModalMode`: `None`, `AddPortfolio`, `AddGoal`, `EditParams`.
  - `FormState`: Interactive form field tracker, handling text input, backspace, numeric parsing, validation, and field cycling via `[Tab]`.
  - `PresetScenario`: 5 preloaded market scenarios (`TechTrio`, `MegaCapSeven`, `DiversifiedEtf`, `CryptoVol`, `DividendIncome`).
- **Core State (`App` struct)**:
  - `is_dark`: Boolean toggling Monotone Dark and Monotone Light themes.
  - `col1_modules` & `col2_modules`: Dynamic vectors storing panel order in each column.
  - `move_mode`: Boolean enabling panel rearrangement with `[U]` (up), `[D]` (down), and `[S]` (swap column).
  - `start_time` & `tick_count`: Live stopwatch and pulse timing engine.
  - `cash` & `fee`: Available liquid capital and per-trade commission fee.
  - `portfolio` & `goals`: Lists of active holdings (`PortfolioItem`) and target bands (`GoalItem`).
  - `selected_port_idx` & `selected_goal_idx`: Selection cursors for in-place row editing (`[Enter]`) or deletion (`[X]`).
- **Asynchronous Search Worker**:
  - `is_searching`: Boolean indicator that calculation is currently executing in background.
  - `search_rx`: Standard library `Receiver<PlanResult>` channel endpoint.
  - `execute_plan()`: Spawns a detached background thread via `std::thread::spawn` and returns immediately without blocking UI rendering.
  - `poll_search_result()`: Called on each tick to non-blockingly check `search_rx.try_recv()` and update results upon completion.
- **Animation Helpers**:
  - `spinner_frame()`: Rotates through 10-frame Braille spinner (`⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏`).
  - `running_dots_str()`: Cycles dynamic progress ellipsis (`.`, `..`, `...`, `....`).
  - `running_progress_bar()`: Generates animated scanning UTF-8 progress bar (`[■■■■■░░░░░]`).
  - `search_elapsed_str()`: Live stopwatch tracking calculation execution time.

---

### C. `src/planner.rs` — Algorithmic Search Engine
Solves the state-space rebalancing problem where each state is represented by $(C, [q_1, q_2, \dots, q_k])$:
- **`SearchAlgorithm` Implementations**:
  1. **`A*` (A-Star Search)**: Evaluates $f(n) = g(n) + h(n)$. Uses an admissible optimistic fee heuristic based on remaining lot discrepancies. Guarantees optimal trade sequence.
  2. **`GBFS` (Greedy Best-First Search)**: Evaluates $f(n) = h(n)$. Aggressively expands states that closest match target allocations.
  3. **`BFS` (Breadth-First Search)**: Explores state space level-by-level with a FIFO queue, guaranteeing minimum number of trade actions.
  4. **`UCS` (Uniform-Cost Search / Dijkstra)**: Priority queue ordering by cumulative path cost $g(n) = \text{total fees} + \text{steps}$, ignoring heuristics.
  5. **`BEAM` (Beam Search, $K=50$)**: Limits memory and combinatorial explosion by keeping only the top 50 candidate states at each search level.
  6. **`DLS` (Depth-Limited Search, $D=12$)**: Strict depth-bounded DFS preventing runaway paths while searching deep combinations.
  7. **`IDDFS` (Iterative Deepening Search)**: Combines DFS memory efficiency ($O(bd)$) with BFS optimality by iteratively increasing depth limit ($D=1 \dots 14$).
- **State Transition Logic**:
  - Generates valid discrete `SELL` actions (respecting current inventory and `sell_only` flags).
  - Generates valid discrete `BUY` actions (respecting liquid cash $C \ge \text{price} + \text{fee}$).
  - Enforces finite lot trade sizes to ensure rapid state convergence.

---

### D. `src/ui.rs` — Monotone Bloomberg Interface & Theming
- **Monotone Theming System (`ThemeColors` & `get_theme`)**:
  - **Monotone Dark**: Deep obsidian background (`#0a0a0a`), graphite containers (`#121212`), high-contrast silver borders (`#3c3c3c` / `#dcdcdc`), and crisp white typography.
  - **Monotone Light**: Clean paper background (`#f0f0f0`), white panels (`#ffffff`), and dark charcoal typography.
  - **Subtle Flashing Highlights**: Smooth periodic breathing highlights on active borders, live heartbeat indicator (`● LIVE`), and blinking terminal cursor (`█`).
- **Render Modules**:
  - `render_header()`: Displays Bloomberg command buttons, active module badge, live session timer, and dynamic `[F5] ⠋ RUNNING....` badge.
  - `render_params_content()`: Cash, transaction fee, and quick adjustment controls.
  - `render_portfolio_content()`: Holdings table with ticker, quantity, market price, total value, and `SELL_ONLY` flags.
  - `render_goals_content()`: Target goal table with min/max target quantities and target valuation bands.
  - `render_telemetry_content()`: Total portfolio value, cash reserve ratio, and asset diversity metrics.
  - `render_engine_content()`: Algorithm information or live **"running...." progress bar** and stopwatch when executing.
  - `render_audit_content()`: Step-by-step transaction logs, fees, and final cash summary or animated node evaluation transcript.
  - `render_status_bar()`: Keybinding legend and animated status line.
  - `render_modal()`: Centered dialog box for adding or editing positions and parameters.

---

## 4. Keyboard Shortcuts & Controls

| Shortcut | Bloomberg Code | Action Description |
|---|---|---|
| `[F5]` or `[R]` | `RUN<GO>` | Execute trade planner search engine asynchronously |
| `[F6]` or `[O]` / `[K]` | `ALGO<GO>` | Cycle search algorithm (`A*` → `GBFS` → `BFS` → `UCS` → `BEAM` → `DLS` → `IDDFS`) |
| `[F7]` or `[W]` / `[Z]` | `BENCH<GO>` | Test All Searches & benchmark fastest algorithm with full comparative metrics |
| `[F2]` or `[B]` / `[D]` | `PRESET<GO>` | Cycle preset market scenarios (Tech Trio, Mag 7, ETFs, Volatile, Dividends) |
| `[A]` or `[Insert]` / `[N]` | `ADD<GO>` | Open Add Entry modal form |
| `[P]` | `PORT<GO>` | Quick-open Add Portfolio Position modal |
| `[G]` | `GOAL<GO>` | Quick-open Add Target Goal modal |
| `[E]` | `PARM<GO>` | Quick-open Edit Cash & Fee modal |
| `[Enter]` | `EDIT<GO>` | Edit currently selected portfolio position or target goal row |
| `[X]` or `[Delete]` | `DEL<GO>` | Delete currently selected portfolio or goal row |
| `[F1]` or `[C]` | `CLR<GO>` | Clear all portfolio positions and target goals |
| `[F3]` or `[T]` | `THEME<GO>` | Toggle between Monotone Dark and Monotone Light themes |
| `[F4]` or `[L]` | `LAY<GO>` | Toggle between 2-Column Split view and Stacked view |
| `[M]` | `MOVE<GO>` | Toggle Move Mode (`[U]` Up, `[D]` Down, `[S]` Swap column) |
| `[Tab]` / `[Shift+Tab]` | `NEXT<GO>` | Cycle active panel focus across workspace |
| `[↑]` / `[↓]` | `NAV` | Navigate rows in Portfolio or Goals table |
| `[+]` / `[-]` | `CASH±` | Adjust liquid cash by ±$500 |
| `[>]` / `[<]` | `FEE±` | Adjust transaction fee by ±$1.00 |
| `[Q]` or `[Esc]` | `QUIT` | Exit application |

---

## 5. Building, Running, and Testing

### Prerequisites
- Rust 1.70+ (`cargo` and `rustc`)

### Build and Run
```bash
# Run in debug mode
cargo run

# Run with maximum optimization
cargo run --release
```

### Running the Test Suite
The project contains unit tests covering all search algorithms, async worker execution, channel polling, animation helpers, benchmark evaluation, and preset scenarios:
```bash
cargo test
```
All 13 unit tests execute in under 0.05s.
