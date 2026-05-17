use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, ContentError>;

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse TOML {path}: {source}")]
    Toml {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("failed to parse JSON {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("{case}: missing required field {field}")]
    MissingField { case: String, field: &'static str },
    #[error("{case}: artifact reference {path} does not exist")]
    MissingArtifact { case: String, path: PathBuf },
    #[error("{case}: duplicate {kind} id {id}")]
    DuplicateId {
        case: String,
        kind: &'static str,
        id: String,
    },
    #[error("{case}: command {command} has no fixture")]
    CommandWithoutFixture { case: String, command: String },
    #[error("{case}: scoring references unknown evidence {evidence}")]
    UnknownScoringEvidence { case: String, evidence: String },
    #[error("{case}: scoring root_fix {fix} is not an authored fix")]
    UnknownRootFix { case: String, fix: String },
    #[error("{case}: diagnosis expectation has no evidence path")]
    DiagnosisWithoutEvidence { case: String },
    #[error("{case}: fix set has no root-cause fix")]
    NoRootCauseFix { case: String },
    #[error("{case}: malformed timestamp in {path}: {timestamp}")]
    MalformedTimestamp {
        case: String,
        path: PathBuf,
        timestamp: String,
    },
    #[error("duplicate case slug {slug}")]
    DuplicateSlug { slug: String },
}

#[derive(Clone, Debug)]
pub struct Case {
    pub root: PathBuf,
    pub metadata: CaseMetadata,
    pub artifacts: LoadedArtifacts,
    pub commands: Vec<CommandFixture>,
    pub scoring: ScoringRules,
    pub diagnosis: DiagnosisExpectation,
    pub fixes: Vec<FixOption>,
    pub hints: Vec<Hint>,
    pub false_trails: Vec<FalseTrail>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CaseMetadata {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub difficulty: String,
    pub component: String,
    pub starts_at: String,
}

#[derive(Clone, Debug)]
pub struct LoadedArtifacts {
    pub brief: String,
    pub logs: Vec<Value>,
    pub metrics: toml::Value,
    pub schema_sql: String,
    pub sql_rows: BTreeMap<String, String>,
    pub traces: Value,
    pub diffs: BTreeMap<String, String>,
    pub runbooks: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CommandFixture {
    pub id: String,
    pub kind: CommandKind,
    pub command: String,
    pub fixture: PathBuf,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub damage: u32,
    #[serde(skip)]
    pub output: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CommandKind {
    Shell,
    Sql,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScoringRules {
    pub max_score: u32,
    pub time_budget_seconds: u32,
    pub evidence: Vec<String>,
    pub root_fix: String,
    #[serde(default)]
    pub symptom_fixes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DiagnosisExpectation {
    pub root_cause: String,
    pub affected_component: String,
    pub blast_radius: String,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FixOption {
    pub id: String,
    pub title: String,
    pub kind: FixKind,
    pub explanation: String,
    pub solves: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FixKind {
    RootCause,
    SymptomMask,
    Unsafe,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Hint {
    pub id: String,
    pub text: String,
    pub cost: u32,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FalseTrail {
    pub id: String,
    pub title: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CaseToml {
    case: CaseMetadata,
    artifacts: ArtifactRefs,
    diagnosis: DiagnosisExpectation,
    fixes: Vec<FixOption>,
    #[serde(default)]
    hints: Vec<Hint>,
    #[serde(default)]
    false_trails: Vec<FalseTrail>,
}

#[derive(Debug, Deserialize)]
struct ArtifactRefs {
    brief: PathBuf,
    logs: PathBuf,
    metrics: PathBuf,
    schema: PathBuf,
    #[serde(default)]
    rows: Vec<PathBuf>,
    traces: PathBuf,
    #[serde(default)]
    diffs: Vec<PathBuf>,
    #[serde(default)]
    runbooks: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct CommandsToml {
    commands: Vec<CommandFixture>,
}

#[derive(Debug, Deserialize)]
struct ScoringToml {
    scoring: ScoringRules,
}

pub fn load_case_dir(path: impl AsRef<Path>) -> Result<Case> {
    let root = path.as_ref();
    let case_path = root.join("case.toml");
    let commands_path = root.join("commands.toml");
    let scoring_path = root.join("scoring.toml");

    let case_toml: CaseToml = read_toml(&case_path)?;
    let mut commands_toml: CommandsToml = read_toml(&commands_path)?;
    let scoring_toml: ScoringToml = read_toml(&scoring_path)?;
    validate_required_metadata(&case_toml.case)?;

    let artifacts = load_artifacts(root, &case_toml.case.slug, &case_toml.artifacts)?;
    load_command_outputs(root, &case_toml.case.slug, &mut commands_toml.commands)?;

    let case = Case {
        root: root.to_path_buf(),
        metadata: case_toml.case,
        artifacts,
        commands: commands_toml.commands,
        scoring: scoring_toml.scoring,
        diagnosis: case_toml.diagnosis,
        fixes: case_toml.fixes,
        hints: case_toml.hints,
        false_trails: case_toml.false_trails,
    };
    validate_case(&case)?;
    Ok(case)
}

pub fn load_cases(root: impl AsRef<Path>) -> Result<Vec<Case>> {
    let root = root.as_ref();
    let mut cases = Vec::new();
    let mut slugs = BTreeSet::new();
    for entry in fs::read_dir(root).map_err(|source| ContentError::Read {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ContentError::Read {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() && path.join("case.toml").exists() {
            let case = load_case_dir(&path)?;
            if !slugs.insert(case.metadata.slug.clone()) {
                return Err(ContentError::DuplicateSlug {
                    slug: case.metadata.slug,
                });
            }
            cases.push(case);
        }
    }
    cases.sort_by(|left, right| left.metadata.slug.cmp(&right.metadata.slug));
    Ok(cases)
}

fn validate_required_metadata(metadata: &CaseMetadata) -> Result<()> {
    let case = metadata.slug.clone();
    for (field, value) in [
        ("id", &metadata.id),
        ("slug", &metadata.slug),
        ("title", &metadata.title),
        ("summary", &metadata.summary),
        ("difficulty", &metadata.difficulty),
        ("component", &metadata.component),
        ("starts_at", &metadata.starts_at),
    ] {
        if value.trim().is_empty() {
            return Err(ContentError::MissingField { case, field });
        }
    }
    validate_timestamp(&metadata.slug, Path::new("case.toml"), &metadata.starts_at)
}

fn load_artifacts(root: &Path, case: &str, refs: &ArtifactRefs) -> Result<LoadedArtifacts> {
    let brief = read_required_string(root, case, &refs.brief)?;
    let logs_path = root.join(&refs.logs);
    let logs = read_logs(case, &logs_path)?;
    let metrics = read_toml_value(&root.join(&refs.metrics))?;
    let schema_sql = read_required_string(root, case, &refs.schema)?;
    let mut sql_rows = BTreeMap::new();
    for rows in &refs.rows {
        sql_rows.insert(path_key(rows), read_required_string(root, case, rows)?);
    }
    let traces_path = root.join(&refs.traces);
    let traces = read_json(&traces_path)?;

    let mut diffs = BTreeMap::new();
    for diff in &refs.diffs {
        diffs.insert(path_key(diff), read_required_string(root, case, diff)?);
    }
    let mut runbooks = BTreeMap::new();
    for runbook in &refs.runbooks {
        runbooks.insert(
            path_key(runbook),
            read_required_string(root, case, runbook)?,
        );
    }

    Ok(LoadedArtifacts {
        brief,
        logs,
        metrics,
        schema_sql,
        sql_rows,
        traces,
        diffs,
        runbooks,
    })
}

fn load_command_outputs(root: &Path, case: &str, commands: &mut [CommandFixture]) -> Result<()> {
    let mut ids = BTreeSet::new();
    for command in commands {
        if !ids.insert(command.id.clone()) {
            return Err(ContentError::DuplicateId {
                case: case.to_owned(),
                kind: "command",
                id: command.id.clone(),
            });
        }
        if command.fixture.as_os_str().is_empty() {
            return Err(ContentError::CommandWithoutFixture {
                case: case.to_owned(),
                command: command.command.clone(),
            });
        }
        command.output = read_required_string(root, case, &command.fixture)?;
    }
    Ok(())
}

fn validate_case(case: &Case) -> Result<()> {
    let evidence_ids: BTreeSet<_> = case.scoring.evidence.iter().cloned().collect();
    if case.diagnosis.evidence.is_empty() {
        return Err(ContentError::DiagnosisWithoutEvidence {
            case: case.metadata.slug.clone(),
        });
    }
    for evidence in &case.diagnosis.evidence {
        if !evidence_ids.contains(evidence) {
            return Err(ContentError::UnknownScoringEvidence {
                case: case.metadata.slug.clone(),
                evidence: evidence.clone(),
            });
        }
    }
    for command in &case.commands {
        for evidence in &command.evidence {
            if !evidence_ids.contains(evidence) {
                return Err(ContentError::UnknownScoringEvidence {
                    case: case.metadata.slug.clone(),
                    evidence: evidence.clone(),
                });
            }
        }
    }

    let fix_ids: BTreeSet<_> = case.fixes.iter().map(|fix| fix.id.as_str()).collect();
    if !fix_ids.contains(case.scoring.root_fix.as_str()) {
        return Err(ContentError::UnknownRootFix {
            case: case.metadata.slug.clone(),
            fix: case.scoring.root_fix.clone(),
        });
    }
    if !case
        .fixes
        .iter()
        .any(|fix| fix.kind == FixKind::RootCause && fix.solves)
    {
        return Err(ContentError::NoRootCauseFix {
            case: case.metadata.slug.clone(),
        });
    }
    Ok(())
}

fn read_required_string(root: &Path, case: &str, rel: &Path) -> Result<String> {
    let path = root.join(rel);
    if !path.exists() {
        return Err(ContentError::MissingArtifact {
            case: case.to_owned(),
            path: rel.to_path_buf(),
        });
    }
    fs::read_to_string(&path).map_err(|source| ContentError::Read { path, source })
}

fn read_logs(case: &str, path: &Path) -> Result<Vec<Value>> {
    let source = fs::read_to_string(path).map_err(|source| ContentError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut logs = Vec::new();
    for line in source.lines().filter(|line| !line.trim().is_empty()) {
        let value: Value = serde_json::from_str(line).map_err(|source| ContentError::Json {
            path: path.to_path_buf(),
            source,
        })?;
        if let Some(timestamp) = value.get("timestamp").and_then(Value::as_str) {
            validate_timestamp(case, path, timestamp)?;
        } else {
            return Err(ContentError::MalformedTimestamp {
                case: case.to_owned(),
                path: path.to_path_buf(),
                timestamp: "<missing>".to_owned(),
            });
        }
        logs.push(value);
    }
    Ok(logs)
}

fn validate_timestamp(case: &str, path: &Path, timestamp: &str) -> Result<()> {
    let looks_utc = timestamp.len() >= 20 && timestamp.contains('T') && timestamp.ends_with('Z');
    if looks_utc {
        Ok(())
    } else {
        Err(ContentError::MalformedTimestamp {
            case: case.to_owned(),
            path: path.to_path_buf(),
            timestamp: timestamp.to_owned(),
        })
    }
}

fn read_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let source = fs::read_to_string(path).map_err(|source| ContentError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&source).map_err(|source| ContentError::Toml {
        path: path.to_path_buf(),
        source,
    })
}

fn read_toml_value(path: &Path) -> Result<toml::Value> {
    read_toml(path)
}

fn read_json(path: &Path) -> Result<Value> {
    let source = fs::read_to_string(path).map_err(|source| ContentError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&source).map_err(|source| ContentError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_case() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cases/slow-checkout")
    }

    #[test]
    fn loads_seed_case_artifacts_and_commands() {
        let case = load_case_dir(fixture_case()).expect("seed case loads");
        assert_eq!(case.metadata.slug, "slow-checkout");
        assert!(case.artifacts.brief.contains("Checkout latency"));
        assert_eq!(case.artifacts.logs.len(), 6);
        assert!(case.artifacts.sql_rows.contains_key("rows/orders.csv"));
        assert!(case.artifacts.diffs.contains_key("diffs/query-shape.diff"));
        assert_eq!(case.commands.len(), 6);
        assert!(
            case.commands
                .iter()
                .all(|command| !command.output.is_empty())
        );
    }

    #[test]
    fn validates_case_collection_without_duplicate_slugs() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cases");
        let cases = load_cases(root).expect("case collection loads");
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].metadata.title, "Slow Checkout");
    }
}
