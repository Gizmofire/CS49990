# Bloomberg-Style Modular Monotone Trade Planner

An automated A* portfolio rebalancing planner built with a blocky, modular financial terminal design inspired by Bloomberg terminals. Available in both **Python (Mesop Web)** and **Rust (Ratatui TUI)**.

---

## 🦀 Running the Rust Terminal Version

The Rust application runs natively in your terminal with fast rendering, full keyboard shortcuts, modular panel movement, and dark/light monotone themes.

### Build and Run:
```bash
cargo run --release
```

### Run Tests:
```bash
cargo test
```

### Shortcuts & Controls:
- **`[F1]` or `[C]`**: `CLR<GO>` - Clear portfolio and goals
- **`[F2]` or `[D]`**: `DEMO<GO>` - Load demo scenario
- **`[F3]` or `[T]`**: `THEME<GO>` - Toggle Dark / Light monotone theme
- **`[F4]` or `[L]`**: `LAY<GO>` - Toggle 2-column Split / Stacked layout
- **`[F5]`, `[R]`, or `[Enter]`**: `RUN<GO>` - Execute A* optimal trade search
- **`[M]`**: `MOVE MODE` - Reorder active module (`[U]` Up, `[D]` Down, `[S]` Swap column)
- **`[Tab]` / `[Shift+Tab]`**: Cycle focused module
- **`[X]`**: Delete selected row from active module
- **`[+]` / `[-]`**: Adjust cash ($500 increments)
- **`[>]` / `[<]`**: Adjust transaction fee ($1.00 increments)
- **`[Q]` or `[Esc]`**: Quit

---

## 🐍 Running the Python (Mesop Web) Version

### Start the Mesop Server:
```bash
.venv/bin/mesop trade_planner.py
```
Open [http://localhost:32123](http://localhost:32123) in your browser.

---

## 📐 Design Documentation
See [Design.md](file:///home/gizmo/Downloads/School/CS49990/Design.md) for full design philosophy, color tokens, and modular layout documentation.
