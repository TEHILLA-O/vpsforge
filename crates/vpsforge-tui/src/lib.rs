//! Interactive first-run console.

use std::io::{self, stdout};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use ratatui::backend::CrosstermBackend;
use vpsforge_core::{format, HostFacts, InstallPlan};
use vpsforge_installer::plan_profiles;
use vpsforge_profiles::{load_embedded, Profile, ProfileFlags};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Menu,
    Plan,
    Quit,
}

pub struct TuiOutcome {
    pub selected: Vec<String>,
    pub apply: bool,
}

pub fn run(host: &HostFacts) -> io::Result<Option<TuiOutcome>> {
    let profiles = load_embedded().map_err(|e| io::Error::other(e.to_string()))?;
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = App::new(host.clone(), profiles);
    let result = loop {
        terminal.draw(|f| app.draw(f))?;
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.mode = Mode::Quit;
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev(),
                    KeyCode::Enter => app.enter(),
                    KeyCode::Char('y') if app.mode == Mode::Plan => {
                        break Ok(Some(TuiOutcome {
                            selected: vec![app.current_id()],
                            apply: true,
                        }));
                    }
                    KeyCode::Char('n') if app.mode == Mode::Plan => {
                        app.mode = Mode::Menu;
                    }
                    _ => {}
                }
            }
        }
        if app.mode == Mode::Quit {
            break Ok(None);
        }
    };
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    result
}

struct App {
    host: HostFacts,
    profiles: Vec<Profile>,
    state: ListState,
    mode: Mode,
    plan: Option<InstallPlan>,
}

impl App {
    fn new(host: HostFacts, profiles: Vec<Profile>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            host,
            profiles,
            state,
            mode: Mode::Menu,
            plan: None,
        }
    }

    fn current_id(&self) -> String {
        let idx = self.state.selected().unwrap_or(0);
        if idx == self.profiles.len() {
            return "custom".into();
        }
        self.profiles
            .get(idx)
            .map(|p| p.id.clone())
            .unwrap_or_default()
    }

    fn item_count(&self) -> usize {
        self.profiles.len() + 2
    }

    fn next(&mut self) {
        let len = self.item_count();
        let i = match self.state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn prev(&mut self) {
        let len = self.item_count();
        let i = match self.state.selected() {
            Some(0) => len - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn enter(&mut self) {
        let idx = self.state.selected().unwrap_or(0);
        if idx == self.profiles.len() + 1 {
            self.mode = Mode::Quit;
            return;
        }
        if idx == self.profiles.len() {
            if let Ok(plan) = vpsforge_installer::build_custom_plan(
                &self.host,
                &[
                    "docker".into(),
                    "postgresql".into(),
                    "redis".into(),
                    "nginx".into(),
                    "python3".into(),
                    "git".into(),
                    "tmux".into(),
                ],
            ) {
                self.plan = Some(plan);
                self.mode = Mode::Plan;
            }
            return;
        }
        if let Ok(plan) = plan_profiles(
            &self.host,
            &[self.profiles[idx].clone()],
            &ProfileFlags::default(),
        ) {
            self.plan = Some(plan);
            self.mode = Mode::Plan;
        }
    }

    fn draw(&self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let block = Block::default()
            .title(" VPSFORGE ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        frame.render_widget(block, area);

        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(14),
                Constraint::Min(8),
                Constraint::Length(2),
            ])
            .split(inner);

        frame.render_widget(host_panel(&self.host), chunks[0]);

        match self.mode {
            Mode::Menu => {
                let mut items: Vec<ListItem> = self
                    .profiles
                    .iter()
                    .map(|p| ListItem::new(format!("  {}", p.name)))
                    .collect();
                items.push(ListItem::new("  Custom Build"));
                items.push(ListItem::new("  Quit"));
                let list = List::new(items)
                    .block(
                        Block::default()
                            .title(" Choose configuration ")
                            .borders(Borders::TOP),
                    )
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("> ");
                frame.render_stateful_widget(list, chunks[1], &mut self.state.clone());
            }
            Mode::Plan => {
                let text = self
                    .plan
                    .as_ref()
                    .map(render_plan_brief)
                    .unwrap_or_default();
                frame.render_widget(
                    Paragraph::new(text).wrap(Wrap { trim: false }),
                    chunks[1],
                );
            }
            Mode::Quit => {}
        }

        let help = match self.mode {
            Mode::Menu => "↑/↓ move   enter plan   q quit",
            Mode::Plan => "y apply (Linux)   n back   q quit",
            Mode::Quit => "",
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                help,
                Style::default().fg(Color::DarkGray),
            ))),
            chunks[2],
        );
    }
}

fn host_panel(host: &HostFacts) -> Paragraph<'static> {
    let gpu = host
        .gpu
        .as_ref()
        .map(|g| format!("{} detected", g.model))
        .unwrap_or_else(|| "Not detected".into());
    let lines = vec![
        Line::from(format!(" Host       {}", host.hostname)),
        Line::from(format!(
            " OS         {} {}",
            host.os.distribution, host.os.version
        )),
        Line::from(format!(" Kernel     {}", host.kernel)),
        Line::from(format!(" CPU        {} vCPU", host.cpu_cores)),
        Line::from(format!(" RAM        {}", format::gib(host.ram_bytes))),
        Line::from(format!(
            " Disk       {} {}",
            format::gib(host.disk_bytes),
            host.disk_kind
        )),
        Line::from(format!(" GPU        {gpu}")),
        Line::from(format!(" Package    {}", host.pkg.as_str())),
        Line::from(format!(" Init       {}", host.init.as_str())),
        Line::from(format!(" Container  {}", host.virt.kind.as_str())),
    ];
    Paragraph::new(lines)
}

fn render_plan_brief(plan: &InstallPlan) -> String {
    let mut out = vec![plan.title.clone(), String::new()];
    out.push(format!(
        "Install {:>6} packages",
        plan.package_count()
    ));
    out.push(format!("Add repositories {:>4}", plan.repos));
    out.push(format!("Create services {:>5}", plan.services));
    out.push(format!(
        "Open firewall ports {:>2}",
        plan.firewall_ports.len()
    ));
    out.push(format!(
        "Estimated disk use {:>7}",
        format::bytes_human(plan.estimated_bytes)
    ));
    out.push(String::new());
    out.push("No changes have been made.".into());
    out.push("Apply? [y/N]".into());
    out.join("\n")
}
