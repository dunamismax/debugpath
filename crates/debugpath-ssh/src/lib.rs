pub const LOCAL_DEV_BIND_ADDR: &str = "127.0.0.1:2222";
pub const PRODUCTION_ENTRYPOINT: &str = "ssh debugpath.dev";

pub fn smoke_summary() -> String {
    format!("{PRODUCTION_ENTRYPOINT} planned; local development will bind {LOCAL_DEV_BIND_ADDR}")
}

pub mod abuse {
    use std::collections::BTreeMap;
    use std::net::{IpAddr, SocketAddr};

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct AbuseConfig {
        pub max_sessions_per_peer: u32,
        pub max_connections_per_window: u32,
        pub window_seconds: u64,
        pub max_command_bytes: usize,
    }

    impl Default for AbuseConfig {
        fn default() -> Self {
            Self {
                max_sessions_per_peer: 2,
                max_connections_per_window: 10,
                window_seconds: 60,
                max_command_bytes: 512,
            }
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum AbuseDecision {
        Accepted,
        Rejected(RejectReason),
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum RejectReason {
        TooManySessions,
        RateLimited,
        CommandTooLarge,
        EmptyCommand,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct AuditEvent {
        pub at_seconds: u64,
        pub peer: String,
        pub action: AuditAction,
        pub decision: AbuseDecision,
        pub detail: String,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum AuditAction {
        SessionStart,
        SessionEnd,
        Command,
    }

    #[derive(Clone, Debug)]
    pub struct AbuseControls {
        config: AbuseConfig,
        peers: BTreeMap<String, PeerState>,
        audit_events: Vec<AuditEvent>,
    }

    #[derive(Clone, Debug, Default)]
    struct PeerState {
        active_sessions: u32,
        window_started_at: u64,
        connection_attempts: u32,
    }

    impl AbuseControls {
        pub fn new(config: AbuseConfig) -> Self {
            Self {
                config,
                peers: BTreeMap::new(),
                audit_events: Vec::new(),
            }
        }

        pub fn config(&self) -> &AbuseConfig {
            &self.config
        }

        pub fn audit_events(&self) -> &[AuditEvent] {
            &self.audit_events
        }

        pub fn active_sessions(&self, peer: &str) -> u32 {
            self.peers
                .get(&peer_key(peer))
                .map(|state| state.active_sessions)
                .unwrap_or_default()
        }

        pub fn start_session(&mut self, peer: &str, now_seconds: u64) -> AbuseDecision {
            let key = peer_key(peer);
            let state = self.peers.entry(key.clone()).or_default();
            reset_window_if_expired(state, now_seconds, self.config.window_seconds);
            state.connection_attempts = state.connection_attempts.saturating_add(1);

            let decision = if state.connection_attempts > self.config.max_connections_per_window {
                AbuseDecision::Rejected(RejectReason::RateLimited)
            } else if state.active_sessions >= self.config.max_sessions_per_peer {
                AbuseDecision::Rejected(RejectReason::TooManySessions)
            } else {
                state.active_sessions = state.active_sessions.saturating_add(1);
                AbuseDecision::Accepted
            };

            self.audit(
                now_seconds,
                peer,
                AuditAction::SessionStart,
                decision.clone(),
                "session start",
            );
            decision
        }

        pub fn end_session(&mut self, peer: &str, now_seconds: u64) {
            let key = peer_key(peer);
            if let Some(state) = self.peers.get_mut(&key) {
                state.active_sessions = state.active_sessions.saturating_sub(1);
            }
            self.audit(
                now_seconds,
                peer,
                AuditAction::SessionEnd,
                AbuseDecision::Accepted,
                "session end",
            );
        }

        pub fn inspect_command(
            &mut self,
            peer: &str,
            command: &str,
            now_seconds: u64,
        ) -> AbuseDecision {
            let decision = if command.trim().is_empty() {
                AbuseDecision::Rejected(RejectReason::EmptyCommand)
            } else if command.len() > self.config.max_command_bytes {
                AbuseDecision::Rejected(RejectReason::CommandTooLarge)
            } else {
                AbuseDecision::Accepted
            };
            self.audit(
                now_seconds,
                peer,
                AuditAction::Command,
                decision.clone(),
                &format!("command_bytes={}", command.len()),
            );
            decision
        }

        fn audit(
            &mut self,
            at_seconds: u64,
            peer: &str,
            action: AuditAction,
            decision: AbuseDecision,
            detail: &str,
        ) {
            self.audit_events.push(AuditEvent {
                at_seconds,
                peer: redact_peer(peer),
                action,
                decision,
                detail: redact_detail(detail),
            });
        }
    }

    fn reset_window_if_expired(state: &mut PeerState, now_seconds: u64, window_seconds: u64) {
        if state.window_started_at == 0
            || now_seconds.saturating_sub(state.window_started_at) >= window_seconds
        {
            state.window_started_at = now_seconds;
            state.connection_attempts = 0;
        }
    }

    fn peer_key(peer: &str) -> String {
        parse_peer_ip(peer)
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| peer.to_owned())
    }

    pub fn redact_peer(peer: &str) -> String {
        match parse_peer_ip(peer) {
            Some(IpAddr::V4(ip)) => {
                let octets = ip.octets();
                format!("{}.{}.{}.x", octets[0], octets[1], octets[2])
            }
            Some(IpAddr::V6(ip)) => {
                let segments = ip.segments();
                format!(
                    "{:x}:{:x}:{:x}:redacted",
                    segments[0], segments[1], segments[2]
                )
            }
            None => "unparsed-peer".to_owned(),
        }
    }

    pub fn redact_detail(detail: &str) -> String {
        detail
            .split_whitespace()
            .map(redact_detail_token)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn redact_detail_token(token: &str) -> String {
        for marker in ["token=", "password=", "secret=", "key="] {
            if let Some((prefix, _)) = token.split_once(marker) {
                return format!("{prefix}{marker}[redacted]");
            }
        }
        token.to_owned()
    }

    fn parse_peer_ip(peer: &str) -> Option<IpAddr> {
        peer.parse::<SocketAddr>()
            .map(|addr| addr.ip())
            .or_else(|_| peer.parse::<IpAddr>())
            .ok()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn rate_limits_connection_attempts_per_peer_window() {
            let mut controls = AbuseControls::new(AbuseConfig {
                max_sessions_per_peer: 10,
                max_connections_per_window: 2,
                window_seconds: 60,
                max_command_bytes: 512,
            });

            assert_eq!(
                controls.start_session("203.0.113.10:4400", 10),
                AbuseDecision::Accepted
            );
            controls.end_session("203.0.113.10:4400", 11);
            assert_eq!(
                controls.start_session("203.0.113.10:4401", 12),
                AbuseDecision::Accepted
            );
            controls.end_session("203.0.113.10:4401", 13);
            assert_eq!(
                controls.start_session("203.0.113.10:4402", 14),
                AbuseDecision::Rejected(RejectReason::RateLimited)
            );
            assert_eq!(
                controls.start_session("203.0.113.10:4403", 75),
                AbuseDecision::Accepted
            );
        }

        #[test]
        fn enforces_active_session_limit() {
            let mut controls = AbuseControls::new(AbuseConfig {
                max_sessions_per_peer: 1,
                ..AbuseConfig::default()
            });

            assert_eq!(
                controls.start_session("198.51.100.24:5000", 1),
                AbuseDecision::Accepted
            );
            assert_eq!(
                controls.start_session("198.51.100.24:5001", 2),
                AbuseDecision::Rejected(RejectReason::TooManySessions)
            );
            assert_eq!(controls.active_sessions("198.51.100.24"), 1);
            controls.end_session("198.51.100.24:5000", 3);
            assert_eq!(controls.active_sessions("198.51.100.24"), 0);
        }

        #[test]
        fn rejects_empty_or_oversized_commands_before_engine_handling() {
            let mut controls = AbuseControls::new(AbuseConfig {
                max_command_bytes: 4,
                ..AbuseConfig::default()
            });

            assert_eq!(
                controls.inspect_command("127.0.0.1:2222", "logs", 1),
                AbuseDecision::Accepted
            );
            assert_eq!(
                controls.inspect_command("127.0.0.1:2222", "", 2),
                AbuseDecision::Rejected(RejectReason::EmptyCommand)
            );
            assert_eq!(
                controls.inspect_command("127.0.0.1:2222", "logss", 3),
                AbuseDecision::Rejected(RejectReason::CommandTooLarge)
            );
        }

        #[test]
        fn audit_events_redact_connection_metadata_and_secret_like_details() {
            let mut controls = AbuseControls::new(AbuseConfig::default());
            controls.start_session("203.0.113.44:3300", 7);
            controls.audit(
                8,
                "2001:db8:abcd:0012::1",
                AuditAction::Command,
                AbuseDecision::Accepted,
                "command_bytes=12 token=abc123 password=hunter2",
            );

            assert_eq!(controls.audit_events()[0].peer, "203.0.113.x");
            assert_eq!(controls.audit_events()[1].peer, "2001:db8:abcd:redacted");
            assert!(
                controls.audit_events()[1]
                    .detail
                    .contains("token=[redacted]")
            );
            assert!(
                controls.audit_events()[1]
                    .detail
                    .contains("password=[redacted]")
            );
            assert!(!controls.audit_events()[1].detail.contains("hunter2"));
        }
    }
}
