# Compatibility and Screenshot Retirement Review

## Status and scope

- State: complete
- Baseline: `a9fa0bb0` on `codex/grafana12-screenshot-retirement`.
- Review the preceding UID mutation, screenshot removal, release matrix,
  operator documentation, and acceptance claims; fix confirmed defects and commit.
- Preserve Grafana 12.0+ compatibility, stable API boundaries, and unrelated work.
- Do not implement the separate playlist and portable-export roadmap slices here.

## Execution and acceptance

- [x] Inspect changed code, CLI contracts, CI/scripts, and documentation together.
- [x] Reproduce confirmed findings and add behavioral regressions where useful.
- [x] Fix findings and reconcile documentation with actual evidence.
- [x] Run relevant Rust/Python, generated-doc, and workflow checks.
- [x] Review the final diff, record verification limits, and commit scoped changes.

## Findings and evidence

- P1: Rust live smoke ignored Cargo's chosen target directory and executed a
  fixed repository binary. Use the executable from Cargo's artifact message.
- P2: Rust's cleanup function was never registered; passing and failed smoke
  runs leaked containers and temporary files. Register exit/signal cleanup and
  delete containers by the ID Docker records for this run. Independent review
  also found the successful-start ownership flag missed containers created
  before a startup failure; the ID-file fix covers that case in both scripts.
- P2: Both org-delete exceptions swallowed non-500 failures and malformed JSON
  based on a substring match across all Grafana 13 versions. Require the pinned
  13.2.0 patch, curl HTTP-error status, HTTP 500, and one valid JSON response.
- P2: The org-delete exception also skipped independent org export/diff/update
  coverage. Keep those checks running and skip only deletion-dependent steps.
- P2: The prior Python quality claim missed three Black formatting failures.
  Reproduced with the full Python quality script and formatted the affected tests.
- Documentation: explain the matrix exception and artifact/cleanup behavior;
  correct the parent tracker that claimed a 13.1.3 run although evidence used 13.2.0.

## Verification

- The new shell behavior regressions reproduced artifact-selection, missing
  cleanup, and org-delete exception failures before the fixes; all four test
  methods now pass, including success/failure/startup-error cleanup and status/body subcases
  against both smoke scripts.
- `make quality-rust`: 1,957 library tests and 38 integration tests passed;
  one existing test ignored. Rustfmt and Clippy with warnings denied passed.
- `make quality-rust-feature-matrix-full`: default and no-default-features
  Cargo checks passed with build outputs outside the repository.
- `cd python && poetry run bash ../scripts/check-python-quality.sh`: 1,213
  tests run, two optional-httpx tests skipped because httpx is not installed,
  no failures; Ruff, mypy (109 sources), and Black
  (172 files) passed. The earlier generated-HTML comparison ran while docs
  were being regenerated; the final full run used stable regenerated files.
- `make man html man-check html-check quality-ai-workflow` and
  `make quality-docs-surface`: passed, including generated outputs and 204
  documentation surface checks.
- `CARGO_TARGET_DIR=/tmp/grafana-utils-screenshot-retirement-cargo make
  test-grafana-compatibility-matrix`: passed all five local Docker lanes:
  Rust 12.0.10, 12.4.9, 13.2.0; Python datasource 12.0.10, 13.2.0.
  Each reported the matching health version. The corrected scripts removed
  their containers afterward; the unrelated renderer container was preserved.
- `bash -n` for both smoke scripts and `git diff --check`: passed.
- Independent final review found the created-but-not-started container cleanup
  gap. After the ID-file repair and its failing-then-passing regression,
  re-review found no remaining important issues; the five live lanes and full
  Python quality/docs checks passed again with the final files.

## Verification boundaries

- These are local container compatibility results, not native or production
  acceptance, and not a remote CI run.
- Grafana 13.2.0 still returns the upstream org-delete failure. Only that
  exact patch's curl HTTP-error/HTTP 500/single expected JSON response permits
  deletion-dependent fixture skips. Org creation/export/diff/existing-org
  import and ordinary datasource/dashboard workflows remain covered. The
  production CLI does not suppress the server error.
- Shell regression tests require Bash and jq; Python-only environments without
  them skip these tests. Both tools were available in this verification run.
- No beta API scope, playlist work, portable-export work, merge, or push is
  included in this review.
