mod app;
mod planner;
mod ui;

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, ModalMode, ModuleId};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error executing Trade Planner: {err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    while app.running {
        app.on_tick();
        terminal.draw(|f| ui::render_ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.running = false;
                    break;
                }

                // If modal is active, delegate all inputs to the form handler
                if app.modal_mode != ModalMode::None {
                    match key.code {
                        KeyCode::Esc => {
                            app.close_modal();
                        }
                        KeyCode::Tab | KeyCode::Down => {
                            app.form_next_field();
                        }
                        KeyCode::BackTab | KeyCode::Up => {
                            app.form_prev_field();
                        }
                        KeyCode::Enter => {
                            app.form_submit();
                        }
                        KeyCode::Backspace => {
                            app.form_backspace();
                        }
                        KeyCode::Char(' ') => {
                            if app.modal_mode == ModalMode::AddPortfolio && app.form.field_idx == 3 {
                                app.form_toggle_checkbox();
                            } else {
                                app.form_input_char(' ');
                            }
                        }
                        KeyCode::Char(c) => {
                            app.form_input_char(c);
                        }
                        _ => {}
                    }
                    continue;
                }

                // Normal Dashboard Mode
                match key.code {
                    // Global Quick Actions / Bloomberg Function Keys
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.running = false;
                    }
                    KeyCode::F(1) | KeyCode::Char('c') if !app.move_mode => {
                        app.clear_all();
                    }
                    KeyCode::F(2) | KeyCode::Char('b') | KeyCode::Char('B') | KeyCode::Char('d') if !app.move_mode => {
                        app.next_preset();
                    }
                    KeyCode::F(3) | KeyCode::Char('t') if !app.move_mode => {
                        app.toggle_theme();
                    }
                    KeyCode::F(4) | KeyCode::Char('l') if !app.move_mode => {
                        app.toggle_layout();
                    }
                    KeyCode::F(5) | KeyCode::Char('r') => {
                        app.execute_plan();
                    }
                    KeyCode::F(6) | KeyCode::Char('o') | KeyCode::Char('O') | KeyCode::Char('k') | KeyCode::Char('K') if !app.move_mode => {
                        app.next_algorithm();
                    }
                    KeyCode::F(7) | KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Char('z') | KeyCode::Char('Z') if !app.move_mode => {
                        app.execute_benchmark();
                    }

                    // Open Add / Edit Modal Forms
                    KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Insert | KeyCode::Char('n') => {
                        match app.active_module() {
                            Some(ModuleId::Goals) => app.open_add_goal_modal(),
                            Some(ModuleId::Params) => app.open_edit_params_modal(),
                            _ => app.open_add_portfolio_modal(),
                        }
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') if !app.move_mode => {
                        app.open_add_portfolio_modal();
                    }
                    KeyCode::Char('g') | KeyCode::Char('G') if !app.move_mode => {
                        app.open_add_goal_modal();
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') if !app.move_mode => {
                        app.open_edit_params_modal();
                    }

                    // Edit Selected Row with Enter
                    KeyCode::Enter if !app.move_mode => {
                        match app.active_module() {
                            Some(ModuleId::Portfolio) => {
                                app.open_edit_portfolio_modal(app.selected_port_idx);
                            }
                            Some(ModuleId::Goals) => {
                                app.open_edit_goal_modal(app.selected_goal_idx);
                            }
                            Some(ModuleId::Params) => {
                                app.open_edit_params_modal();
                            }
                            Some(ModuleId::Engine) => {
                                app.execute_plan();
                            }
                            _ => {}
                        }
                    }

                    // Move Mode Toggle & Movement Controls
                    KeyCode::Char('m') => {
                        app.move_mode = !app.move_mode;
                        app.status_message = if app.move_mode {
                            "MOVE MODE ACTIVE // PRESS [U] UP, [D] DOWN, [S] SWAP COL, [M] EXIT"
                                .to_string()
                        } else {
                            "MOVE MODE EXITED".to_string()
                        };
                    }
                    KeyCode::Char('u') | KeyCode::Up if app.move_mode => {
                        app.move_active_module_up();
                    }
                    KeyCode::Char('d') | KeyCode::Down if app.move_mode => {
                        app.move_active_module_down();
                    }
                    KeyCode::Char('s') | KeyCode::Left | KeyCode::Right if app.move_mode => {
                        app.move_active_module_col();
                    }

                    // Algorithm Cycling via Left/Right when Engine is active
                    KeyCode::Left if !app.move_mode => {
                        if app.active_module() == Some(ModuleId::Engine) {
                            app.prev_algorithm();
                        }
                    }
                    KeyCode::Right if !app.move_mode => {
                        if app.active_module() == Some(ModuleId::Engine) {
                            app.next_algorithm();
                        }
                    }

                    // Navigation & Row Selection
                    KeyCode::Tab => {
                        app.next_module();
                    }
                    KeyCode::BackTab => {
                        app.prev_module();
                    }
                    KeyCode::Up if !app.move_mode => {
                        if let Some(m) = app.active_module() {
                            match m {
                                ModuleId::Portfolio => {
                                    if app.selected_port_idx > 0 {
                                        app.selected_port_idx -= 1;
                                    }
                                }
                                ModuleId::Goals => {
                                    if app.selected_goal_idx > 0 {
                                        app.selected_goal_idx -= 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Down if !app.move_mode => {
                        if let Some(m) = app.active_module() {
                            match m {
                                ModuleId::Portfolio => {
                                    if app.selected_port_idx + 1 < app.portfolio.len() {
                                        app.selected_port_idx += 1;
                                    }
                                }
                                ModuleId::Goals => {
                                    if app.selected_goal_idx + 1 < app.goals.len() {
                                        app.selected_goal_idx += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char('x') => {
                        if let Some(m) = app.active_module() {
                            match m {
                                ModuleId::Portfolio => app.delete_selected_portfolio_item(),
                                ModuleId::Goals => app.delete_selected_goal_item(),
                                _ => {}
                            }
                        }
                    }

                    // Adjust Cash & Fee Parameters directly
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        app.cash += 500.0;
                        app.status_message = format!("CASH INCREASED: ${:.2}", app.cash);
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        if app.cash >= 500.0 {
                            app.cash -= 500.0;
                        }
                        app.status_message = format!("CASH DECREASED: ${:.2}", app.cash);
                    }
                    KeyCode::Char('>') | KeyCode::Char('.') => {
                        app.fee += 1.0;
                        app.status_message = format!("FEE SET TO: ${:.2}", app.fee);
                    }
                    KeyCode::Char('<') | KeyCode::Char(',') => {
                        if app.fee >= 1.0 {
                            app.fee -= 1.0;
                        }
                        app.status_message = format!("FEE SET TO: ${:.2}", app.fee);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
