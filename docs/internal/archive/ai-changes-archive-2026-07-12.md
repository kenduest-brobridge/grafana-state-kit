# ai-changes-archive-2026-07-12

## 2026-05-25 - Shared review narrative and impact projection
- Summary: Moved access plan narrative and impact row projection into the shared review contract so mutation review surfaces can reuse action/status/changed-field guidance text.
- Tests: cargo test --quiet review_mutation_action_narrative_and_impact_lines_project_action_guidance; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same Narrative and Why this matters rows while review_contract now owns the generic action narrative and changed-field impact projection. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent projection code into review_contract, removes the now-unused access review action alias, and focused access/review-contract tests cover the old output.
- Follow-up: Continue auditing remaining compatible local artifact browser review/detail projections before declaring the broader TUI design work complete.
