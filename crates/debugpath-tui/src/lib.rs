use debugpath_content::{Case, CommandKind};
use debugpath_engine::{DiagnosisSubmission, EngineError, ReplayEvent, Session};
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
    mode: AppMode,
    command_line: String,
    palette_query: String,
    palette_selected: usize,
    diagnosis_draft: DiagnosisDraft,
    diagnosis_field: DiagnosisField,
    fix_selected: usize,
    notes: Vec<String>,
    last_output: String,
    status: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppMode {
    Command,
    Palette,
    Diagnosis,
    FixSelect,
    Results,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DiagnosisDraft {
    root_cause: String,
    evidence: String,
    affected_component: String,
    proposed_fix: String,
    blast_radius: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DiagnosisField {
    RootCause,
    Evidence,
    AffectedComponent,
    ProposedFix,
    BlastRadius,
}

impl DiagnosisField {
    fn label(self) -> &'static str {
        match self {
            Self::RootCause => "Root cause",
            Self::Evidence => "Evidence IDs",
            Self::AffectedComponent => "Affected component",
            Self::ProposedFix => "Proposed fix",
            Self::BlastRadius => "Blast radius",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::RootCause => Self::Evidence,
            Self::Evidence => Self::AffectedComponent,
            Self::AffectedComponent => Self::ProposedFix,
            Self::ProposedFix => Self::BlastRadius,
            Self::BlastRadius => Self::BlastRadius,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::RootCause => Self::RootCause,
            Self::Evidence => Self::RootCause,
            Self::AffectedComponent => Self::Evidence,
            Self::ProposedFix => Self::AffectedComponent,
            Self::BlastRadius => Self::ProposedFix,
        }
    }

    fn is_last(self) -> bool {
        self == Self::BlastRadius
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PaletteAction {
    label: String,
    detail: String,
    command: PaletteCommand,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PaletteCommand {
    RunCommand(String),
    ShowArtifact(String),
    UseHint(String),
    OpenNotes,
    OpenDiagnosis,
    OpenFixes,
    ShowResults,
    ListCommands,
    ListArtifacts,
}

impl AppModel {
    pub fn new(case: Case) -> Self {
        let title = case.metadata.title.clone();
        Self {
            session: Session::new(case),
            active_pane: 0,
            mode: AppMode::Command,
            command_line: String::new(),
            palette_query: String::new(),
            palette_selected: 0,
            diagnosis_draft: DiagnosisDraft::default(),
            diagnosis_field: DiagnosisField::RootCause,
            fix_selected: 0,
            notes: Vec::new(),
            last_output: "Type `commands` for fixtures or `artifacts` for browsable case files."
                .to_owned(),
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
                0x1b => {
                    if bytes.get(index + 1) == Some(&b'[') {
                        match bytes.get(index + 2).copied() {
                            Some(b'C') => {
                                self.handle_escape_key(b'C');
                                index += 2;
                            }
                            Some(b'D') | Some(b'Z') | Some(b'A') | Some(b'B') => {
                                self.handle_escape_key(bytes[index + 2]);
                                index += 2;
                            }
                            _ => self.cancel_mode_or_clear(),
                        }
                    } else {
                        self.cancel_mode_or_clear();
                    }
                }
                byte => self.handle_plain_byte(byte, &mut quit),
            }
            index += 1;
        }

        if quit {
            AppOutcome::quit()
        } else {
            AppOutcome::redraw()
        }
    }

    fn handle_plain_byte(&mut self, byte: u8, quit: &mut bool) {
        match self.mode {
            AppMode::Command => self.handle_command_byte(byte, quit),
            AppMode::Palette => self.handle_palette_byte(byte, quit),
            AppMode::Diagnosis => self.handle_diagnosis_byte(byte, quit),
            AppMode::FixSelect => self.handle_fix_select_byte(byte, quit),
            AppMode::Results => self.handle_results_byte(byte, quit),
        }
    }

    fn handle_command_byte(&mut self, byte: u8, quit: &mut bool) {
        match byte {
            b'\t' => self.next_pane(),
            b'\r' | b'\n' => self.submit_command_line(),
            0x03 => *quit = true,
            0x08 | 0x7f => {
                self.command_line.pop();
            }
            0x10 => self.open_palette(),
            b'q' if self.command_line.is_empty() => *quit = true,
            b'?' if self.command_line.is_empty() => self.use_hint(None),
            byte if byte.is_ascii_graphic() || byte == b' ' => {
                self.command_line.push(byte as char);
            }
            _ => {}
        }
    }

    fn handle_palette_byte(&mut self, byte: u8, quit: &mut bool) {
        match byte {
            b'\t' => self.next_palette_action(),
            b'\r' | b'\n' => self.accept_palette_action(),
            0x03 => *quit = true,
            0x08 | 0x7f => {
                self.palette_query.pop();
                self.palette_selected = 0;
            }
            byte if byte.is_ascii_graphic() || byte == b' ' => {
                self.palette_query.push(byte as char);
                self.palette_selected = 0;
            }
            _ => {}
        }
    }

    fn handle_diagnosis_byte(&mut self, byte: u8, quit: &mut bool) {
        match byte {
            b'\t' => self.advance_diagnosis_field(),
            b'\r' | b'\n' => {
                if self.diagnosis_field.is_last() {
                    self.submit_diagnosis_form();
                } else {
                    self.advance_diagnosis_field();
                }
            }
            0x03 => *quit = true,
            0x08 | 0x7f => {
                self.command_line.pop();
            }
            byte if byte.is_ascii_graphic() || byte == b' ' => {
                self.command_line.push(byte as char);
            }
            _ => {}
        }
    }

    fn handle_fix_select_byte(&mut self, byte: u8, quit: &mut bool) {
        match byte {
            b'\t' => self.next_fix_option(),
            b'\r' | b'\n' => self.apply_selected_fix(),
            0x03 => *quit = true,
            b'j' => self.next_fix_option(),
            b'k' => self.previous_fix_option(),
            byte if byte.is_ascii_digit() => {
                let index = usize::from(byte - b'1');
                if index < self.session.case().fixes.len() {
                    self.fix_selected = index;
                    self.apply_selected_fix();
                }
            }
            _ => {}
        }
    }

    fn handle_results_byte(&mut self, byte: u8, quit: &mut bool) {
        match byte {
            b'\t' => self.next_pane(),
            b'\r' | b'\n' => self.mode = AppMode::Command,
            0x03 => *quit = true,
            b'q' => *quit = true,
            _ => {}
        }
    }

    fn handle_escape_key(&mut self, key: u8) {
        match self.mode {
            AppMode::Command => match key {
                b'C' | b'B' => self.next_pane(),
                b'D' | b'A' | b'Z' => self.previous_pane(),
                _ => self.command_line.clear(),
            },
            AppMode::Palette => match key {
                b'C' | b'B' => self.next_palette_action(),
                b'D' | b'A' | b'Z' => self.previous_palette_action(),
                _ => self.cancel_mode_or_clear(),
            },
            AppMode::Diagnosis => match key {
                b'C' | b'B' => self.advance_diagnosis_field(),
                b'D' | b'A' | b'Z' => self.rewind_diagnosis_field(),
                _ => self.cancel_mode_or_clear(),
            },
            AppMode::FixSelect => match key {
                b'C' | b'B' => self.next_fix_option(),
                b'D' | b'A' | b'Z' => self.previous_fix_option(),
                _ => self.cancel_mode_or_clear(),
            },
            AppMode::Results => self.mode = AppMode::Command,
        }
    }

    fn cancel_mode_or_clear(&mut self) {
        match self.mode {
            AppMode::Command => self.command_line.clear(),
            AppMode::Palette => {
                self.mode = AppMode::Command;
                self.palette_query.clear();
                self.palette_selected = 0;
                self.status = "Command palette closed.".to_owned();
            }
            AppMode::Diagnosis => {
                self.mode = AppMode::Command;
                self.command_line.clear();
                self.status = "Diagnosis form closed.".to_owned();
            }
            AppMode::FixSelect => {
                self.mode = AppMode::Command;
                self.status = "Fix selection closed.".to_owned();
            }
            AppMode::Results => {
                self.mode = AppMode::Command;
                self.status = "Results view closed.".to_owned();
            }
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
        if input == "palette" {
            self.open_palette();
            return;
        }

        if input == "diagnosis" || input == "diagnose" {
            self.open_diagnosis_form();
            return;
        }

        if input == "fixes" || input == "fix" {
            self.open_fix_selection();
            return;
        }

        if input == "results" || input == "score" {
            self.open_results_view();
            return;
        }

        if input == "commands" {
            self.last_output = self.command_catalog();
            self.status = "Listed fixture-backed commands.".to_owned();
            return;
        }

        if input == "artifacts" {
            self.last_output = self.artifact_catalog();
            self.status = "Listed browsable case artifacts.".to_owned();
            return;
        }

        if let Some(path) = input
            .strip_prefix("show ")
            .or_else(|| input.strip_prefix("open "))
        {
            match self.artifact_content(path.trim()) {
                Some(output) => {
                    self.last_output = output;
                    self.status = format!("Viewing artifact `{}`.", path.trim());
                }
                None => {
                    self.last_output = format!(
                        "Unknown artifact `{}`.\nRun `artifacts` to list browsable case files.",
                        path.trim()
                    );
                    self.status = "Artifact not found.".to_owned();
                }
            }
            return;
        }

        if input == "hints" {
            self.last_output = self.hint_catalog();
            self.status = "Listed authored hints.".to_owned();
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
                    self.mode = AppMode::Results;
                    self.status = "Fix applied; showing results.".to_owned();
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

        let command_kind = self
            .session
            .case()
            .commands
            .iter()
            .find(|command| command.command == input)
            .map(|command| command.kind.clone());

        match self.session.run_command(input) {
            Ok(output) => {
                self.last_output = output;
                if let Some(kind) = command_kind {
                    self.active_pane = match kind {
                        CommandKind::Shell => CORE_PANES
                            .iter()
                            .position(|pane| *pane == "Shell")
                            .unwrap_or(self.active_pane),
                        CommandKind::Sql => CORE_PANES
                            .iter()
                            .position(|pane| *pane == "SQL")
                            .unwrap_or(self.active_pane),
                    };
                }
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
            Paragraph::new(self.body_content())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(self.body_title()),
                )
                .wrap(Wrap { trim: false }),
            body,
        );

        frame.render_widget(
            Paragraph::new(self.input_value()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.input_title()),
            ),
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

    fn open_palette(&mut self) {
        self.mode = AppMode::Palette;
        self.palette_query.clear();
        self.palette_selected = 0;
        self.status = "Command palette open.".to_owned();
    }

    fn open_diagnosis_form(&mut self) {
        self.mode = AppMode::Diagnosis;
        self.diagnosis_draft = DiagnosisDraft::default();
        self.set_diagnosis_field(DiagnosisField::RootCause);
        self.status = "Diagnosis form open.".to_owned();
    }

    fn open_fix_selection(&mut self) {
        self.mode = AppMode::FixSelect;
        self.fix_selected = self
            .fix_selected
            .min(self.session.case().fixes.len().saturating_sub(1));
        self.command_line.clear();
        self.status = "Fix selection open.".to_owned();
    }

    fn open_results_view(&mut self) {
        self.mode = AppMode::Results;
        self.command_line.clear();
        self.last_output = self.results_content();
        self.status = "Results view open.".to_owned();
    }

    fn next_palette_action(&mut self) {
        let len = self.filtered_palette_actions().len();
        if len > 0 {
            self.palette_selected = (self.palette_selected + 1) % len;
        }
    }

    fn previous_palette_action(&mut self) {
        let len = self.filtered_palette_actions().len();
        if len > 0 {
            self.palette_selected = if self.palette_selected == 0 {
                len - 1
            } else {
                self.palette_selected - 1
            };
        }
    }

    fn accept_palette_action(&mut self) {
        let actions = self.filtered_palette_actions();
        let Some(action) = actions.get(self.palette_selected.min(actions.len().saturating_sub(1)))
        else {
            self.status = "No palette action matches the query.".to_owned();
            return;
        };
        let command = action.command.clone();
        self.mode = AppMode::Command;
        self.palette_query.clear();
        self.palette_selected = 0;
        match command {
            PaletteCommand::RunCommand(command) => self.execute(&command),
            PaletteCommand::ShowArtifact(path) => {
                if let Some(output) = self.artifact_content(&path) {
                    self.last_output = output;
                    self.status = format!("Viewing artifact `{path}`.");
                }
            }
            PaletteCommand::UseHint(hint_id) => self.use_hint(Some(&hint_id)),
            PaletteCommand::OpenNotes => {
                self.active_pane = CORE_PANES
                    .iter()
                    .position(|pane| *pane == "Notes")
                    .unwrap_or(self.active_pane);
                self.status = "Notes pane selected.".to_owned();
            }
            PaletteCommand::OpenDiagnosis => self.open_diagnosis_form(),
            PaletteCommand::OpenFixes => self.open_fix_selection(),
            PaletteCommand::ShowResults => self.open_results_view(),
            PaletteCommand::ListCommands => {
                self.last_output = self.command_catalog();
                self.status = "Listed fixture-backed commands.".to_owned();
            }
            PaletteCommand::ListArtifacts => {
                self.last_output = self.artifact_catalog();
                self.status = "Listed browsable case artifacts.".to_owned();
            }
        }
    }

    fn palette_actions(&self) -> Vec<PaletteAction> {
        let mut actions = vec![
            PaletteAction {
                label: "diagnosis form".to_owned(),
                detail: "submit root cause, evidence, component, fix, and blast radius".to_owned(),
                command: PaletteCommand::OpenDiagnosis,
            },
            PaletteAction {
                label: "fix selection".to_owned(),
                detail: "choose an authored fix option".to_owned(),
                command: PaletteCommand::OpenFixes,
            },
            PaletteAction {
                label: "results view".to_owned(),
                detail: "show score and replay events".to_owned(),
                command: PaletteCommand::ShowResults,
            },
            PaletteAction {
                label: "notes pane".to_owned(),
                detail: "review session-local notes".to_owned(),
                command: PaletteCommand::OpenNotes,
            },
            PaletteAction {
                label: "list commands".to_owned(),
                detail: "show fixture-backed shell and SQL commands".to_owned(),
                command: PaletteCommand::ListCommands,
            },
            PaletteAction {
                label: "list artifacts".to_owned(),
                detail: "show browsable case artifacts".to_owned(),
                command: PaletteCommand::ListArtifacts,
            },
        ];

        actions.extend(
            self.session
                .case()
                .commands
                .iter()
                .map(|command| PaletteAction {
                    label: format!("run: {}", command.command),
                    detail: format!("{:?} fixture", command.kind),
                    command: PaletteCommand::RunCommand(command.command.clone()),
                }),
        );
        actions.extend(self.artifact_paths().into_iter().map(|path| PaletteAction {
            label: format!("artifact: {path}"),
            detail: "open case artifact".to_owned(),
            command: PaletteCommand::ShowArtifact(path),
        }));
        actions.extend(self.session.case().hints.iter().map(|hint| PaletteAction {
            label: format!("hint: {}", hint.id),
            detail: format!("cost {}", hint.cost),
            command: PaletteCommand::UseHint(hint.id.clone()),
        }));
        actions
    }

    fn filtered_palette_actions(&self) -> Vec<PaletteAction> {
        let query = self.palette_query.trim().to_lowercase();
        self.palette_actions()
            .into_iter()
            .filter(|action| {
                query.is_empty()
                    || action.label.to_lowercase().contains(&query)
                    || action.detail.to_lowercase().contains(&query)
            })
            .collect()
    }

    fn advance_diagnosis_field(&mut self) {
        self.commit_diagnosis_field();
        self.set_diagnosis_field(self.diagnosis_field.next());
    }

    fn rewind_diagnosis_field(&mut self) {
        self.commit_diagnosis_field();
        self.set_diagnosis_field(self.diagnosis_field.previous());
    }

    fn set_diagnosis_field(&mut self, field: DiagnosisField) {
        self.diagnosis_field = field;
        self.command_line = self.diagnosis_value(field).to_owned();
    }

    fn commit_diagnosis_field(&mut self) {
        let value = self.command_line.trim().to_owned();
        match self.diagnosis_field {
            DiagnosisField::RootCause => self.diagnosis_draft.root_cause = value,
            DiagnosisField::Evidence => self.diagnosis_draft.evidence = value,
            DiagnosisField::AffectedComponent => self.diagnosis_draft.affected_component = value,
            DiagnosisField::ProposedFix => self.diagnosis_draft.proposed_fix = value,
            DiagnosisField::BlastRadius => self.diagnosis_draft.blast_radius = value,
        }
    }

    fn diagnosis_value(&self, field: DiagnosisField) -> &str {
        match field {
            DiagnosisField::RootCause => &self.diagnosis_draft.root_cause,
            DiagnosisField::Evidence => &self.diagnosis_draft.evidence,
            DiagnosisField::AffectedComponent => &self.diagnosis_draft.affected_component,
            DiagnosisField::ProposedFix => &self.diagnosis_draft.proposed_fix,
            DiagnosisField::BlastRadius => &self.diagnosis_draft.blast_radius,
        }
    }

    fn submit_diagnosis_form(&mut self) {
        self.commit_diagnosis_field();
        let diagnosis = DiagnosisSubmission {
            root_cause: self.diagnosis_draft.root_cause.trim().to_owned(),
            evidence: self
                .diagnosis_draft
                .evidence
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect(),
            affected_component: self.diagnosis_draft.affected_component.trim().to_owned(),
            proposed_fix: self.diagnosis_draft.proposed_fix.trim().to_owned(),
            blast_radius: self.diagnosis_draft.blast_radius.trim().to_owned(),
        };

        match self.session.submit_diagnosis(diagnosis) {
            Ok(()) => {
                self.last_output = "Diagnosis submitted. Choose a fix option next.".to_owned();
                self.status = "Diagnosis replay event captured.".to_owned();
                self.open_fix_selection();
            }
            Err(error) => {
                self.mode = AppMode::Diagnosis;
                self.show_engine_error(error);
            }
        }
    }

    fn next_fix_option(&mut self) {
        let len = self.session.case().fixes.len();
        if len > 0 {
            self.fix_selected = (self.fix_selected + 1) % len;
        }
    }

    fn previous_fix_option(&mut self) {
        let len = self.session.case().fixes.len();
        if len > 0 {
            self.fix_selected = if self.fix_selected == 0 {
                len - 1
            } else {
                self.fix_selected - 1
            };
        }
    }

    fn apply_selected_fix(&mut self) {
        let Some(fix) = self.session.case().fixes.get(self.fix_selected) else {
            self.last_output = "This case has no authored fixes.".to_owned();
            self.status = "No fix available.".to_owned();
            return;
        };
        let fix_id = fix.id.clone();
        match self.session.apply_fix(&fix_id) {
            Ok(()) => {
                self.last_output = self.results_content();
                self.mode = AppMode::Results;
                self.status = format!("Applied fix `{fix_id}`; results ready.");
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

    fn body_title(&self) -> String {
        match self.mode {
            AppMode::Command => self.active_pane().to_owned(),
            AppMode::Palette => "Command Palette".to_owned(),
            AppMode::Diagnosis => "Diagnosis Form".to_owned(),
            AppMode::FixSelect => "Fix Selection".to_owned(),
            AppMode::Results => "Results".to_owned(),
        }
    }

    fn body_content(&self) -> Text<'_> {
        match self.mode {
            AppMode::Command => self.active_content(),
            AppMode::Palette => Text::from(self.palette_content()),
            AppMode::Diagnosis => Text::from(self.diagnosis_content()),
            AppMode::FixSelect => Text::from(self.fix_selection_content()),
            AppMode::Results => Text::from(self.results_content()),
        }
    }

    fn input_title(&self) -> String {
        match self.mode {
            AppMode::Command => "Command".to_owned(),
            AppMode::Palette => "Palette Query".to_owned(),
            AppMode::Diagnosis => self.diagnosis_field.label().to_owned(),
            AppMode::FixSelect => "Fix".to_owned(),
            AppMode::Results => "Results".to_owned(),
        }
    }

    fn input_value(&self) -> &str {
        match self.mode {
            AppMode::Palette => self.palette_query.as_str(),
            AppMode::FixSelect => "Use arrows/j/k or 1-9, then enter.",
            AppMode::Results => "Enter returns to the console. q disconnects.",
            _ => self.command_line.as_str(),
        }
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

    fn palette_content(&self) -> String {
        let actions = self.filtered_palette_actions();
        if actions.is_empty() {
            return "No matching actions.".to_owned();
        }
        let selected = self.palette_selected.min(actions.len().saturating_sub(1));
        let mut lines = vec![
            "Enter runs the selected action. Tab or arrows change selection. Esc closes."
                .to_owned(),
            String::new(),
        ];
        lines.extend(actions.iter().enumerate().map(|(index, action)| {
            let marker = if index == selected { ">" } else { " " };
            format!("{marker} {} - {}", action.label, action.detail)
        }));
        lines.join("\n")
    }

    fn diagnosis_content(&self) -> String {
        let fields = [
            DiagnosisField::RootCause,
            DiagnosisField::Evidence,
            DiagnosisField::AffectedComponent,
            DiagnosisField::ProposedFix,
            DiagnosisField::BlastRadius,
        ];
        let mut lines = vec![
            "Fill each field and press Enter. Evidence IDs are comma-separated. Esc closes."
                .to_owned(),
            String::new(),
        ];
        lines.extend(fields.into_iter().map(|field| {
            let marker = if field == self.diagnosis_field {
                ">"
            } else {
                " "
            };
            let value = if field == self.diagnosis_field {
                self.command_line.as_str()
            } else {
                self.diagnosis_value(field)
            };
            let shown = if value.is_empty() { "<empty>" } else { value };
            format!("{marker} {}: {shown}", field.label())
        }));
        lines.join("\n")
    }

    fn fix_selection_content(&self) -> String {
        let fixes = &self.session.case().fixes;
        if fixes.is_empty() {
            return "This case has no authored fixes.".to_owned();
        }
        let mut lines = vec!["Choose one authored fix option.".to_owned(), String::new()];
        lines.extend(fixes.iter().enumerate().map(|(index, fix)| {
            let marker = if index == self.fix_selected { ">" } else { " " };
            format!(
                "{marker} {}. {} ({:?})\n   id: {}\n   {}",
                index + 1,
                fix.title,
                fix.kind,
                fix.id,
                fix.explanation
            )
        }));
        lines.join("\n")
    }

    fn results_content(&self) -> String {
        let score = self.session.score();
        let replay = self
            .session
            .replay()
            .iter()
            .enumerate()
            .map(|(index, event)| format!("{}. {}", index + 1, replay_event_summary(event)))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "Score: {}/{}\nRoot cause correct: {}\nFix solved: {}\nRequired evidence found: {}\nDamage penalty: {}\nHint penalty: {}\nTime penalty: {}\n\nReplay:\n{}",
            score.total,
            score.max_score,
            score.root_cause_correct,
            score.fix_solved,
            score.evidence_found,
            score.damage_penalty,
            score.hint_penalty,
            score.time_penalty,
            if replay.is_empty() {
                "No replay events yet.".to_owned()
            } else {
                replay
            }
        )
    }

    fn systems_content(&self) -> String {
        let case = self.session.case();
        let fixes = case
            .fixes
            .iter()
            .map(|fix| format!("- {}: {} ({:?})", fix.id, fix.title, fix.kind))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "Summary: {}\nComponent: {}\nStarts: {}\n\nFix options:\n{}\n\n{}\n\n{}",
            case.metadata.summary,
            case.metadata.component,
            case.metadata.starts_at,
            fixes,
            self.hint_catalog(),
            self.artifact_catalog()
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
            "{}\n\nLast output:\n{}\n\nSchema:\n{}\n\nRows:\n{}",
            self.command_catalog_for(CommandKind::Sql),
            self.last_output,
            case.artifacts.schema_sql,
            rows
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
            "Meta commands:\n- commands\n- artifacts\n- show <artifact>\n- hints\n- hint [hint_id]\n- note <text>\n- diagnose <root>|<evidence_csv>|<component>|<fix>|<blast_radius>\n- fix <fix_id>"
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

    fn artifact_catalog(&self) -> String {
        let mut lines = vec!["Browsable artifacts:".to_owned()];
        lines.extend(
            self.artifact_paths()
                .into_iter()
                .map(|path| format!("- {path}")),
        );
        lines.join("\n")
    }

    fn artifact_paths(&self) -> Vec<String> {
        let case = self.session.case();
        let mut paths = vec![
            "brief".to_owned(),
            "logs".to_owned(),
            "metrics".to_owned(),
            "schema.sql".to_owned(),
            "traces".to_owned(),
        ];
        paths.extend(case.artifacts.sql_rows.keys().cloned());
        paths.extend(case.artifacts.diffs.keys().cloned());
        paths.extend(case.artifacts.runbooks.keys().cloned());
        paths
    }

    fn artifact_content(&self, path: &str) -> Option<String> {
        let case = self.session.case();
        match path {
            "brief" | "brief.md" => Some(case.artifacts.brief.clone()),
            "logs" | "logs.ndjson" => Some(logs_content(case)),
            "metrics" | "metrics.toml" => Some(metrics_content(case)),
            "schema" | "schema.sql" => Some(case.artifacts.schema_sql.clone()),
            "traces" | "traces.json" => Some(trace_content(case)),
            _ => case
                .artifacts
                .sql_rows
                .get(path)
                .or_else(|| case.artifacts.diffs.get(path))
                .or_else(|| case.artifacts.runbooks.get(path))
                .cloned(),
        }
    }

    fn hint_catalog(&self) -> String {
        let hints = self
            .session
            .case()
            .hints
            .iter()
            .map(|hint| format!("- {} (cost {}): {}", hint.id, hint.cost, hint.text))
            .collect::<Vec<_>>()
            .join("\n");
        if hints.is_empty() {
            "This case has no authored hints.".to_owned()
        } else {
            format!("Authored hints:\n{hints}")
        }
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

fn replay_event_summary(event: &ReplayEvent) -> String {
    match event {
        ReplayEvent::CommandRun {
            command,
            evidence,
            damage,
        } => format!(
            "command `{command}` evidence=[{}] damage={damage}",
            evidence.join(",")
        ),
        ReplayEvent::CommandRejected { command, reason } => {
            format!("rejected `{command}`: {reason}")
        }
        ReplayEvent::HintUsed { hint_id, cost } => format!("hint `{hint_id}` cost={cost}"),
        ReplayEvent::DiagnosisSubmitted => "diagnosis submitted".to_owned(),
        ReplayEvent::FixApplied { fix_id, solves } => {
            format!("fix `{fix_id}` applied solves={solves}")
        }
    }
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
    fn browses_case_artifacts_notes_and_hints() {
        let mut model = AppModel::new(fixture_case());

        model.execute("artifacts");
        assert!(model.last_output().contains("logs"));
        assert!(model.last_output().contains("diffs/query-shape.diff"));
        assert!(model.last_output().contains("runbooks/checkout.md"));

        model.execute("show diffs/query-shape.diff");
        assert!(model.last_output().contains("orders"));

        model.execute("show runbooks/checkout.md");
        assert!(model.last_output().contains("Checkout"));

        model.execute("hints");
        assert!(model.last_output().contains("Authored hints"));

        model.execute("hint");
        assert!(matches!(
            model.session().replay().last(),
            Some(debugpath_engine::ReplayEvent::HintUsed { .. })
        ));

        model.execute("note compare deploy diff to query plan");
        for _ in 0..7 {
            model.next_pane();
        }
        let notes = model.render_to_string(96, 28).expect("renders after note");
        assert!(notes.contains("compare deploy diff to query plan"));
    }

    #[test]
    fn renders_predictable_narrow_terminal_fallback() {
        let model = AppModel::new(fixture_case());
        let view = model.render_to_string(30, 8).expect("compact render");

        assert!(view.contains("debugpath.dev"));
        assert!(view.contains("Slow Checkout"));
        assert!(view.contains("Pane: Brief"));
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
    fn command_palette_runs_commands_and_opens_artifacts() {
        let mut model = AppModel::new(fixture_case());

        model.handle_bytes(&[0x10]);
        assert_eq!(model.mode, AppMode::Palette);
        model.handle_bytes(b"logs checkout\r");
        assert_eq!(model.mode, AppMode::Command);
        assert!(model.last_output().contains("checkout-api"));

        model.handle_bytes(&[0x10]);
        model.handle_bytes(b"query-shape\r");
        assert_eq!(model.mode, AppMode::Command);
        assert!(model.last_output().contains("orders"));
    }

    #[test]
    fn interactive_diagnosis_fix_and_results_flow_scores_case() {
        let mut model = AppModel::new(fixture_case());
        model.execute("logs checkout-api --since 10m");
        model.execute("sql explain checkout_recent_orders");
        model.execute("diff deploy checkout-api");

        model.execute("diagnosis");
        assert_eq!(model.mode, AppMode::Diagnosis);
        model.handle_bytes(
            b"missing composite index after query shape change\r\
              checkout-timeouts-after-deploy,seq-scan-orders,query-shape-changed\r\
              checkout-api orders query\r\
              add_orders_status_created_at_index\r\
              checkout order confirmation reads for pending orders\r",
        );
        assert_eq!(model.mode, AppMode::FixSelect);

        model.handle_bytes(b"\r");
        assert_eq!(model.mode, AppMode::Results);
        let score = model.session().score();
        assert!(score.root_cause_correct);
        assert!(score.fix_solved);

        let view = model.render_to_string(96, 28).expect("renders results");
        assert!(view.contains("Results"));
        assert!(view.contains("Score:"));
        assert!(view.contains("diagnosis submitted"));
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
