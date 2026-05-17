use debugpath_content::{Case, load_cases};
use debugpath_tui::AppModel;
use russh::keys::ssh_key;
use russh::server::{Auth, Config, Msg, Server as _, Session};
use russh::{Channel, ChannelId, Pty};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;

pub const LOCAL_DEV_BIND_ADDR: &str = "127.0.0.1:2222";
pub const PRODUCTION_ENTRYPOINT: &str = "ssh debugpath.dev";
pub const DEFAULT_DEV_CASE_SLUG: &str = "slow-checkout";

pub type Result<T> = std::result::Result<T, anyhow::Error>;

pub fn smoke_summary() -> String {
    format!(
        "{PRODUCTION_ENTRYPOINT} planned; local development binds {LOCAL_DEV_BIND_ADDR} by default"
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalSshConfig {
    pub bind_addr: SocketAddr,
    pub cases_root: PathBuf,
    pub case_slug: String,
    pub abuse: abuse::AbuseConfig,
}

impl LocalSshConfig {
    pub fn from_env() -> Result<Self> {
        let bind_addr = std::env::var("DEBUGPATH_SSH_BIND")
            .unwrap_or_else(|_| LOCAL_DEV_BIND_ADDR.to_owned())
            .parse()?;
        let cases_root = std::env::var("DEBUGPATH_CASES_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("cases"));
        let case_slug = std::env::var("DEBUGPATH_CASE_SLUG")
            .unwrap_or_else(|_| DEFAULT_DEV_CASE_SLUG.to_owned());
        Ok(Self {
            bind_addr,
            cases_root,
            case_slug,
            abuse: abuse::AbuseConfig::default(),
        })
    }
}

pub async fn run_local_dev_server(config: LocalSshConfig) -> Result<()> {
    let case = load_seed_case(&config.cases_root, &config.case_slug)?;
    let bind_addr = config.bind_addr;
    let socket = TcpListener::bind(bind_addr).await?;
    let local_addr = socket.local_addr()?;
    eprintln!("debugpath local SSH listening on {local_addr}");
    eprintln!("connect with: ssh -p {} localhost", local_addr.port());

    let server_config = Arc::new(ssh_server_config());
    let mut server = LocalServer::new(case, config.abuse);
    server.run_on_socket(server_config, &socket).await?;
    Ok(())
}

pub fn load_seed_case(root: impl AsRef<Path>, slug: &str) -> Result<Case> {
    load_cases(root)?
        .into_iter()
        .find(|case| case.metadata.slug == slug)
        .ok_or_else(|| anyhow::anyhow!("case slug `{slug}` was not found"))
}

fn ssh_server_config() -> Config {
    Config {
        inactivity_timeout: Some(Duration::from_secs(3600)),
        auth_rejection_time: Duration::from_millis(100),
        auth_rejection_time_initial: Some(Duration::from_millis(0)),
        keys: vec![
            russh::keys::PrivateKey::random(&mut rand::rng(), ssh_key::Algorithm::Ed25519)
                .expect("ed25519 host key generation succeeds"),
        ],
        nodelay: true,
        ..Default::default()
    }
}

#[derive(Clone)]
struct LocalServer {
    clients: Arc<tokio::sync::Mutex<BTreeMap<usize, ClientState>>>,
    abuse: Arc<Mutex<abuse::AbuseControls>>,
    case: Case,
    next_id: usize,
    id: usize,
    peer: String,
    accepted: bool,
}

impl LocalServer {
    fn new(case: Case, abuse_config: abuse::AbuseConfig) -> Self {
        Self {
            clients: Arc::new(tokio::sync::Mutex::new(BTreeMap::new())),
            abuse: Arc::new(Mutex::new(abuse::AbuseControls::new(abuse_config))),
            case,
            next_id: 0,
            id: 0,
            peer: "unknown-peer".to_owned(),
            accepted: true,
        }
    }

    fn auth_decision(&self) -> Auth {
        if self.accepted {
            Auth::Accept
        } else {
            Auth::reject()
        }
    }

    fn render_client(
        &self,
        channel: ChannelId,
        session: &mut Session,
        clients: &mut BTreeMap<usize, ClientState>,
    ) -> Result<()> {
        if let Some(client) = clients.get(&self.id) {
            let frame = client.app.render_to_string(client.width, client.height)?;
            session.data(channel, format!("\x1b[2J\x1b[H{frame}").into_bytes())?;
        }
        Ok(())
    }

    async fn render_locked_client(&self, channel: ChannelId, session: &mut Session) -> Result<()> {
        let mut clients = self.clients.lock().await;
        self.render_client(channel, session, &mut clients)?;
        Ok(())
    }

    async fn close_client(&self, now_seconds: u64) {
        self.clients.lock().await.remove(&self.id);
        if let Ok(mut abuse) = self.abuse.lock() {
            abuse.end_session(&self.peer, now_seconds);
        }
    }

    async fn resize_client(&self, width: u32, height: u32) -> Result<()> {
        let width = width.clamp(20, u16::MAX as u32) as u16;
        let height = height.clamp(8, u16::MAX as u32) as u16;
        let mut clients = self.clients.lock().await;
        if let Some(client) = clients.get_mut(&self.id) {
            client.width = width;
            client.height = height;
        }
        Ok(())
    }
}

struct ClientState {
    app: AppModel,
    width: u16,
    height: u16,
}

impl russh::server::Server for LocalServer {
    type Handler = Self;

    fn new_client(&mut self, peer_addr: Option<SocketAddr>) -> Self {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let peer = peer_addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown-peer".to_owned());
        let decision = self
            .abuse
            .lock()
            .map(|mut abuse| abuse.start_session(&peer, now_seconds()))
            .unwrap_or(abuse::AbuseDecision::Rejected(
                abuse::RejectReason::TooManySessions,
            ));

        let mut handler = self.clone();
        handler.id = id;
        handler.peer = peer;
        handler.accepted = decision == abuse::AbuseDecision::Accepted;
        handler
    }

    fn handle_session_error(&mut self, error: <Self::Handler as russh::server::Handler>::Error) {
        eprintln!("debugpath SSH session error: {error:#}");
    }
}

impl russh::server::Handler for LocalServer {
    type Error = anyhow::Error;

    async fn auth_none(&mut self, _user: &str) -> Result<Auth> {
        Ok(self.auth_decision())
    }

    async fn auth_password(&mut self, _user: &str, _password: &str) -> Result<Auth> {
        Ok(self.auth_decision())
    }

    async fn auth_publickey_offered(
        &mut self,
        _user: &str,
        _public_key: &ssh_key::PublicKey,
    ) -> Result<Auth> {
        Ok(self.auth_decision())
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        _public_key: &ssh_key::PublicKey,
    ) -> Result<Auth> {
        Ok(self.auth_decision())
    }

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool> {
        if !self.accepted {
            return Ok(false);
        }

        let app = AppModel::new(self.case.clone());
        self.clients.lock().await.insert(
            self.id,
            ClientState {
                app,
                width: 80,
                height: 24,
            },
        );
        Ok(true)
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<()> {
        self.resize_client(col_width, row_height).await?;
        session.channel_success(channel)?;
        Ok(())
    }

    async fn shell_request(&mut self, channel: ChannelId, session: &mut Session) -> Result<()> {
        session.channel_success(channel)?;
        self.render_locked_client(channel, session).await?;
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<()> {
        let requested = String::from_utf8_lossy(data);
        let message = format!(
            "debugpath.dev does not execute SSH exec requests (`{requested}`).\r\nUse an interactive shell; all commands are fixture-backed.\r\n"
        );
        session.data(channel, message.into_bytes())?;
        session.channel_failure(channel)?;
        session.exit_status_request(channel, 126)?;
        session.close(channel)?;
        Ok(())
    }

    async fn data(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<()> {
        let decision = if is_control_input(data) {
            abuse::AbuseDecision::Accepted
        } else {
            self.abuse
                .lock()
                .map(|mut abuse| {
                    abuse.inspect_command(&self.peer, &String::from_utf8_lossy(data), now_seconds())
                })
                .unwrap_or(abuse::AbuseDecision::Rejected(
                    abuse::RejectReason::CommandTooLarge,
                ))
        };
        if let abuse::AbuseDecision::Rejected(reason) = decision {
            session.data(
                channel,
                format!("\r\ninput rejected before engine handling: {reason:?}\r\n").into_bytes(),
            )?;
            return Ok(());
        }

        let mut should_close = false;
        {
            let mut clients = self.clients.lock().await;
            let mut redraw = false;
            if let Some(client) = clients.get_mut(&self.id) {
                let outcome = client.app.handle_bytes(data);
                should_close = outcome.quit;
                redraw = outcome.redraw;
            }
            if redraw {
                self.render_client(channel, session, &mut clients)?;
            }
        }

        if should_close {
            self.close_client(now_seconds()).await;
            session.close(channel)?;
        }
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        session: &mut Session,
    ) -> Result<()> {
        self.resize_client(col_width, row_height).await?;
        self.render_locked_client(channel, session).await?;
        Ok(())
    }

    async fn env_request(
        &mut self,
        channel: ChannelId,
        _variable_name: &str,
        _variable_value: &str,
        session: &mut Session,
    ) -> Result<()> {
        session.channel_failure(channel)?;
        Ok(())
    }

    async fn subsystem_request(
        &mut self,
        channel: ChannelId,
        _name: &str,
        session: &mut Session,
    ) -> Result<()> {
        session.channel_failure(channel)?;
        Ok(())
    }

    async fn channel_close(&mut self, _channel: ChannelId, _session: &mut Session) -> Result<()> {
        self.close_client(now_seconds()).await;
        Ok(())
    }

    async fn channel_eof(&mut self, _channel: ChannelId, _session: &mut Session) -> Result<()> {
        self.close_client(now_seconds()).await;
        Ok(())
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn is_control_input(data: &[u8]) -> bool {
    !data.is_empty()
        && data.iter().all(|byte| {
            matches!(
                byte,
                b'\t' | b'\r' | b'\n' | 0x03 | 0x08 | 0x1b | b'[' | b'C' | b'D' | b'Z'
            )
        })
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

#[cfg(test)]
mod ssh_tests {
    use super::*;
    use russh::{ChannelMsg, client};
    use std::sync::Arc;
    use tokio::time::{Duration, Instant, timeout};

    struct SmokeClient;

    impl client::Handler for SmokeClient {
        type Error = russh::Error;

        async fn check_server_key(
            &mut self,
            _server_public_key: &ssh_key::PublicKey,
        ) -> std::result::Result<bool, Self::Error> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn local_ssh_session_renders_tui_and_rejects_host_escape() {
        let cases_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cases");
        let case = load_seed_case(cases_root, DEFAULT_DEV_CASE_SLUG).expect("seed case loads");
        let socket = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind random local port");
        let addr = socket.local_addr().expect("local addr");
        let server_config = Arc::new(ssh_server_config());
        let mut server = LocalServer::new(case, abuse::AbuseConfig::default());
        let server_task =
            tokio::spawn(async move { server.run_on_socket(server_config, &socket).await });

        let client_config = Arc::new(client::Config {
            inactivity_timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        });
        let mut client = client::connect(client_config, addr, SmokeClient)
            .await
            .expect("client connects");
        let auth = client
            .authenticate_none("anonymous")
            .await
            .expect("anonymous auth returns");
        assert!(auth.success());

        let mut channel = client.channel_open_session().await.expect("channel opens");
        channel
            .request_pty(true, "xterm-256color", 96, 28, 0, 0, &[])
            .await
            .expect("pty accepted");
        channel.request_shell(true).await.expect("shell accepted");

        let initial = read_until(&mut channel, "Slow Checkout")
            .await
            .expect("initial TUI output");
        assert!(initial.contains("Brief"));

        channel
            .data(&b"cat /etc/passwd\n"[..])
            .await
            .expect("send rejected command");
        let rejected = read_until(&mut channel, "unknown command: cat /etc/passwd")
            .await
            .expect("rejection output");
        assert!(!rejected.contains("root:"));

        channel
            .data(&b"logs checkout-api --since 10m\n"[..])
            .await
            .expect("send fixture command");
        let accepted = read_until(&mut channel, "checkout-api")
            .await
            .expect("fixture output");
        assert!(accepted.contains("Fixture-backed command ran"));

        channel.data(&b"q"[..]).await.expect("quit");
        let _ = client
            .disconnect(russh::Disconnect::ByApplication, "done", "en")
            .await;
        server_task.abort();
    }

    async fn read_until(
        channel: &mut russh::Channel<client::Msg>,
        needle: &str,
    ) -> anyhow::Result<String> {
        let mut output = String::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match timeout(Duration::from_millis(250), channel.wait()).await {
                Ok(Some(ChannelMsg::Data { data })) => {
                    output.push_str(&String::from_utf8_lossy(&data));
                    if output.contains(needle) {
                        return Ok(output);
                    }
                }
                Ok(Some(_)) => {}
                Ok(None) => anyhow::bail!(
                    "channel closed before {needle:?}; captured {} bytes: {:?}",
                    output.len(),
                    output
                ),
                Err(_) if Instant::now() >= deadline => {
                    anyhow::bail!(
                        "timed out waiting for {needle:?}; captured {} bytes: {:?}",
                        output.len(),
                        output
                    );
                }
                Err(_) => {}
            }
        }
    }
}
