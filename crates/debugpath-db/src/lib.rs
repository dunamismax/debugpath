#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptRecord {
    pub player_handle: String,
    pub case_slug: String,
    pub score: u32,
}

impl AttemptRecord {
    pub fn new(player_handle: impl Into<String>, case_slug: impl Into<String>, score: u32) -> Self {
        Self {
            player_handle: player_handle.into(),
            case_slug: case_slug.into(),
            score,
        }
    }
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
}
