use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Wrap,
    },
    Frame,
};

use crate::app::{App, LayoutMode, ModalMode, ModuleId};

pub struct ThemeColors {
    pub bg: Color,
    pub panel_bg: Color,
    pub header_bg: Color,
    pub border: Color,
    pub border_active: Color,
    pub text: Color,
    pub text_dim: Color,
    pub accent: Color,
    pub highlight_bg: Color,
    pub success: Color,
    pub info: Color,
}

impl ThemeColors {
    pub fn active_border_flash(&self, is_dark: bool, is_bright: bool) -> Color {
        if is_bright {
            self.border_active
        } else if is_dark {
            Color::Rgb(160, 160, 160)
        } else {
            Color::Rgb(100, 100, 100)
        }
    }

    pub fn live_indicator_color(&self, is_dark: bool, is_bright: bool) -> Color {
        if is_dark {
            if is_bright {
                Color::Rgb(100, 255, 120)
            } else {
                Color::Rgb(40, 110, 50)
            }
        } else if is_bright {
            Color::Rgb(0, 180, 0)
        } else {
            Color::Rgb(140, 190, 140)
        }
    }
}

pub fn get_theme(is_dark: bool) -> ThemeColors {
    if is_dark {
        ThemeColors {
            bg: Color::Rgb(10, 10, 10),
            panel_bg: Color::Rgb(18, 18, 18),
            header_bg: Color::Rgb(28, 28, 28),
            border: Color::Rgb(60, 60, 60),
            border_active: Color::Rgb(220, 220, 220),
            text: Color::Rgb(240, 240, 240),
            text_dim: Color::Rgb(140, 140, 140),
            accent: Color::Rgb(255, 255, 255),
            highlight_bg: Color::Rgb(40, 40, 40),
            success: Color::Rgb(100, 255, 100),
            info: Color::Rgb(140, 200, 255),
        }
    } else {
        ThemeColors {
            bg: Color::Rgb(240, 240, 240),
            panel_bg: Color::Rgb(255, 255, 255),
            header_bg: Color::Rgb(225, 225, 225),
            border: Color::Rgb(180, 180, 180),
            border_active: Color::Rgb(20, 20, 20),
            text: Color::Rgb(20, 20, 20),
            text_dim: Color::Rgb(90, 90, 90),
            accent: Color::Rgb(0, 0, 0),
            highlight_bg: Color::Rgb(215, 215, 215),
            success: Color::Rgb(0, 140, 0),
            info: Color::Rgb(0, 80, 180),
        }
    }
}

pub fn render_ui(frame: &mut Frame, app: &App) {
    let theme = get_theme(app.is_dark);
    let size = frame.area();

    // Background fill
    let bg_block = Block::default().style(Style::default().bg(theme.bg));
    frame.render_widget(bg_block, size);

    // Root layout: Header, Workspace, Footer Status Bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top Command Header
            Constraint::Min(5),    // Workspace Panels
            Constraint::Length(2), // Bottom Status / Keybinding Legend
        ])
        .split(size);

    render_header(frame, main_chunks[0], app, &theme);
    render_workspace(frame, main_chunks[1], app, &theme);
    render_status_bar(frame, main_chunks[2], app, &theme);

    // If a modal is active, render it over the workspace
    if app.modal_mode != ModalMode::None {
        render_modal(frame, size, app, &theme);
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let active_mod_str = app
        .active_module()
        .map(|m| m.code())
        .unwrap_or("NONE");

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),    // Left command buttons & status
            Constraint::Length(26), // Right Timer & Live Pulse Indicator
        ])
        .split(area);

    let is_bright = app.is_flash_bright();

    let header_line = Line::from(vec![
        Span::styled(
            " BBG::TRADE_PLANNER // RUST_V2 ",
            Style::default()
                .fg(theme.bg)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled("[A] ADD<GO> ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("[F1] CLR ", Style::default().fg(theme.text)),
        Span::styled("[F2] PRESET ", Style::default().fg(theme.text)),
        Span::styled(
            format!("[F3] THEME:{} ", if app.is_dark { "DARK" } else { "LIGHT" }),
            Style::default().fg(theme.text),
        ),
        Span::styled(
            format!("[F4] LAY:{:?} ", app.layout_mode),
            Style::default().fg(theme.text),
        ),
        Span::styled(
            format!("[F6] ALGO:{} ", app.algorithm.code()),
            Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
        ),
        if app.is_searching {
            Span::styled(
                format!("[F5] {} RUNNING{} ", app.spinner_frame(), app.running_dots_str()),
                Style::default()
                    .fg(theme.info)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                "[F5] RUN<GO> ",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            )
        },
        Span::styled("[F7] BENCH<GO> ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(
            if app.move_mode {
                "[M] MOVE:ON "
            } else {
                "[M] MOVE "
            },
            Style::default().fg(if app.move_mode {
                theme.success
            } else {
                theme.text_dim
            }),
        ),
        Span::styled(
            format!("| ACTIVE:[{}]", active_mod_str),
            Style::default().fg(theme.text_dim),
        ),
    ]);

    let left_header_widget = Paragraph::new(header_line)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.header_bg)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(left_header_widget, header_chunks[0]);

    // Top Right Timer & Live Pulse Indicator Block
    let timer_line = Line::from(vec![
        Span::styled(
            if is_bright { "● " } else { "○ " },
            Style::default()
                .fg(theme.live_indicator_color(app.is_dark, is_bright))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("LIVE ", Style::default().fg(theme.text_dim)),
        Span::styled(
            format!("⏱ {} ", app.elapsed_timer_str()),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let right_timer_widget = Paragraph::new(timer_line)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.header_bg)),
        )
        .alignment(Alignment::Right);

    frame.render_widget(right_timer_widget, header_chunks[1]);
}

fn render_workspace(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    match app.layout_mode {
        LayoutMode::Split => {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            render_module_column(frame, cols[0], &app.col1_modules, app, theme);
            render_module_column(frame, cols[1], &app.col2_modules, app, theme);
        }
        LayoutMode::Stacked => {
            let mut all_mods = app.col1_modules.clone();
            all_mods.extend(app.col2_modules.clone());
            render_module_column(frame, area, &all_mods, app, theme);
        }
    }
}

fn render_module_column(
    frame: &mut Frame,
    area: Rect,
    modules: &[ModuleId],
    app: &App,
    theme: &ThemeColors,
) {
    if modules.is_empty() {
        let empty_p = Paragraph::new("--- EMPTY COLUMN (USE [S] IN MOVE MODE TO SHIFT HERE) ---")
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(theme.border)),
            );
        frame.render_widget(empty_p, area);
        return;
    }

    let constraints: Vec<Constraint> = modules
        .iter()
        .map(|m| match m {
            ModuleId::Params => Constraint::Length(5),
            ModuleId::Portfolio => Constraint::Min(8),
            ModuleId::Goals => Constraint::Min(8),
            ModuleId::Telemetry => Constraint::Length(6),
            ModuleId::Engine => Constraint::Length(6),
            ModuleId::Audit => Constraint::Min(10),
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (i, &mod_id) in modules.iter().enumerate() {
        if i < chunks.len() {
            render_module(frame, chunks[i], mod_id, app, theme);
        }
    }
}

fn render_module(
    frame: &mut Frame,
    area: Rect,
    mod_id: ModuleId,
    app: &App,
    theme: &ThemeColors,
) {
    let is_active = app.active_module() == Some(mod_id);
    let is_bright = app.is_flash_bright();

    let border_color = if is_active {
        theme.active_border_flash(app.is_dark, is_bright)
    } else {
        theme.border
    };

    let title_line = Line::from(vec![
        if is_active {
            Span::styled(
                if is_bright { " [● " } else { " [○ " },
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(" [", Style::default().fg(theme.text_dim))
        },
        Span::styled(
            format!("{}] {} ", mod_id.code(), mod_id.title()),
            Style::default()
                .fg(if is_active { theme.accent } else { theme.text })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" [▲ U] [▼ D] [⇄ S] ", Style::default().fg(theme.text_dim)),
    ]);

    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.panel_bg));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    match mod_id {
        ModuleId::Params => render_params_content(frame, inner_area, app, theme),
        ModuleId::Portfolio => render_portfolio_content(frame, inner_area, app, theme),
        ModuleId::Goals => render_goals_content(frame, inner_area, app, theme),
        ModuleId::Telemetry => render_telemetry_content(frame, inner_area, app, theme),
        ModuleId::Engine => render_engine_content(frame, inner_area, app, theme),
        ModuleId::Audit => render_audit_content(frame, inner_area, app, theme),
    }
}

fn render_params_content(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    theme: &ThemeColors,
) {
    let text = vec![
        Line::from(vec![
            Span::styled("  AVAILABLE CASH : ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("${:.2}", app.cash),
                Style::default()
                    .fg(theme.text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("    TRANSACTION FEE : ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("${:.2}", app.fee),
                Style::default()
                    .fg(theme.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  [+/-] Quick Cash | [</>] Quick Fee | [E] Edit Parameters | [B] Cycle Presets",
                Style::default().fg(theme.text_dim),
            ),
        ]),
    ];

    let p = Paragraph::new(text);
    frame.render_widget(p, area);
}

fn render_portfolio_content(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    theme: &ThemeColors,
) {
    let header_cells = ["SYM", "QTY", "PRICE ($)", "SELL ONLY", "VAL ($)"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(theme.text_dim).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.portfolio.iter().enumerate().map(|(i, item)| {
        let val = (item.qty as f64) * item.price;
        let is_sel = i == app.selected_port_idx;
        let style = if is_sel {
            Style::default()
                .bg(theme.highlight_bg)
                .fg(theme.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };

        let cells = vec![
            Cell::from(item.symbol.clone()),
            Cell::from(item.qty.to_string()),
            Cell::from(format!("{:.2}", item.price)),
            Cell::from(if item.sell_only { "YES" } else { "-" }),
            Cell::from(format!("{:.2}", val)),
        ];
        Row::new(cells).style(style).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ],
    )
    .header(header);

    frame.render_widget(table, area);
}

fn render_goals_content(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    theme: &ThemeColors,
) {
    let header_cells = ["SYM", "TARGET BAND [MIN, MAX]", "PRICE ($)"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(theme.text_dim).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.goals.iter().enumerate().map(|(i, item)| {
        let is_sel = i == app.selected_goal_idx;
        let style = if is_sel {
            Style::default()
                .bg(theme.highlight_bg)
                .fg(theme.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };

        let cells = vec![
            Cell::from(item.symbol.clone()),
            Cell::from(format!("[{}, {}]", item.min_qty, item.max_qty)),
            Cell::from(format!("{:.2}", item.price)),
        ];
        Row::new(cells).style(style).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(26),
            Constraint::Length(12),
        ],
    )
    .header(header);

    frame.render_widget(table, area);
}

fn format_money(val: f64) -> String {
    let s = format!("{:.2}", val);
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let dec_part = if parts.len() > 1 { parts[1] } else { "00" };
    let mut result = String::new();
    let chars: Vec<char> = int_part.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 && *c != '-' {
            result.push(',');
        }
        result.push(*c);
    }
    format!("${}.{}", result, dec_part)
}

fn render_telemetry_content(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    theme: &ThemeColors,
) {
    let port_val: f64 = app.portfolio.iter().map(|p| (p.qty as f64) * p.price).sum();
    let net_worth = port_val + app.cash;

    let text = vec![
        Line::from(vec![
            Span::styled("  PORT VALUE : ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format_money(port_val),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    NET WORTH  : ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format_money(net_worth),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  HOLDINGS   : ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{} ASSETS", app.portfolio.len()),
                Style::default().fg(theme.text),
            ),
            Span::styled("         TARGET GOALS : ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{} TARGETS", app.goals.len()),
                Style::default().fg(theme.text),
            ),
        ]),
    ];

    let p = Paragraph::new(text);
    frame.render_widget(p, area);
}

fn render_engine_content(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    theme: &ThemeColors,
) {
    let lines = if app.is_searching {
        let bar_width = (area.width.saturating_sub(22) as usize).clamp(10, 36);
        let progress_bar = app.running_progress_bar(bar_width);
        let dots = app.running_dots_str();
        let spinner = app.spinner_frame();
        let elapsed = app.search_elapsed_str();

        vec![
            Line::from(vec![
                Span::styled(format!("  {} SEARCH ENGINE RUNNING{} ", spinner, dots), Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
                Span::styled(format!("[ALGO: {}]", app.algorithm.code()), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  EVALUATING : ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("[{}] ", progress_bar), Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
                Span::styled("NON-BLOCKING WORKER", Style::default().fg(theme.text_dim)),
            ]),
            Line::from(vec![
                Span::styled(format!("  RUNTIME    : ⏱ {} ", elapsed), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(" | UI THREAD: 100% RESPONSIVE", Style::default().fg(theme.success)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("  ALGORITHM: ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    format!("[{}] {}", app.algorithm.code(), app.algorithm.name()),
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  [F6 / O TO CYCLE]", Style::default().fg(theme.text_dim)),
            ]),
            Line::from(vec![
                Span::styled(
                    format!("  ℹ {}", app.algorithm.description()),
                    Style::default().fg(theme.text_dim),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  ▶ [PRESS F5 OR R TO EXECUTE TRADE PLANNER] ◀",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ]
    };

    let p = Paragraph::new(lines);
    frame.render_widget(p, area);
}

fn render_audit_content(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    theme: &ThemeColors,
) {
    let mut lines = Vec::new();

    if app.is_searching {
        let spinner = app.spinner_frame();
        let dots = app.running_dots_str();
        let elapsed = app.search_elapsed_str();
        let bar_width = (area.width.saturating_sub(26) as usize).clamp(10, 36);
        let progress_bar = app.running_progress_bar(bar_width);

        lines.push(Line::from(vec![
            Span::styled(format!(">> {} [RUNNING{}] ", spinner, dots), Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("COMPUTING TRADE REBALANCING WITH [{}]", app.algorithm.code()),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!(
                "   STATUS: ASYNC WORKER ACTIVE | ELAPSED: {} | UI INTERACTIVE: YES",
                elapsed
            ),
            Style::default().fg(theme.text_dim),
        )));
        lines.push(Line::from(vec![
            Span::styled("   FRONTIER: ", Style::default().fg(theme.text_dim)),
            Span::styled(format!("[{}] ", progress_bar), Style::default().fg(theme.info)),
            Span::styled("SEARCHING EXPANDED GRAPH NODES...", Style::default().fg(theme.text_dim)),
        ]));
        lines.push(Line::from(Span::styled(
            "===========================================================",
            Style::default().fg(theme.border),
        )));
        lines.push(Line::from(Span::styled(
            "   CALCULATING OPTIMAL PATH & MINIMIZING TRANSACTION FRICTION...",
            Style::default().fg(theme.text_dim),
        )));
    } else if let Some(bench) = &app.last_benchmark {
        let total_time_str = if bench.total_time_us > 1000 {
            format!("{:.2} ms", (bench.total_time_us as f64) / 1000.0)
        } else {
            format!("{} µs", bench.total_time_us)
        };

        lines.push(Line::from(vec![
            Span::styled(">> [BENCHMARK RESULTS] ", Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("7 SEARCH ALGORITHMS EVALUATED | TOTAL BENCHMARK: {}", total_time_str),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]));

        if let Some(idx) = bench.fastest_idx {
            let fastest = &bench.results[idx];
            let f_time_str = if fastest.execution_time_us > 1000 {
                format!("{:.2} ms", (fastest.execution_time_us as f64) / 1000.0)
            } else {
                format!("{} µs", fastest.execution_time_us)
            };

            lines.push(Line::from(vec![
                Span::styled("   ★ FASTEST ALGORITHM : ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("[{}] {} ", fastest.algorithm.code(), fastest.algorithm.name()),
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("(TIME: {} | NODES: {} | TRADES: {})", f_time_str, fastest.nodes_evaluated, fastest.trades.len()),
                    Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        lines.push(Line::from(Span::styled(
            "=================================================================================",
            Style::default().fg(theme.border),
        )));

        // Comparative Table Header
        lines.push(Line::from(vec![
            Span::styled("   ALGO   ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("STATUS  ", Style::default().fg(theme.text_dim)),
            Span::styled("RUNTIME   ", Style::default().fg(theme.text_dim)),
            Span::styled("NODES ", Style::default().fg(theme.text_dim)),
            Span::styled("TRADES ", Style::default().fg(theme.text_dim)),
            Span::styled("FEES    ", Style::default().fg(theme.text_dim)),
            Span::styled("FINAL CASH ", Style::default().fg(theme.text_dim)),
            Span::styled("BENCHMARK RESULT", Style::default().fg(theme.text_dim)),
        ]));
        lines.push(Line::from(Span::styled(
            "   -----------------------------------------------------------------------------",
            Style::default().fg(theme.border),
        )));

        for (i, res) in bench.results.iter().enumerate() {
            let is_fastest = bench.fastest_idx == Some(i);
            let time_str = if res.execution_time_us > 1000 {
                format!("{:.2} ms", (res.execution_time_us as f64) / 1000.0)
            } else {
                format!("{} µs", res.execution_time_us)
            };

            let status_badge = if res.success {
                Span::styled("OK      ", Style::default().fg(theme.success))
            } else {
                Span::styled("FAIL    ", Style::default().fg(Color::Red))
            };

            let fastest_marker = if is_fastest {
                Span::styled(" ★ FASTEST (WINNER)", Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("", Style::default().fg(theme.text_dim))
            };

            lines.push(Line::from(vec![
                Span::styled(format!("   {:<7}", res.algorithm.code()), if is_fastest { Style::default().fg(theme.accent).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.text) }),
                status_badge,
                Span::styled(format!("{:<10}", time_str), if is_fastest { Style::default().fg(theme.success).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.text) }),
                Span::styled(format!("{:<6}", res.nodes_evaluated), Style::default().fg(theme.text_dim)),
                Span::styled(format!("{:<7}", res.trades.len()), Style::default().fg(theme.text_dim)),
                Span::styled(format!("${:<7.2}", res.total_fee), Style::default().fg(theme.text_dim)),
                Span::styled(format!("${:<10.2}", res.final_cash), Style::default().fg(theme.text_dim)),
                fastest_marker,
            ]));
        }

        // Detailed Trade Execution Plan for the Fastest Algorithm
        if let Some(idx) = bench.fastest_idx {
            let fastest = &bench.results[idx];
            lines.push(Line::from(Span::styled(
                "=================================================================================",
                Style::default().fg(theme.border),
            )));
            lines.push(Line::from(vec![
                Span::styled(">> FASTEST RESULT PLAN: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("[{}] {} ({} TRADES | FINAL CASH: ${:.2})", fastest.algorithm.code(), fastest.algorithm.name(), fastest.trades.len(), fastest.final_cash),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ]));
            if fastest.trades.is_empty() {
                lines.push(Line::from(Span::styled(
                    "    (PORTFOLIO SATISFIES TARGET GOALS // ZERO TRADES REQUIRED)",
                    Style::default().fg(theme.text_dim),
                )));
            } else {
                for (t_idx, t) in fastest.trades.iter().enumerate() {
                    lines.push(Line::from(vec![
                        Span::styled(format!("    [{:02}] ", t_idx + 1), Style::default().fg(theme.text_dim)),
                        Span::styled(t.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }
    } else if let Some(res) = &app.last_result {
        let time_str = if res.execution_time_us > 1000 {
            format!("{:.2} ms", (res.execution_time_us as f64) / 1000.0)
        } else {
            format!("{} µs", res.execution_time_us)
        };

        if res.success {
            lines.push(Line::from(vec![
                Span::styled(">> [SUCCESS] ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("ALGO: [{}] {} | TRADES: {}", res.algorithm.code(), res.algorithm.name(), res.trades.len()),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                format!(
                    "   FEES: ${:.2} | FINAL CASH: ${:.2} | NODES EVAL: {} | RUNTIME: {}",
                    res.total_fee, res.final_cash, res.nodes_evaluated, time_str
                ),
                Style::default().fg(theme.text_dim),
            )));
            lines.push(Line::from(Span::styled(
                "===========================================================",
                Style::default().fg(theme.border),
            )));
            for (i, t) in res.trades.iter().enumerate() {
                lines.push(Line::from(vec![
                    Span::styled(format!(" [{:02}] ", i + 1), Style::default().fg(theme.text_dim)),
                    Span::styled(t.clone(), Style::default().fg(theme.text)),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled(">> [FAILURE] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("ALGO: [{}] | NODES EVAL: {} | TIME: {}", res.algorithm.code(), res.nodes_evaluated, time_str),
                    Style::default().fg(theme.text_dim),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                format!("   REASON: {}", res.message),
                Style::default().fg(Color::Red),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            ">> AWAITING EXECUTION COMMAND... (PRESS F5 / R TO RUN | F7 / W TO TEST ALL)",
            Style::default().fg(theme.text_dim),
        )));
    }

    let p = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let cursor_span = if app.is_flash_bright() {
        Span::styled(" █", Style::default().fg(theme.accent))
    } else {
        Span::styled("  ", Style::default().fg(theme.text_dim))
    };

    let status_prefix = if app.is_searching {
        format!(" STATUS: {} ", app.spinner_frame())
    } else {
        " STATUS: ".to_string()
    };

    let status_line = Line::from(vec![
        Span::styled(
            format!("{}{}", status_prefix, app.status_message),
            Style::default()
                .fg(theme.text)
                .bg(theme.header_bg)
                .add_modifier(Modifier::BOLD),
        ),
        cursor_span,
        Span::styled(" | ", Style::default().fg(theme.border)),
        Span::styled(
            "[A] ADD | [Enter] EDIT | [X] DEL | [O/F6] ALGO | [B] PRESET | [F7/W] TEST ALL | [Q] QUIT",
            Style::default().fg(theme.text_dim),
        ),
    ]);

    let p = Paragraph::new(status_line).style(Style::default().bg(theme.header_bg));
    frame.render_widget(p, area);
}

fn render_modal(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let modal_width = 66.min(area.width.saturating_sub(4));
    let modal_height = 14.min(area.height.saturating_sub(2));

    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    let is_bright = app.is_flash_bright();
    let flash_pointer = if is_bright { " ◀" } else { " ◁" };

    frame.render_widget(Clear, modal_area);

    let (title, content) = match app.modal_mode {
        ModalMode::AddPortfolio => {
            let title = if app.form.is_editing {
                format!(" [ ✎ EDIT PORTFOLIO POSITION: {} ] ", app.form.symbol)
            } else {
                " [ + ADD NEW PORTFOLIO POSITION ] ".to_string()
            };
            let mut lines = Vec::new();
            lines.push(Line::from(""));
            
            // Field 0: Symbol
            let f0_active = app.form.field_idx == 0;
            lines.push(Line::from(vec![
                Span::styled("   TICKER SYMBOL   : ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    if app.form.symbol.is_empty() && !f0_active {
                        "[ e.g. NVDA ]"
                    } else {
                        &app.form.symbol
                    },
                    if f0_active {
                        Style::default().bg(theme.highlight_bg).fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                if f0_active { Span::styled(flash_pointer, Style::default().fg(theme.success)) } else { Span::raw("") },
            ]));

            // Field 1: Qty
            let f1_active = app.form.field_idx == 1;
            lines.push(Line::from(vec![
                Span::styled("   HOLDING QUANTITY: ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    &app.form.qty,
                    if f1_active {
                        Style::default().bg(theme.highlight_bg).fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                if f1_active { Span::styled(flash_pointer, Style::default().fg(theme.success)) } else { Span::raw("") },
            ]));

            // Field 2: Price
            let f2_active = app.form.field_idx == 2;
            lines.push(Line::from(vec![
                Span::styled("   MARKET PRICE ($): ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    &app.form.price,
                    if f2_active {
                        Style::default().bg(theme.highlight_bg).fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                if f2_active { Span::styled(flash_pointer, Style::default().fg(theme.success)) } else { Span::raw("") },
            ]));

            // Field 3: Sell Only Checkbox
            let f3_active = app.form.field_idx == 3;
            lines.push(Line::from(vec![
                Span::styled("   SELL ONLY (Y/N) : ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    if app.form.sell_only { "[X] YES (SELL ONLY)" } else { "[ ] NO (CAN BUY & SELL)" },
                    if f3_active {
                        Style::default().bg(theme.highlight_bg).fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                if f3_active { Span::styled(format!("{} [SPACE TO TOGGLE]", flash_pointer), Style::default().fg(theme.text_dim)) } else { Span::raw("") },
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("   -------------------------------------------------------", Style::default().fg(theme.border)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("   [TAB] NEXT FIELD   |   [ENTER] SUBMIT   |   [ESC] CANCEL", Style::default().fg(theme.text_dim).add_modifier(Modifier::BOLD)),
            ]));

            (title, lines)
        }
        ModalMode::AddGoal => {
            let title = if app.form.is_editing {
                format!(" [ ✎ EDIT TARGET GOAL: {} ] ", app.form.symbol)
            } else {
                " [ + ADD TARGET GOAL BAND ] ".to_string()
            };
            let mut lines = Vec::new();
            lines.push(Line::from(""));

            // Field 0: Symbol
            let f0_active = app.form.field_idx == 0;
            lines.push(Line::from(vec![
                Span::styled("   TICKER SYMBOL   : ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    if app.form.symbol.is_empty() && !f0_active {
                        "[ e.g. NVDA ]"
                    } else {
                        &app.form.symbol
                    },
                    if f0_active {
                        Style::default().bg(theme.highlight_bg).fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                if f0_active { Span::styled(flash_pointer, Style::default().fg(theme.success)) } else { Span::raw("") },
            ]));

            // Field 1: Min Qty
            let f1_active = app.form.field_idx == 1;
            lines.push(Line::from(vec![
                Span::styled("   MIN TARGET QTY  : ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    &app.form.min_qty,
                    if f1_active {
                        Style::default().bg(theme.highlight_bg).fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                if f1_active { Span::styled(flash_pointer, Style::default().fg(theme.success)) } else { Span::raw("") },
            ]));

            // Field 2: Max Qty
            let f2_active = app.form.field_idx == 2;
            lines.push(Line::from(vec![
                Span::styled("   MAX TARGET QTY  : ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    &app.form.max_qty,
                    if f2_active {
                        Style::default().bg(theme.highlight_bg).fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                if f2_active { Span::styled(flash_pointer, Style::default().fg(theme.success)) } else { Span::raw("") },
            ]));

            // Field 3: Price
            let f3_active = app.form.field_idx == 3;
            lines.push(Line::from(vec![
                Span::styled("   EST. PRICE ($)  : ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    &app.form.price,
                    if f3_active {
                        Style::default().bg(theme.highlight_bg).fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                if f3_active { Span::styled(flash_pointer, Style::default().fg(theme.success)) } else { Span::raw("") },
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("   -------------------------------------------------------", Style::default().fg(theme.border)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("   [TAB] NEXT FIELD   |   [ENTER] SUBMIT   |   [ESC] CANCEL", Style::default().fg(theme.text_dim).add_modifier(Modifier::BOLD)),
            ]));

            (title, lines)
        }
        ModalMode::EditParams => {
            let title = " [ EDIT CAPITAL & FEES ] ".to_string();
            let mut lines = Vec::new();
            lines.push(Line::from(""));

            // Field 0: Cash
            let f0_active = app.form.field_idx == 0;
            lines.push(Line::from(vec![
                Span::styled("   AVAILABLE CASH ($) : ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    &app.form.cash,
                    if f0_active {
                        Style::default().bg(theme.highlight_bg).fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                if f0_active { Span::styled(flash_pointer, Style::default().fg(theme.success)) } else { Span::raw("") },
            ]));

            // Field 1: Fee
            let f1_active = app.form.field_idx == 1;
            lines.push(Line::from(vec![
                Span::styled("   TRANSACTION FEE ($): ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    &app.form.fee,
                    if f1_active {
                        Style::default().bg(theme.highlight_bg).fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                if f1_active { Span::styled(flash_pointer, Style::default().fg(theme.success)) } else { Span::raw("") },
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("   -------------------------------------------------------", Style::default().fg(theme.border)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("   [TAB] NEXT FIELD   |   [ENTER] SUBMIT   |   [ESC] CANCEL", Style::default().fg(theme.text_dim).add_modifier(Modifier::BOLD)),
            ]));

            (title, lines)
        }
        ModalMode::None => ("".to_string(), Vec::new()),
    };

    let modal_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.header_bg));

    let p = Paragraph::new(content).block(modal_block);
    frame.render_widget(p, modal_area);
}
