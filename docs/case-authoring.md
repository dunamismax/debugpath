# Case Authoring Guide

Cases are Git-authored incident simulations. Treat every case as production
code: small, reviewable, deterministic, and testable.

## Required Shape

Each case lives in its own directory under `cases/`:

```text
cases/<slug>/
  case.toml
  brief.md
  logs.ndjson
  metrics.toml
  schema.sql
  traces.json
  commands.toml
  scoring.toml
  diffs/
  fixtures/
  rows/
  runbooks/
```

The loader validates metadata, artifact references, timestamps, duplicate IDs,
command fixtures, diagnosis evidence, hint evidence, false-trail evidence, fix
options, and scoring references.

## Incident Design

Start from the root cause and work backward into evidence. A player should be
able to solve the case by observing system behavior, forming a hypothesis,
testing it through constrained commands, and submitting a specific diagnosis.

A complete case needs:

- a short brief that gives impact and urgency without solving the incident
- realistic logs with noise, timestamps, components, levels, and request IDs
- metrics with enough signal to compare healthy and unhealthy behavior
- trace spans with coherent timing and parent-child relationships
- SQL schema and rows when database behavior matters
- deploy diffs or config snippets when a change triggered the incident
- command fixtures for every shell or SQL command the player can run
- at least one fair false trail backed by evidence
- hints that reveal direction gradually and carry score cost
- fix options that separate root-cause fixes from symptom masking or unsafe
  actions

Do not require real host commands, containers, secrets, production data, or
network access. All player-visible command output must come from fixtures or
explicit engine handlers.

## Evidence Standards

Evidence IDs are the contract between content, commands, hints, diagnosis, and
scoring. Use stable, descriptive IDs such as `seq-scan-orders` or
`health-path-drift`.

Required diagnosis evidence should be enough to prove:

- what failed
- where it failed
- why it failed now
- why the chosen fix addresses the root cause

False trails should be plausible but bounded. They can be noisy or incomplete,
but they must not depend on arbitrary hidden knowledge.

## Fixture Standards

Fixtures should feel inspected under incident pressure:

- keep outputs compact enough to scan in a terminal
- include useful context, not only the winning line
- include warnings or background noise when realistic
- keep timestamps coherent across logs, metrics, traces, and deploy markers
- avoid private names, real customer data, credentials, tokens, or dumps

## Review Checklist

Before checking in a case, verify:

- [ ] `just validate-cases` passes.
- [ ] The case has a clear root cause and one authored root-cause fix.
- [ ] The diagnosis expectation includes root cause, affected component,
      blast radius, and evidence.
- [ ] Every diagnosis evidence ID appears in `scoring.toml`.
- [ ] Every command has a fixture or deterministic engine handler.
- [ ] No command can escape the simulation boundary.
- [ ] Logs and case metadata use UTC timestamps ending in `Z`.
- [ ] At least one fair false trail exists and references known evidence.
- [ ] Hints cost score and reveal no more than their cost justifies.
- [ ] Symptom-masking and unsafe fixes are labeled distinctly from root-cause
      fixes.
- [ ] Fixture data contains no secrets, real customer data, private logs, or
      production identifiers.
- [ ] A test or validation path proves the case can load before runtime.

## Local Validation

Run the case validator after content changes:

```sh
just validate-cases
```

For broad changes to content schema or engine behavior, run the full gate:

```sh
just gate
```
