use std::env;
use std::path::PathBuf;

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        usage_and_exit();
    };

    match command.as_str() {
        "validate-cases" => {
            let root = args.next().unwrap_or_else(|| "cases".to_owned());
            validate_cases(PathBuf::from(root));
        }
        "release-smoke" => {
            let root = args.next().unwrap_or_else(|| "cases".to_owned());
            release_smoke(PathBuf::from(root));
        }
        _ => usage_and_exit(),
    }
}

fn validate_cases(root: PathBuf) {
    match debugpath_content::load_cases(&root) {
        Ok(cases) => {
            println!("validated {} case(s) under {}", cases.len(), root.display());
            for case in cases {
                println!("- {}", case.metadata.slug);
            }
        }
        Err(error) => {
            eprintln!("case validation failed: {error}");
            std::process::exit(1);
        }
    }
}

fn release_smoke(root: PathBuf) {
    let cases = match debugpath_content::load_cases(&root) {
        Ok(cases) => cases,
        Err(error) => {
            eprintln!("case validation failed: {error}");
            std::process::exit(1);
        }
    };
    assert_smoke(!cases.is_empty(), "case catalog is not empty");

    let slow_checkout = cases
        .into_iter()
        .find(|case| case.metadata.slug == "slow-checkout")
        .unwrap_or_else(|| {
            eprintln!("release smoke failed: slow-checkout case is missing");
            std::process::exit(1);
        });
    let root_fix = slow_checkout.scoring.root_fix.clone();
    let diagnosis = slow_checkout.diagnosis.clone();
    let evidence_commands: Vec<_> = slow_checkout
        .commands
        .iter()
        .filter(|command| {
            command
                .evidence
                .iter()
                .any(|evidence| diagnosis.evidence.contains(evidence))
        })
        .map(|command| command.command.clone())
        .collect();

    let mut session = debugpath_engine::Session::new(slow_checkout);
    for command in evidence_commands {
        let output = session.run_command(&command).unwrap_or_else(|error| {
            eprintln!("release smoke failed: command {command:?}: {error}");
            std::process::exit(1);
        });
        assert_smoke(!output.trim().is_empty(), "command fixture returned output");
    }
    session
        .submit_diagnosis(debugpath_engine::DiagnosisSubmission {
            root_cause: diagnosis.root_cause,
            evidence: diagnosis.evidence,
            affected_component: diagnosis.affected_component,
            proposed_fix: root_fix.clone(),
            blast_radius: diagnosis.blast_radius,
        })
        .unwrap_or_else(|error| {
            eprintln!("release smoke failed: diagnosis rejected: {error}");
            std::process::exit(1);
        });
    session.apply_fix(&root_fix).unwrap_or_else(|error| {
        eprintln!("release smoke failed: fix rejected: {error}");
        std::process::exit(1);
    });
    let score = session.score();
    assert_smoke(score.root_cause_correct, "engine scores root cause");
    assert_smoke(score.fix_solved, "engine scores root fix");
    assert_smoke(
        !session.replay().is_empty(),
        "engine captures replay events",
    );

    smoke_db();
    smoke_site(session.replay());
    smoke_tui();
    smoke_ssh();
    println!("release smoke passed");
}

fn smoke_db() {
    let migration = debugpath_db::migrations()[0].sql;
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
        assert_smoke(
            migration.contains(&format!("CREATE TABLE {table}")),
            &format!("database migration contains {table}"),
        );
    }

    for query in [
        debugpath_db::sql::UPSERT_PUBLISHED_CASE,
        debugpath_db::sql::UPSERT_PLAYER,
        debugpath_db::sql::INSERT_ATTEMPT,
        debugpath_db::sql::INSERT_DIAGNOSIS,
        debugpath_db::sql::UPSERT_SCORE,
        debugpath_db::sql::INSERT_REPLAY_EVENT,
        debugpath_db::sql::UPSERT_UNLOCK,
        debugpath_db::sql::SELECT_LEADERBOARD,
        debugpath_db::sql::SELECT_RECENT_SOLVES,
        debugpath_db::sql::SELECT_REPLAY_EVENTS,
    ] {
        assert_smoke(!query.trim().is_empty(), "database query is present");
    }
}

fn smoke_site(replay: &[debugpath_engine::ReplayEvent]) {
    let data = debugpath_site::seeded_site();
    let home = debugpath_site::render_home(&data);
    assert_smoke(
        home.contains("ssh debugpath.dev"),
        "site renders SSH entrypoint",
    );
    assert_smoke(home.contains("Leaderboard"), "site renders leaderboard");
    assert_smoke(home.contains("Recent solves"), "site renders recent solves");

    let replay_html = debugpath_site::render_replay(replay);
    assert_smoke(
        replay_html.contains("diagnosis submitted"),
        "site renders replay events",
    );
}

fn smoke_tui() {
    for pane in [
        "Brief", "Systems", "Logs", "Metrics", "Shell", "SQL", "Trace", "Notes",
    ] {
        assert_smoke(
            debugpath_tui::CORE_PANES.contains(&pane),
            &format!("TUI exposes {pane} pane"),
        );
    }
}

fn smoke_ssh() {
    let summary = debugpath_ssh::smoke_summary();
    assert_smoke(
        summary.contains(debugpath_ssh::PRODUCTION_ENTRYPOINT),
        "SSH smoke includes production entrypoint",
    );
    assert_smoke(
        summary.contains(debugpath_ssh::LOCAL_DEV_BIND_ADDR),
        "SSH smoke includes local development bind address",
    );

    let mut controls =
        debugpath_ssh::abuse::AbuseControls::new(debugpath_ssh::abuse::AbuseConfig {
            max_sessions_per_peer: 1,
            max_connections_per_window: 2,
            window_seconds: 60,
            max_command_bytes: 32,
        });
    assert_smoke(
        controls.start_session("203.0.113.10:5500", 1)
            == debugpath_ssh::abuse::AbuseDecision::Accepted,
        "SSH abuse controls accept first session",
    );
    assert_smoke(
        controls.start_session("203.0.113.10:5501", 2)
            == debugpath_ssh::abuse::AbuseDecision::Rejected(
                debugpath_ssh::abuse::RejectReason::TooManySessions,
            ),
        "SSH abuse controls enforce session limits",
    );
    controls.end_session("203.0.113.10:5500", 3);
    assert_smoke(
        controls.inspect_command("203.0.113.10:5500", "", 4)
            == debugpath_ssh::abuse::AbuseDecision::Rejected(
                debugpath_ssh::abuse::RejectReason::EmptyCommand,
            ),
        "SSH abuse controls reject invalid commands",
    );
    assert_smoke(
        controls
            .audit_events()
            .iter()
            .all(|event| event.peer != "203.0.113.10"),
        "SSH audit events redact peer metadata",
    );
}

fn assert_smoke(condition: bool, message: &str) {
    if condition {
        println!("ok - {message}");
    } else {
        eprintln!("release smoke failed: {message}");
        std::process::exit(1);
    }
}

fn usage_and_exit() -> ! {
    eprintln!("usage: cargo run -p xtask -- <validate-cases|release-smoke> [cases-dir]");
    std::process::exit(2);
}
