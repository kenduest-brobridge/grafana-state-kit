#![cfg(feature = "tui")]
use reqwest::Method;
use serde_json::Value;

use crate::common::Result;

use super::super::super::delete_support::{DashboardDeleteTarget, DeletePlan};
use super::super::browse_actions::{
    build_delete_preview, delete_status_message, execute_delete_plan_with_request,
    refresh_browser_document,
};
use super::super::browse_state::{BrowseSelectionTarget, BrowserState, DeleteReview};
use super::super::browse_support::DashboardBrowseNodeKind;
use super::browse_input_shared::scoped_org_client;
use crate::dashboard::BrowseArgs;

pub(super) fn preview_selected_delete<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
    include_folders: bool,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if state.has_selected_targets() {
        preview_selected_targets_delete(state)?;
        return Ok(());
    }

    let Some(node) = state.selected_node().cloned() else {
        return Ok(());
    };
    if node.kind == DashboardBrowseNodeKind::Org {
        state.status =
            "Org rows do not support delete. Select a folder or dashboard row.".to_string();
        return Ok(());
    }
    let plan = if let Some(client) = scoped_org_client(args, &node)? {
        let mut scoped = |method: Method,
                          path: &str,
                          params: &[(String, String)],
                          payload: Option<&Value>|
         -> Result<Option<Value>> {
            client.request_json(method, path, params, payload)
        };
        build_delete_preview(&mut scoped, args, &node, include_folders)?
    } else {
        build_delete_preview(request_json, args, &node, include_folders)?
    };
    state.pending_delete = Some(DeleteReview::single(
        plan,
        Some(node.clone()),
        BrowseSelectionTarget::from_node(&node)
            .into_iter()
            .collect(),
    ));
    state.detail_scroll = 0;
    state.status = delete_status_message(&node, include_folders);
    Ok(())
}

pub(super) fn confirm_delete<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(review) = state.pending_delete.take() else {
        return Ok(());
    };
    let Some(node) = review
        .scope_node
        .clone()
        .or_else(|| state.selected_node().cloned())
    else {
        return Ok(());
    };
    let deleted = if let Some(client) = scoped_org_client(args, &node)? {
        let mut scoped = |method: Method,
                          path: &str,
                          params: &[(String, String)],
                          payload: Option<&Value>|
         -> Result<Option<Value>> {
            client.request_json(method, path, params, payload)
        };
        execute_delete_plan_with_request(&mut scoped, &review.plan)?
    } else {
        execute_delete_plan_with_request(request_json, &review.plan)?
    };
    let document = refresh_browser_document(request_json, args)?;
    state.replace_document(document);
    state.status = format!("Deleted {} item(s) from the live dashboard tree.", deleted);
    super::ensure_selected_dashboard_view(request_json, args, state, false)?;
    Ok(())
}

fn preview_selected_targets_delete(state: &mut BrowserState) -> Result<()> {
    let targets = state.selected_dashboard_targets();
    if targets.is_empty() {
        state.pending_delete = None;
        state.status = "Only dashboard rows can be selected for batch delete.".to_string();
        return Ok(());
    }
    let org_id = targets[0].org_id.clone();
    if targets.iter().any(|target| target.org_id != org_id) {
        state.pending_delete = None;
        state.status = "Batch delete selections must stay within one org.".to_string();
        return Ok(());
    }
    let scope_node = state
        .document
        .nodes
        .iter()
        .find(|node| {
            node.kind == DashboardBrowseNodeKind::Dashboard
                && node.org_id == org_id
                && node.uid == targets[0].uid
        })
        .cloned();
    let plan = DeletePlan {
        selector_uid: None,
        selector_path: Some("selected dashboards".to_string()),
        delete_folders: false,
        dashboards: targets
            .iter()
            .map(|target| DashboardDeleteTarget {
                uid: target.uid.clone().unwrap_or_default(),
                title: target.title.clone(),
                folder_path: target.path.clone(),
            })
            .collect(),
        folders: Vec::new(),
    };
    state.pending_delete = Some(DeleteReview::single(plan, scope_node, targets));
    state.detail_scroll = 0;
    let count = state
        .pending_delete
        .as_ref()
        .map(|review| review.plan.dashboards.len())
        .unwrap_or(0);
    state.status = format!(
        "Reviewing selected dashboard delete for {count} dashboard(s). Press y to confirm."
    );
    Ok(())
}
