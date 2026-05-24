#![cfg(test)]
use super::*;
use crate::dashboard::delete_support::DeletePlan;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn empty_document() -> super::super::browse_support::DashboardBrowseDocument {
    super::super::browse_support::DashboardBrowseDocument {
        summary: super::super::browse_support::DashboardBrowseSummary {
            root_path: None,
            dashboard_count: 0,
            folder_count: 0,
            org_count: 1,
            scope_label: "current-org".to_string(),
        },
        nodes: Vec::new(),
    }
}

fn dashboard_node(uid: &str, title: &str) -> super::super::browse_support::DashboardBrowseNode {
    super::super::browse_support::DashboardBrowseNode {
        kind: super::super::browse_support::DashboardBrowseNodeKind::Dashboard,
        title: title.to_string(),
        path: "Platform".to_string(),
        uid: Some(uid.to_string()),
        depth: 1,
        meta: format!("uid={uid}"),
        details: Vec::new(),
        url: None,
        org_name: "Acme".to_string(),
        org_id: "42".to_string(),
        child_count: 0,
    }
}

#[test]
fn tree_rows_render_org_header_and_dashboard_metadata() {
    let nodes = vec![
        super::super::browse_support::DashboardBrowseNode {
            kind: super::super::browse_support::DashboardBrowseNodeKind::Org,
            title: "Acme".to_string(),
            path: "Acme".to_string(),
            uid: None,
            depth: 0,
            meta: "2 folder(s) | 1 dashboard(s)".to_string(),
            details: Vec::new(),
            url: None,
            org_name: "Acme".to_string(),
            org_id: "42".to_string(),
            child_count: 2,
        },
        super::super::browse_support::DashboardBrowseNode {
            kind: super::super::browse_support::DashboardBrowseNodeKind::Dashboard,
            title: "CPU Main".to_string(),
            path: "Platform / Infra".to_string(),
            uid: Some("cpu-main".to_string()),
            depth: 1,
            meta: "uid=cpu-main".to_string(),
            details: vec!["Type: Dashboard".to_string()],
            url: Some("https://grafana.example.com/d/cpu-main".to_string()),
            org_name: "Acme".to_string(),
            org_id: "42".to_string(),
            child_count: 0,
        },
    ];
    let items = super::browse_render_rows::build_live_tree_items(&nodes, &[]);
    let debug = items
        .iter()
        .map(|item| format!("{item:?}"))
        .collect::<Vec<_>>();

    assert_eq!(items.len(), 2);
    assert!(debug[0].contains("ORG"));
    assert!(debug[0].contains("Acme"));
    assert!(debug[0].contains("id=42"));
    assert!(debug[1].contains("CPU Main"));
    assert!(debug[1].contains("uid=cpu-main"));
}

#[test]
fn tree_rows_and_summary_surface_selected_dashboard_count() {
    let document = super::super::browse_support::DashboardBrowseDocument {
        summary: super::super::browse_support::DashboardBrowseSummary {
            root_path: None,
            dashboard_count: 2,
            folder_count: 0,
            org_count: 1,
            scope_label: "current-org".to_string(),
        },
        nodes: vec![
            dashboard_node("cpu-main", "CPU Main"),
            dashboard_node("memory-main", "Memory Main"),
        ],
    };
    let mut state = BrowserState::new(document);
    state.toggle_selected_node();

    let summary_lines = render_summary_lines(&state)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(summary_lines[1].contains("Selected"));
    assert!(summary_lines[1].contains('1'));

    let items = super::browse_render_rows::build_live_tree_items(
        &state.document.nodes,
        &state.selected_targets(),
    );
    let debug = items
        .iter()
        .map(|item| format!("{item:?}"))
        .collect::<Vec<_>>();
    assert!(debug[0].contains("[x]"));
    assert!(debug[1].contains("[ ]"));
}

#[test]
fn local_export_tree_rows_render_file_tree_without_live_org_requirement() {
    let nodes = vec![
        super::super::browse_support::DashboardBrowseNode {
            kind: super::super::browse_support::DashboardBrowseNodeKind::Folder,
            title: "Platform".to_string(),
            path: "Platform".to_string(),
            uid: Some("platform".to_string()),
            depth: 0,
            meta: "0 folder(s) | 1 dashboard(s)".to_string(),
            details: Vec::new(),
            url: None,
            org_name: "Local export".to_string(),
            org_id: "1".to_string(),
            child_count: 0,
        },
        super::super::browse_support::DashboardBrowseNode {
            kind: super::super::browse_support::DashboardBrowseNodeKind::Dashboard,
            title: "CPU Main".to_string(),
            path: "Platform".to_string(),
            uid: Some("cpu-main".to_string()),
            depth: 1,
            meta: "uid=cpu-main".to_string(),
            details: vec!["Source file: dashboards/cpu-main.json".to_string()],
            url: None,
            org_name: "Local export".to_string(),
            org_id: "1".to_string(),
            child_count: 0,
        },
    ];
    let items = super::browse_render_rows::build_local_export_tree_items(&nodes, &[]);
    let debug = items
        .iter()
        .map(|item| format!("{item:?}"))
        .collect::<Vec<_>>();

    assert_eq!(items.len(), 2);
    assert!(!debug.iter().any(|item| item.contains("ORG")));
    assert!(debug[0].contains("Platform"));
    assert!(debug[0].contains("0 folder(s) | 1 dashboard(s)"));
    assert!(debug[1].contains("CPU Main"));
    assert!(debug[1].contains("uid=cpu-main"));
}

#[test]
fn summary_lines_move_status_out_of_the_header() {
    let state = BrowserState::new(empty_document());
    let lines = render_summary_lines(&state)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Folders"));
    assert!(lines[0].contains('0'));
    assert!(lines[0].contains("Dashboards"));
    assert!(lines[1].contains("Mode"));
    assert!(lines[1].contains("browse"));
    assert!(lines[1].contains("Focus"));
    assert!(lines[1].contains("Tree"));
    assert!(!lines
        .iter()
        .any(|line| line.contains("Loaded dashboard tree")));
}

#[test]
fn summary_lines_surface_pending_delete_mode() {
    let mut state = BrowserState::new(empty_document());
    state.pending_delete = Some(super::super::browse_state::DeleteReview::single(
        DeletePlan {
            selector_uid: None,
            selector_path: None,
            delete_folders: false,
            dashboards: Vec::new(),
            folders: Vec::new(),
        },
        None,
        Vec::new(),
    ));
    let lines = render_summary_lines(&state)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(lines[1].contains("Mode"));
    assert!(lines[1].contains("confirm-delete"));
    assert!(lines[1].contains("Focus"));
    assert!(lines[1].contains("Tree"));
}

#[test]
fn control_lines_use_consistent_pane_and_exit_labels() {
    let lines = control_lines(false, false, false, false)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(lines[0].contains("next pane"));
    assert!(lines[1].contains("previous pane"));
    assert!(lines[1].contains("search"));
    assert!(lines[2].contains("exit"));
    assert!(lines[2].contains("Esc/q"));
}

#[test]
fn delete_control_lines_use_cancel_labels() {
    let lines = control_lines(true, false, false, false)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(lines[0].contains("confirm delete"));
    assert!(lines[0].contains("cancel"));
    assert!(lines[1].contains("refresh"));
    assert!(!lines.iter().any(|line| line.contains("exit")));
}

#[test]
fn local_mode_summary_and_controls_mark_read_only_state() {
    let state = BrowserState::new_with_mode(empty_document(), true);
    let summary_lines = render_summary_lines(&state)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(summary_lines[1].contains("local-browse"));
    assert!(summary_lines[1].contains("Tree"));

    let lines = control_lines(false, false, false, true)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(lines[0].contains("refresh local tree"));
    assert!(lines[2].contains("read-only"));
}

#[test]
fn external_edit_control_lines_show_preview_save_apply_actions() {
    let lines = control_lines(false, false, true, false)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(lines[0].contains("apply live"));
    assert!(lines[0].contains("draft filename"));
    assert!(lines[0].contains("discard"));
    assert!(lines[1].contains("refresh preview"));
    assert!(!lines.iter().any(|line| line.contains("s ")));
}

#[test]
fn search_prompt_uses_compact_apply_cancel_repeat_hint() {
    let mut terminal = Terminal::new(TestBackend::new(90, 16)).unwrap();

    terminal
        .draw(|frame| render_search_prompt(frame, SearchDirection::Backward, "cpu"))
        .unwrap();

    let screen = format!("{}", terminal.backend());
    assert!(screen.contains("Enter search"));
    assert!(screen.contains("Esc cancel"));
    assert!(screen.contains("n repeat"));
    assert!(!screen.contains("repeat last search"));
}
