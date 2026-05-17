CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE published_cases (
    slug TEXT PRIMARY KEY,
    case_id TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    difficulty TEXT NOT NULL,
    component TEXT NOT NULL,
    content_version TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    retired_at TIMESTAMPTZ
);

CREATE TABLE players (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    handle TEXT NOT NULL UNIQUE,
    display_name TEXT,
    is_anonymous BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    case_slug TEXT NOT NULL REFERENCES published_cases(slug),
    status TEXT NOT NULL CHECK (status IN ('started', 'submitted', 'fixed', 'abandoned')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    submitted_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    engine_version TEXT NOT NULL,
    case_content_version TEXT NOT NULL
);

CREATE INDEX attempts_player_started_idx ON attempts (player_id, started_at DESC);
CREATE INDEX attempts_case_started_idx ON attempts (case_slug, started_at DESC);

CREATE TABLE diagnosis_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attempt_id UUID NOT NULL REFERENCES attempts(id) ON DELETE CASCADE,
    root_cause TEXT NOT NULL,
    evidence JSONB NOT NULL,
    affected_component TEXT NOT NULL,
    proposed_fix TEXT NOT NULL,
    blast_radius TEXT NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX diagnosis_submissions_attempt_idx
    ON diagnosis_submissions (attempt_id, submitted_at DESC);

CREATE TABLE scores (
    attempt_id UUID PRIMARY KEY REFERENCES attempts(id) ON DELETE CASCADE,
    total INTEGER NOT NULL CHECK (total >= 0),
    max_score INTEGER NOT NULL CHECK (max_score >= 0),
    root_cause_correct BOOLEAN NOT NULL,
    fix_solved BOOLEAN NOT NULL,
    evidence_found INTEGER NOT NULL CHECK (evidence_found >= 0),
    damage_penalty INTEGER NOT NULL CHECK (damage_penalty >= 0),
    hint_penalty INTEGER NOT NULL CHECK (hint_penalty >= 0),
    time_penalty INTEGER NOT NULL CHECK (time_penalty >= 0),
    scored_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX scores_leaderboard_idx ON scores (total DESC, scored_at ASC);

CREATE TABLE replay_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attempt_id UUID NOT NULL REFERENCES attempts(id) ON DELETE CASCADE,
    sequence INTEGER NOT NULL CHECK (sequence >= 0),
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (attempt_id, sequence)
);

CREATE INDEX replay_events_attempt_sequence_idx
    ON replay_events (attempt_id, sequence);

CREATE TABLE unlocks (
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    case_slug TEXT NOT NULL REFERENCES published_cases(slug),
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reason TEXT NOT NULL,
    PRIMARY KEY (player_id, case_slug)
);

CREATE TABLE authored_case_drafts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_player_id UUID REFERENCES players(id) ON DELETE SET NULL,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('draft', 'review', 'rejected', 'published')),
    draft JSONB NOT NULL,
    validation_errors JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX authored_case_drafts_owner_updated_idx
    ON authored_case_drafts (owner_player_id, updated_at DESC);

CREATE INDEX authored_case_drafts_slug_idx ON authored_case_drafts (slug);
