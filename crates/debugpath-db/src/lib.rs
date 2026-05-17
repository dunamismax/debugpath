#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptRecord {
    pub player_handle: String,
    pub case_slug: String,
    pub score: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial",
    sql: include_str!("../migrations/0001_initial.sql"),
}];

impl AttemptRecord {
    pub fn new(player_handle: impl Into<String>, case_slug: impl Into<String>, score: u32) -> Self {
        Self {
            player_handle: player_handle.into(),
            case_slug: case_slug.into(),
            score,
        }
    }
}

pub fn migrations() -> &'static [Migration] {
    MIGRATIONS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attempt_record_preserves_leaderboard_fields() {
        let record = AttemptRecord::new("anon", "slow-checkout", 930);
        assert_eq!(record.player_handle, "anon");
        assert_eq!(record.case_slug, "slow-checkout");
        assert_eq!(record.score, 930);
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
}
