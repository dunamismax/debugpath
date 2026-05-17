pub const CORE_PANES: &[&str] = &[
    "Brief", "Systems", "Logs", "Metrics", "Shell", "SQL", "Trace", "Notes",
];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppModel {
    active_pane: usize,
}

impl AppModel {
    pub fn active_pane(&self) -> &'static str {
        CORE_PANES[self.active_pane]
    }

    pub fn next_pane(&mut self) {
        self.active_pane = (self.active_pane + 1) % CORE_PANES.len();
    }

    pub fn previous_pane(&mut self) {
        self.active_pane = if self.active_pane == 0 {
            CORE_PANES.len() - 1
        } else {
            self.active_pane - 1
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut model = AppModel::default();
        assert_eq!(model.active_pane(), "Brief");
        model.previous_pane();
        assert_eq!(model.active_pane(), "Notes");
        model.next_pane();
        assert_eq!(model.active_pane(), "Brief");
    }
}
