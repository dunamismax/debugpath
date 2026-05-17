use debugpath_content::{Case, CommandKind};
use debugpath_engine::{DiagnosisSubmission, EngineError, Session};
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};
use ratatui::{Frame, Terminal};

pub const CORE_PANES: &[&str] = &[
    "Brief", "Systems", "Logs", "Metrics", "Shell", "SQL", "Trace", "Notes",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppOutcome {
    pub redraw: bool,
    pub quit: bool,
}

impl AppOutcome {
    fn redraw() -> Self {
        Self {
            redraw: true,
            quit: false,
        }
    }

    fn quit() -> Self {
        Self {
            redraw: true,
            quit: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppModel {
    session: Session,
    active_pane: usize,
    command_line: String,
    notes: Vec<String>,
    last_output: String,
    status: String,
}

impl AppModel {
    pub fn new(case: Case) -> Self {
        let title = case.metadata.title.clone();
        Self {
            session: Session::new(case),
            active_pane: 0,
            command_line: String::new(),
            notes: Vec::new(),
            last_output: "Type `commands` to list fixture-backed commands.".to_owned(),
            status: format!("Loaded seed case: {title}"),
        }
    }

    pub fn active_pane(&self) -> &'static str {
        CORE_PANES[self.active_pane]
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn command_line(&self) -> &str {
        &self.command_line
    }

    pub fn last_output(&self) -> &str {
        &self.last_output
    }

    pub fn next_pane(&mut self) {
        self.active_pane = (self.active_pane + 1) % CORE_PANES.len();
        self.status = format!("Pane: {}", self.active_pane());
    }

    pub fn previous_pane(&mut self) {
        self.active_pane = if self.active_pane == 0 {
            CORE_PANES.len() - 1
        } else {
            self.active_pane - 1
        };
        self.status = format!("Pane: {}", self.active_pane());
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> AppOutcome {
        let mut index = 0;
        let mut quit = false;
        while index < bytes.len() {
            match bytes[index] {
                b'\t' => self.next_pane(),
                b'\r' | b'\n' => self.submit_command_line(),
                0x03 => quit = true,
                0x08 | 0x7f => {
                    self.command_line.pop();
                }
                b'q' if self.command_line.is_empty() => quit = true,
                b'?' if self.command_line.is_empty() => self.use_hint(None),
                0x1b => {
                    if bytes.get(index + 1) == Some(&b'[') {
                        match bytes.get(index + 2).copied() {
                            Some(b'C') => {
                                self.next_pane();
                                index += 2;
                            }
                            Some(b'D') | Some(b'Z') => {
                                self.previous_pane();
                                index += 2;
                            }
                            _ => self.command_line.clear(),
                        }
                    } else {
                        self.command_line.clear();
                    }
                }
                byte if byte.is_ascii_graphic() || byte == b' ' => {
                    self.command_line.push(byte as char);
                }
                _ => {}
            }
            index += 1;
        }

        if quit {
            AppOutcome::quit()
        } else {
            AppOutcome::redraw()
        }
    }

    pub fn submit_command_line(&mut self) {
        let input = self.command_line.trim().to_owned();
        self.command_line.clear();
        if input.is_empty() {
            self.status = "Enter a fixture-backed command or `commands`.".to_owned();
            return;
        }
        self.execute(&input);
    }

    pub fn execute(&mut self, input: &str) {
        if input == "commands" {
            self.last_output = self.command_catalog();
            self.status = "Listed fixture-backed commands.".to_owned();
            return;
        }

        if let Some(note) = input.strip_prefix("note ") {
            self.notes.push(note.trim().to_owned());
            self.last_output = "Note recorded in this isolated session.".to_owned();
            self.status = format!("Notes: {}", self.notes.len());
            return;
        }

        if let Some(hint_id) = input.strip_prefix("hint") {
            let hint_id = hint_id.trim();
            self.use_hint((!hint_id.is_empty()).then_some(hint_id));
            return;
        }

        if let Some(fix_id) = input.strip_prefix("fix ") {
            match self.session.apply_fix(fix_id.trim()) {
                Ok(()) => {
                    let score = self.session.score();
                    self.last_output = format!(
                        "Applied fix `{}`.\nScore: {}/{} root_cause={} fix_solved={}",
                        fix_id.trim(),
                        score.total,
                        score.max_score,
                        score.root_cause_correct,
                        score.fix_solved
                    );
                    self.status = "Fix applied and replay event captured.".to_owned();
                }
                Err(error) => self.show_engine_error(error),
            }
            return;
        }

        if let Some(raw) = input.strip_prefix("diagnose ") {
            match parse_diagnosis(raw) {
                Some(diagnosis) => match self.session.submit_diagnosis(diagnosis) {
                    Ok(()) => {
                        self.last_output =
                            "Diagnosis submitted. Use `fix <fix_id>` next.".to_owned();
                        self.status = "Diagnosis replay event captured.".to_owned();
                    }
                    Err(error) => self.show_engine_error(error),
                },
                None => {
                    self.last_output = diagnosis_usage();
                    self.status =
                        "Diagnosis rejected: expected five pipe-delimited fields.".to_owned();
                }
            }
            return;
        }

        match self.session.run_command(input) {
            Ok(output) => {
                self.last_output = output;
                self.status = "Fixture-backed command ran; replay event captured.".to_owned();
            }
            Err(error) => self.show_engine_error(error),
        }
    }

    pub fn render<B: Backend>(&self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        terminal.draw(|frame| self.render_frame(frame)).map(|_| ())
    }

    pub fn render_frame(&self, frame: &mut Frame) {
        let area = frame.area();
        if area.width < 40 || area.height < 12 {
            let compact = Paragraph::new(format!(
                "debugpath.dev\n{}\n\nPane: {}\n\n{}",
                self.session.case().metadata.title,
                self.active_pane(),
                self.status
            ))
            .block(Block::default().borders(Borders::ALL).title("Incident"))
            .wrap(Wrap { trim: true });
            frame.render_widget(compact, area);
            return;
        }

        let [header, tabs, body, input, status] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(area);

        let case = self.session.case();
        let header_text = format!(
            "{} | {} | {} | ssh debugpath.dev local",
            case.metadata.title, case.metadata.difficulty, case.metadata.component
        );
        frame.render_widget(
            Paragraph::new(header_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("debugpath.dev"),
            ),
            header,
        );

        let titles = CORE_PANES.iter().map(|pane| {
            Line::from(Span::styled(
                *pane,
                Style::default().add_modifier(Modifier::BOLD),
            ))
        });
        frame.render_widget(
            Tabs::new(titles.collect::<Vec<_>>())
                .select(self.active_pane)
                .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("Panes")),
            tabs,
        );

        frame.render_widget(
            Paragraph::new(self.active_content())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(self.active_pane()),
                )
                .wrap(Wrap { trim: false }),
            body,
        );

        frame.render_widget(
            Paragraph::new(self.command_line.as_str())
                .block(Block::default().borders(Borders::ALL).title("Command")),
            input,
        );
        frame.render_widget(Paragraph::new(self.status.as_str()), status);
    }

    pub fn render_to_string(&self, width: u16, height: u16) -> std::io::Result<String> {
        let backend = ratatui::backend::TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend)?;
        self.render(&mut terminal)?;
        Ok(buffer_to_string(terminal.backend_mut().buffer()))
    }

    fn use_hint(&mut self, hint_id: Option<&str>) {
        let selected = hint_id.map(str::to_owned).or_else(|| {
            self.session
                .case()
                .hints
                .first()
                .map(|hint| hint.id.clone())
        });
        let Some(hint_id) = selected else {
            self.last_output = "This case has no authored hints.".to_owned();
            self.status = "No hint available.".to_owned();
            return;
        };

        match self.session.use_hint(&hint_id) {
            Ok(text) => {
                self.last_output = text;
                self.status = format!("Hint `{hint_id}` used; replay event captured.");
            }
            Err(error) => self.show_engine_error(error),
        }
    }

    fn show_engine_error(&mut self, error: EngineError) {
        self.last_output = match &error {
            EngineError::UnknownCommand(command) => format!(
                "Rejected `{command}`: no authored fixture exists for that command.\nHost shell and host filesystem are not available."
            ),
            _ => error.to_string(),
        };
        self.status = error.to_string();
    }

    fn active_content(&self) -> Text<'_> {
        match self.active_pane() {
            "Brief" => Text::from(self.session.case().artifacts.brief.as_str()),
            "Systems" => Text::from(self.systems_content()),
            "Logs" => Text::from(logs_content(self.session.case())),
            "Metrics" => Text::from(metrics_content(self.session.case())),
            "Shell" => Text::from(self.command_content(CommandKind::Shell)),
            "SQL" => Text::from(self.sql_content()),
            "Trace" => Text::from(trace_content(self.session.case())),
            "Notes" => Text::from(self.notes_content()),
            _ => Text::from("unknown pane"),
        }
    }

    fn systems_content(&self) -> String {
        let case = self.session.case();
        let fixes = case
            .fixes
            .iter()
            .map(|fix| format!("- {}: {} ({:?})", fix.id, fix.title, fix.kind))
            .collect::<Vec<_>>()
            .join("\n");
        let hints = case
            .hints
            .iter()
            .map(|hint| format!("- {} (cost {})", hint.id, hint.cost))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "Summary: {}\nComponent: {}\nStarts: {}\n\nFix options:\n{}\n\nHints:\n{}",
            case.metadata.summary, case.metadata.component, case.metadata.starts_at, fixes, hints
        )
    }

    fn command_content(&self, kind: CommandKind) -> String {
        format!(
            "{}\n\nLast output:\n{}",
            self.command_catalog_for(kind),
            self.last_output
        )
    }

    fn sql_content(&self) -> String {
        let case = self.session.case();
        let rows = case
            .artifacts
            .sql_rows
            .iter()
            .map(|(path, rows)| format!("-- {path}\n{rows}"))
            .collect::<Vec<_>>()
            .join("\n\n");
        format!(
            "{}\n\nSchema:\n{}\n\nRows:\n{}\n\nLast output:\n{}",
            self.command_catalog_for(CommandKind::Sql),
            case.artifacts.schema_sql,
            rows,
            self.last_output
        )
    }

    fn notes_content(&self) -> String {
        if self.notes.is_empty() {
            "Use `note <text>` to record session-local notes.\n\nNotes are isolated per SSH connection."
                .to_owned()
        } else {
            self.notes
                .iter()
                .enumerate()
                .map(|(index, note)| format!("{}. {}", index + 1, note))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    fn command_catalog(&self) -> String {
        [
            self.command_catalog_for(CommandKind::Shell),
            self.command_catalog_for(CommandKind::Sql),
            "Meta commands:\n- commands\n- hint [hint_id]\n- note <text>\n- diagnose <root>|<evidence_csv>|<component>|<fix>|<blast_radius>\n- fix <fix_id>"
                .to_owned(),
        ]
        .join("\n\n")
    }

    fn command_catalog_for(&self, kind: CommandKind) -> String {
        let title = match kind {
            CommandKind::Shell => "Fixture-backed shell commands:",
            CommandKind::Sql => "Fixture-backed SQL commands:",
        };
        let commands = self
            .session
            .case()
            .commands
            .iter()
            .filter(|command| command.kind == kind)
            .map(|command| format!("- {}", command.command))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{title}\n{commands}")
    }
}

fn logs_content(case: &Case) -> String {
    case.artifacts
        .logs
        .iter()
        .map(|value| serde_json::to_string(value).unwrap_or_else(|_| value.to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn metrics_content(case: &Case) -> String {
    toml::to_string_pretty(&case.artifacts.metrics)
        .unwrap_or_else(|_| case.artifacts.metrics.to_string())
}

fn trace_content(case: &Case) -> String {
    serde_json::to_string_pretty(&case.artifacts.traces)
        .unwrap_or_else(|_| case.artifacts.traces.to_string())
}

fn parse_diagnosis(raw: &str) -> Option<DiagnosisSubmission> {
    let parts = raw.split('|').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 5 || parts.iter().any(|part| part.is_empty()) {
        return None;
    }
    Some(DiagnosisSubmission {
        root_cause: parts[0].to_owned(),
        evidence: parts[1]
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_owned)
            .collect(),
        affected_component: parts[2].to_owned(),
        proposed_fix: parts[3].to_owned(),
        blast_radius: parts[4].to_owned(),
    })
}

fn diagnosis_usage() -> String {
    "Usage: diagnose <root>|<evidence_csv>|<component>|<fix>|<blast_radius>".to_owned()
}

fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    let mut output = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            if let Some(cell) = buffer.cell((x, y)) {
                output.push_str(cell.symbol());
            }
        }
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use debugpath_content::load_case_dir;
    use std::path::PathBuf;

    fn fixture_case() -> Case {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cases/slow-checkout");
        load_case_dir(root).expect("seed case loads")
    }

    #[test]
    fn exposes_required_core_panes() {
        for expected in [
            "Brief", "Systems", "Logs", "Metrics", "Shell", "SQL", "Trace", "Notes",
        ] {
            assert!(CORE_PANES.contains(&expected));
        }
    }

    #[test]
    fn pane_focus_wraps_predictably() {
        let mut model = AppModel::new(fixture_case());
        assert_eq!(model.active_pane(), "Brief");
        model.previous_pane();
        assert_eq!(model.active_pane(), "Notes");
        model.next_pane();
        assert_eq!(model.active_pane(), "Brief");
    }

    #[test]
    fn renders_seed_case_and_runs_fixture_command() {
        let mut model = AppModel::new(fixture_case());
        let view = model.render_to_string(96, 28).expect("renders");
        assert!(view.contains("Slow Checkout"));
        assert!(view.contains("Brief"));

        model.execute("logs checkout-api --since 10m");
        assert!(model.last_output().contains("checkout-api"));
        assert!(matches!(
            model.session().replay().last(),
            Some(debugpath_engine::ReplayEvent::CommandRun { command, .. })
                if command == "logs checkout-api --since 10m"
        ));
    }

    #[test]
    fn bad_input_cannot_escape_to_host_shell_or_filesystem() {
        let mut model = AppModel::new(fixture_case());
        model.execute("cat /etc/passwd");

        assert!(model.last_output().contains("no authored fixture"));
        assert!(
            model
                .last_output()
                .contains("Host shell and host filesystem are not available")
        );
        assert!(!model.last_output().contains("root:"));
        assert!(matches!(
            model.session().replay().last(),
            Some(debugpath_engine::ReplayEvent::CommandRejected { command, .. })
                if command == "cat /etc/passwd"
        ));
    }

    #[test]
    fn app_models_keep_session_state_isolated() {
        let case = fixture_case();
        let mut first = AppModel::new(case.clone());
        let second = AppModel::new(case);

        first.execute("logs checkout-api --since 10m");
        first.execute("note first connection only");

        assert_eq!(first.session().replay().len(), 1);
        assert!(second.session().replay().is_empty());
        assert!(!second.notes_content().contains("first connection only"));
    }

    #[test]
    fn diagnosis_and_fix_flow_updates_score_and_replay() {
        let mut model = AppModel::new(fixture_case());
        model.execute("logs checkout-api --since 10m");
        model.execute("sql explain checkout_recent_orders");
        model.execute("diff deploy checkout-api");
        model.execute(
            "diagnose missing composite index after query shape change|checkout-timeouts-after-deploy,seq-scan-orders,query-shape-changed|checkout-api orders query|add_orders_status_created_at_index|checkout order confirmation reads for pending orders",
        );
        model.execute("fix add_orders_status_created_at_index");

        let score = model.session().score();
        assert!(score.root_cause_correct);
        assert!(score.fix_solved);
        assert!(
            model
                .session()
                .replay()
                .iter()
                .any(|event| matches!(event, debugpath_engine::ReplayEvent::DiagnosisSubmitted))
        );
    }
}
