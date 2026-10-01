use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, VecDeque};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SearchAlgorithm {
    AStar,
    GreedyBestFirst,
    BreadthFirst,
    UniformCost,
    BeamSearch,
    DepthLimited,
    IterativeDeepening,
}

impl SearchAlgorithm {
    #[allow(dead_code)]
    pub const ALL: [SearchAlgorithm; 7] = [
        SearchAlgorithm::AStar,
        SearchAlgorithm::GreedyBestFirst,
        SearchAlgorithm::BreadthFirst,
        SearchAlgorithm::UniformCost,
        SearchAlgorithm::BeamSearch,
        SearchAlgorithm::DepthLimited,
        SearchAlgorithm::IterativeDeepening,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            SearchAlgorithm::AStar => "A* HEURISTIC (OPTIMAL)",
            SearchAlgorithm::GreedyBestFirst => "GREEDY BEST-FIRST (FAST)",
            SearchAlgorithm::BreadthFirst => "BREADTH-FIRST (FEWEST TRADES)",
            SearchAlgorithm::UniformCost => "UNIFORM-COST / DIJKSTRA",
            SearchAlgorithm::BeamSearch => "BEAM SEARCH (K=50 BOUNDED)",
            SearchAlgorithm::DepthLimited => "DEPTH-LIMITED (D=12)",
            SearchAlgorithm::IterativeDeepening => "ITERATIVE DEEPENING (IDDFS)",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            SearchAlgorithm::AStar => "A*",
            SearchAlgorithm::GreedyBestFirst => "GBFS",
            SearchAlgorithm::BreadthFirst => "BFS",
            SearchAlgorithm::UniformCost => "UCS",
            SearchAlgorithm::BeamSearch => "BEAM",
            SearchAlgorithm::DepthLimited => "DLS",
            SearchAlgorithm::IterativeDeepening => "IDDFS",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SearchAlgorithm::AStar => "Minimizes f(n) = g(n) + h(n). Guaranteed optimal trade count under admissible heuristic.",
            SearchAlgorithm::GreedyBestFirst => "Minimizes h(n). Aggressively drives allocation distance to zero with minimal node expansions.",
            SearchAlgorithm::BreadthFirst => "FIFO queue expanding level-by-level. Finds shallowest goal without heuristic guidance.",
            SearchAlgorithm::UniformCost => "Minimizes cumulative fee and transaction cost g(n) without heuristic bias.",
            SearchAlgorithm::BeamSearch => "Retains only the top-50 most promising states per level for bounded memory & high speed.",
            SearchAlgorithm::DepthLimited => "DFS exploration up to max depth of 12 steps, preventing unbounded recursion.",
            SearchAlgorithm::IterativeDeepening => "Combines DFS memory efficiency with BFS completeness by iteratively increasing depth limit.",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SearchAlgorithm::AStar => SearchAlgorithm::GreedyBestFirst,
            SearchAlgorithm::GreedyBestFirst => SearchAlgorithm::BreadthFirst,
            SearchAlgorithm::BreadthFirst => SearchAlgorithm::UniformCost,
            SearchAlgorithm::UniformCost => SearchAlgorithm::BeamSearch,
            SearchAlgorithm::BeamSearch => SearchAlgorithm::DepthLimited,
            SearchAlgorithm::DepthLimited => SearchAlgorithm::IterativeDeepening,
            SearchAlgorithm::IterativeDeepening => SearchAlgorithm::AStar,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SearchAlgorithm::AStar => SearchAlgorithm::IterativeDeepening,
            SearchAlgorithm::GreedyBestFirst => SearchAlgorithm::AStar,
            SearchAlgorithm::BreadthFirst => SearchAlgorithm::GreedyBestFirst,
            SearchAlgorithm::UniformCost => SearchAlgorithm::BreadthFirst,
            SearchAlgorithm::BeamSearch => SearchAlgorithm::UniformCost,
            SearchAlgorithm::DepthLimited => SearchAlgorithm::BeamSearch,
            SearchAlgorithm::IterativeDeepening => SearchAlgorithm::DepthLimited,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortfolioItem {
    pub symbol: String,
    pub qty: i64,
    pub price: f64,
    pub sell_only: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GoalItem {
    pub symbol: String,
    pub min_qty: i64,
    pub max_qty: i64,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct PlanResult {
    pub success: bool,
    pub algorithm: SearchAlgorithm,
    pub trades: Vec<String>,
    pub total_fee: f64,
    pub final_cash: f64,
    pub nodes_evaluated: u64,
    pub execution_time_us: u128,
    pub message: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Node {
    priority: i64,
    secondary: i64,
    counter: u64,
    cash_cents: i64,
    portfolio: BTreeMap<String, i64>,
    path: Vec<String>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: lowest priority first
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| other.secondary.cmp(&self.secondary))
            .then_with(|| other.counter.cmp(&self.counter))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn is_goal(
    state_port: &BTreeMap<String, i64>,
    goals: &HashMap<String, (i64, i64)>,
) -> bool {
    for (sym, &(min_q, max_q)) in goals {
        let qty = state_port.get(sym).copied().unwrap_or(0);
        if qty < min_q || qty > max_q {
            return false;
        }
    }
    for (sym, &qty) in state_port {
        if qty > 0 && !goals.contains_key(sym) {
            return false;
        }
    }
    true
}

pub fn heuristic(
    state_port: &BTreeMap<String, i64>,
    goals: &HashMap<String, (i64, i64)>,
) -> i64 {
    let mut h: i64 = 0;
    for (sym, &(min_q, max_q)) in goals {
        let qty = state_port.get(sym).copied().unwrap_or(0);
        if qty < min_q {
            let diff = min_q - qty;
            h += (diff + 99) / 100;
        } else if qty > max_q {
            let diff = qty - max_q;
            h += (diff + 99) / 100;
        }
    }
    for (sym, &qty) in state_port {
        if qty > 0 && !goals.contains_key(sym) {
            h += (qty + 99) / 100;
        }
    }
    h
}

#[derive(Clone)]
struct ActionOutcome {
    cash_cents: i64,
    portfolio: BTreeMap<String, i64>,
    desc: String,
}

fn get_successors(
    current_cash_cents: i64,
    current_port: &BTreeMap<String, i64>,
    prices: &HashMap<String, f64>,
    sell_only_set: &HashSet<String>,
    fee_cents: i64,
) -> Vec<ActionOutcome> {
    let mut successors = Vec::with_capacity(16);
    let lots = [1i64, 10, 100];

    for (sym, &px) in prices {
        let px_cents = (px * 100.0).round() as i64;

        for &lot in &lots {
            // Action: Buy
            if !sell_only_set.contains(sym) {
                let cost = (px_cents * lot) + fee_cents;
                if current_cash_cents >= cost {
                    let mut new_port = current_port.clone();
                    *new_port.entry(sym.clone()).or_insert(0) += lot;
                    if *new_port.get(sym).unwrap() == 0 {
                        new_port.remove(sym);
                    }
                    let new_cash = current_cash_cents - cost;
                    successors.push(ActionOutcome {
                        cash_cents: new_cash,
                        portfolio: new_port,
                        desc: format!("Buy {:3} {:<5} @ ${:.2}", lot, sym, px),
                    });
                }
            }

            // Action: Sell
            let current_holding = current_port.get(sym).copied().unwrap_or(0);
            if current_holding >= lot {
                let revenue = (px_cents * lot) - fee_cents;
                let new_cash = current_cash_cents + revenue;
                if new_cash >= 0 {
                    let mut new_port = current_port.clone();
                    let rem = current_holding - lot;
                    if rem == 0 {
                        new_port.remove(sym);
                    } else {
                        new_port.insert(sym.clone(), rem);
                    }
                    successors.push(ActionOutcome {
                        cash_cents: new_cash,
                        portfolio: new_port,
                        desc: format!("Sel {:3} {:<5} @ ${:.2}", lot, sym, px),
                    });
                }
            }
        }
    }

    successors
}

pub fn calculate_plan(
    algorithm: SearchAlgorithm,
    start_cash: f64,
    fee: f64,
    portfolio: &[PortfolioItem],
    goals: &[GoalItem],
) -> PlanResult {
    let start_time = Instant::now();

    let mut start_port = BTreeMap::new();
    let mut prices: HashMap<String, f64> = HashMap::new();
    let mut sell_only_set: HashSet<String> = HashSet::new();

    for p in portfolio {
        if p.qty > 0 {
            start_port.insert(p.symbol.clone(), p.qty);
        }
        prices.insert(p.symbol.clone(), p.price);
        if p.sell_only {
            sell_only_set.insert(p.symbol.clone());
        }
    }

    let mut goal_map: HashMap<String, (i64, i64)> = HashMap::new();
    for g in goals {
        goal_map.insert(g.symbol.clone(), (g.min_qty, g.max_qty));
        if !prices.contains_key(&g.symbol) {
            prices.insert(g.symbol.clone(), g.price);
        }
    }

    let start_cash_cents = (start_cash * 100.0).round() as i64;
    let fee_cents = (fee * 100.0).round() as i64;

    // Check if initial state is already at goal
    if is_goal(&start_port, &goal_map) {
        let elapsed = start_time.elapsed().as_micros();
        return PlanResult {
            success: true,
            algorithm,
            trades: Vec::new(),
            total_fee: 0.0,
            final_cash: start_cash,
            nodes_evaluated: 1,
            execution_time_us: elapsed,
            message: "INITIAL HOLDINGS ALREADY SATISFY ALL TARGET GOALS".to_string(),
        };
    }

    let res = match algorithm {
        SearchAlgorithm::AStar => run_astar(
            start_cash_cents,
            fee_cents,
            start_port,
            prices,
            sell_only_set,
            goal_map,
            fee,
        ),
        SearchAlgorithm::GreedyBestFirst => run_gbfs(
            start_cash_cents,
            fee_cents,
            start_port,
            prices,
            sell_only_set,
            goal_map,
            fee,
        ),
        SearchAlgorithm::BreadthFirst => run_bfs(
            start_cash_cents,
            fee_cents,
            start_port,
            prices,
            sell_only_set,
            goal_map,
            fee,
        ),
        SearchAlgorithm::UniformCost => run_ucs(
            start_cash_cents,
            fee_cents,
            start_port,
            prices,
            sell_only_set,
            goal_map,
            fee,
        ),
        SearchAlgorithm::BeamSearch => run_beam(
            start_cash_cents,
            fee_cents,
            start_port,
            prices,
            sell_only_set,
            goal_map,
            fee,
            50, // Beam width K=50
        ),
        SearchAlgorithm::DepthLimited => run_dls(
            start_cash_cents,
            fee_cents,
            start_port,
            prices,
            sell_only_set,
            goal_map,
            fee,
            12, // Depth limit
        ),
        SearchAlgorithm::IterativeDeepening => run_iddfs(
            start_cash_cents,
            fee_cents,
            start_port,
            prices,
            sell_only_set,
            goal_map,
            fee,
            12, // Max depth
        ),
    };

    let elapsed = start_time.elapsed().as_micros();
    PlanResult {
        execution_time_us: elapsed,
        ..res
    }
}

// 1. A* Search: priority = g + h
fn run_astar(
    start_cash_cents: i64,
    fee_cents: i64,
    start_port: BTreeMap<String, i64>,
    prices: HashMap<String, f64>,
    sell_only_set: HashSet<String>,
    goal_map: HashMap<String, (i64, i64)>,
    fee: f64,
) -> PlanResult {
    let mut open_set = BinaryHeap::new();
    let mut counter: u64 = 0;
    let initial_h = heuristic(&start_port, &goal_map);

    open_set.push(Node {
        priority: initial_h,
        secondary: 0,
        counter,
        cash_cents: start_cash_cents,
        portfolio: start_port.clone(),
        path: Vec::new(),
    });

    let mut best_cash_for_port: HashMap<BTreeMap<String, i64>, i64> = HashMap::new();
    best_cash_for_port.insert(start_port, start_cash_cents);

    while let Some(current) = open_set.pop() {
        if is_goal(&current.portfolio, &goal_map) {
            let total_fee = (current.path.len() as f64) * fee;
            let final_cash = (current.cash_cents as f64) / 100.0;
            return PlanResult {
                success: true,
                algorithm: SearchAlgorithm::AStar,
                trades: current.path,
                total_fee,
                final_cash,
                nodes_evaluated: counter,
                execution_time_us: 0,
                message: "OPTIMAL TRAJECTORY FOUND (A*)".to_string(),
            };
        }

        if counter >= 100_000 {
            break;
        }

        for outcome in get_successors(
            current.cash_cents,
            &current.portfolio,
            &prices,
            &sell_only_set,
            fee_cents,
        ) {
            let should_explore = match best_cash_for_port.get(&outcome.portfolio) {
                Some(&prev_cash) => outcome.cash_cents > prev_cash,
                None => true,
            };

            if should_explore {
                best_cash_for_port.insert(outcome.portfolio.clone(), outcome.cash_cents);
                counter += 1;
                let mut new_path = current.path.clone();
                new_path.push(outcome.desc);
                let g = new_path.len() as i64;
                let h = heuristic(&outcome.portfolio, &goal_map);
                open_set.push(Node {
                    priority: g + h,
                    secondary: g,
                    counter,
                    cash_cents: outcome.cash_cents,
                    portfolio: outcome.portfolio,
                    path: new_path,
                });
            }
        }
    }

    PlanResult {
        success: false,
        algorithm: SearchAlgorithm::AStar,
        trades: Vec::new(),
        total_fee: 0.0,
        final_cash: (start_cash_cents as f64) / 100.0,
        nodes_evaluated: counter,
        execution_time_us: 0,
        message: "NO VALID PLAN FOUND WITHIN SEARCH BUDGET (A*)".to_string(),
    }
}

// 2. Greedy Best-First Search: priority = h
fn run_gbfs(
    start_cash_cents: i64,
    fee_cents: i64,
    start_port: BTreeMap<String, i64>,
    prices: HashMap<String, f64>,
    sell_only_set: HashSet<String>,
    goal_map: HashMap<String, (i64, i64)>,
    fee: f64,
) -> PlanResult {
    let mut open_set = BinaryHeap::new();
    let mut counter: u64 = 0;
    let initial_h = heuristic(&start_port, &goal_map);

    open_set.push(Node {
        priority: initial_h,
        secondary: 0,
        counter,
        cash_cents: start_cash_cents,
        portfolio: start_port.clone(),
        path: Vec::new(),
    });

    let mut best_cash_for_port: HashMap<BTreeMap<String, i64>, i64> = HashMap::new();
    best_cash_for_port.insert(start_port, start_cash_cents);

    while let Some(current) = open_set.pop() {
        if is_goal(&current.portfolio, &goal_map) {
            let total_fee = (current.path.len() as f64) * fee;
            let final_cash = (current.cash_cents as f64) / 100.0;
            return PlanResult {
                success: true,
                algorithm: SearchAlgorithm::GreedyBestFirst,
                trades: current.path,
                total_fee,
                final_cash,
                nodes_evaluated: counter,
                execution_time_us: 0,
                message: "SOLUTION FOUND (GREEDY BEST-FIRST)".to_string(),
            };
        }

        if counter >= 100_000 {
            break;
        }

        for outcome in get_successors(
            current.cash_cents,
            &current.portfolio,
            &prices,
            &sell_only_set,
            fee_cents,
        ) {
            let should_explore = match best_cash_for_port.get(&outcome.portfolio) {
                Some(&prev_cash) => outcome.cash_cents > prev_cash,
                None => true,
            };

            if should_explore {
                best_cash_for_port.insert(outcome.portfolio.clone(), outcome.cash_cents);
                counter += 1;
                let mut new_path = current.path.clone();
                new_path.push(outcome.desc);
                let h = heuristic(&outcome.portfolio, &goal_map);
                open_set.push(Node {
                    priority: h,
                    secondary: new_path.len() as i64,
                    counter,
                    cash_cents: outcome.cash_cents,
                    portfolio: outcome.portfolio,
                    path: new_path,
                });
            }
        }
    }

    PlanResult {
        success: false,
        algorithm: SearchAlgorithm::GreedyBestFirst,
        trades: Vec::new(),
        total_fee: 0.0,
        final_cash: (start_cash_cents as f64) / 100.0,
        nodes_evaluated: counter,
        execution_time_us: 0,
        message: "NO VALID PLAN FOUND WITHIN BUDGET (GBFS)".to_string(),
    }
}

// 3. Breadth-First Search (FIFO queue)
fn run_bfs(
    start_cash_cents: i64,
    fee_cents: i64,
    start_port: BTreeMap<String, i64>,
    prices: HashMap<String, f64>,
    sell_only_set: HashSet<String>,
    goal_map: HashMap<String, (i64, i64)>,
    fee: f64,
) -> PlanResult {
    let mut queue = VecDeque::new();
    let mut counter: u64 = 0;

    queue.push_back((start_cash_cents, start_port.clone(), Vec::<String>::new()));
    let mut best_cash_for_port: HashMap<BTreeMap<String, i64>, i64> = HashMap::new();
    best_cash_for_port.insert(start_port, start_cash_cents);

    while let Some((curr_cash, curr_port, path)) = queue.pop_front() {
        if is_goal(&curr_port, &goal_map) {
            let total_fee = (path.len() as f64) * fee;
            let final_cash = (curr_cash as f64) / 100.0;
            return PlanResult {
                success: true,
                algorithm: SearchAlgorithm::BreadthFirst,
                trades: path,
                total_fee,
                final_cash,
                nodes_evaluated: counter,
                execution_time_us: 0,
                message: "SHORTEST PATH FOUND (BREADTH-FIRST SEARCH)".to_string(),
            };
        }

        if counter >= 100_000 {
            break;
        }

        for outcome in get_successors(
            curr_cash,
            &curr_port,
            &prices,
            &sell_only_set,
            fee_cents,
        ) {
            let should_explore = match best_cash_for_port.get(&outcome.portfolio) {
                Some(&prev_cash) => outcome.cash_cents > prev_cash,
                None => true,
            };

            if should_explore {
                best_cash_for_port.insert(outcome.portfolio.clone(), outcome.cash_cents);
                counter += 1;
                let mut new_path = path.clone();
                new_path.push(outcome.desc);
                queue.push_back((outcome.cash_cents, outcome.portfolio, new_path));
            }
        }
    }

    PlanResult {
        success: false,
        algorithm: SearchAlgorithm::BreadthFirst,
        trades: Vec::new(),
        total_fee: 0.0,
        final_cash: (start_cash_cents as f64) / 100.0,
        nodes_evaluated: counter,
        execution_time_us: 0,
        message: "NO VALID PLAN FOUND WITHIN BUDGET (BFS)".to_string(),
    }
}

// 4. Uniform-Cost Search / Dijkstra (priority = g_score / cumulative cost)
fn run_ucs(
    start_cash_cents: i64,
    fee_cents: i64,
    start_port: BTreeMap<String, i64>,
    prices: HashMap<String, f64>,
    sell_only_set: HashSet<String>,
    goal_map: HashMap<String, (i64, i64)>,
    fee: f64,
) -> PlanResult {
    let mut open_set = BinaryHeap::new();
    let mut counter: u64 = 0;

    open_set.push(Node {
        priority: 0,
        secondary: 0,
        counter,
        cash_cents: start_cash_cents,
        portfolio: start_port.clone(),
        path: Vec::new(),
    });

    let mut best_cash_for_port: HashMap<BTreeMap<String, i64>, i64> = HashMap::new();
    best_cash_for_port.insert(start_port, start_cash_cents);

    while let Some(current) = open_set.pop() {
        if is_goal(&current.portfolio, &goal_map) {
            let total_fee = (current.path.len() as f64) * fee;
            let final_cash = (current.cash_cents as f64) / 100.0;
            return PlanResult {
                success: true,
                algorithm: SearchAlgorithm::UniformCost,
                trades: current.path,
                total_fee,
                final_cash,
                nodes_evaluated: counter,
                execution_time_us: 0,
                message: "OPTIMAL COST PLAN FOUND (UNIFORM COST SEARCH)".to_string(),
            };
        }

        if counter >= 100_000 {
            break;
        }

        for outcome in get_successors(
            current.cash_cents,
            &current.portfolio,
            &prices,
            &sell_only_set,
            fee_cents,
        ) {
            let should_explore = match best_cash_for_port.get(&outcome.portfolio) {
                Some(&prev_cash) => outcome.cash_cents > prev_cash,
                None => true,
            };

            if should_explore {
                best_cash_for_port.insert(outcome.portfolio.clone(), outcome.cash_cents);
                counter += 1;
                let mut new_path = current.path.clone();
                new_path.push(outcome.desc);
                let g = new_path.len() as i64;
                open_set.push(Node {
                    priority: g,
                    secondary: 0,
                    counter,
                    cash_cents: outcome.cash_cents,
                    portfolio: outcome.portfolio,
                    path: new_path,
                });
            }
        }
    }

    PlanResult {
        success: false,
        algorithm: SearchAlgorithm::UniformCost,
        trades: Vec::new(),
        total_fee: 0.0,
        final_cash: (start_cash_cents as f64) / 100.0,
        nodes_evaluated: counter,
        execution_time_us: 0,
        message: "NO VALID PLAN FOUND WITHIN BUDGET (UCS)".to_string(),
    }
}

// 5. Beam Search (Top K states kept per generation)
fn run_beam(
    start_cash_cents: i64,
    fee_cents: i64,
    start_port: BTreeMap<String, i64>,
    prices: HashMap<String, f64>,
    sell_only_set: HashSet<String>,
    goal_map: HashMap<String, (i64, i64)>,
    fee: f64,
    beam_width: usize,
) -> PlanResult {
    let mut current_beam = vec![(start_cash_cents, start_port, Vec::<String>::new())];
    let mut counter: u64 = 0;
    let max_depth = 20;

    for _ in 0..max_depth {
        let mut candidates = Vec::new();

        for (curr_cash, curr_port, path) in current_beam {
            if is_goal(&curr_port, &goal_map) {
                let total_fee = (path.len() as f64) * fee;
                let final_cash = (curr_cash as f64) / 100.0;
                return PlanResult {
                    success: true,
                    algorithm: SearchAlgorithm::BeamSearch,
                    trades: path,
                    total_fee,
                    final_cash,
                    nodes_evaluated: counter,
                    execution_time_us: 0,
                    message: "SOLUTION FOUND (BEAM SEARCH K=50)".to_string(),
                };
            }

            for outcome in get_successors(
                curr_cash,
                &curr_port,
                &prices,
                &sell_only_set,
                fee_cents,
            ) {
                counter += 1;
                let mut new_path = path.clone();
                new_path.push(outcome.desc);
                let h = heuristic(&outcome.portfolio, &goal_map);
                candidates.push((h, outcome.cash_cents, outcome.portfolio, new_path));
            }
        }

        if candidates.is_empty() {
            break;
        }

        // Sort candidates by heuristic score (lowest h is best)
        candidates.sort_by_key(|c| c.0);
        candidates.truncate(beam_width);

        current_beam = candidates
            .into_iter()
            .map(|(_, cash, port, path)| (cash, port, path))
            .collect();
    }

    PlanResult {
        success: false,
        algorithm: SearchAlgorithm::BeamSearch,
        trades: Vec::new(),
        total_fee: 0.0,
        final_cash: (start_cash_cents as f64) / 100.0,
        nodes_evaluated: counter,
        execution_time_us: 0,
        message: "NO VALID PLAN FOUND WITHIN BEAM LIMITS".to_string(),
    }
}

// 6. Depth-Limited Search (DFS up to depth limit)
fn run_dls(
    start_cash_cents: i64,
    fee_cents: i64,
    start_port: BTreeMap<String, i64>,
    prices: HashMap<String, f64>,
    sell_only_set: HashSet<String>,
    goal_map: HashMap<String, (i64, i64)>,
    fee: f64,
    max_depth: usize,
) -> PlanResult {
    let mut stack = vec![(start_cash_cents, start_port, Vec::<String>::new())];
    let mut counter: u64 = 0;
    let mut visited = HashSet::new();

    while let Some((curr_cash, curr_port, path)) = stack.pop() {
        counter += 1;
        if is_goal(&curr_port, &goal_map) {
            let total_fee = (path.len() as f64) * fee;
            let final_cash = (curr_cash as f64) / 100.0;
            return PlanResult {
                success: true,
                algorithm: SearchAlgorithm::DepthLimited,
                trades: path,
                total_fee,
                final_cash,
                nodes_evaluated: counter,
                execution_time_us: 0,
                message: format!("SOLUTION FOUND (DEPTH-LIMITED D={})", max_depth),
            };
        }

        if path.len() >= max_depth || counter >= 50_000 {
            continue;
        }

        let state_key = (curr_cash, curr_port.clone());
        if !visited.insert(state_key) {
            continue;
        }

        let successors = get_successors(
            curr_cash,
            &curr_port,
            &prices,
            &sell_only_set,
            fee_cents,
        );

        for outcome in successors.into_iter().rev() {
            let mut new_path = path.clone();
            new_path.push(outcome.desc);
            stack.push((outcome.cash_cents, outcome.portfolio, new_path));
        }
    }

    PlanResult {
        success: false,
        algorithm: SearchAlgorithm::DepthLimited,
        trades: Vec::new(),
        total_fee: 0.0,
        final_cash: (start_cash_cents as f64) / 100.0,
        nodes_evaluated: counter,
        execution_time_us: 0,
        message: format!("NO VALID PLAN WITHIN DEPTH LIMIT D={}", max_depth),
    }
}

// 7. Iterative Deepening Search (IDDFS)
fn run_iddfs(
    start_cash_cents: i64,
    fee_cents: i64,
    start_port: BTreeMap<String, i64>,
    prices: HashMap<String, f64>,
    sell_only_set: HashSet<String>,
    goal_map: HashMap<String, (i64, i64)>,
    fee: f64,
    max_limit: usize,
) -> PlanResult {
    let mut total_nodes: u64 = 0;

    for depth in 1..=max_limit {
        let res = run_dls(
            start_cash_cents,
            fee_cents,
            start_port.clone(),
            prices.clone(),
            sell_only_set.clone(),
            goal_map.clone(),
            fee,
            depth,
        );
        total_nodes += res.nodes_evaluated;
        if res.success {
            return PlanResult {
                success: true,
                algorithm: SearchAlgorithm::IterativeDeepening,
                trades: res.trades,
                total_fee: res.total_fee,
                final_cash: res.final_cash,
                nodes_evaluated: total_nodes,
                execution_time_us: 0,
                message: format!("OPTIMAL PATH FOUND VIA IDDFS (DEPTH: {})", depth),
            };
        }
    }

    PlanResult {
        success: false,
        algorithm: SearchAlgorithm::IterativeDeepening,
        trades: Vec::new(),
        total_fee: 0.0,
        final_cash: (start_cash_cents as f64) / 100.0,
        nodes_evaluated: total_nodes,
        execution_time_us: 0,
        message: "NO VALID PLAN FOUND WITHIN ITERATIVE DEPTH LIMIT".to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub results: Vec<PlanResult>,
    pub fastest_idx: Option<usize>,
    pub total_time_us: u128,
}

pub fn benchmark_all_algorithms(
    cash: f64,
    fee: f64,
    portfolio: &[PortfolioItem],
    goals: &[GoalItem],
) -> BenchmarkResult {
    let start_all = Instant::now();
    let mut results = Vec::new();
    let mut fastest_idx = None;
    let mut min_time = u128::MAX;

    for (i, algo) in SearchAlgorithm::ALL.iter().enumerate() {
        let res = calculate_plan(*algo, cash, fee, portfolio, goals);
        if res.success && res.execution_time_us < min_time {
            min_time = res.execution_time_us;
            fastest_idx = Some(i);
        }
        results.push(res);
    }

    if fastest_idx.is_none() && !results.is_empty() {
        let mut min_time_all = u128::MAX;
        for (i, res) in results.iter().enumerate() {
            if res.execution_time_us < min_time_all {
                min_time_all = res.execution_time_us;
                fastest_idx = Some(i);
            }
        }
    }

    BenchmarkResult {
        results,
        fastest_idx,
        total_time_us: start_all.elapsed().as_micros(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_scenario() -> (Vec<PortfolioItem>, Vec<GoalItem>) {
        let portfolio = vec![
            PortfolioItem { symbol: "AAPL".to_string(), qty: 10, price: 180.0, sell_only: false },
            PortfolioItem { symbol: "GOOG".to_string(), qty: 5, price: 140.0, sell_only: false },
        ];
        let goals = vec![
            GoalItem { symbol: "AAPL".to_string(), min_qty: 0, max_qty: 5, price: 180.0 },
            GoalItem { symbol: "GOOG".to_string(), min_qty: 5, max_qty: 10, price: 140.0 },
            GoalItem { symbol: "MSFT".to_string(), min_qty: 2, max_qty: 5, price: 300.0 },
        ];
        (portfolio, goals)
    }

    #[test]
    fn test_astar_search() {
        let (portfolio, goals) = test_scenario();
        let result = calculate_plan(SearchAlgorithm::AStar, 1000.0, 2.0, &portfolio, &goals);
        assert!(result.success, "A* search should find a plan");
        assert!(!result.trades.is_empty());
    }

    #[test]
    fn test_greedy_search() {
        let (portfolio, goals) = test_scenario();
        let result = calculate_plan(SearchAlgorithm::GreedyBestFirst, 1000.0, 2.0, &portfolio, &goals);
        assert!(result.success, "GBFS should find a plan");
        assert!(!result.trades.is_empty());
    }

    #[test]
    fn test_bfs_search() {
        let (portfolio, goals) = test_scenario();
        let result = calculate_plan(SearchAlgorithm::BreadthFirst, 1000.0, 2.0, &portfolio, &goals);
        assert!(result.success, "BFS should find a plan");
        assert!(!result.trades.is_empty());
    }

    #[test]
    fn test_beam_search() {
        let (portfolio, goals) = test_scenario();
        let result = calculate_plan(SearchAlgorithm::BeamSearch, 1000.0, 2.0, &portfolio, &goals);
        assert!(result.success, "Beam search should find a plan");
        assert!(!result.trades.is_empty());
    }

    #[test]
    fn test_ucs_search() {
        let (portfolio, goals) = test_scenario();
        let result = calculate_plan(SearchAlgorithm::UniformCost, 1000.0, 2.0, &portfolio, &goals);
        assert!(result.success, "UCS should find a plan");
        assert!(!result.trades.is_empty());
    }

    #[test]
    fn test_dls_search() {
        let (portfolio, goals) = test_scenario();
        let result = calculate_plan(SearchAlgorithm::DepthLimited, 1000.0, 2.0, &portfolio, &goals);
        assert!(result.success, "DLS should find a plan");
        assert!(!result.trades.is_empty());
    }

    #[test]
    fn test_iddfs_search() {
        let (portfolio, goals) = test_scenario();
        let result = calculate_plan(SearchAlgorithm::IterativeDeepening, 1000.0, 2.0, &portfolio, &goals);
        assert!(result.success, "IDDFS should find a plan");
        assert!(!result.trades.is_empty());
    }

    #[test]
    fn test_all_algorithms_cycle() {
        for algo in SearchAlgorithm::ALL {
            let next = algo.next();
            assert_eq!(next.prev(), algo);
        }
    }

    #[test]
    fn test_benchmark_all_algorithms() {
        let (portfolio, goals) = test_scenario();
        let bench = benchmark_all_algorithms(1000.0, 2.0, &portfolio, &goals);
        assert_eq!(bench.results.len(), 7);
        assert!(bench.fastest_idx.is_some());
        let fastest = &bench.results[bench.fastest_idx.unwrap()];
        assert!(fastest.success);
    }
}
