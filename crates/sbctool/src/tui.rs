use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub kernel: String,
    pub architecture: String,
    pub chip: Option<String>,
    pub cpu_info: String,
    pub memory: String,
    pub uptime: String,
    pub os_info: String,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub struct TuiApp {
    pub system_info: Arc<Mutex<Option<SystemInfo>>>,
    pub logs: Arc<Mutex<Vec<LogEntry>>>,
    pub should_quit: bool,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            system_info: Arc::new(Mutex::new(None)),
            logs: Arc::new(Mutex::new(Vec::new())),
            should_quit: false,
        }
    }

    pub fn update_system_info(&self, info: SystemInfo) {
        if let Ok(mut system_info) = self.system_info.lock() {
            *system_info = Some(info);
        }
    }

    pub fn add_log(&self, entry: LogEntry) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(entry);
            // Keep only last 100 logs
            if logs.len() > 100 {
                let len = logs.len();
                logs.drain(0..len - 100);
            }
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // Add exit log
                        self.add_log(LogEntry {
                            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                            level: "INFO".to_string(),
                            message: "Exiting TUI...".to_string(),
                        });
                        self.should_quit = true;
                        break;
                    }
                    KeyCode::Char('r') => {
                        // Refresh system info
                        self.add_log(LogEntry {
                            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                            level: "INFO".to_string(),
                            message: "Refreshing system information...".to_string(),
                        });
                    }
                    _ => {}
                }
            }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn ui(&self, f: &mut Frame) {
        // Create main layout with helper bar at bottom
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(f.area());

        // Create horizontal layout for system info and logs
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_chunks[0]);

        self.render_system_info(f, content_chunks[0]);
        self.render_logs(f, content_chunks[1]);
        self.render_helper_bar(f, main_chunks[1]);
    }

    fn render_system_info(&self, f: &mut Frame, area: Rect) {
        let system_info = self.system_info.lock().unwrap();
        
        let mut lines = vec![
            Line::from(vec![
                Span::styled("SBC System Information", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
        ];

        if let Some(info) = system_info.as_ref() {
            lines.extend(vec![
                Line::from(vec![
                    Span::styled("Hostname: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&info.hostname),
                ]),
                Line::from(vec![
                    Span::styled("Kernel: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&info.kernel),
                ]),
                Line::from(vec![
                    Span::styled("Architecture: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&info.architecture),
                ]),
                Line::from(""),
            ]);

            if let Some(chip) = &info.chip {
                lines.push(Line::from(vec![
                    Span::styled("Chip: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(chip, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(""));
            }

            lines.extend(vec![
                Line::from(vec![
                    Span::styled("CPU: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&info.cpu_info),
                ]),
                Line::from(vec![
                    Span::styled("Memory: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&info.memory),
                ]),
                Line::from(vec![
                    Span::styled("Uptime: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&info.uptime),
                ]),
                Line::from(vec![
                    Span::styled("OS: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&info.os_info),
                ]),
            ]);
        } else {
            lines.push(Line::from(vec![
                Span::styled("No system information available", Style::default().fg(Color::Red))
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("System Info"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_logs(&self, f: &mut Frame, area: Rect) {
        let logs = self.logs.lock().unwrap();
        
        let items: Vec<ListItem> = logs
            .iter()
            .rev() // Show newest first
            .take(20) // Show last 20 entries
            .map(|log| {
                let level_color = match log.level.as_str() {
                    "ERROR" => Color::Red,
                    "WARN" => Color::Yellow,
                    "INFO" => Color::Green,
                    "DEBUG" => Color::Blue,
                    _ => Color::White,
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", log.timestamp),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(
                        format!("{}: ", log.level),
                        Style::default().fg(level_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&log.message),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Logs"))
            .style(Style::default().fg(Color::White));

        f.render_widget(list, area);
    }

    fn render_helper_bar(&self, f: &mut Frame, area: Rect) {
        let helper_text = Line::from(vec![
            Span::styled("q: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Quit", Style::default().fg(Color::White)),
            Span::raw("  "),
            Span::styled("r: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Refresh", Style::default().fg(Color::White)),
            Span::raw("  "),
            Span::styled("ESC: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Exit", Style::default().fg(Color::White)),
        ]);

        let paragraph = Paragraph::new(helper_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(paragraph, area);
    }
}

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
