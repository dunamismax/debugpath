use debugpath_content::{Case, FixKind};
use std::collections::BTreeSet;

pub type Result<T> = std::result::Result<T, EngineError>;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("unknown hint: {0}")]
    UnknownHint(String),
    #[error("unknown fix: {0}")]
    UnknownFix(String),
    #[error(
        "diagnosis must include root cause, evidence, affected component, fix, and blast radius"
    )]
    IncompleteDiagnosis,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosisSubmission {
    pub root_cause: String,
    pub evidence: Vec<String>,
    pub affected_component: String,
    pub proposed_fix: String,
    pub blast_radius: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayEvent {
    CommandRun {
        command: String,
        evidence: Vec<String>,
        damage: u32,
    },
    HintUsed {
        hint_id: String,
        cost: u32,
    },
    DiagnosisSubmitted,
    FixApplied {
        fix_id: String,
        solves: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Score {
    pub total: u32,
    pub max_score: u32,
    pub root_cause_correct: bool,
    pub fix_solved: bool,
    pub evidence_found: usize,
    pub damage_penalty: u32,
    pub hint_penalty: u32,
    pub time_penalty: u32,
}

#[derive(Clone, Debug)]
pub struct Session {
    case: Case,
    elapsed_seconds: u32,
    evidence: BTreeSet<String>,
    hint_penalty: u32,
    damage_penalty: u32,
    diagnosis: Option<DiagnosisSubmission>,
    applied_fix: Option<String>,
    replay: Vec<ReplayEvent>,
}

impl Session {
    pub fn new(case: Case) -> Self {
        Self {
            case,
            elapsed_seconds: 0,
            evidence: BTreeSet::new(),
            hint_penalty: 0,
            damage_penalty: 0,
            diagnosis: None,
            applied_fix: None,
            replay: Vec::new(),
        }
    }

    pub fn case(&self) -> &Case {
        &self.case
    }

    pub fn replay(&self) -> &[ReplayEvent] {
        &self.replay
    }

    pub fn run_command(&mut self, input: &str) -> Result<String> {
        let command = self
            .case
            .commands
            .iter()
            .find(|command| command.command == input)
            .ok_or_else(|| EngineError::UnknownCommand(input.to_owned()))?;
        for evidence in &command.evidence {
            self.evidence.insert(evidence.clone());
        }
        self.damage_penalty = self.damage_penalty.saturating_add(command.damage);
        self.elapsed_seconds = self.elapsed_seconds.saturating_add(15);
        self.replay.push(ReplayEvent::CommandRun {
            command: command.command.clone(),
            evidence: command.evidence.clone(),
            damage: command.damage,
        });
        Ok(command.output.clone())
    }

    pub fn use_hint(&mut self, hint_id: &str) -> Result<String> {
        let hint = self
            .case
            .hints
            .iter()
            .find(|hint| hint.id == hint_id)
            .ok_or_else(|| EngineError::UnknownHint(hint_id.to_owned()))?;
        for evidence in &hint.evidence_refs {
            self.evidence.insert(evidence.clone());
        }
        self.hint_penalty = self.hint_penalty.saturating_add(hint.cost);
        self.replay.push(ReplayEvent::HintUsed {
            hint_id: hint.id.clone(),
            cost: hint.cost,
        });
        Ok(hint.text.clone())
    }

    pub fn submit_diagnosis(&mut self, diagnosis: DiagnosisSubmission) -> Result<()> {
        if diagnosis.root_cause.trim().is_empty()
            || diagnosis.evidence.is_empty()
            || diagnosis.affected_component.trim().is_empty()
            || diagnosis.proposed_fix.trim().is_empty()
            || diagnosis.blast_radius.trim().is_empty()
        {
            return Err(EngineError::IncompleteDiagnosis);
        }
        for evidence in &diagnosis.evidence {
            self.evidence.insert(evidence.clone());
        }
        self.diagnosis = Some(diagnosis);
        self.replay.push(ReplayEvent::DiagnosisSubmitted);
        Ok(())
    }

    pub fn apply_fix(&mut self, fix_id: &str) -> Result<()> {
        let fix = self
            .case
            .fixes
            .iter()
            .find(|fix| fix.id == fix_id)
            .ok_or_else(|| EngineError::UnknownFix(fix_id.to_owned()))?;
        self.applied_fix = Some(fix.id.clone());
        self.replay.push(ReplayEvent::FixApplied {
            fix_id: fix.id.clone(),
            solves: fix.solves,
        });
        Ok(())
    }

    pub fn score(&self) -> Score {
        let root_cause_correct = self.diagnosis.as_ref().is_some_and(|diagnosis| {
            same_answer(&diagnosis.root_cause, &self.case.diagnosis.root_cause)
                && same_answer(
                    &diagnosis.affected_component,
                    &self.case.diagnosis.affected_component,
                )
                && same_answer(&diagnosis.blast_radius, &self.case.diagnosis.blast_radius)
        });
        let fix_solved = self.applied_fix.as_ref().is_some_and(|applied_fix| {
            self.case
                .fixes
                .iter()
                .any(|fix| fix.id == *applied_fix && fix.kind == FixKind::RootCause && fix.solves)
        });
        let required_evidence: BTreeSet<_> = self.case.diagnosis.evidence.iter().collect();
        let found_required = required_evidence
            .iter()
            .filter(|evidence| self.evidence.contains(evidence.as_str()))
            .count();
        let missing_evidence = required_evidence.len().saturating_sub(found_required) as u32;
        let evidence_penalty = missing_evidence * 25;
        let time_penalty = self
            .elapsed_seconds
            .saturating_sub(self.case.scoring.time_budget_seconds)
            / 60;
        let correctness_penalty = if root_cause_correct { 0 } else { 200 };
        let fix_penalty = if fix_solved {
            0
        } else if self
            .applied_fix
            .as_ref()
            .is_some_and(|id| self.case.scoring.symptom_fixes.contains(id))
        {
            125
        } else {
            200
        };
        let penalty = correctness_penalty
            + fix_penalty
            + evidence_penalty
            + self.hint_penalty
            + self.damage_penalty
            + time_penalty;
        Score {
            total: self.case.scoring.max_score.saturating_sub(penalty),
            max_score: self.case.scoring.max_score,
            root_cause_correct,
            fix_solved,
            evidence_found: found_required,
            damage_penalty: self.damage_penalty,
            hint_penalty: self.hint_penalty,
            time_penalty,
        }
    }
}

fn same_answer(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use debugpath_content::{load_case_dir, load_cases};
    use std::path::PathBuf;

    fn session() -> Session {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cases/slow-checkout");
        Session::new(load_case_dir(root).expect("seed case loads"))
    }

    fn cases_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cases")
    }

    #[test]
    fn constrained_commands_return_fixtures_and_record_replay() {
        let mut session = session();
        let output = session
            .run_command("logs checkout-api --since 10m")
            .expect("fixture command runs");
        assert!(output.contains("checkout-api"));
        assert_eq!(session.replay().len(), 1);
        assert!(session.run_command("cat /etc/passwd").is_err());
    }

    #[test]
    fn diagnosis_fix_and_score_reward_root_cause_evidence() {
        let mut session = session();
        session
            .run_command("logs checkout-api --since 10m")
            .expect("logs command");
        session
            .run_command("sql explain checkout_recent_orders")
            .expect("sql command");
        session
            .run_command("diff deploy checkout-api")
            .expect("diff command");
        session
            .submit_diagnosis(DiagnosisSubmission {
                root_cause: "missing composite index after query shape change".to_owned(),
                evidence: vec![
                    "checkout-timeouts-after-deploy".to_owned(),
                    "seq-scan-orders".to_owned(),
                    "query-shape-changed".to_owned(),
                ],
                affected_component: "checkout-api orders query".to_owned(),
                proposed_fix: "add_orders_status_created_at_index".to_owned(),
                blast_radius: "checkout order confirmation reads for pending orders".to_owned(),
            })
            .expect("diagnosis accepted");
        session
            .apply_fix("add_orders_status_created_at_index")
            .expect("fix accepted");
        let score = session.score();
        assert!(score.root_cause_correct);
        assert!(score.fix_solved);
        assert_eq!(score.evidence_found, 3);
        assert!(score.total > 800);
    }

    #[test]
    fn all_seed_cases_can_be_diagnosed_fixed_scored_and_replayed() {
        let cases = load_cases(cases_root()).expect("case collection loads");
        assert_eq!(cases.len(), 3);

        for case in cases {
            let required_evidence = case.diagnosis.evidence.clone();
            let root_cause = case.diagnosis.root_cause.clone();
            let affected_component = case.diagnosis.affected_component.clone();
            let blast_radius = case.diagnosis.blast_radius.clone();
            let root_fix = case.scoring.root_fix.clone();
            let commands: Vec<_> = case
                .commands
                .iter()
                .filter(|command| {
                    command
                        .evidence
                        .iter()
                        .any(|evidence| required_evidence.contains(evidence))
                })
                .map(|command| command.command.clone())
                .collect();

            let mut session = Session::new(case);
            for command in commands {
                session
                    .run_command(&command)
                    .expect("evidence command runs");
            }
            session
                .submit_diagnosis(DiagnosisSubmission {
                    root_cause,
                    evidence: required_evidence.clone(),
                    affected_component,
                    proposed_fix: root_fix.clone(),
                    blast_radius,
                })
                .expect("diagnosis accepted");
            session.apply_fix(&root_fix).expect("root fix accepted");

            let score = session.score();
            assert!(score.root_cause_correct);
            assert!(score.fix_solved);
            assert_eq!(score.evidence_found, required_evidence.len());
            assert!(
                session
                    .replay()
                    .iter()
                    .any(|event| matches!(event, ReplayEvent::DiagnosisSubmitted))
            );
        }
    }
}
