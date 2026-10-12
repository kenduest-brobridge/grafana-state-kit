# Dashboard dormant-test reconciliation — 2026-10-12

## Accounting

- Baseline: 51 unregistered cases at `eb1e0c93`, after three focused-review
  cases were restored in the previous change.
- Each remaining case still exercises supported behavior; no registered case
  was found to replace its specific assertions. Restore all 51, without restoring
  the old list-render → browse-router inclusion tree.
- Register cases with their production owners. Remove the obsolete
  `dashboard_browse_workflow_rust_tests.rs` router. This removes infrastructure,
  not a test assertion. No runtime behavior is changed for this reconciliation.
- Adapt one interactive-review fixture to the current `diff_model` field;
  strengthen text output, dry-run Enter and pre-TTY zero-request assertions.
- With `tui`: all 51 should run. Without default features: 24 list plus three
  delete plus two history cases should run; the other 22 require TUI types.

## Per-case dispositions

Every row below has disposition **restore unique current behavior**. Module
names refer to registered namespaces below `dashboard::`; function names are
kept intact so Cargo listings can be reconciled against this inventory.

| Case | Registered owner | Retained behavior |
| --- | --- | --- |
| `format_dashboard_summary_line_uses_uid_name_and_folder_details` | `list::tests` | Exact UID/title/folder/org line |
| `format_dashboard_summary_line_appends_sources_when_present` | `list::tests` | Source names in text output |
| `render_dashboard_summary_text_uses_line_format` | `list::tests` | Exact public text result plus empty input |
| `render_dashboard_summary_table_uses_headers_and_defaults` | `list::tests` | Org columns and missing-folder defaults |
| `render_dashboard_summary_table_includes_sources_column_when_present` | `list::tests` | Source column and rendered row |
| `render_dashboard_summary_table_can_omit_header` | `list::tests` | Headerless single-row output |
| `render_dashboard_summary_csv_uses_headers_and_escaping` | `list::tests` | Stable headers, comma and quote escaping |
| `render_dashboard_summary_csv_includes_sources_column_when_present` | `list::tests` | Source names and UIDs in CSV |
| `render_dashboard_summary_json_returns_objects` | `list::tests` | Full JSON field projection and defaults |
| `render_dashboard_summary_json_can_be_rendered_as_yaml` | `list::tests` | YAML projection including org ID |
| `render_dashboard_summary_json_includes_sources_when_present` | `list::tests` | Source arrays in JSON |
| `render_dashboard_summary_json_includes_ownership_and_provenance_when_present` | `list::tests` | Ownership/provenance projection |
| `render_dashboard_summary_table_respects_selected_columns` | `list::tests` | User-selected table columns |
| `render_dashboard_summary_json_respects_selected_columns` | `list::tests` | JSON aliases and selected fields |
| `build_folder_path_joins_parents_and_title` | `list::tests` | Parent hierarchy path |
| `attach_dashboard_folder_paths_with_request_uses_folder_hierarchy` | `list::tests` | Fetched parent path and General fallback |
| `list_dashboards_with_request_returns_dashboard_count` | `list::tests` | Count and no unnecessary source reads |
| `collect_dashboard_source_names_prefers_datasource_names` | `list::tests` | Catalog lookup, built-in exclusion, deduplication |
| `collect_dashboard_source_names_accepts_preserved_raw_dashboard_documents` | `list::tests` | Raw document source lookup |
| `collect_dashboard_ownership_provenance_reports_api_file_and_git_sync_states` | `list::tests` | API/file/Git ownership distinctions |
| `list_dashboards_with_request_json_fetches_dashboards_and_datasources_by_default` | `list::tests` | Default JSON source enrichment reads |
| `list_dashboards_with_request_output_columns_sources_fetches_dashboard_sources` | `list::tests` | Explicit source-column enrichment |
| `list_dashboards_with_request_output_columns_all_expands_full_human_columns` | `list::tests` | All-column human list request path |
| `list_dashboards_with_request_with_org_id_scopes_requests` | `list::tests` | Org-scoped search, folder and source reads |
| `dashboard_delete_validate_args_requires_yes_without_dry_run` | `delete::tests` | Explicit destructive confirmation |
| `dashboard_delete_build_plan_matches_path_subtree` | `delete::tests` | Subtree selection and deepest folder first |
| `dashboard_delete_with_request_deletes_dashboards_then_folders` | `delete::tests` | Exact dashboard-before-folder delete sequence |
| `dashboard_browse_document_builds_tree_with_general_and_nested_folders` | `browse::tree_state_tests` | General/nested folder labels, depths and counts |
| `dashboard_browse_document_filters_to_requested_root_path` | `browse::tree_state_tests` | Root filtering and rebased depth |
| `browser_state_replace_document_preserves_selected_dashboard_uid` | `browse::tree_state_tests` | Preserve focused UID after folder move |
| `dashboard_edit_resolves_destination_folder_uid_from_browser_tree` | `edit::tests` | Folder path resolves destination UID |
| `dashboard_edit_fetch_draft_reads_current_live_title_and_tags` | `edit::tests` | Current draft identity/title/folder/tags |
| `dashboard_edit_apply_posts_updated_title_tags_and_folder_uid` | `edit::tests` | Exact edited POST with overwrite policy |
| `dashboard_edit_dialog_folder_picker_selects_existing_folder_path` | `edit::tests` | Keyboard folder selection and saved path |
| `dashboard_edit_dialog_ctrl_x_closes_dialog` | `edit::tests` | Cancel without saving |
| `dashboard_view_lines_include_recent_versions_when_history_exists` | `history::browse_history_tests` | Limit five and recent-version details |
| `dashboard_view_lines_ignore_missing_versions_endpoint` | `history::browse_history_tests` | 404 retains current version display |
| `dashboard_history_versions_lists_recent_versions_by_uid` | `history::browse_history_tests` | History wrapper, limit, version and creator |
| `dashboard_history_restore_reimports_selected_version_payload` | `history::browse_history_tests` | Old title with current ID/version/folder |
| `dashboard_history_dialog_escape_and_q_close_dialog` | `history::browse_history_tests` | Escape/q cancel |
| `dashboard_history_dialog_restore_review_uses_human_message` | `history::browse_history_tests` | Restore review UID/version/message |
| `dashboard_raw_edit_validation_rejects_overwrite_in_user_payload` | `edit_external::tests::raw_edit_workflow` | User overwrite override rejected |
| `dashboard_raw_edit_review_summarizes_title_tags_and_folder_uid_changes` | `edit_external::tests::raw_edit_workflow` | Change summary includes identity/folder/tags |
| `dashboard_raw_edit_apply_posts_payload_with_overwrite_and_message` | `edit_external::tests::raw_edit_workflow` | Controlled POST metadata injection |
| `interactive_import_loads_dashboard_titles_and_folder_paths` | `import_interactive::tests` | Exported identity/title/folder extraction |
| `interactive_import_state_toggles_and_confirms_selected_files` | `import_interactive::tests` | Space/all/Enter selection batch |
| `interactive_import_grouping_cycles_folder_action_flat` | `import_interactive::tests` | Complete grouping cycle |
| `interactive_import_context_view_scope_and_diff_depth_cycle` | `import_interactive::tests` | Context/scope/depth keyboard state |
| `interactive_import_summary_counts_track_pending_selected_and_reviewed_actions` | `import_interactive::tests` | Pending/selected/reviewed/update counts |
| `interactive_import_dry_run_state_uses_dry_run_status_and_enter_copy` | `import_interactive::tests` | Dry-run status and Enter execution mode |
| `interactive_import_with_use_export_org_falls_through_to_tty_validation` | `import_interactive::tests` | Aggregate org source reaches TTY gate before reads |

## Verification

- Initial list-owner restoration: 24 tests actually passed. Registration was
  subsequently moved from the aggregate test owner to `list::tests`; final
  execution must cover that delivered registration.
- Final registration, execution counts and feature-gate checks are recorded
  in `2026-10-12-development-follow-through.md` after combined verification.
- These are automated state/request tests, not physical terminal interaction
  acceptance. Age or an unregistered filename was never used as deletion proof.
