# Trade Planner UI Design Choices & Architecture

> **Note on Scope**: Development is focused exclusively on the **Rust product** (`src/`, `Cargo.toml`, `ratatui` + `crossterm`).

## 1. Design Vision & Aesthetics
The goal of this redesign is to deliver a **professional, data-dense, minimalist financial tool** inspired by the Bloomberg Terminal.
We avoided generic styling in favor of:
- **Blocky Geometry**: Eliminating curved edges entirely. `border-radius` is set to `0px` globally (and single-line ASCII/Unicode box drawing in the terminal) to reinforce a structural, technical feel.
- **Monotone Palette**: Rather than using colorful elements, the UI relies strictly on shades of black, gray, silver, and crisp white. Statuses and emphasis are handled via contrast and crisp monochrome borders.
- **Typography & Grid**: Strict monospace typography (`JetBrains Mono`, `Courier New`, `Consolas`, and terminal fixed-width cells). This guarantees perfect column alignment for financial tables and execution transcripts.

## 2. Modular Layout Engine
To satisfy the requirement that users can "move parts of modules around," we implemented a dynamic **State-Driven Layout Engine**:
- **Component Separation**: The app is divided into six distinct modules: `CONFIG_PARAMS`, `PORTFOLIO_HOLDINGS`, `TARGET_GOALS`, `SEARCH_ENGINE`, `TRACE_AUDIT_LOG`, and `TELEMETRY_METRICS`.
- **Movement Controls**: Every module features header controls `[▲]`, `[▼]`, and `[⇄]`. These interact with `col1_modules` and `col2_modules` state lists, allowing users to shift modules vertically within their column or laterally across panes.
- **Layout Presets**: The command bar includes a `LAY<GO>` button/shortcut toggling between a 2-column split view and a stacked single-stream view.

## 3. Dark Mode, Theming & Live Visual Accents
A dynamic monotone theme system is supported:
- **Dark Mode**: Uses deep obsidian (`#0a0a0a` / RGB 10,10,10) for the background, graphite surfaces, and crisp white text. Borders are stark high-contrast lines (`#333333` / RGB 60,60,60).
- **Light Mode**: Inverts the palette to crisp whites and light grays (`#f0f0f0` / RGB 240,240,240), maintaining the strict monotone rule.
- **Theme Engine**: Centralized in `get_theme()` and switchable on the fly via `THEME<GO>` (`[F3]` / `[T]`).
- **Live Menu Timer**: A high-contrast elapsed session clock (`⏱ MM:SS` / `HH:MM:SS`) is rendered in the top-right corner of the menu bar alongside a pulsating `● LIVE` heartbeat indicator.
- **Subtle Flashing Highlights**: A non-distracting periodic pulse (approx. 500ms cycle) drives:
  - Active module border breathing between high-contrast focus and softened silver.
  - Active module header indicator (`[●]` / `[○]`).
  - Terminal cursor block `█` blinking in the status bar.
  - Interactive modal field pointers (`◀` / `◁`).

## 4. Multi-Entry Management & Scenario Presets
Users have flexible tools to populate and customize large portfolios and target bands:
- **Direct Interactive Modal Entry**: Press `[A]` (or `[P]` for Portfolio, `[G]` for Goals, `[E]` for Parameters) to open keyboard-navigable forms with `[Tab]`, `[Space]`, and `[Enter]`.
- **In-Place Row Editing**: Pressing `[Enter]` on any selected portfolio or goal row opens the editor prefilled with current quantities, prices, and target bands for fast updates.
- **Scenario Presets (`[F2]` / `[B]`)**: Instantly cycle through diverse multi-asset market scenarios:
  1. `TECH TRIO`: Classic AAPL / GOOG / MSFT rebalancing.
  2. `MAGNIFICENT SEVEN`: 7 Mega-Caps (NVDA, AAPL, MSFT, AMZN, GOOGL, META, TSLA).
  3. `DIVERSIFIED ETF`: Core asset allocation across SPY, QQQ, VTI, GLD, and BND.
  4. `HIGH VOLATILITY`: High-beta & crypto assets (BTC, ETH, SOL, COIN, MSTR).
  5. `DIVIDEND INCOME`: Stable cash-flow portfolio (JNJ, PG, KO, SCHD, O).

## 5. Comprehensive Search Algorithm Suite (`src/planner.rs`)
The search engine incorporates 7 distinct search algorithms:
1. **A\* Heuristic Search (`A*`)**: Evaluates $f(n) = g(n) + h(n)$ using an admissible trade-lot heuristic. Guaranteed optimal transaction steps.
2. **Greedy Best-First Search (`GBFS`)**: Evaluates $f(n) = h(n)$ to aggressively minimize allocation distance with minimal node expansions.
3. **Breadth-First Search (`BFS`)**: Unweighted level-by-level traversal guaranteeing the shortest sequence of trade actions.
4. **Uniform-Cost Search / Dijkstra (`UCS`)**: Expands lowest cumulative fee and transaction step cost $g(n)$ without heuristic bias.
5. **Beam Search (`BEAM`)**: Memory-bounded search retaining only top $K=50$ best candidate states per level for high-speed multi-asset planning.
6. **Depth-Limited Search (`DLS`)**: Depth-first exploration with a strict depth boundary ($D=12$) preventing unbounded search trees.
7. **Iterative Deepening Search (`IDDFS`)**: Combines depth-first memory efficiency with breadth-first optimality by iteratively increasing depth limits.

## 6. Non-Blocking Search Engine & "Running...." Animations
To guarantee that the UI thread never freezes even on large combinatorial portfolios or deep search spaces:
- **Multithreaded Worker Execution**: When a search is triggered (`[F5]` / `[R]`), `App::execute_plan` spawns a detached worker thread via `std::thread::spawn` and returns immediately. Communication occurs across a lock-free `std::sync::mpsc::channel`.
- **Zero-Freeze Responsive Event Loop**: The main terminal event loop continues drawing at full framerate, processing keystrokes, resizing windows, and updating timers without any lag or blocking.
- **Multi-Panel "Running...." Animation Suite**:
  - **Braille Spinner**: Real-time rotating spinner `["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]`.
  - **Dynamic Progress Dots**: Cycling `RUNNING.`, `RUNNING..`, `RUNNING...`, `RUNNING....`.
  - **Active Frontier Progress Bar**: Animated scanning bar (`[■■■■■░░░░░]`) rendered across the `SEARCH_ENGINE` and `TRACE_AUDIT_LOG` panels.
  - **Real-Time Execution Stopwatch**: Millisecond/second live timer showing elapsed computation time.
  - **Header & Status Bar Badges**: Dynamic `[F5] ⠋ RUNNING....` badge and animated status line.

## 7. Comparative Search Benchmarking ("Test All Searches")
Users can evaluate all 7 search algorithms simultaneously on the active portfolio scenario (`[F7]` / `[W]`):
- **Full Comparative Evaluation**: Executes `A*`, `GBFS`, `BFS`, `UCS`, `BEAM`, `DLS`, and `IDDFS` across background worker threads.
- **Fastest Algorithm Highlight**: Identifies the fastest search strategy (`★ FASTEST WINNER`) and highlights runtime, nodes expanded, transaction fees, and final cash.
- **Audit Panel Benchmark Table**: Renders an ASCII comparative breakdown table directly inside `TRACE_AUDIT_LOG`, followed by the fastest algorithm's complete step-by-step trade execution plan.

## 8. Rust Terminal Shortcuts
- `[F1]` / `[C]`: `CLR` (Clear all portfolio & goals)
- `[F2]` / `[B]` / `[D]`: `PRESET` (Cycle through multi-asset market scenarios)
- `[F3]` / `[T]`: `THEME` (Toggle Dark / Light Monotone Theme)
- `[F4]` / `[L]`: `LAY` (Toggle Split / Stacked Layout)
- `[F5]` / `[R]`: `RUN<GO>` (Execute active search planner non-blockingly)
- `[F6]` / `[O]` / `[K]`: `ALGO` (Cycle active search algorithm)
- `[F7]` / `[W]` / `[Z]`: `BENCH<GO>` (Test All Searches & find the fastest)
- `[A]` / `[Insert]` / `[N]`: Open Add Entry Modal Form
- `[P]` / `[G]` / `[E]`: Quick open Add Portfolio / Add Goal / Edit Params
- `[Enter]`: Edit selected row or execute planner
- `[X]`: Delete selected row
- `[M]`: `MOVE MODE` (Use `[U]` up, `[D]` down, `[S]` swap column)
- `[Tab]` / `[Shift+Tab]`: Cycle active module focus
- `[+]` / `[-]`: Adjust Cash ($500 increments)
- `[>]` / `[<]`: Adjust Fee ($1.00 increments)
- `[Q]` / `[Esc]`: Quit
