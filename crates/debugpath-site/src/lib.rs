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
            r#"<section aria-label="case detail">
  <p class="backlink"><a href="/cases">Case catalog</a></p>
  <h1>{}</h1>
  <p class="lede">{}</p>
  <dl class="metadata">
    <div><dt>Difficulty</dt><dd>{}</dd></div>
    <div><dt>Component</dt><dd>{}</dd></div>
  </dl>
  <dl class="metric-row" aria-label="case investigation surface">
    <div><dt>Commands</dt><dd>{}</dd></div>
    <div><dt>Evidence IDs</dt><dd>{}</dd></div>
    <div><dt>Hints</dt><dd>{}</dd></div>
    <div><dt>False trails</dt><dd>{}</dd></div>
  </dl>
  <div class="action-row">
    <a class="primary-action" href="/">SSH in now</a>
    <a href="/standards">Review case standards</a>
  </div>
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
            r#"<section aria-label="player profile">
  <h1>{}</h1>
  <p><code>@{}</code></p>
  <dl class="metric-row">
    <div><dt>Solved cases</dt><dd>{}</dd></div>
    <div><dt>Best score</dt><dd>{}</dd></div>
    <div><dt>Recent case</dt><dd>{}</dd></div>
  </dl>
  <div class="action-row">
    <a href="/leaderboard">Leaderboard</a>
    <a href="/replays">Replay viewer</a>
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
            r#"<section aria-label="replay summary">
  <p class="backlink"><a href="/replays">Replay index</a></p>
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
        r#"<section aria-label="authoring docs">
  <h1>Authoring Docs</h1>
  <p>Cases are Git-authored incidents with deterministic artifacts, constrained commands, diagnosis expectations, fix options, hints, false trails, and scoring rules.</p>
  <ol>
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
        r#"<section aria-label="case quality standards">
  <h1>Case Quality Standards</h1>
  <ul>
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
        view! {
            <section aria-label="hero" class="hero">
                <div class="hero-copy">
                    <p class="kicker">"Terminal incident lab"</p>
                    <h1>"debugpath.dev"</h1>
                    <p class="entrypoint"><code>{ssh_entrypoint}</code></p>
                    <p class="lede">
                        "Solve production incidents from the terminal. Read logs, query fixtures, inspect traces, chase false leads, and prove the root cause."
                    </p>
                    <div class="action-row">
                        <a class="primary-action" href="/cases">"Open case catalog"</a>
                        <a href="/replays">"Watch a replay"</a>
                    </div>
                </div>
                <div class="ops-snapshot" aria-label="site snapshot">
                    <span>"cases online" <strong>{case_count}</strong></span>
                    <span>"seeded solves" <strong>{solve_count}</strong></span>
                    <span>"public replays" <strong>{replay_count}</strong></span>
                </div>
            </section>
            <section aria-label="featured incident" class="band">
                <p class="kicker">"Featured incident"</p>
                <h2><a href=featured_href>{featured.title}</a></h2>
                <p>{featured.summary}</p>
                <dl class="metadata">
                    <div><dt>"Difficulty"</dt><dd>{featured.difficulty}</dd></div>
                    <div><dt>"Component"</dt><dd>{featured.component}</dd></div>
                </dl>
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
            <section aria-label="case catalog" class="band">
                <p class="kicker">"Playable incidents"</p>
                <h1>"Case Catalog"</h1>
                <ul class="case-grid">
                    {cases
                        .into_iter()
                        .map(|case| {
                            let href = format!("/cases/{}", case.slug);
                            view! {
                                <li>
                                    <a href=href>{case.title}</a>
                                    <span>{case.difficulty}</span>
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
            <section aria-label="replay index" class="band">
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
                                    <a href=href>{replay.id}</a>
                                    <a href=player_href>{player_label}</a>
                                    <a href=case_href>{replay.case_slug}</a>
                                    <strong>{format!("{} events", replay.events.len())}</strong>
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
            <section aria-label="status" class="band">
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
        <section aria-label="leaderboard" class="band">
            <div class="section-heading">
                <p class="kicker">"Seeded score table"</p>
                <h2>"Leaderboard"</h2>
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
                                        <td>{entry.rank}</td>
                                        <td><a href=player_href>{player_label}</a></td>
                                        <td><a href=case_href>{entry.case_slug}</a></td>
                                        <td>{entry.score}</td>
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
        <section aria-label="recent solves" class="band">
            <div class="section-heading">
                <p class="kicker">"Latest completions"</p>
                <h2>"Recent solves"</h2>
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
                                <a href=player_href>{player_label}</a>
                                <span>"solved"</span>
                                <a href=case_href>{solve.case_title}</a>
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
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{}</title>
  <style>
    :root {{
      color-scheme: light dark;
      font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      line-height: 1.5;
      font-size: 16px;
    }}
    body {{
      margin: 0;
      color: #17202a;
      background: #f4f7f9;
    }}
    .skip-link {{
      position: absolute;
      left: 12px;
      top: -48px;
      padding: 8px 10px;
      background: #ffffff;
      border: 1px solid #17202a;
      z-index: 2;
    }}
    .skip-link:focus {{
      top: 12px;
    }}
    .site-header,
    .site-footer {{
      width: min(1180px, calc(100vw - 32px));
      margin: 0 auto;
    }}
    .site-header {{
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 20px;
      padding: 16px 0;
      border-bottom: 1px solid #d8dee6;
    }}
    .site-header nav,
    .site-footer {{
      display: flex;
      flex-wrap: wrap;
      gap: 10px 18px;
    }}
    .brand {{
      color: #17202a;
      font-weight: 800;
      text-decoration: none;
    }}
    .site-footer {{
      justify-content: space-between;
      padding: 20px 0 32px;
      border-top: 1px solid #d8dee6;
      color: #536170;
    }}
    main {{
      width: min(1180px, calc(100vw - 32px));
      margin: 0 auto;
      padding: 32px 0 48px;
    }}
    h1, h2, p {{
      overflow-wrap: anywhere;
    }}
    h1, h2 {{
      margin: 0 0 12px;
      letter-spacing: 0;
    }}
    h1 {{
      font-size: 3rem;
      line-height: 1;
    }}
    h2 {{
      font-size: 1.35rem;
    }}
    .lede {{
      max-width: 76ch;
      font-size: 1.06rem;
    }}
    section, nav {{
      padding: 24px 0;
      border-bottom: 1px solid #d8dee6;
    }}
    a {{
      color: #006b74;
      font-weight: 650;
    }}
    code {{
      display: inline-block;
      max-width: 100%;
      padding: 4px 8px;
      border: 1px solid #cad2dc;
      background: #ffffff;
      overflow-wrap: anywhere;
    }}
    .hero {{
      display: grid;
      grid-template-columns: minmax(0, 1fr) minmax(220px, 320px);
      gap: 24px;
      align-items: end;
      padding-top: 12px;
    }}
    .hero-copy p {{
      max-width: 68ch;
    }}
    .entrypoint code {{
      font-size: 1.2rem;
      border-color: #8fb1b7;
    }}
    .action-row {{
      display: flex;
      flex-wrap: wrap;
      gap: 10px 16px;
      align-items: center;
      margin-top: 18px;
    }}
    .primary-action {{
      display: inline-block;
      padding: 8px 12px;
      color: #ffffff;
      background: #006b74;
      text-decoration: none;
    }}
    .primary-action:focus,
    .primary-action:hover {{
      background: #00545b;
    }}
    .kicker {{
      margin: 0 0 8px;
      color: #6a4a00;
      font-size: 0.78rem;
      font-weight: 800;
      text-transform: uppercase;
    }}
    .ops-snapshot {{
      display: grid;
      gap: 8px;
      padding: 14px;
      border: 1px solid #cad2dc;
      background: #ffffff;
    }}
    .ops-snapshot span,
    .metadata div,
    .activity-list li {{
      display: flex;
      justify-content: space-between;
      gap: 16px;
    }}
    .metadata {{
      display: flex;
      flex-wrap: wrap;
      gap: 12px 24px;
      margin: 16px 0 0;
    }}
    .metadata dt {{
      color: #536170;
      font-size: 0.82rem;
      font-weight: 750;
    }}
    .metadata dd {{
      margin: 0;
    }}
    .metric-row {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
      gap: 10px;
      margin: 18px 0 0;
    }}
    .metric-row div,
    .mini-metrics div {{
      border-left: 3px solid #006b74;
      background: #ffffff;
      padding: 10px 12px;
    }}
    .metric-row dt,
    .mini-metrics dt {{
      color: #536170;
      font-size: 0.76rem;
      font-weight: 800;
      text-transform: uppercase;
    }}
    .metric-row dd,
    .mini-metrics dd {{
      margin: 2px 0 0;
      font-weight: 760;
    }}
    .table-wrap {{
      overflow-x: auto;
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
    }}
    th, td {{
      padding: 8px 10px;
      border-bottom: 1px solid #d8dee6;
      text-align: left;
      white-space: nowrap;
    }}
    .section-nav {{
      display: flex;
      flex-wrap: wrap;
      gap: 12px 18px;
    }}
    .case-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 12px;
      padding: 0;
      list-style: none;
    }}
    .case-grid li {{
      padding: 14px;
      border: 1px solid #d8dee6;
      background: #ffffff;
    }}
    .case-grid span,
    .case-grid small {{
      display: block;
      margin-top: 8px;
      color: #536170;
    }}
    .mini-metrics {{
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 6px;
      margin: 12px 0 0;
    }}
    .mini-metrics div {{
      padding: 7px 8px;
      border-left-color: #9a5b00;
    }}
    .activity-list,
    .replay-events {{
      display: grid;
      gap: 10px;
      padding: 0;
      list-style: none;
    }}
    .activity-list li,
    .replay-events li {{
      padding: 12px 0;
      border-top: 1px solid #d8dee6;
    }}
    .replay-events li {{
      display: grid;
      grid-template-columns: 42px 110px minmax(0, 1fr);
      gap: 12px;
      align-items: start;
    }}
    .replay-events span {{
      color: #9a5b00;
      font-weight: 800;
    }}
    .replay-events p {{
      margin: 0;
    }}
    @media (max-width: 700px) {{
      main {{
        width: min(100vw - 24px, 1180px);
        padding-top: 20px;
      }}
      .site-header,
      .site-footer {{
        width: min(100vw - 24px, 1180px);
      }}
      .site-header {{
        align-items: flex-start;
        flex-direction: column;
      }}
      .hero {{
        grid-template-columns: 1fr;
      }}
      h1 {{
        font-size: 2.35rem;
      }}
      .activity-list li,
      .replay-events li {{
        display: grid;
        grid-template-columns: 1fr;
        gap: 4px;
      }}
    }}
    @media (prefers-color-scheme: dark) {{
      body {{
        color: #e7edf3;
        background: #111820;
      }}
      section, nav, th, td, .case-grid li, .activity-list li, .replay-events li, .ops-snapshot, .site-header, .site-footer {{
        border-color: #2b3948;
      }}
      .case-grid li, .ops-snapshot, .metric-row div, .mini-metrics div, .skip-link {{
        background: #16212d;
      }}
      a, .brand {{
        color: #8bd3dd;
      }}
      .primary-action {{
        color: #071013;
        background: #8bd3dd;
      }}
      .primary-action:focus,
      .primary-action:hover {{
        background: #aadfe6;
      }}
      .kicker {{
        color: #d2b15f;
      }}
      code {{
        border-color: #3a4a5c;
        background: #182330;
      }}
    }}
  </style>
</head>
<body>
  <a class="skip-link" href="#main">Skip to content</a>
  <header class="site-header">
    <a class="brand" href="/">debugpath.dev</a>
    <nav aria-label="primary navigation">
      <a href="/cases">Cases</a>
      <a href="/leaderboard">Leaderboard</a>
      <a href="/replays">Replays</a>
      <a href="/authoring">Authoring</a>
      <a href="/status">Status</a>
    </nav>
  </header>
  <main id="main">{}</main>
  <footer class="site-footer">
    <span>SSH-native incident lab.</span>
    <a href="/standards">Case quality standards</a>
  </footer>
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
