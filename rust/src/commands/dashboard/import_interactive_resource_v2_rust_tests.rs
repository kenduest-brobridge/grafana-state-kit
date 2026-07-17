use std::path::PathBuf;

use super::import_interactive_resource_v2::{
    decide_interactive_mode, handle_confirmation_choice, InteractiveModeDecision,
    ResourceV2InteractiveAction, ResourceV2InteractiveState, ResourceV2Ownership,
    ResourceV2ReviewAction, ResourceV2ReviewRow, SearchDirection,
};

fn row(name: &str, action: ResourceV2ReviewAction) -> ResourceV2ReviewRow {
    ResourceV2ReviewRow {
        source_path: PathBuf::from(format!("objects/{name}.json")),
        namespace: "team-a".to_string(),
        api_version: "dashboard.grafana.app/v2".to_string(),
        name: name.to_string(),
        folder: Some("observability".to_string()),
        ownership: ResourceV2Ownership::ApiManaged,
        action,
        blocker: None,
        changed_fields: vec!["spec.title".to_string()],
    }
}

#[test]
fn adapter_exposes_the_full_resource_v2_review_identity() {
    let row = ResourceV2ReviewRow {
        ownership: ResourceV2Ownership::GitSyncManaged,
        blocker: Some("managed by repository".to_string()),
        changed_fields: vec!["spec.panels".to_string(), "spec.title".to_string()],
        ..row("latency", ResourceV2ReviewAction::Blocked)
    };

    let state = ResourceV2InteractiveState::new(vec![row]);
    let item = state.focused_item().unwrap();

    assert_eq!(item.source_path, PathBuf::from("objects/latency.json"));
    assert_eq!(item.namespace, "team-a");
    assert_eq!(item.api_version, "dashboard.grafana.app/v2");
    assert_eq!(item.name, "latency");
    assert_eq!(item.folder.as_deref(), Some("observability"));
    assert_eq!(item.ownership.label(), "git-sync-managed");
    assert_eq!(item.action.label(), "blocked");
    assert_eq!(item.blocker.as_deref(), Some("managed by repository"));
    assert_eq!(item.changed_fields, ["spec.panels", "spec.title"]);
}

#[test]
fn search_and_repeat_wrap_over_matching_resource_fields() {
    let mut state = ResourceV2InteractiveState::new(vec![
        row("latency", ResourceV2ReviewAction::Update),
        row("errors", ResourceV2ReviewAction::Create),
        row("saturation", ResourceV2ReviewAction::Update),
    ]);

    assert!(state.search("update"));
    assert_eq!(state.focused_item().unwrap().name, "latency");
    assert!(state.repeat_search(SearchDirection::Forward));
    assert_eq!(state.focused_item().unwrap().name, "saturation");
    assert!(state.repeat_search(SearchDirection::Forward));
    assert_eq!(state.focused_item().unwrap().name, "latency");
    assert!(state.repeat_search(SearchDirection::Backward));
    assert_eq!(state.focused_item().unwrap().name, "saturation");
}

#[test]
fn detail_scroll_is_clamped_when_content_or_viewport_changes() {
    let mut state =
        ResourceV2InteractiveState::new(vec![row("latency", ResourceV2ReviewAction::Update)]);

    state.set_detail_layout(20, 5);
    state.scroll_detail(100);
    assert_eq!(state.detail_scroll(), 15);
    state.set_detail_layout(7, 5);
    assert_eq!(state.detail_scroll(), 2);
    state.scroll_detail(-100);
    assert_eq!(state.detail_scroll(), 0);
    state.set_detail_layout(3, 5);
    assert_eq!(state.detail_scroll(), 0);
}

#[test]
fn footer_hints_change_for_review_and_confirmation() {
    let mut state =
        ResourceV2InteractiveState::new(vec![row("latency", ResourceV2ReviewAction::Update)]);

    let review = state.footer_hints();
    assert!(review.contains("/ search"));
    assert!(review.contains("n/N repeat"));
    assert!(review.contains("Enter confirm"));
    assert!(review.contains("Esc cancel"));

    assert_eq!(
        state.request_confirmation(),
        ResourceV2InteractiveAction::Continue
    );
    assert_eq!(state.footer_hints(), "y confirm  n/Esc cancel");
}

#[test]
fn confirmation_and_cancel_are_distinct_terminal_outcomes() {
    let mut confirmed =
        ResourceV2InteractiveState::new(vec![row("latency", ResourceV2ReviewAction::Update)]);
    confirmed.request_confirmation();
    assert_eq!(
        confirmed.confirm(),
        ResourceV2InteractiveAction::Confirm(vec![PathBuf::from("objects/latency.json")])
    );

    let mut cancelled =
        ResourceV2InteractiveState::new(vec![row("latency", ResourceV2ReviewAction::Update)]);
    assert_eq!(cancelled.cancel(), ResourceV2InteractiveAction::Cancel);
}

#[test]
fn confirmation_choice_returns_a_terminal_cancel_outcome() {
    let mut state =
        ResourceV2InteractiveState::new(vec![row("latency", ResourceV2ReviewAction::Update)]);
    state.request_confirmation();

    assert_eq!(
        handle_confirmation_choice(&mut state, false),
        ResourceV2InteractiveAction::Cancel
    );
}

#[test]
fn dry_run_confirmation_keeps_non_mutating_review_rows() {
    let mut state = ResourceV2InteractiveState::new_for_mode(
        vec![
            row("unchanged", ResourceV2ReviewAction::NoChange),
            row("blocked", ResourceV2ReviewAction::Blocked),
        ],
        true,
    );

    assert_eq!(
        state.request_confirmation(),
        ResourceV2InteractiveAction::Continue
    );
    assert_eq!(
        state.confirm(),
        ResourceV2InteractiveAction::Confirm(vec![
            PathBuf::from("objects/unchanged.json"),
            PathBuf::from("objects/blocked.json"),
        ])
    );
}

#[test]
fn no_tty_uses_an_explicit_non_interactive_fallback_decision() {
    assert_eq!(
        decide_interactive_mode(true, true, true),
        InteractiveModeDecision::Launch
    );
    assert_eq!(
        decide_interactive_mode(true, false, true),
        InteractiveModeDecision::UseNonInteractive {
            reason: "resource-v2 interactive review requires stdin and stdout TTYs".to_string(),
        }
    );
    assert_eq!(
        decide_interactive_mode(false, false, false),
        InteractiveModeDecision::UseNonInteractive {
            reason: "interactive review was not requested".to_string(),
        }
    );
}

#[test]
fn adapter_redacts_credentials_from_review_text() {
    let sensitive = ResourceV2ReviewRow {
        blocker: Some("request failed: Authorization: Bearer abc123 password=hunter2".to_string()),
        changed_fields: vec![
            "metadata.annotations.url=https://grafana.test/x?token=secret&org=1".to_string(),
            "client_secret=super-secret".to_string(),
        ],
        ..row("latency", ResourceV2ReviewAction::Blocked)
    };

    let state = ResourceV2InteractiveState::new(vec![sensitive]);
    let item = state.focused_item().unwrap();
    let detail = item.detail_lines().join("\n");

    assert!(!detail.contains("abc123"));
    assert!(!detail.contains("hunter2"));
    assert!(!detail.contains("secret&"));
    assert!(!detail.contains("super-secret"));
    assert!(detail.contains("Bearer [REDACTED]"));
    assert!(detail.contains("password=[REDACTED]"));
    assert!(detail.contains("token=[REDACTED]"));
    assert!(detail.contains("client_secret=[REDACTED]"));
}

#[test]
fn adapter_redacts_credentials_from_list_and_status_surfaces() {
    let sensitive = ResourceV2ReviewRow {
        namespace: "team?password=hunter2".to_string(),
        name: "latency?token=secret".to_string(),
        ..row("latency", ResourceV2ReviewAction::Update)
    };
    let mut state = ResourceV2InteractiveState::new(vec![sensitive]);

    let line = state.focused_item().unwrap().list_line();
    assert!(!line.contains("hunter2"));
    assert!(!line.contains("secret"));
    assert!(line.contains("password=[REDACTED]"));
    assert!(line.contains("token=[REDACTED]"));

    assert!(state.search("token=secret"));
    let status = state.status_text();
    assert!(!status.contains("secret"));
    assert!(status.contains("token=[REDACTED]"));

    assert!(!state.search("password=another-secret"));
    let status = state.status_text();
    assert!(!status.contains("another-secret"));
    assert!(status.contains("password=[REDACTED]"));
}
