//! Alert runtime plan review projection.

use crate::common::{message, value_as_object, Result};
use crate::review_contract::{
    build_review_mutation_envelope, review_action_rank, ReviewBlockedReason, ReviewMutationAction,
    ReviewMutationActionInput, ReviewMutationEnvelope, REVIEW_ACTION_BLOCKED, REVIEW_ACTION_SAME,
    REVIEW_ACTION_WOULD_CREATE, REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE,
};
use serde_json::{Map, Value};

use super::alert_runtime_plan_document::{runtime_schema, ALERT_PLAN_KIND};

#[derive(Debug, Clone)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct AlertPlanReviewProjection {
    pub(crate) domains: Vec<&'static str>,
    pub(crate) actions: Vec<ReviewMutationAction>,
}

fn row_object<'a>(row: &'a Value, label: &str) -> Result<&'a Map<String, Value>> {
    value_as_object(row, label)
}

fn alert_plan_review_action(action: &str) -> Option<&'static str> {
    match action {
        "create" => Some(REVIEW_ACTION_WOULD_CREATE),
        "update" => Some(REVIEW_ACTION_WOULD_UPDATE),
        "noop" => Some(REVIEW_ACTION_SAME),
        "delete" => Some(REVIEW_ACTION_WOULD_DELETE),
        "blocked" => Some(REVIEW_ACTION_BLOCKED),
        _ => None,
    }
}

fn alert_plan_review_hints(row: &Map<String, Value>) -> Vec<String> {
    row.get(runtime_schema::row::REVIEW_HINTS)
        .and_then(Value::as_array)
        .map(|hints| {
            hints
                .iter()
                .filter_map(|hint| match hint {
                    Value::String(text) if !text.trim().is_empty() => Some(text.clone()),
                    Value::Object(object) => object
                        .get("code")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|code| !code.is_empty())
                        .map(ToOwned::to_owned),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn alert_plan_review_details(row: &Map<String, Value>) -> Option<String> {
    let changed_fields = row
        .get(runtime_schema::row::CHANGED_FIELDS)
        .and_then(Value::as_array)
        .map(|fields| {
            fields
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    (!changed_fields.is_empty()).then(|| format!("fields={}", changed_fields.join(",")))
}

fn alert_plan_row_to_review_projection(row: &Map<String, Value>) -> Result<ReviewMutationAction> {
    let raw = Value::Object(row.clone());
    let action = row
        .get(runtime_schema::row::ACTION)
        .and_then(Value::as_str)
        .ok_or_else(|| message("Alert plan row is missing action."))?;
    let mapped_action = alert_plan_review_action(action).ok_or_else(|| {
        message(format!(
            "Alert plan row action {action:?} is not supported."
        ))
    })?;
    let action_id = row
        .get(runtime_schema::row::ACTION_ID)
        .and_then(Value::as_str)
        .ok_or_else(|| message("Alert plan row is missing actionId."))?;
    let resource_kind = row
        .get(runtime_schema::row::RESOURCE_KIND)
        .or_else(|| row.get(runtime_schema::row::KIND))
        .and_then(Value::as_str)
        .ok_or_else(|| message("Alert plan row is missing resourceKind."))?;
    let identity = row
        .get(runtime_schema::row::IDENTITY)
        .and_then(Value::as_str)
        .ok_or_else(|| message("Alert plan row is missing identity."))?;
    let status = row
        .get(runtime_schema::row::STATUS)
        .and_then(Value::as_str)
        .ok_or_else(|| message("Alert plan row is missing status."))?;
    let blocked_reason = ReviewBlockedReason::from_optional_text(
        row.get(runtime_schema::row::BLOCKED_REASON)
            .and_then(Value::as_str),
    )
    .map(ReviewBlockedReason::into_string);
    Ok(ReviewMutationActionInput {
        action_id: action_id.to_string(),
        action: mapped_action.to_string(),
        domain: "alert".to_string(),
        resource_kind: resource_kind.to_string(),
        identity: identity.to_string(),
        status: status.to_string(),
        blocked_reason,
        details: alert_plan_review_details(row),
        review_hints: alert_plan_review_hints(row),
        raw,
    }
    .into())
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_alert_plan_review_projection(
    plan_document: &Value,
) -> Result<AlertPlanReviewProjection> {
    let plan = value_as_object(plan_document, "Alert plan document")?;
    if plan
        .get(runtime_schema::document::KIND)
        .and_then(Value::as_str)
        != Some(ALERT_PLAN_KIND)
    {
        return Err(message("Alert plan document kind is not supported."));
    }
    let rows = plan
        .get(runtime_schema::document::ROWS)
        .and_then(Value::as_array)
        .ok_or_else(|| message("Alert plan document is missing rows."))?;
    let mut actions = rows
        .iter()
        .map(|row| row_object(row, "Alert plan row").and_then(alert_plan_row_to_review_projection))
        .collect::<Result<Vec<_>>>()?;
    actions.sort_by(|left, right| {
        left.kind_order
            .cmp(&right.kind_order)
            .then_with(|| review_action_rank(&left.action).cmp(&review_action_rank(&right.action)))
            .then_with(|| left.identity.cmp(&right.identity))
            .then_with(|| left.action_id.cmp(&right.action_id))
    });
    Ok(AlertPlanReviewProjection {
        domains: vec!["alert"],
        actions,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_alert_plan_review_envelope(
    plan_document: &Value,
) -> Result<ReviewMutationEnvelope> {
    let projection = build_alert_plan_review_projection(plan_document)?;
    Ok(build_review_mutation_envelope(
        projection.actions,
        &projection.domains,
    ))
}
