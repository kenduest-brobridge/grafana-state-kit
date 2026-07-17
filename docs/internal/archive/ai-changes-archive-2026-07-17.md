# ai-changes-archive-2026-07-17

## 2026-05-25 - TUI completion audit
- Summary: Replaced the open-ended TUI follow-up section with a completion audit that maps current evidence to the finished shared review/detail/diff projection work and records why domain-specific input loops remain local.
- Tests: cargo test --quiet; cargo test --quiet user_browse; cargo test --quiet team_browse; cargo test --quiet datasource_browse; cargo test --quiet status_tui; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow; git diff --check
- Impact: The roadmap now has an evidence-backed completion audit instead of stale continue-follow-up items. Public CLI paths, help text, generated docs, Rust runtime behavior, and command contracts are unchanged.
- Rollback/Risk: Low. Documentation-only audit update based on current inventory/search/test evidence.
- Follow-up: Treat future TUI work as new scoped feature work unless a fresh inventory or user report identifies a concrete regression or duplication.
