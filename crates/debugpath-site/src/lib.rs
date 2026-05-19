use axum::Router;
use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use debugpath_content::{Case, ContentError, load_cases};
use debugpath_engine::ReplayEvent;
use leptos::prelude::*;
use std::env;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseSummary {
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub difficulty: String,
    pub component: String,
    pub command_count: usize,
    pub evidence_count: usize,
    pub hint_count: usize,
    pub false_trail_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub player_handle: String,
    pub case_slug: String,
    pub score: u32,
    pub solved_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecentSolve {
    pub player_handle: String,
    pub case_slug: String,
    pub case_title: String,
    pub score: u32,
    pub solved_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerProfile {
    pub handle: String,
    pub display_name: String,
    pub solved_cases: u32,
    pub best_score: u32,
    pub recent_case: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replay {
    pub id: String,
    pub player_handle: String,
    pub case_slug: String,
    pub events: Vec<ReplayEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SiteData {
    pub ssh_entrypoint: String,
    pub data_source: String,
    pub public_base_url: String,
    pub cases: Vec<CaseSummary>,
    pub featured_slug: String,
    pub leaderboard: Vec<LeaderboardEntry>,
    pub recent_solves: Vec<RecentSolve>,
    pub players: Vec<PlayerProfile>,
    pub replays: Vec<Replay>,
}

pub fn app(data: SiteData) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/cases", get(case_catalog))
        .route("/cases/{slug}", get(case_detail))
        .route("/leaderboard", get(leaderboard))
        .route("/solves", get(recent_solves))
        .route("/players/{handle}", get(player_profile))
        .route("/replays", get(replay_index))
        .route("/replays/{id}", get(replay_detail))
        .route("/authoring", get(authoring_docs))
        .route("/standards", get(case_standards))
        .route("/status", get(status))
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ready" }))
        .with_state(Arc::new(data))
}

pub fn seeded_site() -> SiteData {
    SiteData {
        ssh_entrypoint: "ssh debugpath.dev".to_owned(),
        data_source: "seeded public fixture data".to_owned(),
        public_base_url: "https://debugpath.dev".to_owned(),
        cases: vec![
            CaseSummary::new(
                "slow-checkout",
                "Slow Checkout",
                "API latency jumps after a deploy and points toward a query shape change.",
                "intro",
                "checkout-api orders query",
                6,
                5,
                2,
                1,
            ),
            CaseSummary::new(
                "pinned-postgres",
                "Pinned Postgres",
                "Dashboard traffic pins database CPU after a feature flag enables heavier joins.",
                "intermediate",
                "analytics dashboard",
                6,
                5,
                2,
                1,
            ),
            CaseSummary::new(
                "green-ci-bad-prod",
                "Green CI, Bad Prod",
                "A deploy passes CI while production returns 502s because health checks drift.",
                "intro",
                "edge routing",
                6,
                5,
                2,
                1,
            ),
            CaseSummary::new(
                "memory-tide",
                "Memory Tide",
                "Upload API memory climbs under load after body buffering changes.",
                "intermediate",
                "upload-api",
                6,
                5,
                2,
                1,
            ),
            CaseSummary::new(
                "corrupt-uploads",
                "Corrupt Uploads",
                "Large archive uploads intermittently fail because chunks are reassembled out of order.",
                "intermediate",
                "upload assembler",
                6,
                5,
                2,
                1,
            ),
        ],
        featured_slug: "slow-checkout".to_owned(),
        leaderboard: vec![
            LeaderboardEntry::new(1, "rootcause", "slow-checkout", 955, "2026-05-17T13:10:00Z"),
            LeaderboardEntry::new(
                2,
                "tracefan",
                "pinned-postgres",
                920,
                "2026-05-17T13:02:00Z",
            ),
            LeaderboardEntry::new(
                3,
                "diffhound",
                "green-ci-bad-prod",
                890,
                "2026-05-17T12:44:00Z",
            ),
        ],
        recent_solves: vec![
            RecentSolve::new(
                "rootcause",
                "slow-checkout",
                "Slow Checkout",
                955,
                "2026-05-17T13:10:00Z",
            ),
            RecentSolve::new(
                "tracefan",
                "pinned-postgres",
                "Pinned Postgres",
                920,
                "2026-05-17T13:02:00Z",
            ),
            RecentSolve::new(
                "diffhound",
                "green-ci-bad-prod",
                "Green CI, Bad Prod",
                890,
                "2026-05-17T12:44:00Z",
            ),
        ],
        players: vec![
            PlayerProfile::new("rootcause", "Root Cause", 3, 955, "Slow Checkout"),
            PlayerProfile::new("tracefan", "Trace Fan", 2, 920, "Pinned Postgres"),
            PlayerProfile::new("diffhound", "Diff Hound", 2, 890, "Green CI, Bad Prod"),
        ],
        replays: vec![Replay {
            id: "seed-slow-checkout".to_owned(),
            player_handle: "rootcause".to_owned(),
            case_slug: "slow-checkout".to_owned(),
            events: vec![
                ReplayEvent::CommandRun {
                    command: "logs checkout-api --since 10m".to_owned(),
                    evidence: vec!["checkout-timeouts-after-deploy".to_owned()],
                    damage: 0,
                },
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
        }],
    }
}

pub fn site_from_env() -> Result<SiteData, ContentError> {
    let mut data = match env::var("DEBUGPATH_CASES_DIR") {
        Ok(root) => SiteData::from_cases_root(&root)?,
        Err(_) => seeded_site(),
    };
    data.apply_runtime_env();
    Ok(data)
}

impl CaseSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        slug: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        difficulty: impl Into<String>,
        component: impl Into<String>,
        command_count: usize,
        evidence_count: usize,
        hint_count: usize,
        false_trail_count: usize,
    ) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            summary: summary.into(),
            difficulty: difficulty.into(),
            component: component.into(),
            command_count,
            evidence_count,
            hint_count,
            false_trail_count,
        }
    }
}

impl From<&Case> for CaseSummary {
    fn from(case: &Case) -> Self {
        Self {
            slug: case.metadata.slug.clone(),
            title: case.metadata.title.clone(),
            summary: case.metadata.summary.clone(),
            difficulty: case.metadata.difficulty.clone(),
            component: case.metadata.component.clone(),
            command_count: case.commands.len(),
            evidence_count: case.scoring.evidence.len(),
            hint_count: case.hints.len(),
            false_trail_count: case.false_trails.len(),
        }
    }
}

impl LeaderboardEntry {
    pub fn new(
        rank: u32,
        player_handle: impl Into<String>,
        case_slug: impl Into<String>,
        score: u32,
        solved_at: impl Into<String>,
    ) -> Self {
        Self {
            rank,
            player_handle: player_handle.into(),
            case_slug: case_slug.into(),
            score,
            solved_at: solved_at.into(),
        }
    }
}

impl RecentSolve {
    pub fn new(
        player_handle: impl Into<String>,
        case_slug: impl Into<String>,
        case_title: impl Into<String>,
        score: u32,
        solved_at: impl Into<String>,
    ) -> Self {
        Self {
            player_handle: player_handle.into(),
            case_slug: case_slug.into(),
            case_title: case_title.into(),
            score,
            solved_at: solved_at.into(),
        }
    }
}

impl PlayerProfile {
    pub fn new(
        handle: impl Into<String>,
        display_name: impl Into<String>,
        solved_cases: u32,
        best_score: u32,
        recent_case: impl Into<String>,
    ) -> Self {
        Self {
            handle: handle.into(),
            display_name: display_name.into(),
            solved_cases,
            best_score,
            recent_case: recent_case.into(),
        }
    }
}

async fn home(State(data): State<Arc<SiteData>>) -> Html<String> {
    Html(render_home(&data))
}

async fn case_catalog(State(data): State<Arc<SiteData>>) -> Html<String> {
    Html(render_case_catalog(&data.cases))
}

async fn case_detail(
    State(data): State<Arc<SiteData>>,
    AxumPath(slug): AxumPath<String>,
) -> Result<Html<String>, NotFound> {
    let case = data.case(&slug).ok_or(NotFound)?;
    Ok(Html(page(
        &case.title,
        &format!(
            r#"<section aria-label="case detail" class="page-band detail-layout">
  <p class="backlink"><a href="/cases">Case catalog</a></p>
  <div>
    <p class="kicker">Incident packet</p>
    <h1>{}</h1>
    <p class="lede">{}</p>
    <div class="action-row">
      <a class="primary-action" href="/#ssh-entrypoint">SSH in now</a>
      <a class="secondary-action" href="/replays">Watch a replay</a>
    </div>
  </div>
  <dl class="metadata rail">
    <div><dt>Difficulty</dt><dd>{}</dd></div>
    <div><dt>Component</dt><dd>{}</dd></div>
  </dl>
  <dl class="metric-row span-all" aria-label="case investigation surface">
    <div><dt>Commands</dt><dd>{}</dd></div>
    <div><dt>Evidence IDs</dt><dd>{}</dd></div>
    <div><dt>Hints</dt><dd>{}</dd></div>
    <div><dt>False trails</dt><dd>{}</dd></div>
  </dl>
</section>"#,
            escape_html(&case.title),
            escape_html(&case.summary),
            escape_html(&case.difficulty),
            escape_html(&case.component),
            case.command_count,
            case.evidence_count,
            case.hint_count,
            case.false_trail_count,
        ),
    )))
}

async fn leaderboard(State(data): State<Arc<SiteData>>) -> Html<String> {
    Html(page("Leaderboard", &leaderboard_section(&data.leaderboard)))
}

async fn recent_solves(State(data): State<Arc<SiteData>>) -> Html<String> {
    Html(page(
        "Recent Solves",
        &recent_solves_section(&data.recent_solves),
    ))
}

async fn player_profile(
    State(data): State<Arc<SiteData>>,
    AxumPath(handle): AxumPath<String>,
) -> Result<Html<String>, NotFound> {
    let player = data
        .players
        .iter()
        .find(|player| player.handle == handle)
        .ok_or(NotFound)?;
    Ok(Html(page(
        &player.display_name,
        &format!(
            r#"<section aria-label="player profile" class="page-band">
  <p class="kicker">Player profile</p>
  <h1>{}</h1>
  <p class="lede"><code>@{}</code> has public seeded activity for leaderboard, replay, and case-solve surfaces.</p>
  <dl class="metric-row">
    <div><dt>Solved cases</dt><dd>{}</dd></div>
    <div><dt>Best score</dt><dd>{}</dd></div>
    <div><dt>Recent case</dt><dd>{}</dd></div>
  </dl>
  <div class="action-row">
    <a class="secondary-action" href="/leaderboard">Leaderboard</a>
    <a class="secondary-action" href="/replays">Replay viewer</a>
  </div>
</section>"#,
            escape_html(&player.display_name),
            escape_html(&player.handle),
            player.solved_cases,
            player.best_score,
            escape_html(&player.recent_case)
        ),
    )))
}

async fn replay_index(State(data): State<Arc<SiteData>>) -> Html<String> {
    Html(render_replay_index(&data.replays))
}

async fn replay_detail(
    State(data): State<Arc<SiteData>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Html<String>, NotFound> {
    let replay = data
        .replays
        .iter()
        .find(|replay| replay.id == id)
        .ok_or(NotFound)?;
    Ok(Html(page(
        "Replay",
        &format!(
            r#"<section aria-label="replay summary" class="page-band">
  <p class="backlink"><a href="/replays">Replay index</a></p>
  <p class="kicker">Replay detail</p>
  <h1>Replay</h1>
  <p class="lede"><a href="/players/{player}">@{player}</a> solved <a href="/cases/{case_slug}">{case_slug}</a>.</p>
  {}
</section>"#,
            render_replay(&replay.events),
            player = escape_html(&replay.player_handle),
            case_slug = escape_html(&replay.case_slug)
        ),
    )))
}

async fn authoring_docs() -> Html<String> {
    Html(page(
        "Authoring Docs",
        r#"<section aria-label="authoring docs" class="page-band prose-grid">
  <div>
  <p class="kicker">Case production</p>
  <h1>Authoring Docs</h1>
  <p>Cases are Git-authored incidents with deterministic artifacts, constrained commands, diagnosis expectations, fix options, hints, false trails, and scoring rules.</p>
  </div>
  <ol class="checklist">
    <li>Write the brief and realistic artifacts.</li>
    <li>Author command fixtures that never execute on the host.</li>
    <li>Link evidence to diagnosis and scoring.</li>
    <li>Run <code>just validate-cases</code> before review.</li>
  </ol>
</section>"#,
    ))
}

async fn case_standards() -> Html<String> {
    Html(page(
        "Case Quality Standards",
        r#"<section aria-label="case quality standards" class="page-band prose-grid">
  <div>
  <p class="kicker">Review bar</p>
  <h1>Case Quality Standards</h1>
  <p class="lede">The web surface should make case quality inspectable before a player connects over SSH.</p>
  </div>
  <ul class="checklist">
    <li>Every answer must be discoverable from evidence inside the case.</li>
    <li>Every case needs at least one fair false trail.</li>
    <li>Logs, metrics, SQL rows, traces, diffs, and runbooks must be coherent.</li>
    <li>Unsafe commands can only cause modeled damage inside the simulation.</li>
  </ul>
</section>"#,
    ))
}

async fn status(State(data): State<Arc<SiteData>>) -> Html<String> {
    Html(render_status(&data))
}

impl SiteData {
    pub fn from_cases_root(root: impl AsRef<Path>) -> Result<Self, ContentError> {
        let root = root.as_ref();
        let cases = load_cases(root)?;
        let mut data = seeded_site();
        data.cases = cases.iter().map(CaseSummary::from).collect();
        data.featured_slug = data
            .cases
            .iter()
            .find(|case| case.slug == "slow-checkout")
            .or_else(|| data.cases.first())
            .map(|case| case.slug.clone())
            .unwrap_or_else(|| "slow-checkout".to_owned());
        data.data_source = format!("validated case fixtures from {}", root.display());
        Ok(data)
    }

    fn apply_runtime_env(&mut self) {
        if let Ok(entrypoint) = env::var("DEBUGPATH_SSH_ENTRYPOINT") {
            self.ssh_entrypoint = entrypoint;
        }
        if let Ok(base_url) = env::var("DEBUGPATH_PUBLIC_BASE_URL") {
            self.public_base_url = base_url;
        }
    }

    fn case(&self, slug: &str) -> Option<&CaseSummary> {
        self.cases.iter().find(|case| case.slug == slug)
    }
}

pub fn render_home(data: &SiteData) -> String {
    let featured = data
        .case(&data.featured_slug)
        .or_else(|| data.cases.first())
        .cloned()
        .expect("seeded site has at least one case");
    let leaderboard = data.leaderboard.clone();
    let recent_solves = data.recent_solves.clone();
    let ssh_entrypoint = data.ssh_entrypoint.clone();
    let case_count = data.cases.len();
    let solve_count = recent_solves.len();
    let replay_count = data.replays.len();
    render_page("debugpath.dev", move || {
        let featured_href = format!("/cases/{}", featured.slug);
        let featured_cta_href = featured_href.clone();
        view! {
            <section aria-label="hero" class="hero">
                <div class="hero-copy">
                    <p class="kicker">"SSH-native incident lab"</p>
                    <h1>"debugpath.dev"</h1>
                    <p class="lede">
                        "Solve production incidents from the terminal. Read logs, query fixtures, inspect traces, chase false leads, and prove the root cause."
                    </p>
                    <div id="ssh-entrypoint" class="command-strip" aria-label="ssh entrypoint">
                        <span>"$"</span>
                        <code>{ssh_entrypoint.clone()}</code>
                    </div>
                    <div class="action-row">
                        <a class="primary-action" href="#ssh-entrypoint">"SSH in now"</a>
                        <a class="secondary-action" href="/cases">"Open case catalog"</a>
                        <a class="secondary-action" href="/replays">"Watch a replay"</a>
                    </div>
                </div>
                <div class="terminal-panel" aria-label="incident console preview">
                    <div class="terminal-toolbar">
                        <span class="terminal-dots" aria-hidden="true"><i></i><i></i><i></i></span>
                        <span>"player@debugpath ~ ssh debugpath.dev"</span>
                        <strong>"● live"</strong>
                    </div>
                    <pre>"┌─ Brief ──── Systems ── Logs ── Metrics ── Shell ── SQL ── Trace ── Notes ─┐
│ case: slow-checkout                            status: investigating       │
└────────────────────────────────────────────────────────────────────────────┘
$ logs checkout-api --since 10m
  WARN  p95=4.2s   deploy=checkout-query-shape   trace=8d3a…
  WARN  pool=exhausted  waiters=42  acquired=avg 1.8s
$ sql explain checkout_recent_orders
  Seq Scan on orders  rows=1.2M  cost=18421
  Filter: status = 'pending' AND created_at > now() - '24h'
$ diagnose
  root_cause   : missing composite index (status, created_at)
  evidence     : seq-scan-orders, deploy-checkout-query-shape
  fix          : add_orders_status_created_at_index"</pre>
                </div>
            </section>
            <dl class="ops-snapshot" aria-label="site snapshot">
                <div><dt>"Cases online"</dt><dd>{case_count}</dd></div>
                <div><dt>"Seeded solves"</dt><dd>{solve_count}</dd></div>
                <div><dt>"Public replays"</dt><dd>{replay_count}</dd></div>
            </dl>
            <section aria-label="featured incident" class="page-band feature-grid">
                <div>
                    <p class="kicker">"Featured incident"</p>
                    <h2><a href=featured_href>{featured.title}</a></h2>
                    <p>{featured.summary}</p>
                    <dl class="metadata">
                        <div><dt>"Difficulty"</dt><dd>{featured.difficulty}</dd></div>
                        <div><dt>"Component"</dt><dd>{featured.component}</dd></div>
                    </dl>
                    <div class="action-row">
                        <a class="secondary-action" href=featured_cta_href>"Open incident packet"</a>
                    </div>
                </div>
                <dl class="metric-row" aria-label="featured investigation surface">
                    <div><dt>"Commands"</dt><dd>{featured.command_count}</dd></div>
                    <div><dt>"Evidence IDs"</dt><dd>{featured.evidence_count}</dd></div>
                    <div><dt>"Hints"</dt><dd>{featured.hint_count}</dd></div>
                    <div><dt>"False trails"</dt><dd>{featured.false_trail_count}</dd></div>
                </dl>
            </section>
            {leaderboard_section_view(leaderboard)}
            {recent_solves_section_view(recent_solves)}
            <nav aria-label="product sections" class="section-nav">
                <a href="/cases">"Case catalog"</a>
                <a href="/leaderboard">"Leaderboard"</a>
                <a href="/solves">"Recent solves"</a>
                <a href="/replays">"Replay viewer"</a>
                <a href="/authoring">"Authoring docs"</a>
                <a href="/standards">"Case standards"</a>
                <a href="/status">"Status"</a>
            </nav>
        }
    })
}

pub fn render_case_catalog(cases: &[CaseSummary]) -> String {
    let cases = cases.to_vec();
    render_page("Case Catalog", move || {
        view! {
            <section aria-label="case catalog" class="page-band">
                <p class="kicker">"Playable incidents"</p>
                <h1>"Case Catalog"</h1>
                <p class="lede">"Each incident is a deterministic case with fixture-backed commands, evidence IDs, scored fixes, and fair false trails."</p>
                <ul class="case-grid">
                    {cases
                        .into_iter()
                        .map(|case| {
                            let href = format!("/cases/{}", case.slug);
                            view! {
                                <li>
                                    <div class="case-card-head">
                                        <a href=href>{case.title}</a>
                                        <span>{case.difficulty}</span>
                                    </div>
                                    <p>{case.summary}</p>
                                    <small>{case.component}</small>
                                    <dl class="mini-metrics">
                                        <div><dt>"cmd"</dt><dd>{case.command_count}</dd></div>
                                        <div><dt>"evidence"</dt><dd>{case.evidence_count}</dd></div>
                                        <div><dt>"trails"</dt><dd>{case.false_trail_count}</dd></div>
                                    </dl>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </section>
        }
    })
}

pub fn render_replay_index(replays: &[Replay]) -> String {
    let replays = replays.to_vec();
    render_page("Replays", move || {
        view! {
            <section aria-label="replay index" class="page-band">
                <p class="kicker">"Inspect process"</p>
                <h1>"Replay Viewer"</h1>
                <p class="lede">
                    "Replay pages show the commands, evidence, hints, diagnosis, and fix sequence that led to a solve."
                </p>
                <ul class="activity-list">
                    {replays
                        .into_iter()
                        .map(|replay| {
                            let href = format!("/replays/{}", replay.id);
                            let player_href = format!("/players/{}", replay.player_handle);
                            let player_label = format!("@{}", replay.player_handle);
                            let case_href = format!("/cases/{}", replay.case_slug);
                            view! {
                                <li>
                                    <div>
                                        <a href=href>{replay.id}</a>
                                        <span>{format!("{} events", replay.events.len())}</span>
                                    </div>
                                    <a href=player_href>{player_label}</a>
                                    <a href=case_href>{replay.case_slug}</a>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </section>
        }
    })
}

pub fn render_status(data: &SiteData) -> String {
    let case_count = data.cases.len();
    let solve_count = data.recent_solves.len();
    let replay_count = data.replays.len();
    let data_source = data.data_source.clone();
    let public_base_url = data.public_base_url.clone();
    let ssh_entrypoint = data.ssh_entrypoint.clone();
    render_page("Status", move || {
        view! {
            <section aria-label="status" class="page-band">
                <p class="kicker">"Operational status"</p>
                <h1>"Status"</h1>
                <dl class="metric-row">
                    <div><dt>"Site"</dt><dd>"ready"</dd></div>
                    <div><dt>"Cases"</dt><dd>{case_count}</dd></div>
                    <div><dt>"Solves"</dt><dd>{solve_count}</dd></div>
                    <div><dt>"Replays"</dt><dd>{replay_count}</dd></div>
                </dl>
                <dl class="metadata">
                    <div><dt>"SSH entrypoint"</dt><dd><code>{ssh_entrypoint}</code></dd></div>
                    <div><dt>"Public base URL"</dt><dd>{public_base_url}</dd></div>
                    <div><dt>"Data source"</dt><dd>{data_source}</dd></div>
                    <div><dt>"Health checks"</dt><dd><a href="/healthz">"/healthz"</a> " " <a href="/readyz">"/readyz"</a></dd></div>
                </dl>
            </section>
        }
    })
}

pub fn render_replay(events: &[ReplayEvent]) -> String {
    let events = events.to_vec();
    render_fragment(move || replay_events_view(events))
}

fn leaderboard_section(entries: &[LeaderboardEntry]) -> String {
    let entries = entries.to_vec();
    render_fragment(move || leaderboard_section_view(entries))
}

fn recent_solves_section(solves: &[RecentSolve]) -> String {
    let solves = solves.to_vec();
    render_fragment(move || recent_solves_section_view(solves))
}

fn leaderboard_section_view(entries: Vec<LeaderboardEntry>) -> impl IntoView {
    view! {
        <section aria-label="leaderboard" class="page-band">
            <div class="section-heading">
                <p class="kicker">"Seeded score table"</p>
                <h2>"Leaderboard"</h2>
                <a class="section-link" href="/leaderboard">"Full table"</a>
            </div>
            <div class="table-wrap">
                <table>
                    <thead>
                        <tr>
                            <th>"Rank"</th>
                            <th>"Player"</th>
                            <th>"Case"</th>
                            <th>"Score"</th>
                            <th>"Solved"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {entries
                            .into_iter()
                            .map(|entry| {
                                let player_href = format!("/players/{}", entry.player_handle);
                                let case_href = format!("/cases/{}", entry.case_slug);
                                let player_label = format!("@{}", entry.player_handle);
                                view! {
                                    <tr>
                                        <td><span class="rank-pill">{entry.rank}</span></td>
                                        <td><a href=player_href>{player_label}</a></td>
                                        <td><a href=case_href>{entry.case_slug}</a></td>
                                        <td><strong>{entry.score}</strong></td>
                                        <td>{entry.solved_at}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </section>
    }
}

fn recent_solves_section_view(solves: Vec<RecentSolve>) -> impl IntoView {
    view! {
        <section aria-label="recent solves" class="page-band">
            <div class="section-heading">
                <p class="kicker">"Latest completions"</p>
                <h2>"Recent solves"</h2>
                <a class="section-link" href="/solves">"All solves"</a>
            </div>
            <ul class="activity-list">
                {solves
                    .into_iter()
                    .map(|solve| {
                        let player_href = format!("/players/{}", solve.player_handle);
                        let player_label = format!("@{}", solve.player_handle);
                        let case_href = format!("/cases/{}", solve.case_slug);
                        view! {
                            <li>
                                <div>
                                    <a href=player_href>{player_label}</a>
                                    <span>"solved"</span>
                                    <a href=case_href>{solve.case_title}</a>
                                </div>
                                <strong>{solve.score}</strong>
                                <time>{solve.solved_at}</time>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </section>
    }
}

fn replay_events_view(events: Vec<ReplayEvent>) -> impl IntoView {
    view! {
        <ol aria-label="replay events" class="replay-events">
            {events
                .into_iter()
                .enumerate()
                .map(|(index, event)| {
                    view! {
                        <li>
                            <span>{format!("{:02}", index + 1)}</span>
                            <strong>{replay_event_kind(&event)}</strong>
                            <p>{replay_event_label(&event)}</p>
                        </li>
                    }
                })
                .collect_view()}
        </ol>
    }
}

fn replay_event_kind(event: &ReplayEvent) -> &'static str {
    match event {
        ReplayEvent::CommandRun { .. } => "command",
        ReplayEvent::CommandRejected { .. } => "rejected",
        ReplayEvent::HintUsed { .. } => "hint",
        ReplayEvent::DiagnosisSubmitted => "diagnosis",
        ReplayEvent::FixApplied { .. } => "fix",
    }
}

fn replay_event_label(event: &ReplayEvent) -> String {
    match event {
        ReplayEvent::CommandRun {
            command,
            evidence,
            damage,
        } => format!(
            "command: {} evidence: {} damage: {}",
            command,
            evidence.join(","),
            damage
        ),
        ReplayEvent::CommandRejected { command, reason } => {
            format!("command rejected: {command} reason: {reason}")
        }
        ReplayEvent::HintUsed { hint_id, cost } => {
            format!("hint: {hint_id} cost: {cost}")
        }
        ReplayEvent::DiagnosisSubmitted => "diagnosis submitted".to_owned(),
        ReplayEvent::FixApplied { fix_id, solves } => {
            format!("fix: {fix_id} solves: {solves}")
        }
    }
}

fn render_fragment<V>(view: impl FnOnce() -> V + 'static) -> String
where
    V: IntoView + 'static,
{
    view().to_html()
}

fn render_page<V>(title: &str, body: impl FnOnce() -> V + 'static) -> String
where
    V: IntoView + 'static,
{
    page(title, &render_fragment(body))
}

fn page(title: &str, body: &str) -> String {
    format!(
        r##"<!doctype html>
<html lang="en" data-theme="dark">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="color-scheme" content="dark light">
  <meta name="description" content="debugpath.dev is an SSH-native incident lab. Solve production incidents from a terminal: read logs, query fixtures, inspect traces, and prove the root cause.">
  <title>{}</title>
  <script>
    (function() {{
      try {{
        var stored = localStorage.getItem('debugpath-theme');
        if (stored === 'light' || stored === 'dark') {{
          document.documentElement.setAttribute('data-theme', stored);
        }}
      }} catch (e) {{}}
    }})();
  </script>
  <style>
    :root {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji";
      line-height: 1.5;
      font-size: 16px;
      --mono: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
    }}
    [data-theme="dark"] {{
      color-scheme: dark;
      --canvas: #0d1117;
      --canvas-subtle: #161b22;
      --canvas-inset: #010409;
      --canvas-overlay: #1c2128;
      --border: #30363d;
      --border-muted: #21262d;
      --fg: #e6edf3;
      --fg-muted: #7d8590;
      --fg-subtle: #6e7681;
      --accent: #2f81f7;
      --accent-fg: #58a6ff;
      --accent-emphasis: #1f6feb;
      --success: #3fb950;
      --attention: #d29922;
      --danger: #f85149;
      --on-accent: #ffffff;
      --shadow: 0 8px 24px rgb(1 4 9 / 50%);
      --header-bg: rgba(13, 17, 23, 0.85);
    }}
    [data-theme="light"] {{
      color-scheme: light;
      --canvas: #ffffff;
      --canvas-subtle: #f6f8fa;
      --canvas-inset: #eaeef2;
      --canvas-overlay: #ffffff;
      --border: #d0d7de;
      --border-muted: #d8dee4;
      --fg: #1f2328;
      --fg-muted: #59636e;
      --fg-subtle: #6e7781;
      --accent: #0969da;
      --accent-fg: #0969da;
      --accent-emphasis: #0550ae;
      --success: #1a7f37;
      --attention: #9a6700;
      --danger: #d1242f;
      --on-accent: #ffffff;
      --shadow: 0 6px 18px rgb(31 35 40 / 8%);
      --header-bg: rgba(255, 255, 255, 0.85);
    }}
    * {{
      box-sizing: border-box;
    }}
    html, body {{
      margin: 0;
    }}
    body {{
      color: var(--fg);
      background: var(--canvas);
      -webkit-font-smoothing: antialiased;
      -moz-osx-font-smoothing: grayscale;
    }}
    ::selection {{
      background: color-mix(in srgb, var(--accent) 35%, transparent);
    }}
    *:focus-visible {{
      outline: 2px solid var(--accent);
      outline-offset: 2px;
    }}
    .skip-link {{
      position: absolute;
      left: 12px;
      top: -48px;
      padding: 8px 12px;
      color: var(--fg);
      background: var(--canvas-overlay);
      border: 1px solid var(--border);
      border-radius: 6px;
      z-index: 10;
    }}
    .skip-link:focus {{
      top: 12px;
    }}
    .site-header,
    .site-footer,
    main {{
      width: min(1180px, calc(100vw - 32px));
      margin: 0 auto;
    }}
    .site-header {{
      position: sticky;
      top: 0;
      z-index: 5;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
      padding: 12px 0;
      border-bottom: 1px solid var(--border-muted);
      background: var(--header-bg);
      backdrop-filter: saturate(160%) blur(12px);
      -webkit-backdrop-filter: saturate(160%) blur(12px);
    }}
    .brand-block {{
      display: flex;
      align-items: center;
      gap: 12px;
      min-width: 0;
    }}
    .brand {{
      display: inline-flex;
      align-items: center;
      gap: 8px;
      color: var(--fg);
      font-size: 0.98rem;
      font-weight: 700;
      letter-spacing: -0.01em;
      text-decoration: none;
    }}
    .brand-mark {{
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 28px;
      height: 28px;
      border: 1px solid var(--border);
      border-radius: 6px;
      background: var(--canvas-subtle);
      color: var(--success);
      font-family: var(--mono);
      font-size: 0.78rem;
      font-weight: 700;
    }}
    .brand-tag {{
      display: none;
      padding: 2px 8px;
      border: 1px solid var(--border);
      border-radius: 999px;
      color: var(--fg-muted);
      font-size: 0.74rem;
      font-weight: 500;
    }}
    @media (min-width: 980px) {{
      .brand-tag {{ display: inline-flex; }}
    }}
    .site-header nav {{
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 4px 6px;
    }}
    .site-header nav a {{
      display: inline-flex;
      align-items: center;
      min-height: 32px;
      padding: 4px 10px;
      border-radius: 6px;
      color: var(--fg-muted);
      font-size: 0.88rem;
      font-weight: 500;
      text-decoration: none;
      transition: color .12s ease, background .12s ease;
    }}
    .site-header nav a:hover {{
      color: var(--fg);
      background: var(--canvas-subtle);
    }}
    .theme-toggle {{
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      width: 34px;
      height: 32px;
      margin-left: 4px;
      padding: 0;
      border: 1px solid var(--border);
      border-radius: 6px;
      background: var(--canvas-subtle);
      color: var(--fg-muted);
      font: inherit;
      cursor: pointer;
      transition: color .12s ease, background .12s ease, border-color .12s ease;
    }}
    .theme-toggle:hover {{
      color: var(--fg);
      border-color: var(--fg-subtle);
      background: var(--canvas-overlay);
    }}
    .theme-toggle .icon {{
      width: 16px;
      height: 16px;
      display: inline-flex;
      align-items: center;
      justify-content: center;
    }}
    [data-theme="dark"] .theme-toggle .icon-sun {{ display: inline-flex; }}
    [data-theme="dark"] .theme-toggle .icon-moon {{ display: none; }}
    [data-theme="light"] .theme-toggle .icon-sun {{ display: none; }}
    [data-theme="light"] .theme-toggle .icon-moon {{ display: inline-flex; }}
    .site-footer {{
      display: flex;
      flex-wrap: wrap;
      justify-content: space-between;
      align-items: center;
      gap: 10px 16px;
      padding: 20px 0 32px;
      margin-top: 24px;
      border-top: 1px solid var(--border-muted);
      color: var(--fg-muted);
      font-size: 0.86rem;
    }}
    .site-footer a {{
      color: var(--accent-fg);
      text-decoration: none;
    }}
    .site-footer a:hover {{
      text-decoration: underline;
    }}
    main {{
      padding: 28px 0 24px;
    }}
    h1, h2, h3, p, a, code, td, dd {{
      overflow-wrap: anywhere;
    }}
    h1, h2, h3 {{
      margin: 0 0 12px;
      letter-spacing: -0.015em;
      color: var(--fg);
    }}
    h1 {{
      font-size: 2.25rem;
      line-height: 1.15;
      font-weight: 700;
    }}
    h2 {{
      font-size: 1.35rem;
      line-height: 1.2;
      font-weight: 600;
    }}
    h3 {{
      font-size: 1.05rem;
      font-weight: 600;
    }}
    p {{
      margin: 0 0 12px;
    }}
    .lede {{
      max-width: 72ch;
      color: var(--fg-muted);
      font-size: 1.02rem;
    }}
    a {{
      color: var(--accent-fg);
      font-weight: 500;
      text-decoration: none;
    }}
    a:hover {{
      text-decoration: underline;
    }}
    code {{
      display: inline-block;
      max-width: 100%;
      padding: 2px 6px;
      border: 1px solid var(--border-muted);
      border-radius: 6px;
      background: var(--canvas-subtle);
      color: var(--fg);
      font-family: var(--mono);
      font-size: 0.86em;
    }}
    .hero {{
      display: grid;
      grid-template-columns: minmax(0, 0.92fr) minmax(360px, 1.08fr);
      gap: 28px;
      align-items: stretch;
      padding: 24px 0 32px;
      border-bottom: 1px solid var(--border-muted);
    }}
    .hero-copy {{
      display: flex;
      flex-direction: column;
      justify-content: center;
      min-width: 0;
    }}
    .hero-copy .lede {{
      max-width: 60ch;
      font-size: 1.08rem;
    }}
    .command-strip {{
      display: grid;
      grid-template-columns: auto minmax(0, 1fr);
      gap: 12px;
      align-items: center;
      width: min(100%, 540px);
      margin: 18px 0 0;
      padding: 12px 14px;
      border: 1px solid var(--border);
      border-radius: 8px;
      background: var(--canvas-subtle);
      font-family: var(--mono);
    }}
    .command-strip span {{
      color: var(--success);
      font-weight: 700;
    }}
    .command-strip code {{
      padding: 0;
      border: 0;
      background: transparent;
      color: var(--fg);
      font-size: 1.05rem;
      font-weight: 500;
    }}
    .terminal-panel {{
      align-self: center;
      min-width: 0;
      border: 1px solid var(--border);
      border-radius: 10px;
      background: var(--canvas-inset);
      color: #d1d7e0;
      box-shadow: var(--shadow);
      overflow: hidden;
    }}
    [data-theme="light"] .terminal-panel {{
      background: #0d1117;
      border-color: #0d1117;
    }}
    .terminal-toolbar {{
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      padding: 10px 14px;
      color: #8b949e;
      border-bottom: 1px solid #21262d;
      background: linear-gradient(180deg, #161b22 0%, #0d1117 100%);
      font-size: 0.78rem;
      font-family: var(--mono);
    }}
    .terminal-toolbar strong {{
      color: #3fb950;
      font-weight: 600;
    }}
    .terminal-dots {{
      display: inline-flex;
      gap: 6px;
      margin-right: 8px;
    }}
    .terminal-dots i {{
      width: 10px;
      height: 10px;
      border-radius: 50%;
      background: #30363d;
      display: inline-block;
    }}
    .terminal-dots i:nth-child(1) {{ background: #f85149; }}
    .terminal-dots i:nth-child(2) {{ background: #d29922; }}
    .terminal-dots i:nth-child(3) {{ background: #3fb950; }}
    .terminal-panel pre {{
      margin: 0;
      padding: 18px;
      overflow: auto;
      font-family: var(--mono);
      font-size: 0.86rem;
      line-height: 1.6;
      white-space: pre;
      color: #c9d1d9;
    }}
    .action-row {{
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
      margin-top: 20px;
    }}
    .primary-action,
    .secondary-action,
    .section-link,
    .section-nav a {{
      display: inline-flex;
      min-height: 32px;
      align-items: center;
      padding: 5px 14px;
      border: 1px solid var(--border);
      border-radius: 6px;
      background: var(--canvas-subtle);
      color: var(--fg);
      font-size: 0.88rem;
      font-weight: 500;
      text-decoration: none;
      transition: background .12s ease, border-color .12s ease;
    }}
    .secondary-action:hover,
    .section-link:hover,
    .section-nav a:hover {{
      background: var(--canvas-overlay);
      border-color: var(--fg-subtle);
      text-decoration: none;
    }}
    .primary-action {{
      color: var(--on-accent);
      border-color: var(--accent-emphasis);
      background: var(--accent);
      box-shadow: inset 0 1px 0 rgb(255 255 255 / 8%);
    }}
    .primary-action:hover {{
      background: var(--accent-emphasis);
      border-color: var(--accent-emphasis);
      text-decoration: none;
    }}
    .site-header nav .primary-action {{
      min-height: 28px;
      padding: 4px 12px;
      margin-left: 6px;
      color: var(--on-accent);
      border-color: var(--accent-emphasis);
      background: var(--accent);
      font-size: 0.86rem;
    }}
    .site-header nav .primary-action:hover {{
      background: var(--accent-emphasis);
    }}
    .kicker {{
      margin: 0 0 10px;
      color: var(--accent-fg);
      font-family: var(--mono);
      font-size: 0.74rem;
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }}
    .page-band {{
      padding: 28px 0;
      border-bottom: 1px solid var(--border-muted);
    }}
    .page-band:last-of-type {{
      border-bottom: 0;
    }}
    .feature-grid,
    .detail-layout,
    .prose-grid {{
      display: grid;
      grid-template-columns: minmax(0, 1fr) minmax(260px, 0.48fr);
      gap: 24px;
      align-items: start;
    }}
    .span-all {{
      grid-column: 1 / -1;
    }}
    .ops-snapshot {{
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 0;
      margin: 0;
      padding: 0;
      border: 1px solid var(--border);
      border-radius: 8px;
      background: var(--canvas-subtle);
      overflow: hidden;
    }}
    .ops-snapshot div {{
      display: flex;
      justify-content: space-between;
      align-items: baseline;
      gap: 16px;
      padding: 14px 16px;
      border-right: 1px solid var(--border-muted);
    }}
    .ops-snapshot div:last-child {{
      border-right: 0;
    }}
    .ops-snapshot dt,
    .metadata dt,
    .metric-row dt,
    .mini-metrics dt {{
      color: var(--fg-muted);
      font-size: 0.72rem;
      font-weight: 500;
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }}
    .ops-snapshot dd,
    .metadata dd,
    .metric-row dd,
    .mini-metrics dd {{
      margin: 0;
      color: var(--fg);
      font-family: var(--mono);
      font-size: 1.05rem;
      font-weight: 600;
    }}
    .metadata {{
      display: flex;
      flex-wrap: wrap;
      gap: 0;
      margin: 18px 0 0;
    }}
    .metadata div {{
      flex: 1 1 180px;
      min-width: 0;
      padding: 10px 14px;
      border-top: 1px solid var(--border-muted);
    }}
    .metadata div + div {{
      border-left: 1px solid var(--border-muted);
    }}
    .metadata.rail {{
      display: grid;
      gap: 0;
      margin: 0;
      padding: 4px 0;
      border: 1px solid var(--border);
      border-radius: 8px;
      background: var(--canvas-subtle);
    }}
    .metadata.rail div {{
      flex: none;
      padding: 10px 16px;
      border-top: 0;
      border-left: 3px solid var(--accent);
    }}
    .metric-row {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
      gap: 10px;
      margin: 20px 0 0;
    }}
    .feature-grid .metric-row {{
      margin: 0;
    }}
    .metric-row div,
    .mini-metrics div {{
      border: 1px solid var(--border);
      border-left: 3px solid var(--accent);
      border-radius: 6px;
      background: var(--canvas-subtle);
      padding: 12px 14px;
    }}
    .metric-row dt,
    .mini-metrics dt {{
      display: block;
      margin-bottom: 4px;
    }}
    .table-wrap {{
      overflow-x: auto;
      border: 1px solid var(--border);
      border-radius: 8px;
      background: var(--canvas-subtle);
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
    }}
    th, td {{
      padding: 10px 14px;
      border-bottom: 1px solid var(--border-muted);
      text-align: left;
      white-space: nowrap;
      font-size: 0.9rem;
    }}
    thead th {{
      background: var(--canvas-overlay);
      color: var(--fg-muted);
      font-size: 0.72rem;
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }}
    tbody tr:hover {{
      background: var(--canvas-overlay);
    }}
    tr:last-child td {{
      border-bottom: 0;
    }}
    td strong {{
      font-family: var(--mono);
      font-weight: 600;
    }}
    .rank-pill {{
      display: inline-flex;
      min-width: 28px;
      justify-content: center;
      padding: 2px 8px;
      border: 1px solid var(--border);
      border-radius: 999px;
      background: var(--canvas-overlay);
      color: var(--accent-fg);
      font-family: var(--mono);
      font-size: 0.84rem;
      font-weight: 600;
    }}
    .section-heading {{
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 12px;
      align-items: end;
      margin-bottom: 16px;
    }}
    .section-heading .kicker {{
      grid-column: 1 / -1;
      margin-bottom: -4px;
    }}
    .section-heading h2 {{
      margin-bottom: 0;
    }}
    .section-nav {{
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      padding: 28px 0;
      border-top: 1px solid var(--border-muted);
    }}
    .case-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
      gap: 14px;
      padding: 0;
      list-style: none;
    }}
    .case-grid li {{
      display: grid;
      gap: 10px;
      min-height: 240px;
      padding: 16px;
      border: 1px solid var(--border);
      border-radius: 8px;
      background: var(--canvas-subtle);
      transition: border-color .12s ease, transform .12s ease;
    }}
    .case-grid li:hover {{
      border-color: var(--accent);
    }}
    .case-card-head {{
      display: flex;
      justify-content: space-between;
      gap: 12px;
      align-items: baseline;
    }}
    .case-card-head a {{
      color: var(--fg);
      font-weight: 600;
      font-size: 1.02rem;
    }}
    .case-card-head a:hover {{
      color: var(--accent-fg);
    }}
    .case-grid span {{
      padding: 2px 8px;
      border: 1px solid var(--border);
      border-radius: 999px;
      color: var(--attention);
      background: color-mix(in srgb, var(--attention) 12%, transparent);
      font-size: 0.7rem;
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }}
    .case-grid p {{
      margin: 0;
      color: var(--fg-muted);
      font-size: 0.93rem;
    }}
    .case-grid small {{
      color: var(--fg-muted);
      font-family: var(--mono);
      font-size: 0.78rem;
    }}
    .mini-metrics {{
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 6px;
      margin: 4px 0 0;
      align-self: end;
    }}
    .mini-metrics div {{
      padding: 7px 10px;
      border-left-color: var(--attention);
    }}
    .mini-metrics dd {{
      font-size: 0.95rem;
    }}
    .activity-list,
    .replay-events,
    .checklist {{
      display: grid;
      gap: 10px;
      margin: 0;
      padding: 0;
      list-style: none;
    }}
    .activity-list li,
    .replay-events li,
    .checklist li {{
      border: 1px solid var(--border);
      border-radius: 8px;
      background: var(--canvas-subtle);
    }}
    .activity-list li {{
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto minmax(150px, auto);
      gap: 14px;
      align-items: center;
      padding: 14px 16px;
    }}
    .activity-list li div {{
      display: flex;
      flex-wrap: wrap;
      gap: 6px 10px;
      align-items: baseline;
    }}
    .activity-list strong {{
      color: var(--success);
      font-family: var(--mono);
      font-weight: 600;
    }}
    .activity-list time,
    .activity-list span {{
      color: var(--fg-muted);
      font-family: var(--mono);
      font-size: 0.84rem;
    }}
    .replay-events li {{
      display: grid;
      grid-template-columns: 44px 120px minmax(0, 1fr);
      gap: 14px;
      align-items: start;
      padding: 14px 16px;
    }}
    .replay-events li > span {{
      color: var(--fg-muted);
      font-family: var(--mono);
      font-weight: 600;
    }}
    .replay-events strong {{
      padding: 2px 8px;
      border: 1px solid var(--border);
      border-radius: 999px;
      color: var(--accent-fg);
      background: color-mix(in srgb, var(--accent) 12%, transparent);
      font-family: var(--mono);
      font-size: 0.78rem;
      font-weight: 600;
      text-align: center;
      align-self: start;
      justify-self: start;
    }}
    .replay-events p {{
      margin: 0;
      color: var(--fg-muted);
      font-family: var(--mono);
      font-size: 0.88rem;
    }}
    .checklist li {{
      padding: 14px 16px;
      border-left: 3px solid var(--accent);
    }}
    .backlink {{
      grid-column: 1 / -1;
      margin: 0 0 8px;
      font-size: 0.88rem;
    }}
    .backlink a::before {{
      content: "← ";
      color: var(--fg-muted);
    }}
    @media (max-width: 860px) {{
      .hero,
      .feature-grid,
      .detail-layout,
      .prose-grid {{
        grid-template-columns: 1fr;
      }}
      .ops-snapshot {{
        grid-template-columns: 1fr;
      }}
      .ops-snapshot div {{
        border-right: 0;
        border-bottom: 1px solid var(--border-muted);
      }}
      .ops-snapshot div:last-child {{
        border-bottom: 0;
      }}
      .feature-grid .metric-row {{
        margin-top: 0;
      }}
    }}
    @media (max-width: 700px) {{
      main,
      .site-header,
      .site-footer {{
        width: min(100vw - 24px, 1180px);
      }}
      main {{
        padding-top: 20px;
      }}
      .site-header {{
        position: static;
        align-items: flex-start;
        flex-direction: column;
        gap: 12px;
      }}
      .site-header nav {{
        width: 100%;
        justify-content: flex-start;
      }}
      h1 {{
        font-size: 1.9rem;
      }}
      .hero {{
        padding: 18px 0 24px;
      }}
      .terminal-panel pre {{
        font-size: 0.78rem;
      }}
      .activity-list li,
      .replay-events li,
      .section-heading {{
        grid-template-columns: 1fr;
      }}
      .mini-metrics {{
        grid-template-columns: 1fr;
      }}
      .metadata div + div {{
        border-left: 0;
      }}
      th, td {{
        padding: 9px 10px;
      }}
    }}
    @media (prefers-reduced-motion: reduce) {{
      * {{
        transition: none !important;
      }}
    }}
  </style>
</head>
<body>
  <a class="skip-link" href="#main">Skip to content</a>
  <header class="site-header">
    <div class="brand-block">
      <a class="brand" href="/">
        <span class="brand-mark" aria-hidden="true">$_</span>
        <span>debugpath.dev</span>
      </a>
      <span class="brand-tag">SSH-native incident lab</span>
    </div>
    <nav aria-label="primary navigation">
      <a href="/cases">Cases</a>
      <a href="/leaderboard">Leaderboard</a>
      <a href="/replays">Replays</a>
      <a href="/authoring">Authoring</a>
      <a href="/status">Status</a>
      <button class="theme-toggle" type="button" aria-label="Toggle color theme" title="Toggle color theme" data-theme-toggle>
        <span class="icon icon-sun" aria-hidden="true"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16" fill="currentColor"><path d="M8 12.5a4.5 4.5 0 1 1 0-9 4.5 4.5 0 0 1 0 9ZM8 0a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0V.75A.75.75 0 0 1 8 0Zm0 13a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0v-1.5A.75.75 0 0 1 8 13ZM2.343 2.343a.75.75 0 0 1 1.06 0l1.06 1.06a.75.75 0 1 1-1.06 1.06l-1.06-1.06a.75.75 0 0 1 0-1.06Zm9.193 9.193a.75.75 0 0 1 1.061 0l1.06 1.06a.75.75 0 1 1-1.06 1.061l-1.06-1.06a.75.75 0 0 1 0-1.061ZM16 8a.75.75 0 0 1-.75.75h-1.5a.75.75 0 0 1 0-1.5h1.5A.75.75 0 0 1 16 8ZM3 8a.75.75 0 0 1-.75.75H.75a.75.75 0 0 1 0-1.5h1.5A.75.75 0 0 1 3 8Zm10.657-5.657a.75.75 0 0 1 0 1.06l-1.06 1.06a.75.75 0 1 1-1.061-1.06l1.06-1.06a.75.75 0 0 1 1.061 0Zm-9.193 9.193a.75.75 0 0 1 0 1.061l-1.06 1.06a.75.75 0 1 1-1.061-1.06l1.06-1.061a.75.75 0 0 1 1.061 0Z"/></svg></span>
        <span class="icon icon-moon" aria-hidden="true"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16" fill="currentColor"><path d="M9.598 1.591a.749.749 0 0 1 .785-.175 7.001 7.001 0 1 1-8.967 8.967.75.75 0 0 1 .961-.96 5.5 5.5 0 0 0 7.046-7.046.75.75 0 0 1 .175-.786Zm1.616 1.945a7 7 0 0 1-7.678 7.678 5.499 5.499 0 1 0 7.678-7.678Z"/></svg></span>
      </button>
      <a class="primary-action" href="/#ssh-entrypoint">SSH in now</a>
    </nav>
  </header>
  <main id="main">{}</main>
  <footer class="site-footer">
    <span>SSH-native incident lab · deterministic, inspectable, self-hostable.</span>
    <a href="/standards">Case quality standards</a>
  </footer>
  <script>
    (function() {{
      var btn = document.querySelector('[data-theme-toggle]');
      if (!btn) return;
      btn.addEventListener('click', function() {{
        var current = document.documentElement.getAttribute('data-theme') || 'dark';
        var next = current === 'dark' ? 'light' : 'dark';
        document.documentElement.setAttribute('data-theme', next);
        try {{ localStorage.setItem('debugpath-theme', next); }} catch (e) {{}}
      }});
    }})();
  </script>
</body>
</html>"##,
        escape_html(title),
        body
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[derive(Debug)]
struct NotFound;

impl IntoResponse for NotFound {
    fn into_response(self) -> Response {
        (
            StatusCode::NOT_FOUND,
            Html(page("Not Found", "<h1>Not Found</h1>")),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use std::path::PathBuf;
    use tower::ServiceExt;

    async fn route(uri: &str) -> (StatusCode, String) {
        let response = app(seeded_site())
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[test]
    fn home_route_html_keeps_ssh_entrypoint_primary() {
        let html = render_home(&seeded_site());
        assert!(html.contains("ssh debugpath.dev"));
        assert!(html.contains("/cases/slow-checkout"));
        assert!(html.contains("Leaderboard"));
        assert!(html.contains("Recent solves"));
    }

    #[test]
    fn catalog_links_cases() {
        let html = render_case_catalog(&[CaseSummary::new(
            "slow-checkout",
            "Slow Checkout",
            "Latency jumps after deploy.",
            "intro",
            "checkout-api",
            6,
            5,
            2,
            1,
        )]);
        assert!(html.contains("Case Catalog"));
        assert!(html.contains("Slow Checkout"));
    }

    #[test]
    fn cases_can_be_loaded_from_repo_fixtures_for_site_data() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cases");
        let data = SiteData::from_cases_root(root).expect("cases load into site data");
        assert_eq!(data.cases.len(), 5);
        assert_eq!(data.featured_slug, "slow-checkout");
        assert!(data.data_source.contains("validated case fixtures"));
    }

    #[test]
    fn replay_route_html_renders_events() {
        let html = render_replay(&[
            ReplayEvent::CommandRun {
                command: "sql explain checkout_recent_orders".to_owned(),
                evidence: vec!["seq-scan-orders".to_owned()],
                damage: 0,
            },
            ReplayEvent::FixApplied {
                fix_id: "add_orders_status_created_at_index".to_owned(),
                solves: true,
            },
        ]);
        assert!(html.contains("replay events"));
        assert!(html.contains("seq-scan-orders"));
        assert!(html.contains("add_orders_status_created_at_index"));
    }

    #[tokio::test]
    async fn axum_routes_expose_public_product_surface() {
        for (uri, expected) in [
            ("/", "ssh debugpath.dev"),
            ("/cases", "Corrupt Uploads"),
            ("/cases/slow-checkout", "checkout-api orders query"),
            ("/leaderboard", "@rootcause"),
            ("/solves", "Recent solves"),
            ("/players/rootcause", "Solved cases"),
            ("/replays", "Replay Viewer"),
            ("/replays/seed-slow-checkout", "diagnosis submitted"),
            ("/authoring", "just validate-cases"),
            ("/standards", "fair false trail"),
            ("/status", "Health checks"),
            ("/readyz", "ready"),
        ] {
            let (status, body) = route(uri).await;
            assert_eq!(status, StatusCode::OK, "{uri}");
            assert!(body.contains(expected), "{uri} did not contain {expected}");
        }
    }

    #[tokio::test]
    async fn axum_routes_return_404_for_missing_resources() {
        for uri in ["/cases/missing", "/players/missing", "/replays/missing"] {
            let (status, body) = route(uri).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
            assert!(body.contains("Not Found"));
        }
    }
}
