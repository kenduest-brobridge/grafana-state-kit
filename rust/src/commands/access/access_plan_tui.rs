//! Read-only interactive review surface for access plan documents.

use crate::common::Result;
#[cfg(any(feature = "tui", test))]
use crate::review_contract::{
    append_review_evidence_section, build_review_mutation_action_change_detail_lines,
    build_review_mutation_action_context_lines, build_review_mutation_action_detail_lines,
    build_review_mutation_action_diff_preview_lines, build_review_mutation_action_impact_line,
    build_review_mutation_action_narrative_line, build_review_mutation_action_next_check_lines,
    build_review_mutation_action_target_evidence_lines,
};

#[cfg(feature = "tui")]
use crate::interactive_browser::run_interactive_browser;
#[cfg(any(feature = "tui", test))]
use crate::interactive_browser::BrowserItem;

use super::AccessPlanDocument;

#[cfg(feature = "tui")]
fn build_access_plan_summary_lines(document: &AccessPlanDocument) -> Vec<String> {
    let review = document.build_review_envelope();
    let mut lines = vec![
        "Access plan review".to_string(),
        format!(
            "Resources: {}  checked: {}  same: {}",
            document.summary.resource_count, document.summary.checked, document.summary.same
        ),
        format!(
            "Create: {}  update: {}  extra remote: {}  delete: {}",
            document.summary.create,
            document.summary.update,
            document.summary.extra_remote,
            document.summary.delete
        ),
        format!(
            "Blocked: {}  warning: {}  prune: {}  actions: {}",
            document.summary.blocked,
            document.summary.warning,
            document.summary.prune,
            document.actions.len()
        ),
    ];
    if !review.blocked_reasons.is_empty() {
        lines.push(format!(
            "Blocked reasons: {}",
            review.blocked_reasons.join(" | ")
        ));
    }
    lines
}

#[cfg(any(feature = "tui", test))]
fn resource_focus_line(resource: &super::AccessPlanResourceReport) -> Option<String> {
    let detail = if resource.blocked > 0 {
        Some(format!(
            "{} blocked actions need operator resolution before this bundle can be applied cleanly",
            resource.blocked
        ))
    } else if resource.warning > 0 {
        Some(format!(
            "{} warning actions need operator review before approval",
            resource.warning
        ))
    } else if resource.create + resource.update + resource.delete > 0 {
        Some("this bundle contains live changes that should be reviewed before apply".to_string())
    } else if resource.same == resource.checked && resource.checked > 0 {
        Some("this bundle currently matches live state".to_string())
    } else {
        None
    }?;
    Some(format!("Review focus: {detail}."))
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn build_access_plan_browser_items(document: &AccessPlanDocument) -> Vec<BrowserItem> {
    let projection = document.build_review_projection();
    let mut items = Vec::with_capacity(document.resources.len() + projection.actions.len());

    for resource in &document.resources {
        items.push(BrowserItem {
            kind: "resource".to_string(),
            title: format!("{} bundle", resource.resource_kind),
            meta: format!(
                "{}  checked={} same={} create={} update={} extra={} delete={} blocked={} warning={}",
                if resource.bundle_present {
                    "present"
                } else {
                    "missing"
                },
                resource.checked,
                resource.same,
                resource.create,
                resource.update,
                resource.extra_remote,
                resource.delete,
                resource.blocked,
                resource.warning
            ),
            details: {
                let mut details = vec![
                    format!("Resource kind: {}", resource.resource_kind),
                    format!("Source path: {}", resource.source_path),
                    format!("Bundle present: {}", resource.bundle_present),
                    format!("Source count: {}", resource.source_count),
                    format!("Live count: {}", resource.live_count),
                ];
                if let Some(scope) = &resource.scope {
                    details.push(format!("Scope: {}", scope));
                }
                if let Some(focus) = resource_focus_line(resource) {
                    details.push(focus);
                }
                details.extend(resource.notes.iter().map(|note| format!("Note: {}", note)));
                details
            },
        });
    }

    for action in projection.actions {
        let source_path = action
            .raw
            .get("sourcePath")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let scope = action
            .raw
            .get("scope")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let mut details = vec![build_review_mutation_action_narrative_line(&action)];
        if !scope.is_empty() {
            details.push(format!("Scope: {}", scope));
        }
        if let Some(impact) = build_review_mutation_action_impact_line(&action) {
            details.push(impact);
        }
        append_review_evidence_section(
            &mut details,
            build_review_mutation_action_detail_lines(&action),
        );
        if !source_path.is_empty() {
            details.push(format!("Source path: {}", source_path));
        }
        details.extend(build_review_mutation_action_change_detail_lines(&action));
        details.extend(build_review_mutation_action_diff_preview_lines(&action));
        details.extend(build_review_mutation_action_target_evidence_lines(&action));
        details.extend(build_review_mutation_action_context_lines(&action));
        details.extend(
            action
                .review_hints
                .iter()
                .map(|hint| format!("Hint: {}", hint)),
        );
        details.extend(build_review_mutation_action_next_check_lines(&action));
        items.push(BrowserItem {
            kind: action.resource_kind.clone(),
            title: action.identity,
            meta: format!(
                "{}  {}  {}",
                action.status, action.action, action.order_group
            ),
            details,
        });
    }

    items
}

#[cfg(feature = "tui")]
pub(crate) fn run_access_plan_interactive(document: &AccessPlanDocument) -> Result<()> {
    let summary_lines = build_access_plan_summary_lines(document);
    let items = build_access_plan_browser_items(document);
    run_interactive_browser("Access plan review", &summary_lines, &items)
}

#[cfg(not(feature = "tui"))]
pub(crate) fn run_access_plan_interactive(_document: &AccessPlanDocument) -> Result<()> {
    Err(crate::common::tui_feature_required(
        "Access plan --interactive",
    ))
}
