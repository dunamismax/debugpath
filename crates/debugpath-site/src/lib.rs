use debugpath_engine::ReplayEvent;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseSummary {
    pub slug: String,
    pub title: String,
    pub difficulty: String,
}

pub fn render_home(featured: &CaseSummary) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>debugpath.dev</title></head>
<body>
  <main>
    <h1>debugpath.dev</h1>
    <p><code>ssh debugpath.dev</code></p>
    <section aria-label="featured incident">
      <h2>Featured incident</h2>
      <a href="/cases/{slug}">{title}</a>
      <p>{difficulty}</p>
    </section>
  </main>
</body>
</html>"#,
        slug = featured.slug,
        title = featured.title,
        difficulty = featured.difficulty
    )
}

pub fn render_case_catalog(cases: &[CaseSummary]) -> String {
    let mut html = String::from("<ul>");
    for case in cases {
        html.push_str(&format!(
            r#"<li><a href="/cases/{slug}">{title}</a> <span>{difficulty}</span></li>"#,
            slug = case.slug,
            title = case.title,
            difficulty = case.difficulty
        ));
    }
    html.push_str("</ul>");
    html
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

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_route_html_keeps_ssh_entrypoint_primary() {
        let html = render_home(&CaseSummary {
            slug: "slow-checkout".to_owned(),
            title: "Slow Checkout".to_owned(),
            difficulty: "intro".to_owned(),
        });
        assert!(html.contains("ssh debugpath.dev"));
        assert!(html.contains("/cases/slow-checkout"));
    }

    #[test]
    fn catalog_links_cases() {
        let html = render_case_catalog(&[CaseSummary {
            slug: "slow-checkout".to_owned(),
            title: "Slow Checkout".to_owned(),
            difficulty: "intro".to_owned(),
        }]);
        assert!(html.contains("<ul>"));
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
}
