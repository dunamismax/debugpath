use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use debugpath_engine::ReplayEvent;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseSummary {
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub difficulty: String,
    pub component: String,
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
        .route("/replays/{id}", get(replay_detail))
        .route("/authoring", get(authoring_docs))
        .route("/standards", get(case_standards))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(Arc::new(data))
}

pub fn seeded_site() -> SiteData {
    SiteData {
        cases: vec![
            CaseSummary::new(
                "slow-checkout",
                "Slow Checkout",
                "API latency jumps after a deploy and points toward a query shape change.",
                "intro",
                "checkout-api orders query",
            ),
            CaseSummary::new(
                "pinned-postgres",
                "Pinned Postgres",
                "Dashboard traffic pins database CPU after a feature flag enables heavier joins.",
                "intermediate",
                "analytics dashboard",
            ),
            CaseSummary::new(
                "green-ci-bad-prod",
                "Green CI, Bad Prod",
                "A deploy passes CI while production returns 502s because health checks drift.",
                "intro",
                "edge routing",
            ),
            CaseSummary::new(
                "memory-tide",
                "Memory Tide",
                "Upload API memory climbs under load after body buffering changes.",
                "intermediate",
                "upload-api",
            ),
            CaseSummary::new(
                "corrupt-uploads",
                "Corrupt Uploads",
                "Large archive uploads intermittently fail because chunks are reassembled out of order.",
                "intermediate",
                "upload assembler",
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

impl CaseSummary {
    pub fn new(
        slug: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        difficulty: impl Into<String>,
        component: impl Into<String>,
    ) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            summary: summary.into(),
            difficulty: difficulty.into(),
            component: component.into(),
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
    Path(slug): Path<String>,
) -> Result<Html<String>, NotFound> {
    let case = data.case(&slug).ok_or(NotFound)?;
    Ok(Html(page(
        &case.title,
        &format!(
            r#"<section aria-label="case detail">
  <p><a href="/cases">Case catalog</a></p>
  <h1>{}</h1>
  <p>{}</p>
  <dl>
    <dt>Difficulty</dt><dd>{}</dd>
    <dt>Component</dt><dd>{}</dd>
  </dl>
</section>"#,
            escape_html(&case.title),
            escape_html(&case.summary),
            escape_html(&case.difficulty),
            escape_html(&case.component)
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
    Path(handle): Path<String>,
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
  <dl>
    <dt>Solved cases</dt><dd>{}</dd>
    <dt>Best score</dt><dd>{}</dd>
    <dt>Recent case</dt><dd>{}</dd>
  </dl>
</section>"#,
            escape_html(&player.display_name),
            escape_html(&player.handle),
            player.solved_cases,
            player.best_score,
            escape_html(&player.recent_case)
        ),
    )))
}

async fn replay_detail(
    State(data): State<Arc<SiteData>>,
    Path(id): Path<String>,
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
  <h1>Replay</h1>
  <p><a href="/players/{player}">@{player}</a> solved <a href="/cases/{case_slug}">{case_slug}</a>.</p>
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

impl SiteData {
    fn case(&self, slug: &str) -> Option<&CaseSummary> {
        self.cases.iter().find(|case| case.slug == slug)
    }
}

pub fn render_home(data: &SiteData) -> String {
    let featured = data
        .case(&data.featured_slug)
        .or_else(|| data.cases.first())
        .expect("seeded site has at least one case");
    page(
        "debugpath.dev",
        &format!(
            r#"<section aria-label="hero">
  <h1>debugpath.dev</h1>
  <p class="entrypoint"><code>ssh debugpath.dev</code></p>
  <p>Solve production incidents from the terminal. Read logs, query fixtures, inspect traces, chase false leads, and prove the root cause.</p>
</section>
<section aria-label="featured incident">
  <h2>Featured incident</h2>
  <a href="/cases/{slug}">{title}</a>
  <p>{summary}</p>
</section>
{}
{}
<nav aria-label="product sections">
  <a href="/cases">Case catalog</a>
  <a href="/leaderboard">Leaderboard</a>
  <a href="/solves">Recent solves</a>
  <a href="/replays/seed-slow-checkout">Replay viewer</a>
  <a href="/authoring">Authoring docs</a>
  <a href="/standards">Case standards</a>
</nav>"#,
            leaderboard_section(&data.leaderboard),
            recent_solves_section(&data.recent_solves),
            slug = escape_html(&featured.slug),
            title = escape_html(&featured.title),
            summary = escape_html(&featured.summary)
        ),
    )
}

pub fn render_case_catalog(cases: &[CaseSummary]) -> String {
    let mut items = String::new();
    for case in cases {
        items.push_str(&format!(
            r#"<li>
  <a href="/cases/{slug}">{title}</a>
  <span>{difficulty}</span>
  <p>{summary}</p>
</li>"#,
            slug = escape_html(&case.slug),
            title = escape_html(&case.title),
            difficulty = escape_html(&case.difficulty),
            summary = escape_html(&case.summary)
        ));
    }
    page(
        "Case Catalog",
        &format!(
            r#"<section aria-label="case catalog">
  <h1>Case Catalog</h1>
  <ul>{items}</ul>
</section>"#
        ),
    )
}

pub fn render_replay(events: &[ReplayEvent]) -> String {
    let mut html = String::from(r#"<ol aria-label="replay events">"#);
    for event in events {
        let item = match event {
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
        };
        html.push_str(&format!("<li>{}</li>", escape_html(&item)));
    }
    html.push_str("</ol>");
    html
}

fn leaderboard_section(entries: &[LeaderboardEntry]) -> String {
    let mut rows = String::new();
    for entry in entries {
        rows.push_str(&format!(
            r#"<tr>
  <td>{}</td>
  <td><a href="/players/{player}">@{player}</a></td>
  <td><a href="/cases/{case_slug}">{case_slug}</a></td>
  <td>{}</td>
  <td>{}</td>
</tr>"#,
            entry.rank,
            entry.score,
            escape_html(&entry.solved_at),
            player = escape_html(&entry.player_handle),
            case_slug = escape_html(&entry.case_slug)
        ));
    }
    format!(
        r#"<section aria-label="leaderboard">
  <h2>Leaderboard</h2>
  <table>
    <thead><tr><th>Rank</th><th>Player</th><th>Case</th><th>Score</th><th>Solved</th></tr></thead>
    <tbody>{rows}</tbody>
  </table>
</section>"#
    )
}

fn recent_solves_section(solves: &[RecentSolve]) -> String {
    let mut items = String::new();
    for solve in solves {
        items.push_str(&format!(
            r#"<li>
  <a href="/players/{player}">@{player}</a>
  solved <a href="/cases/{case_slug}">{case_title}</a>
  with {score} points at {solved_at}.
</li>"#,
            player = escape_html(&solve.player_handle),
            case_slug = escape_html(&solve.case_slug),
            case_title = escape_html(&solve.case_title),
            score = solve.score,
            solved_at = escape_html(&solve.solved_at)
        ));
    }
    format!(
        r#"<section aria-label="recent solves">
  <h2>Recent solves</h2>
  <ul>{items}</ul>
</section>"#
    )
}

fn page(title: &str, body: &str) -> String {
    format!(
        r#"<!doctype html>
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
    }}
    body {{
      margin: 0;
      color: #17202a;
      background: #f7f9fb;
    }}
    main {{
      width: min(1120px, calc(100vw - 32px));
      margin: 0 auto;
      padding: 32px 0 48px;
    }}
    h1, h2 {{
      margin: 0 0 12px;
      letter-spacing: 0;
    }}
    section, nav {{
      padding: 20px 0;
      border-bottom: 1px solid #d8dee6;
    }}
    a {{
      color: #005f73;
      font-weight: 650;
    }}
    code {{
      padding: 2px 6px;
      border: 1px solid #cad2dc;
      background: #ffffff;
    }}
    .entrypoint code {{
      font-size: 1.25rem;
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
    }}
    th, td {{
      padding: 8px 10px;
      border-bottom: 1px solid #d8dee6;
      text-align: left;
    }}
    nav {{
      display: flex;
      flex-wrap: wrap;
      gap: 12px 18px;
    }}
    @media (prefers-color-scheme: dark) {{
      body {{
        color: #e7edf3;
        background: #111820;
      }}
      section, nav, th, td {{
        border-color: #2b3948;
      }}
      a {{
        color: #8bd3dd;
      }}
      code {{
        border-color: #3a4a5c;
        background: #182330;
      }}
    }}
  </style>
</head>
<body>
  <main>{}</main>
</body>
</html>"#,
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
        )]);
        assert!(html.contains("Case Catalog"));
        assert!(html.contains("Slow Checkout"));
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
            ("/replays/seed-slow-checkout", "diagnosis submitted"),
            ("/authoring", "just validate-cases"),
            ("/standards", "fair false trail"),
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
