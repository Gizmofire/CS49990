from dataclasses import field
import heapq
import math
import mesop as me

# --- App State Management ---
@me.stateclass
class AppState:
    # Theme & Layout State
    is_dark: bool = True
    layout_mode: str = "split" # "split", "stack"
    col1_modules: list[str] = field(default_factory=lambda: ["params", "portfolio", "telemetry"])
    col2_modules: list[str] = field(default_factory=lambda: ["goals", "engine", "audit"])
    
    # Global Parameters
    cash: str = "500.0"
    fee: str = "5.0"
    
    # Portfolio Input Form
    p_symbol: str = "AAPL"
    p_qty: str = "10"
    p_price: str = "180.0"
    p_sell_only: bool = False
    
    # Goal Input Form
    g_symbol: str = "AAPL"
    g_min: str = "129"
    g_max: str = "135"  # Shows tolerance (e.g. within ~5% of 129)
    g_price: str = "180.0"
    
    # Current Portfolio Database
    symbols: list[str] = field(default_factory=list)
    qtys: list[int] = field(default_factory=list)
    prices: list[float] = field(default_factory=list)
    sell_onlys: list[bool] = field(default_factory=list)
    
    # Target Goal Database
    g_symbols: list[str] = field(default_factory=list)
    g_mins: list[int] = field(default_factory=list)
    g_maxs: list[int] = field(default_factory=list)
    g_prices: list[float] = field(default_factory=list)
    
    result_text: str = "SYS.READY // AWAITING SPECIFICATION"

# --- Event Handlers ---
def toggle_theme(e: me.ClickEvent):
    state = me.state(AppState)
    state.is_dark = not state.is_dark
    me.set_theme_mode("dark" if state.is_dark else "light")

def set_layout_split(e: me.ClickEvent):
    me.state(AppState).layout_mode = "split"

def set_layout_stack(e: me.ClickEvent):
    me.state(AppState).layout_mode = "stack"

def move_up(e: me.ClickEvent):
    state = me.state(AppState)
    mod_id = e.key.replace("up_", "")
    if mod_id in state.col1_modules:
        idx = state.col1_modules.index(mod_id)
        if idx > 0:
            state.col1_modules[idx], state.col1_modules[idx - 1] = state.col1_modules[idx - 1], state.col1_modules[idx]
    elif mod_id in state.col2_modules:
        idx = state.col2_modules.index(mod_id)
        if idx > 0:
            state.col2_modules[idx], state.col2_modules[idx - 1] = state.col2_modules[idx - 1], state.col2_modules[idx]

def move_down(e: me.ClickEvent):
    state = me.state(AppState)
    mod_id = e.key.replace("dn_", "")
    if mod_id in state.col1_modules:
        idx = state.col1_modules.index(mod_id)
        if idx < len(state.col1_modules) - 1:
            state.col1_modules[idx], state.col1_modules[idx + 1] = state.col1_modules[idx + 1], state.col1_modules[idx]
    elif mod_id in state.col2_modules:
        idx = state.col2_modules.index(mod_id)
        if idx < len(state.col2_modules) - 1:
            state.col2_modules[idx], state.col2_modules[idx + 1] = state.col2_modules[idx + 1], state.col2_modules[idx]

def move_col(e: me.ClickEvent):
    state = me.state(AppState)
    mod_id = e.key.replace("sw_", "")
    if mod_id in state.col1_modules:
        state.col1_modules.remove(mod_id)
        state.col2_modules.append(mod_id)
    elif mod_id in state.col2_modules:
        state.col2_modules.remove(mod_id)
        state.col1_modules.append(mod_id)

def set_cash(e: me.InputEvent): me.state(AppState).cash = e.value
def set_fee(e: me.InputEvent): me.state(AppState).fee = e.value

def set_p_symbol(e: me.InputEvent): me.state(AppState).p_symbol = e.value
def set_p_qty(e: me.InputEvent): me.state(AppState).p_qty = e.value
def set_p_price(e: me.InputEvent): me.state(AppState).p_price = e.value
def set_p_sell_only(e: me.CheckboxChangeEvent): me.state(AppState).p_sell_only = e.checked

def set_g_symbol(e: me.InputEvent): me.state(AppState).g_symbol = e.value
def set_g_min(e: me.InputEvent): me.state(AppState).g_min = e.value
def set_g_max(e: me.InputEvent): me.state(AppState).g_max = e.value
def set_g_price(e: me.InputEvent): me.state(AppState).g_price = e.value

def add_portfolio(e: me.ClickEvent):
    state = me.state(AppState)
    if state.p_symbol and state.p_qty.isdigit():
        state.symbols.append(state.p_symbol.upper())
        state.qtys.append(int(state.p_qty))
        state.prices.append(float(state.p_price))
        state.sell_onlys.append(state.p_sell_only)
        state.p_symbol = ""
        state.p_qty = ""

def delete_portfolio(e: me.ClickEvent):
    state = me.state(AppState)
    idx = int(e.key.replace("del_p_", ""))
    state.symbols.pop(idx)
    state.qtys.pop(idx)
    state.prices.pop(idx)
    state.sell_onlys.pop(idx)

def add_goal(e: me.ClickEvent):
    state = me.state(AppState)
    if state.g_symbol and state.g_min.isdigit() and state.g_max.isdigit():
        state.g_symbols.append(state.g_symbol.upper())
        state.g_mins.append(int(state.g_min))
        state.g_maxs.append(int(state.g_max))
        state.g_prices.append(float(state.g_price))
        state.g_symbol = ""
        state.g_min = ""
        state.g_max = ""

def delete_goal(e: me.ClickEvent):
    state = me.state(AppState)
    idx = int(e.key.replace("del_g_", ""))
    state.g_symbols.pop(idx)
    state.g_mins.pop(idx)
    state.g_maxs.pop(idx)
    state.g_prices.pop(idx)

def clear_all(e: me.ClickEvent):
    state = me.state(AppState)
    state.symbols.clear()
    state.qtys.clear()
    state.prices.clear()
    state.sell_onlys.clear()
    state.g_symbols.clear()
    state.g_mins.clear()
    state.g_maxs.clear()
    state.g_prices.clear()
    state.result_text = "SYS.CLEARED"

def load_demo(e: me.ClickEvent):
    clear_all(e)
    state = me.state(AppState)
    state.cash = "1000.0"
    state.fee = "2.0"
    state.symbols = ["AAPL", "GOOG"]
    state.qtys = [10, 5]
    state.prices = [180.0, 140.0]
    state.sell_onlys = [False, False]
    
    state.g_symbols = ["AAPL", "GOOG", "MSFT"]
    state.g_mins = [0, 5, 2]
    state.g_maxs = [5, 10, 5]
    state.g_prices = [180.0, 140.0, 300.0]
    state.result_text = "DEMO.LOADED // AAPL REBALANCE"

# --- A* Search Logic ---
def is_goal(state_portfolio, goal_portfolio):
    state_dict = dict(state_portfolio)
    for sym, (min_q, max_q) in goal_portfolio.items():
        qty = state_dict.get(sym, 0)
        if qty < min_q or qty > max_q:
            return False
    # Ensure no extra stocks are held outside the target portfolio
    for sym, qty in state_dict.items():
        if qty > 0 and sym not in goal_portfolio:
            return False
    return True

def get_successors(cash, portfolio, prices, sell_only, fee):
    successors = []
    lots = [1, 10, 100]
    
    for sym, price in prices.items():
        for lot in lots:
            # Action: Buy
            if sym not in sell_only:
                cost = (price * lot) + fee
                if cash >= cost:
                    new_port = dict(portfolio)
                    new_port[sym] = new_port.get(sym, 0) + lot
                    if new_port[sym] == 0: del new_port[sym]
                    successors.append((cash - cost, tuple(sorted(new_port.items())), f"Buy {lot:3} {sym:5} @ ${price:6.2f}"))
            
            # Action: Sell
            if portfolio.get(sym, 0) >= lot:
                revenue = (price * lot) - fee
                new_cash = cash + revenue
                if new_cash >= 0:
                    new_port = dict(portfolio)
                    new_port[sym] -= lot
                    if new_port[sym] == 0: del new_port[sym]
                    successors.append((new_cash, tuple(sorted(new_port.items())), f"Sel {lot:3} {sym:5} @ ${price:6.2f}"))
    return successors

def heuristic(state_portfolio, goal_portfolio):
    state_dict = dict(state_portfolio)
    h = 0
    for sym, (min_q, max_q) in goal_portfolio.items():
        qty = state_dict.get(sym, 0)
        if qty < min_q:
            h += math.ceil((min_q - qty) / 100.0)
        elif qty > max_q:
            h += math.ceil((qty - max_q) / 100.0)
            
    for sym, qty in state_dict.items():
        if qty > 0 and sym not in goal_portfolio:
            h += math.ceil(qty / 100.0)
    return h

def calculate_plan(e: me.ClickEvent):
    state = me.state(AppState)
    state.result_text = "EXEC.RUNNING // OPTIMIZING"
    
    try:
        start_cash = float(state.cash)
        trade_fee = float(state.fee)
    except:
        state.result_text = "ERR.FATAL // INVALID_CASH_OR_FEE"
        return

    # Build Dictionaries
    start_portfolio = {s: q for s, q in zip(state.symbols, state.qtys) if q > 0}
    goal_portfolio = {s: (mn, mx) for s, mn, mx in zip(state.g_symbols, state.g_mins, state.g_maxs)}
    sell_only_set = {s for s, so in zip(state.symbols, state.sell_onlys) if so}
    
    # Consolidate standard pricing
    global_prices = {s: p for s, p in zip(state.symbols, state.prices)}
    for s, p in zip(state.g_symbols, state.g_prices):
        if s not in global_prices:
            global_prices[s] = p

    start_port_tuple = tuple(sorted(start_portfolio.items()))
    
    # Priority Queue setup: (f_score, g_score, counter, state, path)
    counter = 0
    open_set = []
    heapq.heappush(open_set, (0, 0, counter, (start_cash, start_port_tuple), []))
    
    # Cash pruning: store max cash seen for a specific portfolio arrangement
    best_cash_for_port = {start_port_tuple: start_cash}
    
    while open_set and counter < 100000:
        f, g, _, current_state, path = heapq.heappop(open_set)
        curr_cash, curr_port = current_state
        
        if is_goal(curr_port, goal_portfolio):
            total_fee = len(path) * trade_fee
            res = f"[SUCCESS] OPTIMAL TRAJECTORY FOUND\n"
            res += f"TRADES: {len(path)} | FEES: ${total_fee:.2f} | FINAL CASH: ${curr_cash:.2f}\n"
            res += "=" * 50 + "\n"
            for i, step in enumerate(path):
                res += f"[{i+1:02d}] {step}\n"
            state.result_text = res
            return
            
        for nxt_cash, nxt_port, action in get_successors(curr_cash, dict(curr_port), global_prices, sell_only_set, trade_fee):
            if nxt_port in best_cash_for_port and best_cash_for_port[nxt_port] >= nxt_cash:
                continue # Prune: we've achieved this portfolio before with equal or better cash
                
            best_cash_for_port[nxt_port] = nxt_cash
            new_g = g + 1
            new_h = heuristic(nxt_port, goal_portfolio)
            new_f = new_g + new_h
            
            counter += 1
            heapq.heappush(open_set, (new_f, new_g, counter, (nxt_cash, nxt_port), path + [action]))
            
    state.result_text = "ERR.TIMEOUT // NO_VALID_PLAN_WITHIN_LIMITS"

# --- UI Helpers & Styling ---

def get_theme():
    state = me.state(AppState)
    is_dark = state.is_dark
    return {
        "bg_app": "#0a0a0a" if is_dark else "#f0f0f0",
        "bg_panel": "#141414" if is_dark else "#ffffff",
        "bg_header": "#1c1c1c" if is_dark else "#e5e5e5",
        "bg_row_alt": "#111111" if is_dark else "#f9f9f9",
        "border": "#333333" if is_dark else "#cccccc",
        "text_primary": "#ffffff" if is_dark else "#111111",
        "text_secondary": "#aaaaaa" if is_dark else "#555555",
        "accent_bg": "#ffffff" if is_dark else "#111111",
        "accent_text": "#000000" if is_dark else "#ffffff",
        "btn_bg": "#222222" if is_dark else "#e0e0e0",
        "btn_hover": "#333333" if is_dark else "#cccccc",
    }

def terminal_header():
    theme = get_theme()
    with me.box(style=me.Style(
        display="flex", justify_content="space-between", align_items="center",
        background=theme["bg_header"], color=theme["text_primary"],
        padding=me.Padding.all(12), border=me.Border(bottom=me.BorderSide(width=2, color=theme["border"], style="solid"))
    )):
        me.text("Gordon's::TRADE_PLANNER_OS // V2.0", style=me.Style(font_weight="bold", font_size="16px", letter_spacing="1px"))
        
        with me.box(style=me.Style(display="flex", gap=16, align_items="center")):
            # Commands
            me.button("CLR<GO>", on_click=clear_all, style=me.Style(color=theme["text_primary"], background=theme["btn_bg"]))
            me.button("DEMO<GO>", on_click=load_demo, style=me.Style(color=theme["text_primary"], background=theme["btn_bg"]))
            me.button("THEME<GO>", on_click=toggle_theme, style=me.Style(color=theme["text_primary"], background=theme["btn_bg"]))
            
            # Layout Toggle
            layout_text = "STACKED" if me.state(AppState).layout_mode == "stack" else "SPLIT"
            def toggle_layout(e):
                s = me.state(AppState)
                s.layout_mode = "stack" if s.layout_mode == "split" else "split"
            me.button(f"LAY:{layout_text}<GO>", on_click=toggle_layout, style=me.Style(color=theme["text_primary"], background=theme["btn_bg"]))

def module_wrapper(mod_id: str, title: str, content_fn):
    theme = get_theme()
    with me.box(style=me.Style(
        background=theme["bg_panel"],
        border=me.Border.all(me.BorderSide(width=1, color=theme["border"], style="solid")),
        margin=me.Margin(bottom=16),
        display="flex", flex_direction="column"
    )):
        # Module Header
        with me.box(style=me.Style(
            display="flex", justify_content="space-between", align_items="center",
            background=theme["bg_header"], padding=me.Padding(top=8, bottom=8, left=12, right=12),
            border=me.Border(bottom=me.BorderSide(width=1, color=theme["border"], style="solid"))
        )):
            me.text(f"[{mod_id.upper()}] {title}", style=me.Style(font_weight="bold", color=theme["text_primary"]))
            
            # Controls
            with me.box(style=me.Style(display="flex", gap=4)):
                me.button("▲", key=f"up_{mod_id}", on_click=move_up, style=me.Style(padding=me.Padding.all(4), background=theme["btn_bg"], color=theme["text_primary"]))
                me.button("▼", key=f"dn_{mod_id}", on_click=move_down, style=me.Style(padding=me.Padding.all(4), background=theme["btn_bg"], color=theme["text_primary"]))
                me.button("⇄", key=f"sw_{mod_id}", on_click=move_col, style=me.Style(padding=me.Padding.all(4), background=theme["btn_bg"], color=theme["text_primary"]))
        
        # Module Content
        with me.box(style=me.Style(padding=me.Padding.all(16))):
            content_fn(theme)

# --- Modules ---
def render_params(theme):
    state = me.state(AppState)
    with me.box(style=me.Style(display="flex", gap=16)):
        me.input(label="CASH ($)", value=state.cash, on_input=set_cash, style=me.Style(width="50%"))
        me.input(label="FEE ($)", value=state.fee, on_input=set_fee, style=me.Style(width="50%"))

def render_portfolio(theme):
    state = me.state(AppState)
    
    # Table Header
    with me.box(style=me.Style(display="grid", grid_template_columns="2fr 1fr 1.5fr 1fr 0.5fr", gap=8, margin=me.Margin(bottom=8), color=theme["text_secondary"])):
        me.text("SYM")
        me.text("QTY")
        me.text("PX")
        me.text("SELL")
        me.text("ACT")
        
    # Table Rows
    for i, (s, q, p, so) in enumerate(zip(state.symbols, state.qtys, state.prices, state.sell_onlys)):
        bg = theme["bg_panel"] if i % 2 == 0 else theme["bg_row_alt"]
        with me.box(style=me.Style(display="grid", grid_template_columns="2fr 1fr 1.5fr 1fr 0.5fr", gap=8, padding=me.Padding.all(4), background=bg, align_items="center")):
            me.text(s, style=me.Style(color=theme["text_primary"]))
            me.text(str(q), style=me.Style(color=theme["text_primary"]))
            me.text(f"{p:.2f}", style=me.Style(color=theme["text_primary"]))
            me.text("Y" if so else "-", style=me.Style(color=theme["text_primary"]))
            me.button("✕", key=f"del_p_{i}", on_click=delete_portfolio, style=me.Style(color=theme["text_secondary"]))
            
    me.box(style=me.Style(height="1px", background=theme["border"], margin=me.Margin.symmetric(vertical=16)))
    
    # Add Form
    with me.box(style=me.Style(display="flex", gap=8, align_items="center")):
        me.input(label="SYM", value=state.p_symbol, on_input=set_p_symbol, style=me.Style(width="25%"))
        me.input(label="QTY", value=state.p_qty, on_input=set_p_qty, style=me.Style(width="20%"))
        me.input(label="PX", value=state.p_price, on_input=set_p_price, style=me.Style(width="25%"))
        with me.box(style=me.Style(display="flex", align_items="center", gap=4, width="15%")):
            me.checkbox(checked=state.p_sell_only, on_change=set_p_sell_only)
            me.text("SELL", style=me.Style(font_size="12px", color=theme["text_secondary"]))
        me.button("ADD", on_click=add_portfolio, style=me.Style(background=theme["accent_bg"], color=theme["accent_text"], width="15%"))

def render_goals(theme):
    state = me.state(AppState)
    
    with me.box(style=me.Style(display="grid", grid_template_columns="1.5fr 1.5fr 1.5fr 0.5fr", gap=8, margin=me.Margin(bottom=8), color=theme["text_secondary"])):
        me.text("SYM")
        me.text("BAND")
        me.text("PX")
        me.text("ACT")
        
    for i, (s, mn, mx, p) in enumerate(zip(state.g_symbols, state.g_mins, state.g_maxs, state.g_prices)):
        bg = theme["bg_panel"] if i % 2 == 0 else theme["bg_row_alt"]
        with me.box(style=me.Style(display="grid", grid_template_columns="1.5fr 1.5fr 1.5fr 0.5fr", gap=8, padding=me.Padding.all(4), background=bg, align_items="center")):
            me.text(s, style=me.Style(color=theme["text_primary"]))
            me.text(f"[{mn},{mx}]", style=me.Style(color=theme["text_primary"]))
            me.text(f"{p:.2f}", style=me.Style(color=theme["text_primary"]))
            me.button("✕", key=f"del_g_{i}", on_click=delete_goal, style=me.Style(color=theme["text_secondary"]))

    me.box(style=me.Style(height="1px", background=theme["border"], margin=me.Margin.symmetric(vertical=16)))
    
    with me.box(style=me.Style(display="flex", gap=8, align_items="center")):
        me.input(label="SYM", value=state.g_symbol, on_input=set_g_symbol, style=me.Style(width="25%"))
        me.input(label="MIN", value=state.g_min, on_input=set_g_min, style=me.Style(width="20%"))
        me.input(label="MAX", value=state.g_max, on_input=set_g_max, style=me.Style(width="20%"))
        me.input(label="PX", value=state.g_price, on_input=set_g_price, style=me.Style(width="20%"))
        me.button("ADD", on_click=add_goal, style=me.Style(background=theme["accent_bg"], color=theme["accent_text"], width="15%"))

def render_engine(theme):
    with me.box(style=me.Style(display="flex", flex_direction="column", gap=16)):
        me.button("RUN PLANNER <GO>", on_click=calculate_plan, style=me.Style(
            background=theme["accent_bg"], color=theme["accent_text"], padding=me.Padding.all(24), font_size="18px"
        ))

def render_audit(theme):
    state = me.state(AppState)
    with me.box(style=me.Style(
        background="#050505" if state.is_dark else "#f0f0f0",
        padding=me.Padding.all(16),
        border=me.Border.all(me.BorderSide(width=1, color=theme["border"], style="solid")),
    )):
        me.text(state.result_text, style=me.Style(
            white_space="pre-wrap", color="#00ff00" if state.is_dark else "#008800" if "SUCCESS" in state.result_text else theme["text_primary"], font_size="13px", line_height="1.5"
        ))

def render_telemetry(theme):
    state = me.state(AppState)
    
    val = sum(q * p for q, p in zip(state.qtys, state.prices))
    cash = float(state.cash) if state.cash else 0.0
    net = val + cash
    
    with me.box(style=me.Style(display="grid", grid_template_columns="1fr 1fr", gap=16)):
        with me.box(style=me.Style(background=theme["bg_row_alt"], padding=me.Padding.all(12), border=me.Border.all(me.BorderSide(width=1, color=theme["border"], style="solid")))):
            me.text("PORT VALUE", style=me.Style(color=theme["text_secondary"], font_size="12px"))
            me.text(f"${val:,.2f}", style=me.Style(color=theme["text_primary"], font_size="20px", font_weight="bold"))
        with me.box(style=me.Style(background=theme["bg_row_alt"], padding=me.Padding.all(12), border=me.Border.all(me.BorderSide(width=1, color=theme["border"], style="solid")))):
            me.text("CASH AVAIL", style=me.Style(color=theme["text_secondary"], font_size="12px"))
            me.text(f"${cash:,.2f}", style=me.Style(color=theme["text_primary"], font_size="20px", font_weight="bold"))
        with me.box(style=me.Style(background=theme["bg_row_alt"], padding=me.Padding.all(12), border=me.Border.all(me.BorderSide(width=1, color=theme["border"], style="solid")))):
            me.text("NET WORTH", style=me.Style(color=theme["text_secondary"], font_size="12px"))
            me.text(f"${net:,.2f}", style=me.Style(color=theme["text_primary"], font_size="20px", font_weight="bold"))
        with me.box(style=me.Style(background=theme["bg_row_alt"], padding=me.Padding.all(12), border=me.Border.all(me.BorderSide(width=1, color=theme["border"], style="solid")))):
            me.text("GOALS", style=me.Style(color=theme["text_secondary"], font_size="12px"))
            me.text(str(len(state.g_symbols)), style=me.Style(color=theme["text_primary"], font_size="20px", font_weight="bold"))

# --- Rendering Dispatcher ---
def get_module_renderer(mod_id: str):
    if mod_id == "params": return "CONFIG_PARAMS", render_params
    if mod_id == "portfolio": return "PORTFOLIO", render_portfolio
    if mod_id == "goals": return "GOALS", render_goals
    if mod_id == "engine": return "A_STAR_ENGINE", render_engine
    if mod_id == "audit": return "TRACE_AUDIT", render_audit
    if mod_id == "telemetry": return "TELEMETRY", render_telemetry
    return "UNKNOWN", lambda t: me.text("Unknown Module")

# --- User Interface ---
@me.page(path="/", stylesheets=["/static/terminal.css"])
def trade_planner_app():
    state = me.state(AppState)
    theme = get_theme()
    
    with me.box(style=me.Style(
        background=theme["bg_app"], 
        color=theme["text_primary"],
        min_height="100vh",
        display="flex", flex_direction="column"
    )):
        terminal_header()
        
        # Main Workspace
        with me.box(style=me.Style(padding=me.Padding.all(24), display="flex", flex_direction="column", gap=24, flex_grow=1)):
            if state.layout_mode == "split":
                with me.box(style=me.Style(display="flex", gap=24, flex_wrap="wrap")):
                    with me.box(style=me.Style(flex_basis="calc(50% - 12px)", flex_grow=1, display="flex", flex_direction="column")):
                        for mod_id in state.col1_modules:
                            title, fn = get_module_renderer(mod_id)
                            module_wrapper(mod_id, title, fn)
                    with me.box(style=me.Style(flex_basis="calc(50% - 12px)", flex_grow=1, display="flex", flex_direction="column")):
                        for mod_id in state.col2_modules:
                            title, fn = get_module_renderer(mod_id)
                            module_wrapper(mod_id, title, fn)
            else: # Stacked mode
                with me.box(style=me.Style(display="flex", flex_direction="column")):
                    for mod_id in state.col1_modules + state.col2_modules:
                        title, fn = get_module_renderer(mod_id)
                        module_wrapper(mod_id, title, fn)