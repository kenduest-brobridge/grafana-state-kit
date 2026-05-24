//! Terminal interaction layer for Sync operations and command-driven review flows.
#![cfg_attr(not(feature = "tui"), allow(dead_code, unused_imports))]

use crate::common::Result;
use crate::datasource::is_safe_changed_field;
#[cfg(feature = "tui")]
pub(crate) use crate::review_diff::{
    build_review_diff_controls_lines as build_diff_controls_lines,
    render_review_diff_items as render_diff_items,
};
#[cfg(any(feature = "tui", test))]
pub(crate) use crate::review_diff::{
    clip_text_window, review_diff_pane_title as diff_pane_title,
    review_diff_scroll_max as diff_scroll_max, wrap_text_chunks,
    ReviewDiffControlsState as DiffControlsState, ReviewDiffLine, ReviewDiffModel,
    ReviewDiffPaneFocus as DiffPaneFocus,
};

#[cfg(feature = "tui")]
use ratatui::style::{Color, Modifier, Style};
#[cfg(feature = "tui")]
use ratatui::text::{Line, Span};
use serde_json::Value;
use std::collections::BTreeSet;

use super::super::json::{require_json_array_field, require_json_object};
use super::super::plan_builder::{
    build_sync_alert_assessment_document, build_sync_plan_summary_document,
};

#[cfg(any(feature = "tui", test))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewableOperation {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) operation: Value,
}

#[cfg(any(feature = "tui", test))]
fn operation_key(operation: &serde_json::Map<String, Value>) -> String {
    if let Some(action_id) = operation.get("actionId").and_then(Value::as_str) {
        return action_id.to_string();
    }
    format!(
        "{}::{}",
        operation
            .get("resourceKind")
            .or_else(|| operation.get("kind"))
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        operation
            .get("identity")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    )
}

#[cfg(any(feature = "tui", test))]
fn operation_label(operation: &serde_json::Map<String, Value>) -> String {
    format!(
        "{} {}",
        operation
            .get("resourceKind")
            .or_else(|| operation.get("kind"))
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        operation
            .get("identity")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    )
}

#[cfg(any(feature = "tui", test))]
fn operation_badge(action: &str) -> &'static str {
    match action {
        "would-create" => "CREATE",
        "would-update" => "UPDATE",
        "would-delete" => "DELETE",
        _ => "UNKNOWN",
    }
}

#[cfg(feature = "tui")]
pub(crate) fn operation_badge_color(action: &str) -> Color {
    match action {
        "would-create" => Color::Green,
        "would-update" => Color::Yellow,
        "would-delete" => Color::Red,
        _ => Color::DarkGray,
    }
}

#[cfg(feature = "tui")]
pub(crate) fn operation_row_color(action: &str) -> Color {
    match action {
        "would-create" => Color::LightGreen,
        "would-update" => Color::LightYellow,
        "would-delete" => Color::LightRed,
        _ => Color::Gray,
    }
}

#[cfg(feature = "tui")]
fn selection_mark(selected: bool) -> &'static str {
    if selected {
        "✓"
    } else {
        "·"
    }
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn operation_preview(item: &ReviewableOperation) -> Vec<String> {
    let object = match item.operation.as_object() {
        Some(object) => object,
        None => return vec!["Invalid operation payload".to_string()],
    };
    let action = object
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let kind = object
        .get("resourceKind")
        .or_else(|| object.get("kind"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let identity = object
        .get("identity")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let changed_fields = object
        .get("changedFields")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter(|field| is_safe_changed_field(field))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "none".to_string());
    vec![
        format!("Action: {}", operation_badge(action)),
        format!("Kind: {kind}"),
        format!("Identity: {identity}"),
        format!("Changed: {changed_fields}"),
    ]
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn operation_detail_line_count(item: &ReviewableOperation) -> usize {
    build_review_operation_diff_model(&item.operation)
        .map(|model| model.live_lines.len().max(model.desired_lines.len()))
        .unwrap_or(0)
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn operation_changed_count(item: &ReviewableOperation) -> usize {
    build_review_operation_diff_model(&item.operation)
        .map(|model| model.live_lines.iter().filter(|line| line.changed).count())
        .unwrap_or(0)
}

#[cfg(any(feature = "tui", test))]
fn truncate_text(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    let kept = text.chars().take(max_chars - 1).collect::<String>();
    format!("{kept}…")
}

#[cfg(feature = "tui")]
pub(crate) fn build_checklist_line(
    item: &ReviewableOperation,
    index: usize,
    selected: bool,
    content_width: usize,
) -> Line<'static> {
    let action = item
        .operation
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let prefix = format!("{} {:>2}. ", selection_mark(selected), index + 1);
    let badge_text = format!("[{}]", operation_badge(action));
    let detail_rows = operation_detail_line_count(item);
    let changed = operation_changed_count(item);
    let row_label = if detail_rows == 1 { "row" } else { "rows" };
    let meta = format!("{detail_rows} {row_label} / {changed} changed");
    let reserved =
        prefix.chars().count() + 1 + badge_text.chars().count() + 1 + meta.chars().count();
    let label_width = content_width.saturating_sub(reserved).max(8);
    let label_text = truncate_text(&item.label, label_width);
    let current =
        prefix.chars().count() + badge_text.chars().count() + 1 + label_text.chars().count();
    let gap = content_width
        .saturating_sub(current + meta.chars().count())
        .max(1);

    Line::from(vec![
        Span::raw(prefix),
        Span::styled(
            badge_text,
            Style::default()
                .fg(operation_badge_color(action))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::raw(label_text),
        Span::raw(" ".repeat(gap)),
        Span::styled(meta, Style::default().fg(Color::DarkGray)),
    ])
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn selection_title_with_position(
    item: Option<&ReviewableOperation>,
    position: Option<usize>,
    total: Option<usize>,
) -> String {
    let Some(item) = item else {
        return "Selection".to_string();
    };
    let action = item
        .operation
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let identity = item
        .operation
        .get("identity")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    match (position, total) {
        (Some(position), Some(total)) if total > 0 => format!(
            "Selection {}/{} [{}] {identity}",
            position + 1,
            total,
            operation_badge(action)
        ),
        _ => format!("Selection [{}] {identity}", operation_badge(action)),
    }
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn collect_reviewable_operations(plan: &Value) -> Result<Vec<ReviewableOperation>> {
    let plan = require_json_object(plan, "Sync plan document")?;
    let operations = match plan.get("actions").or_else(|| plan.get("operations")) {
        Some(Value::Array(items)) => items,
        _ => require_json_array_field(plan, "operations", "Sync plan document")?,
    };
    Ok(operations
        .iter()
        .filter_map(Value::as_object)
        .filter(|operation| {
            matches!(
                operation.get("action").and_then(Value::as_str),
                Some("would-create" | "would-update" | "would-delete")
            )
        })
        .map(|operation| ReviewableOperation {
            key: operation_key(operation),
            label: operation_label(operation),
            operation: Value::Object(operation.clone()),
        })
        .collect())
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn filter_reviewable_operations_by_query(
    items: &[ReviewableOperation],
    query: &str,
) -> Vec<ReviewableOperation> {
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return items.to_vec();
    }
    items
        .iter()
        .filter(|item| reviewable_operation_matches_query(item, &needle))
        .cloned()
        .collect()
}

#[cfg(any(feature = "tui", test))]
fn reviewable_operation_matches_query(item: &ReviewableOperation, needle: &str) -> bool {
    item.key.to_ascii_lowercase().contains(needle)
        || item.label.to_ascii_lowercase().contains(needle)
        || operation_preview(item)
            .iter()
            .any(|line| line.to_ascii_lowercase().contains(needle))
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn filter_review_plan_operations(
    plan: &Value,
    selected_keys: &BTreeSet<String>,
) -> Result<Value> {
    let plan_object = require_json_object(plan, "Sync plan document")?;
    let operations = match plan_object
        .get("actions")
        .or_else(|| plan_object.get("operations"))
    {
        Some(Value::Array(items)) => items,
        _ => require_json_array_field(plan_object, "operations", "Sync plan document")?,
    };
    let filtered_operations = operations
        .iter()
        .filter(|item| {
            let Some(object) = item.as_object() else {
                return false;
            };
            match object.get("action").and_then(Value::as_str) {
                Some("would-create" | "would-update" | "would-delete") => {
                    selected_keys.contains(&operation_key(object))
                }
                _ => true,
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut filtered = plan_object.clone();
    filtered.insert(
        "summary".to_string(),
        build_sync_plan_summary_document(&filtered_operations),
    );
    filtered.insert(
        "alertAssessment".to_string(),
        build_sync_alert_assessment_document(&filtered_operations),
    );
    filtered.insert(
        "operations".to_string(),
        Value::Array(filtered_operations.clone()),
    );
    filtered.insert("actions".to_string(), Value::Array(filtered_operations));
    if let Ok(view) = crate::sync::workspace_preview_review_view::build_workspace_review_view(
        &Value::Object(filtered.clone()),
    ) {
        filtered.insert(
            "domains".to_string(),
            Value::Array(
                view.domains
                    .iter()
                    .map(|domain| domain.raw.clone())
                    .collect(),
            ),
        );
        filtered.insert(
            "blockedReasons".to_string(),
            Value::Array(
                view.blocked_reasons
                    .iter()
                    .map(|reason| Value::String(reason.clone()))
                    .collect(),
            ),
        );
        if let Some(summary) = filtered.get_mut("summary").and_then(Value::as_object_mut) {
            summary
                .entry("actionCount".to_string())
                .or_insert(Value::Number((view.summary.action_count as i64).into()));
            summary
                .entry("domainCount".to_string())
                .or_insert(Value::Number((view.summary.domain_count as i64).into()));
            summary
                .entry("sameCount".to_string())
                .or_insert(Value::Number((view.summary.same_count as i64).into()));
            summary
                .entry("blockedCount".to_string())
                .or_insert(Value::Number((view.summary.blocked_count as i64).into()));
            summary
                .entry("warningCount".to_string())
                .or_insert(Value::Number((view.summary.warning_count as i64).into()));
            if !summary.contains_key("blocked_reasons") {
                summary.insert(
                    "blocked_reasons".to_string(),
                    Value::Array(
                        view.blocked_reasons
                            .iter()
                            .map(|reason| Value::String(reason.clone()))
                            .collect(),
                    ),
                );
            }
        }
    }
    Ok(Value::Object(filtered))
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn build_review_operation_diff_model(operation: &Value) -> Result<ReviewDiffModel> {
    require_json_object(operation, "Sync review operation")?;
    crate::review_diff::build_review_diff_model(
        crate::review_diff::ReviewDiffInput::from_operation(operation)?,
    )
}
