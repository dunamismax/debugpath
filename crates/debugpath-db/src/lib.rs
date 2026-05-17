use chrono::{DateTime, Utc};
use debugpath_engine::{DiagnosisSubmission, ReplayEvent, Score};
use serde::Serialize;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("database migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("failed to encode JSON payload: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptStatus {
    Started,
    Submitted,
    Fixed,
    Abandoned,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedCaseRecord {
    pub slug: String,
    pub case_id: String,
    pub title: String,
    pub summary: String,
    pub difficulty: String,
    pub component: String,
    pub content_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerRecord {
    pub handle: String,
    pub display_name: Option<String>,
    pub is_anonymous: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptRecord {
    pub player_id: Uuid,
    pub case_slug: String,
    pub engine_version: String,
    pub case_content_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnlockRecord {
    pub player_id: Uuid,
    pub case_slug: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AuthoredCaseDraftRecord {
    pub owner_player_id: Option<Uuid>,
    pub slug: String,
    pub title: String,
    pub status: DraftStatus,
    pub draft: Value,
    pub validation_errors: Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum DraftStatus {
    Draft,
    Review,
    Rejected,
    Published,
}

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub struct LeaderboardRow {
    pub rank: i64,
    pub player_handle: String,
    pub case_slug: String,
    pub score: i32,
    pub solved_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub struct RecentSolveRow {
    pub player_handle: String,
    pub case_slug: String,
    pub case_title: String,
    pub score: i32,
    pub solved_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredReplayEvent {
    pub sequence: i32,
    pub event_type: String,
    pub payload: Value,
    pub occurred_at: DateTime<Utc>,
}

pub struct Database {
    pool: PgPool,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial",
    sql: include_str!("../migrations/0001_initial.sql"),
}];

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

impl AttemptStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Submitted => "submitted",
            Self::Fixed => "fixed",
            Self::Abandoned => "abandoned",
        }
    }
}

impl DraftStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Review => "review",
            Self::Rejected => "rejected",
            Self::Published => "published",
        }
    }
}

impl PublishedCaseRecord {
    pub fn new(
        slug: impl Into<String>,
        case_id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        difficulty: impl Into<String>,
        component: impl Into<String>,
        content_version: impl Into<String>,
    ) -> Self {
        Self {
            slug: slug.into(),
            case_id: case_id.into(),
            title: title.into(),
            summary: summary.into(),
            difficulty: difficulty.into(),
            component: component.into(),
            content_version: content_version.into(),
        }
    }
}

impl PlayerRecord {
    pub fn anonymous(handle: impl Into<String>) -> Self {
        Self {
            handle: handle.into(),
            display_name: None,
            is_anonymous: true,
        }
    }
}

impl AttemptRecord {
    pub fn new(
        player_id: Uuid,
        case_slug: impl Into<String>,
        engine_version: impl Into<String>,
        case_content_version: impl Into<String>,
    ) -> Self {
        Self {
            player_id,
            case_slug: case_slug.into(),
            engine_version: engine_version.into(),
            case_content_version: case_content_version.into(),
        }
    }
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self::new(pool))
    }

    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<()> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }

    pub async fn upsert_published_case(&self, case: &PublishedCaseRecord) -> Result<()> {
        sqlx::query(sql::UPSERT_PUBLISHED_CASE)
            .bind(&case.slug)
            .bind(&case.case_id)
            .bind(&case.title)
            .bind(&case.summary)
            .bind(&case.difficulty)
            .bind(&case.component)
            .bind(&case.content_version)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn upsert_player(&self, player: &PlayerRecord) -> Result<Uuid> {
        let id = sqlx::query_scalar::<_, Uuid>(sql::UPSERT_PLAYER)
            .bind(&player.handle)
            .bind(&player.display_name)
            .bind(player.is_anonymous)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn start_attempt(&self, attempt: &AttemptRecord) -> Result<Uuid> {
        let id = sqlx::query_scalar::<_, Uuid>(sql::INSERT_ATTEMPT)
            .bind(attempt.player_id)
            .bind(&attempt.case_slug)
            .bind(AttemptStatus::Started.as_str())
            .bind(&attempt.engine_version)
            .bind(&attempt.case_content_version)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn submit_diagnosis(
        &self,
        attempt_id: Uuid,
        diagnosis: &DiagnosisSubmission,
    ) -> Result<Uuid> {
        let mut tx = self.pool.begin().await?;
        let id = sqlx::query_scalar::<_, Uuid>(sql::INSERT_DIAGNOSIS)
            .bind(attempt_id)
            .bind(&diagnosis.root_cause)
            .bind(Json(diagnosis.evidence.clone()))
            .bind(&diagnosis.affected_component)
            .bind(&diagnosis.proposed_fix)
            .bind(&diagnosis.blast_radius)
            .fetch_one(&mut *tx)
            .await?;
        sqlx::query(sql::MARK_ATTEMPT_SUBMITTED)
            .bind(attempt_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(id)
    }

    pub async fn record_score(&self, attempt_id: Uuid, score: &Score) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(sql::UPSERT_SCORE)
            .bind(attempt_id)
            .bind(score.total as i32)
            .bind(score.max_score as i32)
            .bind(score.root_cause_correct)
            .bind(score.fix_solved)
            .bind(score.evidence_found as i32)
            .bind(score.damage_penalty as i32)
            .bind(score.hint_penalty as i32)
            .bind(score.time_penalty as i32)
            .execute(&mut *tx)
            .await?;
        let status = if score.fix_solved {
            AttemptStatus::Fixed
        } else {
            AttemptStatus::Submitted
        };
        sqlx::query(sql::MARK_ATTEMPT_SCORED)
            .bind(status.as_str())
            .bind(attempt_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn append_replay_events(
        &self,
        attempt_id: Uuid,
        start_sequence: i32,
        events: &[ReplayEvent],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for (offset, event) in events.iter().enumerate() {
            let payload = serde_json::to_value(event)?;
            sqlx::query(sql::INSERT_REPLAY_EVENT)
                .bind(attempt_id)
                .bind(start_sequence + offset as i32)
                .bind(replay_event_type(event))
                .bind(Json(payload))
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn unlock_case(&self, unlock: &UnlockRecord) -> Result<()> {
        sqlx::query(sql::UPSERT_UNLOCK)
            .bind(unlock.player_id)
            .bind(&unlock.case_slug)
            .bind(&unlock.reason)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn save_authored_case_draft(&self, draft: &AuthoredCaseDraftRecord) -> Result<Uuid> {
        let id = sqlx::query_scalar::<_, Uuid>(sql::INSERT_AUTHORED_DRAFT)
            .bind(draft.owner_player_id)
            .bind(&draft.slug)
            .bind(&draft.title)
            .bind(draft.status.as_str())
            .bind(Json(draft.draft.clone()))
            .bind(Json(draft.validation_errors.clone()))
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardRow>> {
        let rows = sqlx::query_as::<_, LeaderboardRow>(sql::SELECT_LEADERBOARD)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn recent_solves(&self, limit: i64) -> Result<Vec<RecentSolveRow>> {
        let rows = sqlx::query_as::<_, RecentSolveRow>(sql::SELECT_RECENT_SOLVES)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn replay_events(&self, attempt_id: Uuid) -> Result<Vec<StoredReplayEvent>> {
        let rows = sqlx::query(sql::SELECT_REPLAY_EVENTS)
            .bind(attempt_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| StoredReplayEvent {
                sequence: row.get("sequence"),
                event_type: row.get("event_type"),
                payload: row.get::<Json<Value>, _>("payload").0,
                occurred_at: row.get("occurred_at"),
            })
            .collect())
    }
}

pub fn migrations() -> &'static [Migration] {
    MIGRATIONS
}

fn replay_event_type(event: &ReplayEvent) -> &'static str {
    match event {
        ReplayEvent::CommandRun { .. } => "command_run",
        ReplayEvent::CommandRejected { .. } => "command_rejected",
        ReplayEvent::HintUsed { .. } => "hint_used",
        ReplayEvent::DiagnosisSubmitted => "diagnosis_submitted",
        ReplayEvent::FixApplied { .. } => "fix_applied",
    }
}

pub mod sql {
    pub const UPSERT_PUBLISHED_CASE: &str = r#"
INSERT INTO published_cases (
    slug, case_id, title, summary, difficulty, component, content_version
)
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (slug) DO UPDATE SET
    case_id = EXCLUDED.case_id,
    title = EXCLUDED.title,
    summary = EXCLUDED.summary,
    difficulty = EXCLUDED.difficulty,
    component = EXCLUDED.component,
    content_version = EXCLUDED.content_version,
    retired_at = NULL
"#;

    pub const UPSERT_PLAYER: &str = r#"
INSERT INTO players (handle, display_name, is_anonymous)
VALUES ($1, $2, $3)
ON CONFLICT (handle) DO UPDATE SET
    display_name = EXCLUDED.display_name,
    is_anonymous = EXCLUDED.is_anonymous,
    last_seen_at = now()
RETURNING id
"#;

    pub const INSERT_ATTEMPT: &str = r#"
INSERT INTO attempts (
    player_id, case_slug, status, engine_version, case_content_version
)
VALUES ($1, $2, $3, $4, $5)
RETURNING id
"#;

    pub const INSERT_DIAGNOSIS: &str = r#"
INSERT INTO diagnosis_submissions (
    attempt_id, root_cause, evidence, affected_component, proposed_fix, blast_radius
)
VALUES ($1, $2, $3, $4, $5, $6)
RETURNING id
"#;

    pub const MARK_ATTEMPT_SUBMITTED: &str = r#"
UPDATE attempts
SET status = 'submitted', submitted_at = now()
WHERE id = $1
"#;

    pub const UPSERT_SCORE: &str = r#"
INSERT INTO scores (
    attempt_id, total, max_score, root_cause_correct, fix_solved, evidence_found,
    damage_penalty, hint_penalty, time_penalty
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT (attempt_id) DO UPDATE SET
    total = EXCLUDED.total,
    max_score = EXCLUDED.max_score,
    root_cause_correct = EXCLUDED.root_cause_correct,
    fix_solved = EXCLUDED.fix_solved,
    evidence_found = EXCLUDED.evidence_found,
    damage_penalty = EXCLUDED.damage_penalty,
    hint_penalty = EXCLUDED.hint_penalty,
    time_penalty = EXCLUDED.time_penalty,
    scored_at = now()
"#;

    pub const MARK_ATTEMPT_SCORED: &str = r#"
UPDATE attempts
SET status = $1,
    completed_at = CASE WHEN $1 = 'fixed' THEN now() ELSE completed_at END
WHERE id = $2
"#;

    pub const INSERT_REPLAY_EVENT: &str = r#"
INSERT INTO replay_events (attempt_id, sequence, event_type, payload)
VALUES ($1, $2, $3, $4)
ON CONFLICT (attempt_id, sequence) DO UPDATE SET
    event_type = EXCLUDED.event_type,
    payload = EXCLUDED.payload
"#;

    pub const UPSERT_UNLOCK: &str = r#"
INSERT INTO unlocks (player_id, case_slug, reason)
VALUES ($1, $2, $3)
ON CONFLICT (player_id, case_slug) DO UPDATE SET
    reason = EXCLUDED.reason,
    unlocked_at = now()
"#;

    pub const INSERT_AUTHORED_DRAFT: &str = r#"
INSERT INTO authored_case_drafts (
    owner_player_id, slug, title, status, draft, validation_errors
)
VALUES ($1, $2, $3, $4, $5, $6)
RETURNING id
"#;

    pub const SELECT_LEADERBOARD: &str = r#"
SELECT
    row_number() OVER (ORDER BY scores.total DESC, scores.scored_at ASC)::BIGINT AS rank,
    players.handle AS player_handle,
    attempts.case_slug,
    scores.total AS score,
    scores.scored_at AS solved_at
FROM scores
JOIN attempts ON attempts.id = scores.attempt_id
JOIN players ON players.id = attempts.player_id
WHERE attempts.status = 'fixed'
ORDER BY scores.total DESC, scores.scored_at ASC
LIMIT $1
"#;

    pub const SELECT_RECENT_SOLVES: &str = r#"
SELECT
    players.handle AS player_handle,
    attempts.case_slug,
    published_cases.title AS case_title,
    scores.total AS score,
    scores.scored_at AS solved_at
FROM scores
JOIN attempts ON attempts.id = scores.attempt_id
JOIN players ON players.id = attempts.player_id
JOIN published_cases ON published_cases.slug = attempts.case_slug
WHERE attempts.status = 'fixed'
ORDER BY scores.scored_at DESC
LIMIT $1
"#;

    pub const SELECT_REPLAY_EVENTS: &str = r#"
SELECT sequence, event_type, payload, occurred_at
FROM replay_events
WHERE attempt_id = $1
ORDER BY sequence ASC
"#;
}

#[cfg(test)]
mod tests {
    use super::*;
    use debugpath_engine::{DiagnosisSubmission, ReplayEvent, Score};

    #[test]
    fn attempt_statuses_match_migration_check_constraint() {
        let migration = migrations()[0].sql;
        for status in [
            AttemptStatus::Started,
            AttemptStatus::Submitted,
            AttemptStatus::Fixed,
            AttemptStatus::Abandoned,
        ] {
            assert!(migration.contains(status.as_str()));
        }
    }

    #[test]
    fn attempt_record_preserves_attempt_fields() {
        let player_id = Uuid::new_v4();
        let record = AttemptRecord::new(player_id, "slow-checkout", "engine-1", "case-sha");
        assert_eq!(record.player_id, player_id);
        assert_eq!(record.case_slug, "slow-checkout");
        assert_eq!(record.engine_version, "engine-1");
        assert_eq!(record.case_content_version, "case-sha");
    }

    #[test]
    fn migrations_are_ordered_and_non_empty() {
        let migrations = migrations();
        assert!(!migrations.is_empty());

        for (index, migration) in migrations.iter().enumerate() {
            assert_eq!(migration.version as usize, index + 1);
            assert!(!migration.name.trim().is_empty());
            assert!(!migration.sql.trim().is_empty());
        }
    }

    #[test]
    fn initial_migration_declares_required_storage_surfaces() {
        let sql = migrations()[0].sql;
        for table in [
            "published_cases",
            "players",
            "attempts",
            "diagnosis_submissions",
            "scores",
            "replay_events",
            "unlocks",
            "authored_case_drafts",
        ] {
            assert!(
                sql.contains(&format!("CREATE TABLE {table}")),
                "missing table {table}"
            );
        }
    }

    #[test]
    fn storage_queries_cover_mvp_write_and_read_paths() {
        for (name, query, tables) in [
            (
                "published cases",
                sql::UPSERT_PUBLISHED_CASE,
                &["published_cases"][..],
            ),
            ("players", sql::UPSERT_PLAYER, &["players"][..]),
            ("attempts", sql::INSERT_ATTEMPT, &["attempts"][..]),
            (
                "diagnosis",
                sql::INSERT_DIAGNOSIS,
                &["diagnosis_submissions"][..],
            ),
            ("scores", sql::UPSERT_SCORE, &["scores"][..]),
            ("replay", sql::INSERT_REPLAY_EVENT, &["replay_events"][..]),
            ("unlocks", sql::UPSERT_UNLOCK, &["unlocks"][..]),
            (
                "drafts",
                sql::INSERT_AUTHORED_DRAFT,
                &["authored_case_drafts"][..],
            ),
            (
                "leaderboard",
                sql::SELECT_LEADERBOARD,
                &["scores", "attempts", "players"][..],
            ),
            (
                "recent solves",
                sql::SELECT_RECENT_SOLVES,
                &["scores", "attempts", "players", "published_cases"][..],
            ),
        ] {
            for table in tables {
                assert!(
                    query.contains(table),
                    "{name} query does not mention {table}"
                );
            }
        }
    }

    #[test]
    fn replay_events_have_stable_storage_type_names() {
        assert_eq!(
            replay_event_type(&ReplayEvent::CommandRun {
                command: "logs".to_owned(),
                evidence: vec!["evidence".to_owned()],
                damage: 0,
            }),
            "command_run"
        );
        assert_eq!(
            replay_event_type(&ReplayEvent::HintUsed {
                hint_id: "hint".to_owned(),
                cost: 25,
            }),
            "hint_used"
        );
        assert_eq!(
            replay_event_type(&ReplayEvent::DiagnosisSubmitted),
            "diagnosis_submitted"
        );
        assert_eq!(
            replay_event_type(&ReplayEvent::FixApplied {
                fix_id: "fix".to_owned(),
                solves: true,
            }),
            "fix_applied"
        );
    }

    #[tokio::test]
    async fn live_postgres_round_trip_stores_mvp_records() {
        let Ok(database_url) = std::env::var("DEBUGPATH_TEST_DATABASE_URL") else {
            eprintln!("skipping live postgres round trip; DEBUGPATH_TEST_DATABASE_URL is unset");
            return;
        };

        let db = Database::connect(&database_url)
            .await
            .expect("connect to test database");
        db.migrate().await.expect("migrate test database");
        let handle = format!("roundtrip-{}", Uuid::new_v4().simple());

        db.upsert_published_case(&PublishedCaseRecord::new(
            "slow-checkout",
            "case-slow-checkout",
            "Slow Checkout",
            "Latency jumps after deploy.",
            "intro",
            "checkout-api orders query",
            "test-content-version",
        ))
        .await
        .expect("upsert case");
        let player_id = db
            .upsert_player(&PlayerRecord::anonymous(&handle))
            .await
            .expect("upsert player");
        let attempt_id = db
            .start_attempt(&AttemptRecord::new(
                player_id,
                "slow-checkout",
                "engine-test",
                "test-content-version",
            ))
            .await
            .expect("start attempt");

        db.submit_diagnosis(
            attempt_id,
            &DiagnosisSubmission {
                root_cause: "missing index".to_owned(),
                evidence: vec!["seq-scan-orders".to_owned()],
                affected_component: "checkout-api".to_owned(),
                proposed_fix: "add_orders_status_created_at_index".to_owned(),
                blast_radius: "checkout reads".to_owned(),
            },
        )
        .await
        .expect("store diagnosis");
        db.record_score(
            attempt_id,
            &Score {
                total: 950,
                max_score: 1000,
                root_cause_correct: true,
                fix_solved: true,
                evidence_found: 1,
                damage_penalty: 0,
                hint_penalty: 0,
                time_penalty: 0,
            },
        )
        .await
        .expect("store score");
        db.append_replay_events(
            attempt_id,
            0,
            &[
                ReplayEvent::CommandRun {
                    command: "sql explain checkout_recent_orders".to_owned(),
                    evidence: vec!["seq-scan-orders".to_owned()],
                    damage: 0,
                },
                ReplayEvent::DiagnosisSubmitted,
                ReplayEvent::FixApplied {
                    fix_id: "add_orders_status_created_at_index".to_owned(),
                    solves: true,
                },
            ],
        )
        .await
        .expect("store replay events");
        db.unlock_case(&UnlockRecord {
            player_id,
            case_slug: "slow-checkout".to_owned(),
            reason: "completed-prerequisite".to_owned(),
        })
        .await
        .expect("store unlock");
        db.save_authored_case_draft(&AuthoredCaseDraftRecord {
            owner_player_id: Some(player_id),
            slug: "draft-case".to_owned(),
            title: "Draft Case".to_owned(),
            status: DraftStatus::Draft,
            draft: serde_json::json!({"case": {"slug": "draft-case"}}),
            validation_errors: serde_json::json!([]),
        })
        .await
        .expect("store authored draft");

        let leaderboard = db.leaderboard(50).await.expect("fetch leaderboard");
        let leaderboard_row = leaderboard
            .iter()
            .find(|row| row.player_handle == handle)
            .expect("leaderboard includes stored player");
        assert_eq!(leaderboard_row.case_slug, "slow-checkout");
        assert_eq!(leaderboard_row.score, 950);

        let solves = db.recent_solves(50).await.expect("fetch recent solves");
        let solve = solves
            .iter()
            .find(|row| row.player_handle == handle)
            .expect("recent solves includes stored player");
        assert_eq!(solve.case_title, "Slow Checkout");

        let replay = db.replay_events(attempt_id).await.expect("fetch replay");
        assert_eq!(replay.len(), 3);
        assert_eq!(replay[0].event_type, "command_run");
        assert_eq!(replay[1].event_type, "diagnosis_submitted");
        assert_eq!(replay[2].event_type, "fix_applied");
    }
}
