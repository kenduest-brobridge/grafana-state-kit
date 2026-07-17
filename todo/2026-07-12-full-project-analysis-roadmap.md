# Full Project Analysis and Improvement Roadmap

## Status

- State: complete
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

- [x] Record branch, commit, worktree status, repository roots, and toolchain
      versions without changing the environment.
- [x] Capture indexed repository structure and primary/secondary language
      boundaries while excluding `rust/target`.
- [x] Map build, test, documentation, packaging, CI, and release entry points.
- [x] Reconcile historical baseline items with current commits and regression
      tests; label each resolved, still open, regressed, or runtime-dependent.
- [x] Record unavailable tools, credentials, services, or platforms as evidence
      limitations rather than project defects.

**Gate:** scope, baseline commit, validation lanes, exclusions, and evidence
limitations are explicit.

### Phase 1: Architecture and execution flows

- [x] Map Rust crate entry points, command registration and dispatch, shared
      infrastructure, Grafana clients, domain modules, renderers, TUI, and
      optional browser boundaries.
- [x] Trace representative read-only and mutation flows from operator input to
      transport, response handling, output, and error reporting.
- [x] Map Python compatibility, packaging, scripts, contracts, fixtures, and
      generated documentation relationships.
- [x] Document dependency direction, integration boundaries, trust boundaries,
      and failure boundaries; add one Mermaid diagram only if it materially
      clarifies these relationships.
- [x] Identify unclear ownership, mixed responsibilities, high-coupling areas,
      and architecture intent that differs from implementation.

**Gate:** every major component has a responsibility, entry point, dependency,
and evidence reference.

### Phase 2: Correctness, security, reliability, and performance

- [x] Review all public command families and shared infrastructure for the
      required correctness and maintainability concerns.
- [x] Review authentication, authorization, secret handling, destructive
      operations, filesystem/network inputs, logging, TLS, and CI/release trust
      boundaries.
- [x] Review pagination, timeouts, retry/fallback behavior, partial failures,
      idempotency, concurrency, recovery, and failure visibility.
- [x] Review likely hot paths and resource bounds without proposing premature
      optimization.
- [x] Reproduce suspected defects with the smallest existing test, focused
      command, fixture, or source-level execution trace practical.

**Gate:** no candidate bug or vulnerability is called confirmed without a
current reachable path and reproducible evidence.

### Phase 3: Interfaces, Grafana integration, configuration, and observability

- [x] Review CLI parser/help/output/error contracts and destructive-action UX
      across command families.
- [x] Review TUI registry, route coverage, interaction state, terminal-size,
      scrolling, focus, search, modal, redaction, and fallback behavior.
- [x] Review the optional browser lane separately from the lean default build.
- [x] Review classic API and App Platform capability routing, namespaces,
      pagination, history, edition/version constraints, and compatibility.
- [x] Compare unstable Grafana claims with current official sources and label
      inference, public support, experimental status, and edition constraints.
- [x] Review configuration precedence, validation, secrets, unsafe defaults,
      logging context, metrics, tracing, health, and diagnostic surfaces.

**Gate:** public-surface findings are checked against command/docs contracts and
Grafana claims cite current authoritative sources.

### Phase 4: Python, contracts, dependencies, build, and delivery

- [x] Verify which Python modules are maintained product compatibility versus
      repository tooling or historical implementation.
- [x] Review packaging/install behavior, Python 3.9 compatibility, YAML and
      other runtime dependencies, Rust/Python/schema parity, and shared
      fixtures.
- [x] Review Rust and Python manifests and lock files for reproducibility,
      unnecessary direct dependencies, supportability, security relevance, and
      version-policy consistency.
- [x] Review Make targets, GitHub/GitLab workflows, feature matrices, quality
      gates, documentation generation, artifact creation, and release flow.
- [x] Distinguish environment failures from repository defects and record the
      exact command, result, and interpretation for each failure.

**Gate:** compatibility and delivery conclusions name the maintained boundary
and the validation environment.

### Phase 5: Tests, documentation, and operational readiness

- [x] Evaluate critical-path and failure-path test quality, isolation,
      determinism, fixtures, contract coverage, live-test boundaries, and CI
      execution rather than relying on test count.
- [x] Run the smallest relevant existing validation targets first, then broaden
      only where evidence crosses subsystems.
- [x] Compare README, handbook, command reference, maintainer docs, generated
      outputs, architecture notes, troubleshooting, and release guidance with
      current code and contracts.
- [x] Assess deployment assumptions, rollback, backup/restore relevance,
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

- [x] Run focused validations needed to confirm candidate findings.
- [x] Run the applicable broad validation matrix.
- [x] Check worktree status afterward and stop if any validation unexpectedly
      modified tracked files.

**Gate:** the final assessment clearly separates static evidence, automated
validation, live behavior, and untested assumptions.

### Phase 7: Synthesis and backlog construction

- [x] Deduplicate candidates by root cause and assign stable finding IDs.
- [x] Record concrete strengths with the same evidence standard as defects.
- [x] Classify every finding by severity and confidence without inflating High
      or Critical counts.
- [x] Build the risk register from current findings and unresolved assumptions.
- [x] Convert actionable findings into atomic TODO items with objective
      acceptance and validation criteria.
- [x] Group TODO items into safe implementation phases and give one exact,
      dependency-aware execution order.
- [x] Record quick wins, justified refactoring opportunities, missing tests,
      documentation gaps, and deferred or rejected recommendations.
- [x] Self-review the final report for unsupported claims, duplicate findings,
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

- [x] The current commit and environment baseline are recorded.
- [x] Every indexed primary and maintained secondary project surface is mapped
      or explicitly excluded with rationale.
- [x] Representative read-only and mutation runtime flows are traced with
      source evidence.
- [x] All required analysis categories are covered, including categories with
      no current finding.
- [x] Historical Grafana 13 findings are individually revalidated and not
      blindly carried forward.
- [x] Relevant existing tests, lint, build, contract, documentation, and
      feature-lane checks are executed or recorded as unavailable.
- [x] Every meaningful finding has current evidence, severity, confidence,
      impact, minimal recommendation, and verification method.
- [x] Security findings distinguish reachable vulnerabilities from hardening
      opportunities and runtime-dependent hypotheses.
- [x] Performance findings distinguish measured problems, likely bottlenecks,
      and profiling candidates.
- [x] Findings are deduplicated by root cause.
- [x] Every active recommendation becomes an atomic, objectively verifiable
      TODO or is explicitly deferred/rejected.
- [x] The risk register, phased roadmap, and exact recommended execution order
      use consistent finding and TODO IDs.
- [x] The final recommendation identifies the first five TODO items, highest
      unresolved assumptions, and whether incremental remediation is suitable.
- [x] The final worktree check confirms no source or generated artifacts were
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
- 2026-07-17: Final analysis ran on branch `dev` at `b95ad250`, 17 commits
  ahead of `origin/dev`. CodeGraph reported 1,038 files, 20,877 nodes, and
  58,834 edges: Rust 815, Python 219, YAML 3, and JavaScript 1.
- 2026-07-17: `cd rust && cargo test --quiet` completed with 1,887 passing,
  one failing, and one ignored test. The failure was
  `project_status_live_all_orgs_fans_out_across_visible_orgs`, whose suite run
  observed an unscoped `/api/org/users` request where the assertion expected
  scoped requests. The focused rerun passed: one passed, 1,925 filtered out.
- 2026-07-17: `cd rust && cargo fmt --all --check` passed. `cd rust && cargo
  check --features browser` passed without compiling additional crates.
- 2026-07-17: `cd rust && cargo clippy --all-targets -- -D warnings` failed
  with seven diagnostics: one `question_mark`, one `some_filter`, and five
  redundant formatting references.
- 2026-07-17: The previously supplied 392-test Python run had 33 import errors
  and one `httpx`-dependent skip. `cd python && poetry run python -c 'import
  yaml'` reproduced `ModuleNotFoundError`; `python/pyproject.toml` correctly
  declares `PyYAML>=6,<7`, so this is a stale/incomplete local Poetry
  environment, not a missing manifest dependency.
- 2026-07-17: `make quality-ai-workflow`, `make quality-docs-surface`, `make
  man-check`, and `make html-check` passed. This supersedes the earlier
  transient docs-surface failure involving access browse documentation.
- 2026-07-17: Current Grafana documentation confirms the standardized
  `/apis/<GROUP>/<VERSION>/namespaces/<NAMESPACE>/<RESOURCE>` shape,
  `resourceVersion` optimistic concurrency semantics, and dashboard v1/v2
  resource discovery in Grafana CLI workflows. Relevant sources are
  <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/apis/>
  and <https://grafana.com/docs/grafana/latest/as-code/observability-as-code/grafana-cli/grafanactl/grafanacli-workflows/>.
- 2026-07-17: Final status inspection observed concurrent, out-of-scope
  dashboard resource-v2 source and fixture changes associated with the
  separately tracked 2026-07-17 import item. This analysis did not edit,
  stage, revert, or evaluate those changes.

## Executive Summary

**Overall assessment: Generally healthy with targeted issues.**

The Rust-first product has a coherent unified CLI, explicit domain ownership,
safe mutation review patterns, extensive automated coverage, generated-doc
contracts, a lean default build, and a separately verified browser feature.
The completed Grafana 13 work remains substantially intact. No reachable
Critical security, data-loss, or performance defect was confirmed.

Release confidence is nevertheless reduced by four concrete delivery issues:
the Rust lint gate currently fails, one network-style Rust test is
suite-order-sensitive, the maintained Python suite is absent from GitHub CI
and could not run in the stale local Poetry environment, and the GitLab
pipeline references paths and binaries that no longer exist. Incremental
remediation is appropriate; a rewrite or service split is not justified.

## Project Overview

`grafana-util` is an operator CLI for inventory, export, inspection, diff,
preview, safe apply, sync, snapshot, status, and interactive Grafana workflows.
Its primary deliverable is the Rust binary defined by
`rust/src/bin/grafana-util.rs`; Python under `python/grafana_utils/` is the
first-generation compatibility and repository-tooling surface. External trust
boundaries are Grafana HTTP APIs, local profile and secret storage, staged
filesystem artifacts, terminals, optional browser automation, CI runners, and
release distribution.

## Repository Map

| Surface | Ownership and role | Primary evidence |
|---|---|---|
| `rust/src/bin`, `rust/src/cli` | binary entry, parsing, help, dispatch | `grafana-util.rs:main`, `cli/dispatch.rs` |
| `rust/src/commands` | dashboard, alert, access, datasource, sync, snapshot, status, config, resource workflows | `rust/src/lib.rs` module map |
| `rust/src/common`, `rust/src/grafana` | auth, profiles, HTTP transport, shared TUI/browser/output and Grafana clients | `common/auth.rs`, `grafana/http.rs`, `grafana/api/mod.rs` |
| `python/grafana_utils`, `python/tests` | compatibility CLI, packaging, doc/tool support, secondary tests | `python/pyproject.toml`, `unified_cli.py` |
| `tests/fixtures` | cross-language and schema contracts | fixture README and JSON cases |
| `scripts/contracts`, `scripts/*.py` | command/docs/TUI contracts and generated-doc quality gates | `command-surface.json`, `docs-entrypoints.json` |
| `docs`, `Makefile` | operator/maintainer sources, generated man/HTML, canonical tasks | `Makefile` targets |
| `.github/workflows`, `.gitlab-ci.yml` | CI, live smoke, packaging, release | workflow definitions |

`rust/target` was excluded. CodeGraph's 1,038-file view is the structural
inventory used for this assessment.

## Architecture Summary

The normal flow is operator arguments -> Clap parsing/help -> shared dispatch
-> domain command normalization -> profile/auth resolution -> `JsonHttpClient`
and resource clients -> Grafana classic or App Platform API -> domain model ->
text/table/JSON/TUI rendering. Mutations add plan/diff/review/approval gates;
read-only browse and status commands reuse the same transport and scoped org
headers. TUI state stays in domain-owned modules with common shell helpers.
The optional browser dependencies are isolated behind the `browser` feature.

Python repeats portions of the product surface for compatibility, while
shared fixtures, command contracts, and documentation generators constrain
cross-language drift. This duplication is manageable only if CI exercises the
secondary lane. Persistence is file-based; network and filesystem failures are
returned as command errors rather than hidden service state.

## Strengths

- `rust/src/lib.rs` documents and enforces recognizable domain boundaries.
- The unified CLI taxonomy and bilingual docs are protected by 206-page docs
  surface checks plus generated man and HTML drift checks.
- Mutation commands use explicit preview/approval and safe-review contracts;
  current review found no confirmed secret disclosure or authorization bypass.
- Rust has broad focused and full-suite coverage, shared contract fixtures,
  a live Grafana smoke lane, and a tag-gated release installer smoke.
- Default and browser feature boundaries are explicit; the browser check passes.
- Historical Grafana 13 fixes for PyYAML declaration, datasource fallback, TUI
  registry precision, resource history, resource-only export, namespace
  selection, and symlink cycles remain represented by current manifests,
  source, regression tests, or passing contract/docs gates. Dashboard v2 import
  remains a separate active item and is not duplicated below.

## Findings by Category

### CICD-001: Rust lint gate is red

- Category: Build, CI/CD, packaging, and release
- Severity: Medium
- Confidence: Confirmed
- Related baseline item: none

#### Evidence

- `rust/src/commands/access/service_account_workflows_sync.rs:import workflow` — one `question_mark` diagnostic.
- `rust/src/commands/dashboard/browse/state.rs:selection key` — one `some_filter` diagnostic.
- `rust/src/commands/status/overview/section_rows.rs:status rows` — five redundant formatting-reference diagnostics.
- Validation: `cd rust && cargo clippy --all-targets -- -D warnings` — seven errors.

#### Description and impact

Behavioral tests may pass, but the canonical Rust quality gate and GitHub
`rust-quality` job cannot pass. Tag release jobs depend on that job.

#### Recommendation and verification

Apply the semantics-preserving Clippy suggestions, retain focused tests for the
touched paths, and rerun formatting, focused tests, Clippy, and full Rust tests.

### TST-001: All-orgs live-status test is suite-order-sensitive

- Category: Testing and functional correctness
- Severity: Medium
- Confidence: High confidence
- Related baseline item: none

#### Evidence

- `rust/src/commands/status/overview/live_tests.rs:project_status_live_all_orgs_fans_out_across_visible_orgs` — expects org-scoped access requests.
- Validation: full Rust suite — 1,887 pass, one fail, one ignored; focused rerun — pass.

#### Description and impact

The isolated pass rules out a deterministic endpoint-contract failure and
points to test-server teardown, shared state, parallel scheduling, or request
capture isolation. A flaky required suite weakens CI signal and can mask a real
org-scoping regression.

#### Recommendation and verification

Reproduce under repeated parallel and single-threaded runs, identify the shared
state, and make the test deterministic without weakening scoped assertions.

### TST-002: Maintained Python tests lack a dependable CI lane

- Category: Testing, dependencies, and compatibility
- Severity: High
- Confidence: Confirmed
- Related baseline item: `todo/2026-07-10-grafana13-all-improvements.md` PyYAML packaging

#### Evidence

- `.github/workflows/ci.yml` — Rust quality/live/release jobs, no Python test job.
- `.gitlab-ci.yml:python-static-analysis` — explicitly omits unit tests.
- `python/pyproject.toml:project.dependencies` — declares PyYAML.
- Validation: Python suite — 392 discovered, 33 import errors, one skip in the
  existing environment; direct `import yaml` fails.

#### Description and impact

The manifest fix is present, but neither primary CI nor the stale local Poetry
environment proves the secondary implementation. Compatibility regressions can
merge undetected, and local failures are noisy rather than actionable.

#### Recommendation and verification

Add an installed Poetry or wheel-based Python test job using the canonical
manifest and run all tests with an explicit, documented optional-`httpx` skip.

### CICD-002: GitLab pipeline no longer matches repository layout

- Category: Build, CI/CD, packaging, and release
- Severity: High
- Confidence: Confirmed
- Related baseline item: none

#### Evidence

- `.gitlab-ci.yml:version-policy` reads root `pyproject.toml`, which is absent;
  the manifest is `python/pyproject.toml`.
- `.gitlab-ci.yml:python-static-analysis` targets root `grafana_utils`, which is absent.
- `.gitlab-ci.yml:rust-package-*` publishes `grafana-access-utils`, while the
  tracked Rust binary entry is only `rust/src/bin/grafana-util.rs`.

#### Description and impact

Any applicable GitLab branch or tag pipeline fails before providing useful
quality evidence or produces an incomplete artifact set. The large explanatory
header describes a design that is no longer true.

#### Recommendation and verification

Either repair GitLab jobs to call canonical Make targets and current paths, or
remove the pipeline if GitLab is no longer supported. Validate the resulting
configuration in GitLab CI lint and one non-release pipeline.

### Categories with no confirmed active defect

No current Critical/High finding was confirmed for security, secrets,
authorization, performance, configuration precedence, observability, API
pagination/history semantics, browser compilation, generated documentation,
or runtime operations. These conclusions are static/test-backed; no live
authenticated Grafana instance, destructive mutation, load test, package
publication, rollback, or disaster-recovery exercise was performed.

## Quick Wins

1. Fix the seven Clippy diagnostics (`TODO-001`).
2. Add a Poetry dependency preflight message to the Python test target as part
   of `TODO-003`.
3. Replace stale GitLab paths and binary names or formally retire the file
   (`TODO-004`).

## Major Refactoring Opportunities

The only justified refactoring opportunity is CI convergence: GitHub, GitLab,
and local automation should invoke the same Make targets rather than replicate
path, dependency, binary, and version logic. No application rewrite, service
split, async transport conversion, or Python removal is currently justified.

## Missing Tests

- Repeated parallel and `--test-threads=1` coverage for the all-org live-status
  request-capture path.
- A required Python CI test lane from a clean installed environment.
- A GitLab pipeline smoke validating current manifest paths and artifact names.
- Live dashboard resource-v2 import acceptance remains owned by the separate
  2026-07-17 plan.

## Documentation Gaps

- `.gitlab-ci.yml` comments describe obsolete repository paths, artifacts, and
  an intentionally test-free quality model.
- The operator documentation is otherwise internally consistent under the
  current command-surface, bilingual, man, and HTML checks.
- Operational rollback/backup evidence remains procedural rather than proven
  by this analysis because no live environment was authorized.

## Risk Register

| Risk | Likelihood | Impact | Evidence | Mitigation |
|---|---|---|---|---|
| Required Rust quality/release job blocked | High | Medium | CICD-001 | TODO-001 |
| Org-scope regression hidden by flaky test | Medium | High | TST-001 | TODO-002 |
| Python compatibility regression merges unseen | High | High | TST-002 | TODO-003 |
| GitLab pipeline unusable or misleading | High | High | CICD-002 | TODO-004 |
| Live App Platform behavior differs by version/edition | Medium | Medium | no live service | separate v2 plan/TODO-005 |
| Concurrent v2 work changes the baseline | Medium | Medium | final worktree status | rebase/revalidate before implementation |

## Prioritized TODO Backlog

### TODO-001: Restore the Rust lint gate

- Priority: High
- Category: CI/CD and maintainability
- Related findings: CICD-001
- Objective: make canonical Rust lint green without behavior changes.
- Scope: the three files and seven diagnostics named in CICD-001.
- Out of scope: unrelated refactors or warning suppression.
- Rationale: unblocks required quality and release jobs.
- Affected files or modules: access service-account sync, dashboard browse state, status overview rows.
- Proposed implementation: apply idiomatic `?`, condition-first option construction, and owned formatting arguments.
- Acceptance criteria: zero Clippy warnings and unchanged focused-test behavior.
- Validation steps: `cargo fmt --all --check`; focused tests; `cargo clippy --all-targets -- -D warnings`; `cargo test --quiet`.
- Dependencies: none.
- Estimated complexity: XS
- Risk: Low
- Breaking-change potential: None
- Suggested owner profile: Rust maintainer
- Rollback approach: revert the isolated lint commit.

### TODO-002: Make the all-org live-status regression deterministic

- Priority: High
- Category: Correctness and testing
- Related findings: TST-001
- Objective: remove suite-order sensitivity while preserving org-scope assertions.
- Scope: test server, request capture, teardown, and shared-state helpers for live status.
- Out of scope: changing production org-scoping semantics without reproduced evidence.
- Rationale: restore trustworthy full-suite signal.
- Affected files or modules: `rust/src/commands/status/overview/live_tests.rs` and shared live test helpers.
- Proposed implementation: reproduce repeatedly, isolate mutable state/ports, and guarantee server shutdown before assertion completion.
- Acceptance criteria: 100 repeated focused parallel runs, 20 full-suite runs, and a single-threaded run pass.
- Validation steps: repeated filtered `cargo test`, `cargo test -- --test-threads=1`, then `cargo test --quiet`.
- Dependencies: TODO-001 only if CI requires lint first.
- Estimated complexity: S
- Risk: Medium
- Breaking-change potential: None
- Suggested owner profile: Rust test/runtime maintainer
- Rollback approach: revert test-harness-only changes.

### TODO-003: Add a clean Python compatibility test lane

- Priority: High
- Category: Testing and dependencies
- Related findings: TST-002
- Objective: continuously prove the maintained Python package from declared dependencies.
- Scope: GitHub CI, canonical Python test target, optional-httpx skip reporting, installed package smoke.
- Out of scope: expanding Python feature parity.
- Rationale: 392 tests currently have no dependable required lane.
- Affected files or modules: `.github/workflows/ci.yml`, `Makefile`, `python/pyproject.toml`, packaging tests.
- Proposed implementation: create a clean Poetry environment, install with dev dependencies, run unit tests, build wheel/sdist, and smoke-import the wheel.
- Acceptance criteria: all non-optional Python tests pass; the one optional-httpx skip is explicit; `yaml` imports from the installed artifact.
- Validation steps: `make poetry-install`; `make poetry-test`; `make build-python`; isolated wheel import smoke.
- Dependencies: none.
- Estimated complexity: M
- Risk: Low
- Breaking-change potential: None
- Suggested owner profile: Python packaging/CI maintainer
- Rollback approach: revert the independent CI job while retaining local targets.

### TODO-004: Repair or retire the GitLab pipeline

- Priority: High
- Category: CI/CD and release
- Related findings: CICD-002
- Objective: ensure every retained GitLab job uses current paths, targets, and artifacts.
- Scope: `.gitlab-ci.yml` and any narrowly required shared Make targets.
- Out of scope: changing GitHub release policy.
- Rationale: current jobs deterministically reference absent inputs and output.
- Affected files or modules: `.gitlab-ci.yml`, version tooling, build targets.
- Proposed implementation: choose and document support status; if retained, call canonical Make targets and publish only actual artifacts.
- Acceptance criteria: GitLab CI lint passes; a branch pipeline runs tests and packages; a dry-run/tag pipeline resolves the correct version and artifact names.
- Validation steps: GitLab CI lint plus one branch pipeline and release-candidate pipeline.
- Dependencies: TODO-001 and TODO-003 when retained jobs invoke those gates.
- Estimated complexity: M
- Risk: Medium
- Breaking-change potential: Low
- Suggested owner profile: release/CI maintainer
- Rollback approach: revert pipeline changes; do not publish artifacts from a failed validation.

### TODO-005: Revalidate live Grafana compatibility before release

- Priority: Medium
- Category: API and operations
- Related findings: runtime limitation; separate dashboard v2 import plan
- Objective: close the highest remaining runtime assumption on supported Grafana editions and versions.
- Scope: non-destructive classic/API discovery, namespace, v1/v2 history, and approved v2 import dry-run/live acceptance owned by its plan.
- Out of scope: unapproved destructive tests or duplicating the v2 implementation tracker.
- Rationale: static and mock coverage cannot prove edition-specific live behavior.
- Affected files or modules: live smoke scripts, Grafana API clients, release evidence.
- Proposed implementation: run the documented live matrix against supported OSS/Cloud targets and retain sanitized evidence.
- Acceptance criteria: classic compatibility and `/apis` capability routes pass; namespace and optimistic-concurrency behavior are demonstrated; blockers are edition-labeled.
- Validation steps: project live smoke targets plus the v2 plan's acceptance commands.
- Dependencies: separate v2 implementation item; TODO-001 through TODO-004 for release gating.
- Estimated complexity: M
- Risk: Medium
- Breaking-change potential: None
- Suggested owner profile: Grafana integration/release maintainer
- Rollback approach: use dry-run first and delete only test-owned resources through approved cleanup.

## Implementation Phases

### Phase 0: Verification and baseline

- Included: TODO-001 lint reproduction and TODO-002 deterministic reproduction.
- Entry: v2 work is isolated and current branch state recorded.
- Exit: exact failures are reproducible; no production behavior changes are mixed in.
- Main risk: concurrent work invalidates line-level assumptions.
- Expected outcome: stable remediation baseline.

### Phase 1: Critical correctness and security fixes

- Included: none; no Critical finding was confirmed.
- Entry/exit: reopen only with reachable evidence and a regression test.
- Main risk: severity inflation without runtime proof.
- Expected outcome: focus remains on confirmed work.

### Phase 2: Reliability and operational safety

- Included: TODO-002.
- Entry: failure mechanism identified.
- Exit: repeated parallel, serial, and full-suite runs pass.
- Main risk: hiding the failure by weakening assertions.
- Expected outcome: trustworthy org-scoping coverage.

### Phase 3: Test coverage and CI guardrails

- Included: TODO-001, TODO-003, TODO-004.
- Entry: focused behavior remains green.
- Exit: Rust lint, Rust tests, Python tests/package smoke, and retained CI systems pass.
- Main risk: divergent CI-specific logic.
- Expected outcome: one dependable cross-language delivery gate.

### Phase 4: Maintainability and architecture improvements

- Included: CI convergence portion of TODO-004.
- Entry: both CI support decisions are explicit.
- Exit: workflows call shared project targets and duplicate logic is minimized.
- Main risk: accidental release-policy changes.
- Expected outcome: lower automation drift.

### Phase 5: Performance and scalability

- Included: none; no measured performance defect was found.
- Entry: profiling or capacity evidence is available.
- Exit: any future change has a before/after measurement.
- Main risk: premature optimization.
- Expected outcome: no speculative churn.

### Phase 6: Documentation and developer experience

- Included: GitLab support/comments in TODO-004 and release evidence in TODO-005.
- Entry: final automation behavior is known.
- Exit: maintainer and operator instructions describe only supported commands and lanes.
- Main risk: documenting an unverified pipeline.
- Expected outcome: accurate operational guidance.

## Recommended Execution Order

1. TODO-001 — restore the Rust quality gate.
2. TODO-002 — stabilize the only observed Rust suite failure.
3. TODO-003 — establish clean Python compatibility and packaging evidence.
4. TODO-004 — repair or retire GitLab using the now-green shared gates.
5. TODO-005 — run the supported live Grafana release matrix after the separate
   dashboard v2 work is ready.

## Deferred or Rejected Recommendations

- Reject a Rust/Python rewrite or service split: no evidence shows incremental
  remediation is impractical.
- Defer async HTTP conversion, caching, and parallelism changes until profiling
  identifies a bounded bottleneck.
- Defer Python removal: it is explicitly maintained as compatibility/tooling
  surface and still has substantial tests.
- Do not reopen resolved Grafana 13 items solely because their historical plans
  mention them. The dashboard v2 import gap is already tracked independently.
- Defer destructive live, publication, rollback, backup/restore, and load tests
  until credentials and an approved disposable environment are available.

## Final Recommendation

Proceed with incremental remediation in the exact order TODO-001 through
TODO-005. The first four items restore deterministic, cross-language delivery
evidence; the fifth closes edition/version runtime assumptions. The highest
unresolved assumptions are live Grafana behavior across OSS/Enterprise/Cloud,
the cause of the suite-only org-scope failure, and whether GitLab remains an
official delivery channel. None requires an architectural rewrite.

## Completion Boundary

Finishing this planning item means the complete analysis report and roadmap
have been written into this file, all acceptance checks are satisfied or have
an explicit evidence limitation, and `todo/todo.md` marks the item complete.

Do not begin implementing any resulting TODO item without separate explicit
approval. The completed analysis must end with:

Analysis complete. No source files were modified.
