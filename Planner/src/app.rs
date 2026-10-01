use crate::planner::{
    benchmark_all_algorithms, calculate_plan, BenchmarkResult, GoalItem, PlanResult,
    PortfolioItem, SearchAlgorithm,
};

#[derive(Debug, Clone)]
pub enum SearchWorkerMessage {
    Single(PlanResult),
    Benchmark(BenchmarkResult),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleId {
    Params,
    Portfolio,
    Goals,
    Telemetry,
    Engine,
    Audit,
}

impl ModuleId {
    pub fn title(&self) -> &'static str {
        match self {
            ModuleId::Params => "CONFIG_PARAMS",
            ModuleId::Portfolio => "PORTFOLIO_HOLDINGS",
            ModuleId::Goals => "TARGET_GOALS",
            ModuleId::Telemetry => "TELEMETRY_METRICS",
            ModuleId::Engine => "SEARCH_ENGINE",
            ModuleId::Audit => "TRACE_AUDIT_LOG",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            ModuleId::Params => "PARM",
            ModuleId::Portfolio => "PORT",
            ModuleId::Goals => "GOAL",
            ModuleId::Telemetry => "TELEM",
            ModuleId::Engine => "ENG",
            ModuleId::Audit => "AUDIT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Split,
    Stacked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalMode {
    None,
    AddPortfolio,
    AddGoal,
    EditParams,
}

#[derive(Debug, Clone, Default)]
pub struct FormState {
    pub symbol: String,
    pub qty: String,
    pub price: String,
    pub sell_only: bool,
    pub min_qty: String,
    pub max_qty: String,
    pub cash: String,
    pub fee: String,
    pub field_idx: usize,
    pub is_editing: bool,
    pub editing_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetScenario {
    TechTrio,
    MegaCapSeven,
    DiversifiedEtf,
    CryptoVol,
    DividendIncome,
}

impl PresetScenario {
    pub fn name(&self) -> &'static str {
        match self {
            PresetScenario::TechTrio => "TECH TRIO (AAPL / GOOG / MSFT)",
            PresetScenario::MegaCapSeven => "MAGNIFICENT SEVEN (7 MEGA-CAPS)",
            PresetScenario::DiversifiedEtf => "DIVERSIFIED ETF (SPY / QQQ / VTI / GLD)",
            PresetScenario::CryptoVol => "HIGH VOLATILITY (BTC / ETH / SOL / COIN)",
            PresetScenario::DividendIncome => "DIVIDEND INCOME (JNJ / PG / SCHD / O)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            PresetScenario::TechTrio => PresetScenario::MegaCapSeven,
            PresetScenario::MegaCapSeven => PresetScenario::DiversifiedEtf,
            PresetScenario::DiversifiedEtf => PresetScenario::CryptoVol,
            PresetScenario::CryptoVol => PresetScenario::DividendIncome,
            PresetScenario::DividendIncome => PresetScenario::TechTrio,
        }
    }
}

pub struct App {
    pub is_dark: bool,
    pub layout_mode: LayoutMode,
    pub col1_modules: Vec<ModuleId>,
    pub col2_modules: Vec<ModuleId>,
    pub active_module_idx: usize,
    pub move_mode: bool,

    // Timer & Tick State for subtle flashing highlights
    pub start_time: std::time::Instant,
    pub tick_count: u64,

    // Search Algorithm
    pub algorithm: SearchAlgorithm,

    // Preset Scenarios
    pub current_preset: PresetScenario,

    // Modal & Form Input State
    pub modal_mode: ModalMode,
    pub form: FormState,

    // Parameters
    pub cash: f64,
    pub fee: f64,

    // Data lists
    pub portfolio: Vec<PortfolioItem>,
    pub goals: Vec<GoalItem>,

    // Selected rows
    pub selected_port_idx: usize,
    pub selected_goal_idx: usize,

    // Results
    pub last_result: Option<PlanResult>,
    pub last_benchmark: Option<BenchmarkResult>,
    pub status_message: String,
    pub running: bool,

    // Background Async Search & Animation State
    pub is_searching: bool,
    pub search_rx: Option<std::sync::mpsc::Receiver<SearchWorkerMessage>>,
    pub search_start_time: Option<std::time::Instant>,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            is_dark: true,
            layout_mode: LayoutMode::Split,
            col1_modules: vec![ModuleId::Params, ModuleId::Portfolio, ModuleId::Telemetry],
            col2_modules: vec![ModuleId::Goals, ModuleId::Engine, ModuleId::Audit],
            active_module_idx: 0,
            move_mode: false,

            start_time: std::time::Instant::now(),
            tick_count: 0,

            algorithm: SearchAlgorithm::AStar,
            current_preset: PresetScenario::TechTrio,

            modal_mode: ModalMode::None,
            form: FormState::default(),

            cash: 1000.0,
            fee: 2.0,

            portfolio: Vec::new(),
            goals: Vec::new(),

            selected_port_idx: 0,
            selected_goal_idx: 0,

            last_result: None,
            last_benchmark: None,
            status_message: "BBG::OS READY // [A] ADD ENTRY | [O/F6] ALGO | [B] PRESETS | [F5] RUN | [F7] TEST ALL".to_string(),
            running: true,

            is_searching: false,
            search_rx: None,
            search_start_time: None,
        };

        app.load_preset(PresetScenario::TechTrio);
        app
    }
}

impl App {
    pub fn on_tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        self.poll_search_result();
    }

    pub fn poll_search_result(&mut self) {
        if !self.is_searching {
            return;
        }

        if let Some(rx) = &self.search_rx {
            match rx.try_recv() {
                Ok(msg) => {
                    self.is_searching = false;
                    self.search_rx = None;
                    match msg {
                        SearchWorkerMessage::Single(res) => {
                            if res.success {
                                let time_str = if res.execution_time_us > 1000 {
                                    format!("{:.2}ms", (res.execution_time_us as f64) / 1000.0)
                                } else {
                                    format!("{}µs", res.execution_time_us)
                                };
                                self.status_message = format!(
                                    "[{}] SUCCESS // {} TRADES | FEES: ${:.2} | FINAL: ${:.2} | NODES: {} | TIME: {}",
                                    res.algorithm.code(),
                                    res.trades.len(),
                                    res.total_fee,
                                    res.final_cash,
                                    res.nodes_evaluated,
                                    time_str
                                );
                            } else {
                                self.status_message = format!("ERROR // {}", res.message);
                            }
                            self.last_result = Some(res);
                            self.last_benchmark = None;
                        }
                        SearchWorkerMessage::Benchmark(bench) => {
                            let total_time_str = if bench.total_time_us > 1000 {
                                format!("{:.2}ms", (bench.total_time_us as f64) / 1000.0)
                            } else {
                                format!("{}µs", bench.total_time_us)
                            };

                            if let Some(idx) = bench.fastest_idx {
                                let fastest = &bench.results[idx];
                                let f_time_str = if fastest.execution_time_us > 1000 {
                                    format!("{:.2}ms", (fastest.execution_time_us as f64) / 1000.0)
                                } else {
                                    format!("{}µs", fastest.execution_time_us)
                                };
                                self.status_message = format!(
                                    "BENCHMARK COMPLETE // FASTEST: [{}] {} (TIME: {} | NODES: {}) | TOTAL: {}",
                                    fastest.algorithm.code(),
                                    fastest.algorithm.name(),
                                    f_time_str,
                                    fastest.nodes_evaluated,
                                    total_time_str
                                );
                            } else {
                                self.status_message = format!("BENCHMARK COMPLETE // {} ALGORITHMS TESTED ({})", bench.results.len(), total_time_str);
                            }
                            self.last_benchmark = Some(bench);
                            self.last_result = None;
                        }
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    let dots = self.running_dots_str();
                    self.status_message = format!(
                        "OPTIMIZING // EVALUATING STATE SPACE{} (NON-BLOCKING UI)",
                        dots
                    );
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_searching = false;
                    self.search_rx = None;
                    self.status_message = "ENGINE ERROR // SEARCH WORKER THREAD DISCONNECTED".to_string();
                }
            }
        }
    }

    pub fn spinner_frame(&self) -> &'static str {
        const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        SPINNER[(self.tick_count as usize) % SPINNER.len()]
    }

    pub fn running_dots_str(&self) -> &'static str {
        const DOTS: &[&str] = &[".", "..", "...", "...."];
        DOTS[(self.tick_count as usize / 2) % DOTS.len()]
    }

    pub fn running_progress_bar(&self, width: usize) -> String {
        let cycle = (self.tick_count as usize * 2) % (width + 6);
        let mut bar = String::with_capacity(width);
        for i in 0..width {
            if i + 4 >= cycle && i <= cycle {
                bar.push('■');
            } else {
                bar.push('░');
            }
        }
        bar
    }

    pub fn search_elapsed_str(&self) -> String {
        if let Some(st) = self.search_start_time {
            let el = st.elapsed();
            if el.as_millis() < 1000 {
                format!("{} ms", el.as_millis())
            } else {
                format!("{:.2} s", el.as_secs_f64())
            }
        } else {
            "0.00 s".to_string()
        }
    }

    pub fn elapsed_timer_str(&self) -> String {
        let elapsed = self.start_time.elapsed().as_secs();
        let hrs = elapsed / 3600;
        let mins = (elapsed % 3600) / 60;
        let secs = elapsed % 60;
        if hrs > 0 {
            format!("{:02}:{:02}:{:02}", hrs, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    }

    /// Returns a value 0..4 for smooth subtle pulsating highlights (approx 600ms cycle)
    #[allow(dead_code)]
    pub fn flash_phase(&self) -> usize {
        ((self.tick_count / 3) % 4) as usize
    }

    /// Returns true if flash phase is in high brightness
    pub fn is_flash_bright(&self) -> bool {
        (self.tick_count / 4) % 2 == 0
    }
    pub fn toggle_theme(&mut self) {
        self.is_dark = !self.is_dark;
        self.status_message = format!(
            "THEME CHANGED: {}",
            if self.is_dark { "MONOTONE DARK" } else { "MONOTONE LIGHT" }
        );
    }

    pub fn toggle_layout(&mut self) {
        self.layout_mode = match self.layout_mode {
            LayoutMode::Split => LayoutMode::Stacked,
            LayoutMode::Stacked => LayoutMode::Split,
        };
        self.status_message = format!("LAYOUT: {:?}", self.layout_mode);
    }

    pub fn next_algorithm(&mut self) {
        self.algorithm = self.algorithm.next();
        self.status_message = format!("SEARCH ALGORITHM: [{}] {}", self.algorithm.code(), self.algorithm.name());
    }

    pub fn prev_algorithm(&mut self) {
        self.algorithm = self.algorithm.prev();
        self.status_message = format!("SEARCH ALGORITHM: [{}] {}", self.algorithm.code(), self.algorithm.name());
    }

    pub fn next_preset(&mut self) {
        let nxt = self.current_preset.next();
        self.load_preset(nxt);
    }

    pub fn all_visible_modules(&self) -> Vec<ModuleId> {
        let mut v = self.col1_modules.clone();
        v.extend(self.col2_modules.clone());
        v
    }

    pub fn active_module(&self) -> Option<ModuleId> {
        let all = self.all_visible_modules();
        if all.is_empty() {
            None
        } else {
            Some(all[self.active_module_idx % all.len()])
        }
    }

    pub fn next_module(&mut self) {
        let count = self.all_visible_modules().len();
        if count > 0 {
            self.active_module_idx = (self.active_module_idx + 1) % count;
        }
    }

    pub fn prev_module(&mut self) {
        let count = self.all_visible_modules().len();
        if count > 0 {
            self.active_module_idx = (self.active_module_idx + count - 1) % count;
        }
    }

    pub fn move_active_module_up(&mut self) {
        if let Some(mod_id) = self.active_module() {
            if let Some(pos) = self.col1_modules.iter().position(|&m| m == mod_id) {
                if pos > 0 {
                    self.col1_modules.swap(pos, pos - 1);
                    self.status_message = format!("MOVED [{}] UP IN COL 1", mod_id.code());
                }
            } else if let Some(pos) = self.col2_modules.iter().position(|&m| m == mod_id) {
                if pos > 0 {
                    self.col2_modules.swap(pos, pos - 1);
                    self.status_message = format!("MOVED [{}] UP IN COL 2", mod_id.code());
                }
            }
        }
    }

    pub fn move_active_module_down(&mut self) {
        if let Some(mod_id) = self.active_module() {
            if let Some(pos) = self.col1_modules.iter().position(|&m| m == mod_id) {
                if pos + 1 < self.col1_modules.len() {
                    self.col1_modules.swap(pos, pos + 1);
                    self.status_message = format!("MOVED [{}] DOWN IN COL 1", mod_id.code());
                }
            } else if let Some(pos) = self.col2_modules.iter().position(|&m| m == mod_id) {
                if pos + 1 < self.col2_modules.len() {
                    self.col2_modules.swap(pos, pos + 1);
                    self.status_message = format!("MOVED [{}] DOWN IN COL 2", mod_id.code());
                }
            }
        }
    }

    pub fn move_active_module_col(&mut self) {
        if let Some(mod_id) = self.active_module() {
            if let Some(pos) = self.col1_modules.iter().position(|&m| m == mod_id) {
                self.col1_modules.remove(pos);
                self.col2_modules.push(mod_id);
                self.status_message = format!("TRANSFERRED [{}] TO COL 2", mod_id.code());
            } else if let Some(pos) = self.col2_modules.iter().position(|&m| m == mod_id) {
                self.col2_modules.remove(pos);
                self.col1_modules.push(mod_id);
                self.status_message = format!("TRANSFERRED [{}] TO COL 1", mod_id.code());
            }
        }
    }

    // Modal Form Trigger Handlers
    pub fn open_add_portfolio_modal(&mut self) {
        self.modal_mode = ModalMode::AddPortfolio;
        self.form = FormState {
            symbol: String::new(),
            qty: "10".to_string(),
            price: "150.00".to_string(),
            sell_only: false,
            field_idx: 0,
            is_editing: false,
            editing_idx: 0,
            ..Default::default()
        };
        self.status_message = "MODAL: ADD PORTFOLIO POSITION // [TAB] SWITCH FIELD, [ENTER] SUBMIT, [ESC] CANCEL".to_string();
    }

    pub fn open_edit_portfolio_modal(&mut self, idx: usize) {
        if idx < self.portfolio.len() {
            let item = &self.portfolio[idx];
            self.modal_mode = ModalMode::AddPortfolio;
            self.form = FormState {
                symbol: item.symbol.clone(),
                qty: item.qty.to_string(),
                price: format!("{:.2}", item.price),
                sell_only: item.sell_only,
                field_idx: 1, // Start on qty
                is_editing: true,
                editing_idx: idx,
                ..Default::default()
            };
            self.status_message = format!("MODAL: EDITING POSITION [{}] // [ENTER] SAVE CHANGES", item.symbol);
        }
    }

    pub fn open_add_goal_modal(&mut self) {
        self.modal_mode = ModalMode::AddGoal;
        self.form = FormState {
            symbol: String::new(),
            min_qty: "0".to_string(),
            max_qty: "10".to_string(),
            price: "150.00".to_string(),
            field_idx: 0,
            is_editing: false,
            editing_idx: 0,
            ..Default::default()
        };
        self.status_message = "MODAL: ADD TARGET GOAL // [TAB] SWITCH FIELD, [ENTER] SUBMIT, [ESC] CANCEL".to_string();
    }

    pub fn open_edit_goal_modal(&mut self, idx: usize) {
        if idx < self.goals.len() {
            let item = &self.goals[idx];
            self.modal_mode = ModalMode::AddGoal;
            self.form = FormState {
                symbol: item.symbol.clone(),
                min_qty: item.min_qty.to_string(),
                max_qty: item.max_qty.to_string(),
                price: format!("{:.2}", item.price),
                field_idx: 1, // Start on min_qty
                is_editing: true,
                editing_idx: idx,
                ..Default::default()
            };
            self.status_message = format!("MODAL: EDITING TARGET GOAL [{}] // [ENTER] SAVE CHANGES", item.symbol);
        }
    }

    pub fn open_edit_params_modal(&mut self) {
        self.modal_mode = ModalMode::EditParams;
        self.form = FormState {
            cash: format!("{:.2}", self.cash),
            fee: format!("{:.2}", self.fee),
            field_idx: 0,
            ..Default::default()
        };
        self.status_message = "MODAL: EDIT CASH & FEES // [TAB] SWITCH FIELD, [ENTER] SUBMIT, [ESC] CANCEL".to_string();
    }

    pub fn close_modal(&mut self) {
        self.modal_mode = ModalMode::None;
        self.status_message = "MODAL CLOSED // READY".to_string();
    }

    pub fn form_next_field(&mut self) {
        let max_fields = match self.modal_mode {
            ModalMode::AddPortfolio => 4, // 0: sym, 1: qty, 2: price, 3: sell_only
            ModalMode::AddGoal => 4,      // 0: sym, 1: min, 2: max, 3: price
            ModalMode::EditParams => 2,   // 0: cash, 1: fee
            ModalMode::None => 1,
        };
        self.form.field_idx = (self.form.field_idx + 1) % max_fields;
    }

    pub fn form_prev_field(&mut self) {
        let max_fields = match self.modal_mode {
            ModalMode::AddPortfolio => 4,
            ModalMode::AddGoal => 4,
            ModalMode::EditParams => 2,
            ModalMode::None => 1,
        };
        self.form.field_idx = (self.form.field_idx + max_fields - 1) % max_fields;
    }

    pub fn form_input_char(&mut self, c: char) {
        match self.modal_mode {
            ModalMode::AddPortfolio => match self.form.field_idx {
                0 => {
                    if c.is_alphanumeric() && self.form.symbol.len() < 8 {
                        self.form.symbol.push(c.to_ascii_uppercase());
                    }
                }
                1 => {
                    if c.is_ascii_digit() && self.form.qty.len() < 7 {
                        self.form.qty.push(c);
                    }
                }
                2 => {
                    if (c.is_ascii_digit() || c == '.') && self.form.price.len() < 10 {
                        self.form.price.push(c);
                    }
                }
                3 => {
                    if c == ' ' || c == 'y' || c == 'Y' || c == 't' || c == 'T' {
                        self.form.sell_only = !self.form.sell_only;
                    }
                }
                _ => {}
            },
            ModalMode::AddGoal => match self.form.field_idx {
                0 => {
                    if c.is_alphanumeric() && self.form.symbol.len() < 8 {
                        self.form.symbol.push(c.to_ascii_uppercase());
                    }
                }
                1 => {
                    if c.is_ascii_digit() && self.form.min_qty.len() < 7 {
                        self.form.min_qty.push(c);
                    }
                }
                2 => {
                    if c.is_ascii_digit() && self.form.max_qty.len() < 7 {
                        self.form.max_qty.push(c);
                    }
                }
                3 => {
                    if (c.is_ascii_digit() || c == '.') && self.form.price.len() < 10 {
                        self.form.price.push(c);
                    }
                }
                _ => {}
            },
            ModalMode::EditParams => match self.form.field_idx {
                0 => {
                    if (c.is_ascii_digit() || c == '.') && self.form.cash.len() < 12 {
                        self.form.cash.push(c);
                    }
                }
                1 => {
                    if (c.is_ascii_digit() || c == '.') && self.form.fee.len() < 8 {
                        self.form.fee.push(c);
                    }
                }
                _ => {}
            },
            ModalMode::None => {}
        }
    }

    pub fn form_backspace(&mut self) {
        match self.modal_mode {
            ModalMode::AddPortfolio => match self.form.field_idx {
                0 => { self.form.symbol.pop(); }
                1 => { self.form.qty.pop(); }
                2 => { self.form.price.pop(); }
                3 => { self.form.sell_only = false; }
                _ => {}
            },
            ModalMode::AddGoal => match self.form.field_idx {
                0 => { self.form.symbol.pop(); }
                1 => { self.form.min_qty.pop(); }
                2 => { self.form.max_qty.pop(); }
                3 => { self.form.price.pop(); }
                _ => {}
            },
            ModalMode::EditParams => match self.form.field_idx {
                0 => { self.form.cash.pop(); }
                1 => { self.form.fee.pop(); }
                _ => {}
            },
            ModalMode::None => {}
        }
    }

    pub fn form_toggle_checkbox(&mut self) {
        if self.modal_mode == ModalMode::AddPortfolio && self.form.field_idx == 3 {
            self.form.sell_only = !self.form.sell_only;
        }
    }

    pub fn form_submit(&mut self) {
        match self.modal_mode {
            ModalMode::AddPortfolio => {
                let sym = self.form.symbol.trim().to_uppercase();
                let qty = self.form.qty.trim().parse::<i64>().unwrap_or(0);
                let price = self.form.price.trim().parse::<f64>().unwrap_or(0.0);

                if !sym.is_empty() && qty >= 0 && price > 0.0 {
                    if self.form.is_editing && self.form.editing_idx < self.portfolio.len() {
                        let item = &mut self.portfolio[self.form.editing_idx];
                        item.symbol = sym.clone();
                        item.qty = qty;
                        item.price = price;
                        item.sell_only = self.form.sell_only;
                        self.status_message = format!("MODIFIED POSITION: {} (QTY: {}, ${:.2})", sym, qty, price);
                    } else if let Some(existing) = self.portfolio.iter_mut().find(|p| p.symbol == sym) {
                        existing.qty = qty;
                        existing.price = price;
                        existing.sell_only = self.form.sell_only;
                        self.status_message = format!("UPDATED POSITION: {} (QTY: {}, ${:.2})", sym, existing.qty, price);
                    } else {
                        self.portfolio.push(PortfolioItem {
                            symbol: sym.clone(),
                            qty,
                            price,
                            sell_only: self.form.sell_only,
                        });
                        self.selected_port_idx = self.portfolio.len() - 1;
                        self.status_message = format!("ADDED NEW POSITION: {} (QTY: {}, ${:.2})", sym, qty, price);
                    }
                    self.modal_mode = ModalMode::None;
                } else {
                    self.status_message = "VALIDATION ERROR // ENTER VALID SYMBOL, QTY >= 0, PRICE > 0".to_string();
                }
            }
            ModalMode::AddGoal => {
                let sym = self.form.symbol.trim().to_uppercase();
                let min_q = self.form.min_qty.trim().parse::<i64>().unwrap_or(0);
                let max_q = self.form.max_qty.trim().parse::<i64>().unwrap_or(0);
                let price = self.form.price.trim().parse::<f64>().unwrap_or(0.0);

                if !sym.is_empty() && max_q >= min_q && price > 0.0 {
                    if self.form.is_editing && self.form.editing_idx < self.goals.len() {
                        let item = &mut self.goals[self.form.editing_idx];
                        item.symbol = sym.clone();
                        item.min_qty = min_q;
                        item.max_qty = max_q;
                        item.price = price;
                        self.status_message = format!("MODIFIED TARGET GOAL: {} [{}-{}]", sym, min_q, max_q);
                    } else if let Some(existing) = self.goals.iter_mut().find(|g| g.symbol == sym) {
                        existing.min_qty = min_q;
                        existing.max_qty = max_q;
                        existing.price = price;
                        self.status_message = format!("UPDATED TARGET GOAL: {} [{}-{}]", sym, min_q, max_q);
                    } else {
                        self.goals.push(GoalItem {
                            symbol: sym.clone(),
                            min_qty: min_q,
                            max_qty: max_q,
                            price,
                        });
                        self.selected_goal_idx = self.goals.len() - 1;
                        self.status_message = format!("ADDED TARGET GOAL: {} [{}-{}]", sym, min_q, max_q);
                    }
                    self.modal_mode = ModalMode::None;
                } else {
                    self.status_message = "VALIDATION ERROR // MAX QTY MUST BE >= MIN QTY & PRICE > 0".to_string();
                }
            }
            ModalMode::EditParams => {
                if let Ok(c) = self.form.cash.trim().parse::<f64>() {
                    self.cash = c;
                }
                if let Ok(f) = self.form.fee.trim().parse::<f64>() {
                    self.fee = f;
                }
                self.status_message = format!("PARAMETERS SAVED // CASH: ${:.2}, FEE: ${:.2}", self.cash, self.fee);
                self.modal_mode = ModalMode::None;
            }
            ModalMode::None => {}
        }
    }

    pub fn load_preset(&mut self, preset: PresetScenario) {
        self.current_preset = preset;
        self.last_result = None;
        self.selected_port_idx = 0;
        self.selected_goal_idx = 0;

        match preset {
            PresetScenario::TechTrio => {
                self.cash = 1000.0;
                self.fee = 2.0;
                self.portfolio = vec![
                    PortfolioItem { symbol: "AAPL".to_string(), qty: 10, price: 180.0, sell_only: false },
                    PortfolioItem { symbol: "GOOG".to_string(), qty: 5, price: 140.0, sell_only: false },
                ];
                self.goals = vec![
                    GoalItem { symbol: "AAPL".to_string(), min_qty: 0, max_qty: 5, price: 180.0 },
                    GoalItem { symbol: "GOOG".to_string(), min_qty: 5, max_qty: 10, price: 140.0 },
                    GoalItem { symbol: "MSFT".to_string(), min_qty: 2, max_qty: 5, price: 300.0 },
                ];
            }
            PresetScenario::MegaCapSeven => {
                self.cash = 5000.0;
                self.fee = 4.0;
                self.portfolio = vec![
                    PortfolioItem { symbol: "NVDA".to_string(), qty: 25, price: 120.0, sell_only: false },
                    PortfolioItem { symbol: "AAPL".to_string(), qty: 15, price: 180.0, sell_only: false },
                    PortfolioItem { symbol: "MSFT".to_string(), qty: 8, price: 400.0, sell_only: false },
                    PortfolioItem { symbol: "AMZN".to_string(), qty: 12, price: 175.0, sell_only: false },
                    PortfolioItem { symbol: "TSLA".to_string(), qty: 20, price: 210.0, sell_only: true },
                ];
                self.goals = vec![
                    GoalItem { symbol: "NVDA".to_string(), min_qty: 30, max_qty: 40, price: 120.0 },
                    GoalItem { symbol: "AAPL".to_string(), min_qty: 10, max_qty: 15, price: 180.0 },
                    GoalItem { symbol: "MSFT".to_string(), min_qty: 5, max_qty: 10, price: 400.0 },
                    GoalItem { symbol: "AMZN".to_string(), min_qty: 15, max_qty: 20, price: 175.0 },
                    GoalItem { symbol: "GOOGL".to_string(), min_qty: 5, max_qty: 10, price: 165.0 },
                    GoalItem { symbol: "META".to_string(), min_qty: 4, max_qty: 8, price: 480.0 },
                    GoalItem { symbol: "TSLA".to_string(), min_qty: 0, max_qty: 10, price: 210.0 },
                ];
            }
            PresetScenario::DiversifiedEtf => {
                self.cash = 3500.0;
                self.fee = 1.5;
                self.portfolio = vec![
                    PortfolioItem { symbol: "SPY".to_string(), qty: 10, price: 520.0, sell_only: false },
                    PortfolioItem { symbol: "QQQ".to_string(), qty: 5, price: 450.0, sell_only: false },
                    PortfolioItem { symbol: "BND".to_string(), qty: 30, price: 72.0, sell_only: false },
                ];
                self.goals = vec![
                    GoalItem { symbol: "SPY".to_string(), min_qty: 12, max_qty: 16, price: 520.0 },
                    GoalItem { symbol: "QQQ".to_string(), min_qty: 6, max_qty: 10, price: 450.0 },
                    GoalItem { symbol: "VTI".to_string(), min_qty: 10, max_qty: 20, price: 260.0 },
                    GoalItem { symbol: "GLD".to_string(), min_qty: 5, max_qty: 10, price: 215.0 },
                    GoalItem { symbol: "BND".to_string(), min_qty: 20, max_qty: 40, price: 72.0 },
                ];
            }
            PresetScenario::CryptoVol => {
                self.cash = 8000.0;
                self.fee = 5.0;
                self.portfolio = vec![
                    PortfolioItem { symbol: "BTC".to_string(), qty: 2, price: 650.0, sell_only: false },
                    PortfolioItem { symbol: "ETH".to_string(), qty: 10, price: 340.0, sell_only: false },
                    PortfolioItem { symbol: "SOL".to_string(), qty: 25, price: 140.0, sell_only: false },
                ];
                self.goals = vec![
                    GoalItem { symbol: "BTC".to_string(), min_qty: 3, max_qty: 5, price: 650.0 },
                    GoalItem { symbol: "ETH".to_string(), min_qty: 8, max_qty: 12, price: 340.0 },
                    GoalItem { symbol: "SOL".to_string(), min_qty: 30, max_qty: 50, price: 140.0 },
                    GoalItem { symbol: "COIN".to_string(), min_qty: 5, max_qty: 15, price: 220.0 },
                    GoalItem { symbol: "MSTR".to_string(), min_qty: 2, max_qty: 6, price: 1400.0 },
                ];
            }
            PresetScenario::DividendIncome => {
                self.cash = 2500.0;
                self.fee = 1.0;
                self.portfolio = vec![
                    PortfolioItem { symbol: "JNJ".to_string(), qty: 20, price: 160.0, sell_only: false },
                    PortfolioItem { symbol: "PG".to_string(), qty: 15, price: 165.0, sell_only: false },
                    PortfolioItem { symbol: "KO".to_string(), qty: 40, price: 62.0, sell_only: false },
                ];
                self.goals = vec![
                    GoalItem { symbol: "JNJ".to_string(), min_qty: 20, max_qty: 30, price: 160.0 },
                    GoalItem { symbol: "PG".to_string(), min_qty: 15, max_qty: 25, price: 165.0 },
                    GoalItem { symbol: "KO".to_string(), min_qty: 30, max_qty: 50, price: 62.0 },
                    GoalItem { symbol: "SCHD".to_string(), min_qty: 25, max_qty: 50, price: 80.0 },
                    GoalItem { symbol: "O".to_string(), min_qty: 20, max_qty: 40, price: 54.0 },
                ];
            }
        }

        self.last_result = None;
        self.last_benchmark = None;
        self.selected_port_idx = 0;
        self.selected_goal_idx = 0;
        self.status_message = format!("PRESET LOADED: {}", preset.name());
    }

    pub fn clear_all(&mut self) {
        self.portfolio.clear();
        self.goals.clear();
        self.selected_port_idx = 0;
        self.selected_goal_idx = 0;
        self.last_result = None;
        self.last_benchmark = None;
        self.status_message = "SYS.CLEARED // PORTFOLIO AND GOALS EMPTIED".to_string();
    }

    pub fn execute_plan(&mut self) {
        if self.is_searching {
            self.status_message = "ENGINE BUSY // SEARCH ALREADY RUNNING....".to_string();
            return;
        }

        self.is_searching = true;
        self.search_start_time = Some(std::time::Instant::now());
        self.status_message = format!(
            "OPTIMIZING // RUNNING {} SEARCH ENGINE.... (NON-BLOCKING)",
            self.algorithm.code()
        );

        let (tx, rx) = std::sync::mpsc::channel();
        let algorithm = self.algorithm;
        let cash = self.cash;
        let fee = self.fee;
        let portfolio = self.portfolio.clone();
        let goals = self.goals.clone();

        std::thread::spawn(move || {
            let res = calculate_plan(
                algorithm,
                cash,
                fee,
                &portfolio,
                &goals,
            );
            let _ = tx.send(SearchWorkerMessage::Single(res));
        });

        self.search_rx = Some(rx);
    }

    pub fn execute_benchmark(&mut self) {
        if self.is_searching {
            self.status_message = "ENGINE BUSY // SEARCH BENCHMARK ALREADY RUNNING....".to_string();
            return;
        }

        self.is_searching = true;
        self.search_start_time = Some(std::time::Instant::now());
        self.status_message = "BENCHMARKING // TESTING ALL 7 SEARCH ALGORITHMS.... (NON-BLOCKING)".to_string();

        let (tx, rx) = std::sync::mpsc::channel();
        let cash = self.cash;
        let fee = self.fee;
        let portfolio = self.portfolio.clone();
        let goals = self.goals.clone();

        std::thread::spawn(move || {
            let bench = benchmark_all_algorithms(
                cash,
                fee,
                &portfolio,
                &goals,
            );
            let _ = tx.send(SearchWorkerMessage::Benchmark(bench));
        });

        self.search_rx = Some(rx);
    }

    pub fn delete_selected_portfolio_item(&mut self) {
        if !self.portfolio.is_empty() && self.selected_port_idx < self.portfolio.len() {
            let removed = self.portfolio.remove(self.selected_port_idx);
            if self.selected_port_idx >= self.portfolio.len() && self.selected_port_idx > 0 {
                self.selected_port_idx -= 1;
            }
            self.status_message = format!("REMOVED PORTFOLIO SYMBOL: {}", removed.symbol);
        }
    }

    pub fn delete_selected_goal_item(&mut self) {
        if !self.goals.is_empty() && self.selected_goal_idx < self.goals.len() {
            let removed = self.goals.remove(self.selected_goal_idx);
            if self.selected_goal_idx >= self.goals.len() && self.selected_goal_idx > 0 {
                self.selected_goal_idx -= 1;
            }
            self.status_message = format!("REMOVED TARGET GOAL SYMBOL: {}", removed.symbol);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_search_execution_and_polling() {
        let mut app = App::default();
        assert!(!app.is_searching);
        assert!(app.last_result.is_none());

        app.execute_plan();
        assert!(app.is_searching);
        assert!(app.search_rx.is_some());

        // Repeated execute calls while searching are ignored
        app.execute_plan();
        assert!(app.is_searching);

        // Spin until worker thread finishes and poll_search_result receives it
        let mut attempts = 0;
        while app.is_searching && attempts < 100 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            app.on_tick();
            attempts += 1;
        }

        assert!(!app.is_searching);
        assert!(app.last_result.is_some());
        assert!(app.last_result.as_ref().unwrap().success);
    }

    #[test]
    fn test_animation_helpers() {
        let mut app = App::default();
        let s0 = app.spinner_frame();
        let d0 = app.running_dots_str();
        let bar = app.running_progress_bar(16);
        assert_eq!(bar.len(), 16 * 3); // 16 UTF-8 characters (3 bytes each for ■ and ░)
        assert!(!s0.is_empty());
        assert!(!d0.is_empty());

        app.on_tick();
        assert_eq!(app.tick_count, 1);
    }

    #[test]
    fn test_preset_switching() {
        let mut app = App::default();
        assert_eq!(app.current_preset, PresetScenario::TechTrio);
        app.next_preset();
        assert_eq!(app.current_preset, PresetScenario::MegaCapSeven);
        assert_eq!(app.portfolio.len(), 5);
        assert_eq!(app.goals.len(), 7);
    }

    #[test]
    fn test_async_benchmark_execution_and_polling() {
        let mut app = App::default();
        assert!(!app.is_searching);
        assert!(app.last_benchmark.is_none());

        app.execute_benchmark();
        assert!(app.is_searching);
        assert!(app.search_rx.is_some());

        // Spin until worker thread finishes and poll_search_result receives it
        let mut attempts = 0;
        while app.is_searching && attempts < 100 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            app.on_tick();
            attempts += 1;
        }

        assert!(!app.is_searching);
        assert!(app.last_benchmark.is_some());
        let bench = app.last_benchmark.as_ref().unwrap();
        assert_eq!(bench.results.len(), 7);
        assert!(bench.fastest_idx.is_some());
        let fastest = &bench.results[bench.fastest_idx.unwrap()];
        assert!(fastest.success);
    }
}
