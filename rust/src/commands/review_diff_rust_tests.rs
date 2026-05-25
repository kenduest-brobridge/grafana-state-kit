//! Shared review diff visualization model tests.

use crate::review_diff::{
    build_review_diff_model, clip_text_window, review_diff_model_preview_lines,
    review_diff_pane_title, review_diff_scroll_max, wrap_text_chunks, ReviewDiffInput,
    ReviewDiffLine, ReviewDiffModel, ReviewDiffPaneFocus,
};
use serde_json::json;

#[test]
fn shared_review_diff_model_represents_workspace_and_domain_shaped_operations() {
    let workspace_operation = json!({
        "resourceKind": "dashboard",
        "identity": "cpu-main",
        "action": "would-update",
        "changedFields": ["title"],
        "live": {
            "title": "CPU Old"
        },
        "desired": {
            "title": "CPU New"
        }
    });
    let alert_operation = json!({
        "kind": "alert-rule",
        "identity": "latency-slo",
        "action": "would-update",
        "changedFields": ["condition", "for"],
        "live": {
            "condition": "B",
            "for": "5m"
        },
        "desired": {
            "condition": "C",
            "for": "10m"
        }
    });

    let workspace_model =
        build_review_diff_model(ReviewDiffInput::from_operation(&workspace_operation).unwrap())
            .unwrap();
    let alert_model =
        build_review_diff_model(ReviewDiffInput::from_operation(&alert_operation).unwrap())
            .unwrap();

    assert_eq!(workspace_model.title, "dashboard cpu-main");
    assert_eq!(workspace_model.action, "would-update");
    assert_eq!(workspace_model.live_lines[0].marker, '-');
    assert_eq!(workspace_model.desired_lines[0].marker, '+');
    assert!(workspace_model.live_lines[0].content.contains("title"));
    assert!(workspace_model.live_lines[0].highlight_range.is_some());

    assert_eq!(alert_model.title, "alert-rule latency-slo");
    assert_eq!(alert_model.live_lines.len(), 2);
    assert_eq!(alert_model.desired_lines.len(), 2);
    assert!(alert_model.live_lines.iter().all(|line| line.changed));
    assert!(alert_model.desired_lines[1].content.contains("for"));
}

#[test]
fn shared_review_diff_view_helpers_cover_titles_scroll_and_text_windows() {
    let model = ReviewDiffModel {
        title: "dashboard cpu-main".to_string(),
        action: "would-update".to_string(),
        live_lines: vec![ReviewDiffLine {
            changed: true,
            marker: '-',
            content: "  1 | title: \"old\"".to_string(),
            highlight_range: Some((13, 16)),
        }],
        desired_lines: vec![
            ReviewDiffLine {
                changed: true,
                marker: '+',
                content: "  1 | title: \"new\"".to_string(),
                highlight_range: Some((13, 16)),
            },
            ReviewDiffLine {
                changed: true,
                marker: '+',
                content: "  2 | refresh: \"5s\"".to_string(),
                highlight_range: Some((15, 19)),
            },
        ],
    };

    assert_eq!(
        review_diff_pane_title("Live", &model.action, &model.title, 0, 3),
        "Live 1/3 [would-update] dashboard cpu-main"
    );
    assert_eq!(review_diff_scroll_max(&model, ReviewDiffPaneFocus::Live), 0);
    assert_eq!(
        review_diff_scroll_max(&model, ReviewDiffPaneFocus::Desired),
        1
    );

    let wrapped = wrap_text_chunks("  1 | datasourceUid: \"smoke-prom-extra\"", 18);
    assert_eq!(wrapped[0], "  1 | datasourceUi");
    assert!(wrapped.len() > 1);
    assert_eq!(
        clip_text_window("  1 | datasourceUid: \"smoke-prom-extra\"", 6, 16),
        "datasourceUid: \""
    );

    let preview = review_diff_model_preview_lines(&model, 1);
    assert_eq!(preview[0], "Shared Diff: dashboard cpu-main [would-update]");
    assert_eq!(preview[1], "Live -   1 | title: \"old\"");
    assert_eq!(preview[2], "Desired +   1 | title: \"new\"");
    assert_eq!(preview.len(), 3);
}
