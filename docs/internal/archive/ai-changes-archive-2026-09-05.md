# ai-changes-archive-2026-09-05

## 2026-05-25 - Status overview starts on items
- Summary: Changed status overview interactive mode to start with the Items pane focused so Up/Down moves rows immediately after launch instead of requiring Tab first.
- Tests: cargo test --quiet overview_tui_starts_on_items_so_arrow_keys_move_rows_immediately; cargo test --quiet project_home_is_available_and_hands_off_to_first_blocked_section; cargo test --quiet interactive_render_starts_on_project_home_surface; cargo test --quiet status_overview; cargo test --quiet status_tui; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; git diff --check
- Impact: Operators entering status overview interactive mode can navigate the item list with arrow keys immediately. Project Home remains available via h and its handoff behavior is preserved. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. The change only adjusts initial TUI focus and focused tests cover immediate row movement plus Project Home access.
- Follow-up: If operators prefer a visual home summary on launch, consider a visible non-focused home panel while keeping keyboard focus on Items.
