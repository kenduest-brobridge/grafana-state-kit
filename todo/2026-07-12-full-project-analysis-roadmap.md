# Full Project Analysis and Improvement Roadmap

## Status

- State: planned
- Date: 2026-07-12
- Scope type: analysis, read-only validation, and planning only
- Final deliverable: one complete project analysis report and prioritized
  improvement roadmap in this file
- Execution method: phased, evidence-gated repository inspection

## Goal

Produce one comprehensive, evidence-backed assessment of the current
`grafana-utils` repository and convert every meaningful current finding into a
deduplicated, prioritized, independently verifiable improvement backlog.

The assessment must cover the complete product and delivery lifecycle while
treating Rust as the primary implementation surface, Python as a maintained
but secondary compatibility surface, and the completed 2026-07-10 and
2026-07-12 Grafana 13 reviews as historical baseline evidence that requires
current verification.

## Deliverable Model

This work combines two complementary approaches:

1. Produce a single complete analysis report so project health, architecture,
   risks, findings, backlog, and execution order can be evaluated together.
2. Build that report through phased evidence collection with explicit gates so
   broad conclusions are not written before repository, runtime, test, and
   contract evidence has been gathered.

All final analysis, findings, evidence, risk entries, TODO items, and roadmap
phases remain in this dated file. `todo/todo.md` stays a concise routing index.

## Non-Goals

- Do not implement any finding or TODO item.
- Do not modify source code, tests, dependency manifests, lock files,
  generated documentation, build outputs, or deployment artifacts.
- Do not reformat, rename, delete, or regenerate project files.
- Do not create implementation commits, push branches, rewrite history, or
  open a pull request as part of the analysis. A docs-only commit containing
  this roadmap or its completed report is allowed only when separately
  requested.
- Do not install tools or dependencies automatically.
- Do not inspect `rust/target`.
- Do not treat a newer dependency version, stylistic preference, or theoretical
  abstraction as an improvement without a concrete project benefit.
- Do not recommend a rewrite or service split without evidence that incremental
  remediation is impractical.

## Project-Specific Constraints

- Start all implementation and architecture analysis in `rust/src/` and
  `rust/Cargo.toml`.
- Inspect `python/grafana_utils/`, `python/tests/`, and
  `python/pyproject.toml` only for maintained compatibility, packaging,
  repository tooling, or confirmed parity boundaries.
- Use shared fixtures in `tests/fixtures/` when checking Rust, Python, and
  schema contracts.
- Use CodeGraph first for structural questions. Use native search for literal
  text, comments, logs, fixtures, or a specific file already under inspection.
- Treat the default Rust build as lean; evaluate the optional `browser` feature
  separately.
- Check public CLI and documentation conclusions against
  `scripts/contracts/command-surface.json` and
  `scripts/contracts/docs-entrypoints.json`.
- Treat `docs/user-guide/{en,zh-TW}/` and `docs/commands/{en,zh-TW}/` as source
  documentation. Treat `docs/man/*.1` and `docs/html/` as generated outputs.
- Distinguish confirmed current behavior from inference, historical evidence,
  edition-specific Grafana behavior, and claims that require a live service.
- Formal evidence must record the underlying project command, not the
  Codex-local command wrapper used to execute it.

## Historical Baseline

The following completed work is input to the new analysis, not proof of the
current state:

- `todo/2026-07-10-project-wide-grafana13-review.md`
- `todo/2026-07-10-grafana13-all-improvements.md`
- `todo/2026-07-12-grafana13-review-followup-fixes.md`

The baseline reported a healthy Rust validation lane and identified dashboard
resource v2, Python YAML packaging, datasource fallback, TUI registry,
resource history, resource-only export, namespace selection, and Git Sync
symlink issues. Many were subsequently implemented. The new analysis must:

- verify the current implementation and regression guards before closing each
  historical item;
- reopen an item only when the defect remains, regressed, or lacks a necessary
  safety contract;
- record verified resolutions as strengths or closed baseline items rather
  than duplicating them in the active backlog;
- identify new risks independently of the Grafana 13 review taxonomy.

## Evidence and Classification Rules

Every meaningful finding must reference the relevant path plus a symbol,
configuration key, workflow, manifest, command, fixture, or observed runtime
behavior. Line numbers may supplement stable identifiers but cannot be the
only evidence.

### Confidence

- **Confirmed:** directly demonstrated by current code, a focused test,
  reproducible command, or an observed execution path.
- **High confidence:** strongly supported by multiple current sources but not
  reproduced end to end.
- **Medium confidence:** plausible and evidence-backed but dependent on an
  unverified assumption.
- **Requires runtime verification:** cannot be resolved without an appropriate
  live Grafana instance, credentials, platform, or deployment environment.

### Severity

- **Critical:** credible security compromise, data loss, major outage, broken
  access control, severe correctness failure, corruption, or immediate
  deployment failure.
- **High:** likely production defect, significant reliability problem,
  important missing control, critical-path test gap, or major maintainability
  blocker.
- **Medium:** bounded technical debt, operational weakness, non-critical test
  gap, or engineering-efficiency problem.
- **Low:** minor cleanup, small consistency issue, optional tooling, or
  low-impact developer-experience improvement.
- **Informational:** verified context or strength that improves understanding
  but does not require remediation.

### Finding IDs

Use one stable prefix per primary category and do not duplicate one root cause
across categories:

- `COR-*` functional correctness
- `ARC-*` architecture
- `MNT-*` maintainability
- `SEC-*` security
- `PER-*` performance and scalability
- `REL-*` reliability and resilience
- `API-*` API and interface design
- `CFG-*` configuration
- `OBS-*` observability
- `TST-*` testing
- `DEP-*` dependencies and supply chain
- `CICD-*` build, CI/CD, packaging, and release
- `DOC-*` documentation and developer experience
- `OPS-*` deployment and operational readiness

## Required Analysis Coverage

### Project and repository model

- Purpose, audience, capabilities, maturity, runtime components, technologies,
  external integrations, and deployment model.
- Top-level structure, module ownership, package boundaries, naming, generated
  content, obsolete or suspicious areas, and separation of concerns.
- Application entry points, public command families, build entry points, test
  entry points, documentation generators, and release paths.

### Architecture and runtime flows

- Major Rust components and dependency direction.
- CLI parsing and dispatch, configuration/profile loading, authentication,
  Grafana transport, domain command execution, output/rendering, review and
  mutation confirmation, TUI flows, and optional browser flows.
- Python compatibility and tooling boundaries plus shared contract data flow.
- Integration, trust, persistence, concurrency, failure, and edition/version
  boundaries.

### Quality, correctness, and maintainability

- Readability, responsibility, complexity, coupling, duplicate logic, hidden
  effects, global state, error propagation, optional handling, resource
  cleanup, concurrency, interfaces, magic values, dead/deprecated code, and
  over- or underengineering.
- Edge cases involving pagination, retries, timeouts, partial operations,
  idempotency, state synchronization, date/time handling, numeric boundaries,
  validation, fallbacks, and error visibility.
- Literal searches for `TODO`, `FIXME`, `HACK`, `XXX`, deprecation markers,
  temporary workarounds, commented-out code, and unsafe assumptions.

### Security and supply chain

- Authentication, authorization, tokens, secrets, sensitive logging, input and
  output handling, command/path injection, traversal, SSRF, TLS, unsafe shell
  execution, filesystem/archive handling, and destructive-action boundaries.
- Dependency pinning, lock files, unused or duplicated direct dependencies,
  maintenance and compatibility risks, licenses, build reproducibility,
  workflow permissions, CI credentials, artifact provenance, container
  permissions, and release exposure where present.
- Do not claim exploitability without confirming the reachable execution path.

### Performance, reliability, and operations

- Algorithmic cost, repeated work, collection bounds, pagination, streaming,
  serialization, network calls, caching, concurrency, blocking work, startup,
  build performance, TUI rendering, and logging cost.
- Separate confirmed problems from likely bottlenecks and profiling candidates.
- Timeouts, retries, backoff, limits, graceful shutdown, health behavior,
  idempotency, partial failures, recovery, dependency outages, backup/restore,
  rollback, diagnostics, capacity assumptions, and operational runbooks.

### Interfaces, configuration, and observability

- Public CLI taxonomy, help, defaults, validation, output contracts, error
  shape, pagination/filtering/sorting, compatibility, mutation safety, and
  documentation.
- Grafana classic and App Platform API capability discovery, version and
  edition boundaries, namespaces, backward compatibility, and contract tests.
- Configuration sources, precedence, environment overrides, required and
  optional values, secret treatment, unsafe defaults, duplication, and docs.
- Structured logging, levels, correlation context, auditability, metrics,
  label cardinality, traces, health reporting, dashboards, alerting readiness,
  and diagnostic quality where applicable.

### Tests, delivery, and documentation

- Unit, integration, contract, live, failure-path, concurrency, security,
  performance, packaging, generated-doc, and cross-language tests.
- Test determinism, isolation, fixtures, mocks, critical-path coverage, CI
  execution, and flaky-test risk; do not infer quality from test count alone.
- Formatting, linting, static analysis, build matrices, security scans,
  artifact creation, release tagging, promotion, rollback, environment parity,
  and workflow duplication or fragility.
- README, handbook, command reference, maintainer docs, architecture,
  troubleshooting, contribution, release, deployment, and incident guidance
  against actual behavior.

## Phased Investigation Plan

Each phase updates the evidence log and checklist below. A phase may identify
candidates, but candidates do not become final findings until their evidence
and classification pass Phase 7.

### Phase 0: Baseline and scope lock

- [ ] Record branch, commit, worktree status, repository roots, and toolchain
      versions without changing the environment.
- [ ] Capture indexed repository structure and primary/secondary language
      boundaries while excluding `rust/target`.
- [ ] Map build, test, documentation, packaging, CI, and release entry points.
- [ ] Reconcile historical baseline items with current commits and regression
      tests; label each resolved, still open, regressed, or runtime-dependent.
- [ ] Record unavailable tools, credentials, services, or platforms as evidence
      limitations rather than project defects.

**Gate:** scope, baseline commit, validation lanes, exclusions, and evidence
limitations are explicit.

### Phase 1: Architecture and execution flows

- [ ] Map Rust crate entry points, command registration and dispatch, shared
      infrastructure, Grafana clients, domain modules, renderers, TUI, and
      optional browser boundaries.
- [ ] Trace representative read-only and mutation flows from operator input to
      transport, response handling, output, and error reporting.
- [ ] Map Python compatibility, packaging, scripts, contracts, fixtures, and
      generated documentation relationships.
- [ ] Document dependency direction, integration boundaries, trust boundaries,
      and failure boundaries; add one Mermaid diagram only if it materially
      clarifies these relationships.
- [ ] Identify unclear ownership, mixed responsibilities, high-coupling areas,
      and architecture intent that differs from implementation.

**Gate:** every major component has a responsibility, entry point, dependency,
and evidence reference.

### Phase 2: Correctness, security, reliability, and performance

- [ ] Review all public command families and shared infrastructure for the
      required correctness and maintainability concerns.
- [ ] Review authentication, authorization, secret handling, destructive
      operations, filesystem/network inputs, logging, TLS, and CI/release trust
      boundaries.
- [ ] Review pagination, timeouts, retry/fallback behavior, partial failures,
      idempotency, concurrency, recovery, and failure visibility.
- [ ] Review likely hot paths and resource bounds without proposing premature
      optimization.
- [ ] Reproduce suspected defects with the smallest existing test, focused
      command, fixture, or source-level execution trace practical.

**Gate:** no candidate bug or vulnerability is called confirmed without a
current reachable path and reproducible evidence.

### Phase 3: Interfaces, Grafana integration, configuration, and observability

- [ ] Review CLI parser/help/output/error contracts and destructive-action UX
      across command families.
- [ ] Review TUI registry, route coverage, interaction state, terminal-size,
      scrolling, focus, search, modal, redaction, and fallback behavior.
- [ ] Review the optional browser lane separately from the lean default build.
- [ ] Review classic API and App Platform capability routing, namespaces,
      pagination, history, edition/version constraints, and compatibility.
- [ ] Compare unstable Grafana claims with current official sources and label
      inference, public support, experimental status, and edition constraints.
- [ ] Review configuration precedence, validation, secrets, unsafe defaults,
      logging context, metrics, tracing, health, and diagnostic surfaces.

**Gate:** public-surface findings are checked against command/docs contracts and
Grafana claims cite current authoritative sources.

### Phase 4: Python, contracts, dependencies, build, and delivery

- [ ] Verify which Python modules are maintained product compatibility versus
      repository tooling or historical implementation.
- [ ] Review packaging/install behavior, Python 3.9 compatibility, YAML and
      other runtime dependencies, Rust/Python/schema parity, and shared
      fixtures.
- [ ] Review Rust and Python manifests and lock files for reproducibility,
      unnecessary direct dependencies, supportability, security relevance, and
      version-policy consistency.
- [ ] Review Make targets, GitHub/GitLab workflows, feature matrices, quality
      gates, documentation generation, artifact creation, and release flow.
- [ ] Distinguish environment failures from repository defects and record the
      exact command, result, and interpretation for each failure.

**Gate:** compatibility and delivery conclusions name the maintained boundary
and the validation environment.

### Phase 5: Tests, documentation, and operational readiness

- [ ] Evaluate critical-path and failure-path test quality, isolation,
      determinism, fixtures, contract coverage, live-test boundaries, and CI
      execution rather than relying on test count.
- [ ] Run the smallest relevant existing validation targets first, then broaden
      only where evidence crosses subsystems.
- [ ] Compare README, handbook, command reference, maintainer docs, generated
      outputs, architecture notes, troubleshooting, and release guidance with
      current code and contracts.
- [ ] Assess deployment assumptions, rollback, backup/restore relevance,
      monitoring, alerting, capacity, recovery, auditability, upgrade strategy,
      and operator diagnostics.

**Gate:** every missing-test or documentation item names the exact behavior,
audience, location, and validation method.

### Phase 6: Broad non-destructive validation

Run applicable existing commands without installing missing tools or changing
tracked files. Start narrow, then use the relevant subset of:

- `cd rust && cargo test --quiet`
- `cd rust && cargo fmt --all --check`
- `cd rust && cargo clippy --all-targets -- -D warnings`
- `cd rust && cargo check --features browser`
- `cd python && PYTHONPATH=. poetry run python -m unittest discover -s tests -v`
- `make quality-ai-workflow`
- `make quality-docs-surface`
- `make man-check`
- `make html-check`
- `make test`

For every command record status, meaningful counts, duration when relevant,
and whether failure is a product defect, repository/tooling defect, generated
drift, unavailable dependency, missing live service, or local environment
limitation.

- [ ] Run focused validations needed to confirm candidate findings.
- [ ] Run the applicable broad validation matrix.
- [ ] Check worktree status afterward and stop if any validation unexpectedly
      modified tracked files.

**Gate:** the final assessment clearly separates static evidence, automated
validation, live behavior, and untested assumptions.

### Phase 7: Synthesis and backlog construction

- [ ] Deduplicate candidates by root cause and assign stable finding IDs.
- [ ] Record concrete strengths with the same evidence standard as defects.
- [ ] Classify every finding by severity and confidence without inflating High
      or Critical counts.
- [ ] Build the risk register from current findings and unresolved assumptions.
- [ ] Convert actionable findings into atomic TODO items with objective
      acceptance and validation criteria.
- [ ] Group TODO items into safe implementation phases and give one exact,
      dependency-aware execution order.
- [ ] Record quick wins, justified refactoring opportunities, missing tests,
      documentation gaps, and deferred or rejected recommendations.
- [ ] Self-review the final report for unsupported claims, duplicate findings,
      vague TODOs, inconsistent IDs, and scope violations.

**Gate:** every TODO traces to evidence or a documented risk, can be implemented
and reviewed independently, and has an objective definition of done.

## Required Final Report Structure

The completed report in this file must contain these sections after the spec
and evidence log:

1. Executive Summary
2. Project Overview
3. Repository Map
4. Architecture Summary
5. Strengths
6. Findings by category
7. Quick Wins
8. Major Refactoring Opportunities
9. Missing Tests
10. Documentation Gaps
11. Risk Register
12. Prioritized TODO Backlog
13. Implementation Phases
14. Recommended Execution Order
15. Deferred or Rejected Recommendations
16. Final Recommendation

The executive summary must assign one overall assessment:

- Healthy
- Generally healthy with targeted issues
- Moderate technical debt
- High technical debt
- High operational risk
- Critical remediation required

## Finding Template

```markdown
## FINDING-ID: Short specific title

- Category:
- Severity: Critical | High | Medium | Low | Informational
- Confidence: Confirmed | High confidence | Medium confidence | Requires runtime verification
- Related baseline item: none | path and heading

### Evidence

- `path:symbol-or-key` — observed current behavior
- Validation: `underlying project command` — result

### Description

Explain the current issue without speculation.

### Impact

Explain the practical consequence and affected users or workflows.

### Recommendation

Describe the smallest reasonable improvement.

### Verification

Describe how to confirm the finding and validate a future fix.
```

## Atomic TODO Template

```markdown
## TODO-ID: Short title

- Priority: Critical | High | Medium | Low
- Category:
- Related findings:
- Objective:
- Scope:
- Out of scope:
- Rationale:
- Affected files or modules:
- Proposed implementation:
- Acceptance criteria:
- Validation steps:
- Dependencies:
- Estimated complexity: XS | S | M | L | XL
- Risk: Low | Medium | High
- Breaking-change potential: None | Low | Medium | High
- Suggested owner profile:
- Rollback approach:
```

Acceptance criteria must be measurable. Do not create backlog items named only
“improve quality,” “refactor architecture,” “add tests,” “improve security,” or
“optimize performance.”

## Implementation Roadmap Rules

Use these default phases unless current findings justify a narrower grouping:

1. Phase 0: Verification and baseline
2. Phase 1: Critical correctness and security fixes
3. Phase 2: Reliability and operational safety
4. Phase 3: Test coverage and CI guardrails
5. Phase 4: Maintainability and architecture improvements
6. Phase 5: Performance and scalability
7. Phase 6: Documentation and developer experience

For each phase list goals, included TODO IDs, entry conditions, exit criteria,
main risks, and expected outcome. Prioritize preventing data loss, closing
security risks, correcting behavior, establishing tests and baselines,
improving reliability, reducing debt, addressing measured performance, and
then documentation.

## Acceptance Checks

- [ ] The current commit and environment baseline are recorded.
- [ ] Every indexed primary and maintained secondary project surface is mapped
      or explicitly excluded with rationale.
- [ ] Representative read-only and mutation runtime flows are traced with
      source evidence.
- [ ] All required analysis categories are covered, including categories with
      no current finding.
- [ ] Historical Grafana 13 findings are individually revalidated and not
      blindly carried forward.
- [ ] Relevant existing tests, lint, build, contract, documentation, and
      feature-lane checks are executed or recorded as unavailable.
- [ ] Every meaningful finding has current evidence, severity, confidence,
      impact, minimal recommendation, and verification method.
- [ ] Security findings distinguish reachable vulnerabilities from hardening
      opportunities and runtime-dependent hypotheses.
- [ ] Performance findings distinguish measured problems, likely bottlenecks,
      and profiling candidates.
- [ ] Findings are deduplicated by root cause.
- [ ] Every active recommendation becomes an atomic, objectively verifiable
      TODO or is explicitly deferred/rejected.
- [ ] The risk register, phased roadmap, and exact recommended execution order
      use consistent finding and TODO IDs.
- [ ] The final recommendation identifies the first five TODO items, highest
      unresolved assumptions, and whether incremental remediation is suitable.
- [ ] The final worktree check confirms no source or generated artifacts were
      modified by the analysis.

## Evidence Log

- 2026-07-12: Planning baseline is branch `dev` at `c58f17ac`, ahead of
  `origin/dev` by 15 commits before this spec was written.
- 2026-07-12: CodeGraph is initialized with 1,038 indexed files, 20,876 nodes,
  and 58,856 edges. The indexed implementation is primarily Rust with a
  secondary Python surface and repository YAML/JavaScript tooling.
- 2026-07-12: The active todo index contained only completed Grafana 13 review,
  implementation, and follow-up entries; this specification opens the next
  analysis slice without reopening those completed items.
- 2026-07-12: No source files, generated artifacts, dependencies, commits, or
  remote state were changed while preparing this specification.
- 2026-07-15: The uncommitted planning scope was reviewed at `432287bc` on
  branch `dev`, ahead of `origin/dev` by 16 commits. No source changes were
  present; the review clarified the docs-only commit boundary, aligned the
  Rust formatting command with repository guidance, and promoted the active
  roadmap above completed items in `todo/todo.md`.

## Completion Boundary

Finishing this planning item means the complete analysis report and roadmap
have been written into this file, all acceptance checks are satisfied or have
an explicit evidence limitation, and `todo/todo.md` marks the item complete.

Do not begin implementing any resulting TODO item without separate explicit
approval. The completed analysis must end with:

`Analysis complete. No source files were modified.`
