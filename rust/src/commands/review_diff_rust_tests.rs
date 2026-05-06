//! Shared review diff visualization model tests.

use crate::review_diff::{build_review_diff_model, ReviewDiffInput};
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
