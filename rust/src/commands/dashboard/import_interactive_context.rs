#![cfg(feature = "tui")]

use ratatui::text::Line;

use super::import_interactive::{
    InteractiveImportContextView, InteractiveImportDiffDepth, InteractiveImportReviewState,
    InteractiveImportState, InteractiveImportSummaryCounts, InteractiveImportSummaryScope,
};

pub(crate) fn build_context_lines(state: &InteractiveImportState) -> Vec<Line<'static>> {
    match state.context_view {
        InteractiveImportContextView::Summary => build_summary_context_lines(state),
        InteractiveImportContextView::Destination => build_destination_context_lines(state),
        InteractiveImportContextView::Diff => build_diff_context_lines(state),
    }
}

fn build_summary_context_lines(state: &InteractiveImportState) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    match state.summary_scope {
        InteractiveImportSummaryScope::Focused => {
            let Some(item) = state.selected_item() else {
                return vec![Line::from("No dashboard selected.")];
            };
            lines.push(Line::from(format!(
                "Scope=focused   uid={}   title={}",
                item.uid, item.title
            )));
            lines.push(Line::from(format!(
                "Selected={}   Group={}",
                state.selected_paths.contains(&item.path),
                state.grouping.label()
            )));
            match &item.review {
                InteractiveImportReviewState::Pending => {
                    lines.push(Line::from("Review pending for the focused dashboard."));
                }
                InteractiveImportReviewState::Failed(error) => {
                    lines.push(Line::from("Review blocked for the focused dashboard."));
                    lines.push(Line::from(error.clone()));
                }
                InteractiveImportReviewState::Resolved(review) => {
                    lines.push(Line::from(format!(
                        "Preview action={}   destination={}   remote={}",
                        review.action_label, review.destination, review.diff_status
                    )));
                    lines.push(Line::from(format!(
                        "Diff availability: {}",
                        format_diff_availability(review)
                    )));
                    if !review.reason.is_empty() {
                        lines.push(Line::from(format!("Reason={}", review.reason)));
                    }
                }
            }
        }
        InteractiveImportSummaryScope::Selected | InteractiveImportSummaryScope::All => {
            let counts = summary_counts_for_scope(state, state.summary_scope);
            lines.push(Line::from(format!(
                "Scope={}   total={}   selected={}   reviewed={}   pending={}",
                state.summary_scope.label(),
                counts.total,
                counts.selected,
                counts.reviewed,
                counts.pending
            )));
            lines.push(Line::from(format!(
                "create={}   update={}   skip-missing={}   skip-folder={}   blocked={}",
                counts.create,
                counts.update,
                counts.skip_missing,
                counts.skip_folder,
                counts.blocked
            )));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Use `s` to switch focused / selected / all scope.",
    ));
    lines
}

fn build_destination_context_lines(state: &InteractiveImportState) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    match state.summary_scope {
        InteractiveImportSummaryScope::Focused => {
            let Some(item) = state.selected_item() else {
                return vec![Line::from("No dashboard selected.")];
            };
            lines.push(Line::from(format!("Scope=focused   uid={}", item.uid)));
            match &item.review {
                InteractiveImportReviewState::Pending => {
                    lines.push(Line::from("Destination status pending review."));
                }
                InteractiveImportReviewState::Failed(error) => {
                    lines.push(Line::from("Destination status blocked."));
                    lines.push(Line::from(error.clone()));
                }
                InteractiveImportReviewState::Resolved(review) => {
                    lines.push(Line::from(format!(
                        "Destination={}   Action={}",
                        review.destination, review.action_label
                    )));
                    lines.push(Line::from(format!("Target Folder={}", review.folder_path)));
                    if !review.source_folder_path.is_empty()
                        || !review.destination_folder_path.is_empty()
                    {
                        lines.push(Line::from(format!(
                            "Folder guard={}   source={}   existing={}",
                            folder_guard_status(review),
                            display_context_value(&review.source_folder_path),
                            display_context_value(&review.destination_folder_path)
                        )));
                    }
                    if !review.destination_folder_path.is_empty() {
                        lines.push(Line::from(format!(
                            "Existing Folder={}",
                            review.destination_folder_path
                        )));
                    }
                    if review.diff_status == "new dashboard" {
                        lines.push(Line::from(format!(
                            "Missing remote: live lookup did not find {}; import action is {}.",
                            item.uid,
                            missing_remote_action_status(review)
                        )));
                    }
                    lines.push(Line::from(format!("Live Diff={}", review.diff_status)));
                }
            }
        }
        InteractiveImportSummaryScope::Selected | InteractiveImportSummaryScope::All => {
            let destination = destination_counts_for_scope(state, state.summary_scope);
            lines.push(Line::from(format!(
                "Scope={}   total={}   exists={}   missing={}   blocked={}",
                state.summary_scope.label(),
                destination.total,
                destination.exists,
                destination.missing,
                destination.blocked
            )));
            lines.push(Line::from(format!(
                "matches-live={}   changed={}   new-dashboard={}",
                destination.matches_live, destination.changed, destination.new_dashboard
            )));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Use `s` to switch destination scope."));
    lines
}

fn build_diff_context_lines(state: &InteractiveImportState) -> Vec<Line<'static>> {
    let Some(item) = state.selected_item() else {
        return vec![Line::from("No dashboard selected.")];
    };
    let mut lines = vec![Line::from(format!(
        "Scope=focused   uid={}   diff-depth={}",
        item.uid,
        state.diff_depth.label()
    ))];
    match &item.review {
        InteractiveImportReviewState::Pending => {
            lines.push(Line::from("Diff pending review."));
        }
        InteractiveImportReviewState::Failed(error) => {
            lines.push(Line::from("Diff blocked because review failed."));
            lines.push(Line::from(error.clone()));
        }
        InteractiveImportReviewState::Resolved(review) => {
            lines.push(Line::from(format!(
                "Remote status={}   {}",
                review.diff_status,
                format_diff_availability(review)
            )));
            let selected_lines = match state.diff_depth {
                InteractiveImportDiffDepth::Summary => &review.diff_summary_lines,
                InteractiveImportDiffDepth::Structural => &review.diff_structural_lines,
                InteractiveImportDiffDepth::Raw => &review.diff_raw_lines,
            };
            if selected_lines.is_empty() {
                lines.push(Line::from("No diff details available."));
            } else {
                lines.extend(selected_lines.iter().cloned().map(Line::from));
            }
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Use `d` to switch summary / structural / raw diff.",
    ));
    lines
}

fn summary_counts_for_scope(
    state: &InteractiveImportState,
    scope: InteractiveImportSummaryScope,
) -> InteractiveImportSummaryCounts {
    let mut counts = InteractiveImportSummaryCounts::default();
    for item in &state.items {
        let included = match scope {
            InteractiveImportSummaryScope::Focused => state
                .selected_item()
                .is_some_and(|focused| focused.path == item.path),
            InteractiveImportSummaryScope::Selected => state.selected_paths.contains(&item.path),
            InteractiveImportSummaryScope::All => true,
        };
        if !included {
            continue;
        }
        counts.total += 1;
        if state.selected_paths.contains(&item.path) {
            counts.selected += 1;
        }
        match &item.review {
            InteractiveImportReviewState::Pending => counts.pending += 1,
            InteractiveImportReviewState::Failed(_) => counts.blocked += 1,
            InteractiveImportReviewState::Resolved(review) => match review.action_label.as_str() {
                "create" => counts.create += 1,
                "update" => counts.update += 1,
                "skip-missing" => counts.skip_missing += 1,
                "skip-folder-mismatch" => counts.skip_folder += 1,
                "blocked-existing" => counts.blocked += 1,
                _ => {}
            },
        }
    }
    counts.reviewed = counts.total.saturating_sub(counts.pending);
    counts
}

#[derive(Default)]
struct DestinationCounts {
    total: usize,
    exists: usize,
    missing: usize,
    blocked: usize,
    matches_live: usize,
    changed: usize,
    new_dashboard: usize,
}

fn destination_counts_for_scope(
    state: &InteractiveImportState,
    scope: InteractiveImportSummaryScope,
) -> DestinationCounts {
    let mut counts = DestinationCounts::default();
    for item in &state.items {
        let included = match scope {
            InteractiveImportSummaryScope::Focused => state
                .selected_item()
                .is_some_and(|focused| focused.path == item.path),
            InteractiveImportSummaryScope::Selected => state.selected_paths.contains(&item.path),
            InteractiveImportSummaryScope::All => true,
        };
        if !included {
            continue;
        }
        counts.total += 1;
        match &item.review {
            InteractiveImportReviewState::Pending => {}
            InteractiveImportReviewState::Failed(_) => counts.blocked += 1,
            InteractiveImportReviewState::Resolved(review) => {
                if review.destination == "exists" {
                    counts.exists += 1;
                } else if review.destination == "missing" {
                    counts.missing += 1;
                }
                match review.diff_status.as_str() {
                    "matches live" => counts.matches_live += 1,
                    "changed" => counts.changed += 1,
                    "new dashboard" => counts.new_dashboard += 1,
                    _ => {}
                }
            }
        }
    }
    counts
}

fn format_diff_availability(review: &super::import_interactive::InteractiveImportReview) -> String {
    format!(
        "summary={} structural={} raw={}",
        yes_no(!review.diff_summary_lines.is_empty()),
        yes_no(!review.diff_structural_lines.is_empty()),
        yes_no(!review.diff_raw_lines.is_empty())
    )
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn folder_guard_status(
    review: &super::import_interactive::InteractiveImportReview,
) -> &'static str {
    if review.action_label == "skip-folder-mismatch" {
        "blocked"
    } else if review.destination_folder_path.is_empty() {
        "not-checked"
    } else if review.source_folder_path == review.destination_folder_path {
        "matched"
    } else {
        "mismatch"
    }
}

fn display_context_value(value: &str) -> &str {
    if value.is_empty() {
        "-"
    } else {
        value
    }
}

fn missing_remote_action_status(
    review: &super::import_interactive::InteractiveImportReview,
) -> &'static str {
    if review.action_label == "skip-folder-mismatch" {
        "blocked by folder guard"
    } else if review.action_label == "skip-missing" {
        "skipped by update-only mode"
    } else {
        "ready to create"
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::dashboard::import_interactive::{
        InteractiveImportItem, InteractiveImportReview, InteractiveImportReviewState,
    };

    fn line_texts(lines: Vec<Line<'static>>) -> Vec<String> {
        lines
            .into_iter()
            .map(|line| {
                line.spans
                    .into_iter()
                    .map(|span| span.content.into_owned())
                    .collect::<String>()
            })
            .collect()
    }

    fn reviewed_state() -> InteractiveImportState {
        let path = PathBuf::from("dashboards/service.json");
        let item = InteractiveImportItem {
            path: path.clone(),
            uid: "service-overview".to_string(),
            title: "Service Overview".to_string(),
            folder_path: "Team / Service".to_string(),
            file_label: "service.json".to_string(),
            review: InteractiveImportReviewState::Resolved(Box::new(InteractiveImportReview {
                action: "would-skip-folder-mismatch".to_string(),
                destination: "exists".to_string(),
                action_label: "skip-folder-mismatch".to_string(),
                folder_path: "Team / Service".to_string(),
                source_folder_path: "Team / Service".to_string(),
                destination_folder_path: "Team / Other".to_string(),
                reason: "folder path mismatch: source=Team / Service destination=Team / Other"
                    .to_string(),
                diff_status: "new dashboard".to_string(),
                diff_summary_lines: vec![
                    "No live dashboard exists yet; import would create a new item.".to_string(),
                ],
                diff_structural_lines: vec!["No live dashboard exists yet.".to_string()],
                diff_raw_lines: vec![
                    "REMOTE <missing>".to_string(),
                    "LOCAL <new dashboard payload>".to_string(),
                ],
            })),
        };
        let mut state = InteractiveImportState::new(vec![item], "create-only".to_string(), true);
        state.selected_paths.insert(path);
        state
    }

    #[test]
    fn focused_preview_context_explains_action_guard_remote_status_and_diff_availability() {
        let mut state = reviewed_state();

        let summary = line_texts(build_context_lines(&state));
        assert!(summary.contains(
            &"Preview action=skip-folder-mismatch   destination=exists   remote=new dashboard"
                .to_string()
        ));
        assert!(
            summary.contains(&"Diff availability: summary=yes structural=yes raw=yes".to_string())
        );

        state.context_view = InteractiveImportContextView::Destination;
        let destination = line_texts(build_context_lines(&state));
        assert!(destination.contains(
            &"Folder guard=blocked   source=Team / Service   existing=Team / Other".to_string()
        ));
        assert!(destination.contains(
            &"Missing remote: live lookup did not find service-overview; import action is blocked by folder guard."
                .to_string()
        ));

        state.context_view = InteractiveImportContextView::Diff;
        let diff = line_texts(build_context_lines(&state));
        assert!(diff.contains(
            &"Remote status=new dashboard   summary=yes structural=yes raw=yes".to_string()
        ));
    }
}
