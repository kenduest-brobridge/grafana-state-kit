# ai-changes-archive-2026-09-10

## 2026-05-25 - TUI empty selection key handling
- Summary: Kept datasource/access browse edit and delete keys inside the TUI when no row is selected, surfacing status messages instead of propagating selected-row errors.
- Tests: cargo test --quiet edit_key_on_empty_document_stays_in_browser; cargo test --quiet delete_key_on_empty_document_stays_in_browser; cargo test --quiet empty_user_browse_edit_and_delete_keys_stay_in_browser; cargo test --quiet empty_team_browse_edit_and_delete_keys_stay_in_browser; cargo test --quiet datasource_browse_input; cargo test --quiet user_browse_input; cargo test --quiet team_browse_input; cargo test --quiet datasource_browse; cargo test --quiet access::user_browse; cargo test --quiet access::team_browse; cargo test --quiet browse; cargo fmt --check; git diff --check
- Impact: Datasource browse, access user browse, and access team browse now treat empty edit/delete key presses as in-browser no-selection states.
- Rollback/Risk: Low; changes are limited to empty-selection branches and regression tests cover the key paths.
- Follow-up: none
