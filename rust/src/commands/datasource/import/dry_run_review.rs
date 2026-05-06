//! Datasource import dry-run review projection helpers.

use serde_json::{Map, Value};

use crate::review_contract::{
    build_review_mutation_envelope, review_action_rank, ReviewMutationAction,
    ReviewMutationActionInput, ReviewMutationEnvelope, REVIEW_ACTION_BLOCKED,
    REVIEW_ACTION_BLOCKED_AMBIGUOUS, REVIEW_ACTION_BLOCKED_READ_ONLY, REVIEW_ACTION_BLOCKED_TARGET,
    REVIEW_ACTION_BLOCKED_UID_MISMATCH, REVIEW_ACTION_SAME, REVIEW_ACTION_WOULD_CREATE,
    REVIEW_ACTION_WOULD_UPDATE, REVIEW_REASON_AMBIGUOUS_LIVE_NAME_MATCH,
    REVIEW_REASON_TARGET_READ_ONLY, REVIEW_REASON_UID_NAME_MISMATCH, REVIEW_STATUS_BLOCKED,
    REVIEW_STATUS_READY, REVIEW_STATUS_SAME, REVIEW_STATUS_WARNING,
};

use super::datasource_import_export_support::DatasourceImportDryRunReport;

#[derive(Debug, Clone)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DatasourceImportDryRunReviewProjection {
    pub(crate) domains: Vec<&'static str>,
    pub(crate) actions: Vec<ReviewMutationAction>,
}

fn optional_string_value(value: &str) -> Value {
    if value.trim().is_empty() {
        Value::Null
    } else {
        Value::String(value.to_string())
    }
}

fn optional_i64_value(value: &str) -> Value {
    value.parse::<i64>().map(Value::from).unwrap_or(Value::Null)
}

fn optional_bool_value(value: &str) -> Value {
    value
        .parse::<bool>()
        .map(Value::from)
        .unwrap_or(Value::Null)
}

fn import_row_value(row: &[String], index: usize) -> &str {
    row.get(index).map(String::as_str).unwrap_or("")
}

fn build_import_row_raw(row: &[String]) -> Value {
    Value::Object(Map::from_iter(vec![
        (
            "uid".to_string(),
            Value::String(import_row_value(row, 0).to_string()),
        ),
        (
            "name".to_string(),
            Value::String(import_row_value(row, 1).to_string()),
        ),
        (
            "type".to_string(),
            Value::String(import_row_value(row, 2).to_string()),
        ),
        (
            "matchBasis".to_string(),
            Value::String(import_row_value(row, 3).to_string()),
        ),
        (
            "destination".to_string(),
            Value::String(import_row_value(row, 4).to_string()),
        ),
        (
            "action".to_string(),
            Value::String(import_row_value(row, 5).to_string()),
        ),
        (
            "orgId".to_string(),
            Value::String(import_row_value(row, 6).to_string()),
        ),
        (
            "file".to_string(),
            Value::String(import_row_value(row, 7).to_string()),
        ),
        (
            "targetUid".to_string(),
            optional_string_value(import_row_value(row, 8)),
        ),
        (
            "targetVersion".to_string(),
            optional_i64_value(import_row_value(row, 9)),
        ),
        (
            "targetReadOnly".to_string(),
            optional_bool_value(import_row_value(row, 10)),
        ),
        (
            "blockedReason".to_string(),
            optional_string_value(import_row_value(row, 11)),
        ),
    ]))
}

fn import_review_identity(row: &[String]) -> String {
    let uid = import_row_value(row, 0).trim();
    if !uid.is_empty() {
        return uid.to_string();
    }
    let name = import_row_value(row, 1).trim();
    if !name.is_empty() {
        return name.to_string();
    }
    let file = import_row_value(row, 7).trim();
    if !file.is_empty() {
        return file.to_string();
    }
    "unknown".to_string()
}

fn import_review_action_id(row: &[String], identity: &str) -> String {
    let org_id = import_row_value(row, 6).trim();
    let file = import_row_value(row, 7).trim();
    let identity_kind = if import_row_value(row, 0).trim().is_empty() {
        "identity"
    } else {
        "uid"
    };
    format!(
        "datasource-import-dry-run:org:{}:file:{}:{}:{}",
        if org_id.is_empty() { "unknown" } else { org_id },
        if file.is_empty() { "unknown" } else { file },
        identity_kind,
        identity
    )
}

fn import_review_details(row: &[String]) -> Option<String> {
    let mut parts = vec![
        format!("matchBasis={}", import_row_value(row, 3)),
        format!("destination={}", import_row_value(row, 4)),
        format!("file={}", import_row_value(row, 7)),
    ];
    if !import_row_value(row, 8).trim().is_empty() {
        parts.push(format!("targetUid={}", import_row_value(row, 8)));
    }
    if !import_row_value(row, 9).trim().is_empty() {
        parts.push(format!("targetVersion={}", import_row_value(row, 9)));
    }
    if !import_row_value(row, 10).trim().is_empty() {
        parts.push(format!("targetReadOnly={}", import_row_value(row, 10)));
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

fn normalize_import_review_action(row: &[String]) -> (&'static str, &'static str, Option<String>) {
    match import_row_value(row, 5) {
        "would-create" => (REVIEW_ACTION_WOULD_CREATE, REVIEW_STATUS_READY, None),
        "would-update" => (REVIEW_ACTION_WOULD_UPDATE, REVIEW_STATUS_READY, None),
        "would-skip-missing" => (REVIEW_ACTION_SAME, REVIEW_STATUS_WARNING, None),
        "blocked-read-only" => (
            REVIEW_ACTION_BLOCKED_READ_ONLY,
            REVIEW_STATUS_BLOCKED,
            Some(REVIEW_REASON_TARGET_READ_ONLY.to_string()),
        ),
        "blocked-target-evidence" => (
            REVIEW_ACTION_BLOCKED_TARGET,
            REVIEW_STATUS_BLOCKED,
            optional_string_value(import_row_value(row, 11))
                .as_str()
                .map(str::to_string),
        ),
        "would-fail-ambiguous" => (
            REVIEW_ACTION_BLOCKED_AMBIGUOUS,
            REVIEW_STATUS_BLOCKED,
            Some(REVIEW_REASON_AMBIGUOUS_LIVE_NAME_MATCH.to_string()),
        ),
        "would-fail-uid-mismatch" => (
            REVIEW_ACTION_BLOCKED_UID_MISMATCH,
            REVIEW_STATUS_BLOCKED,
            Some(REVIEW_REASON_UID_NAME_MISMATCH.to_string()),
        ),
        "same" => (REVIEW_ACTION_SAME, REVIEW_STATUS_SAME, None),
        _ => (
            REVIEW_ACTION_BLOCKED,
            REVIEW_STATUS_BLOCKED,
            optional_string_value(import_row_value(row, 11))
                .as_str()
                .map(str::to_string),
        ),
    }
}

fn import_row_to_review_action(row: &[String]) -> ReviewMutationAction {
    let identity = import_review_identity(row);
    let action_id = import_review_action_id(row, &identity);
    let (action, status, blocked_reason) = normalize_import_review_action(row);
    ReviewMutationActionInput {
        action_id,
        action: action.to_string(),
        domain: "datasource".to_string(),
        resource_kind: "datasource".to_string(),
        identity,
        status: status.to_string(),
        blocked_reason,
        details: import_review_details(row),
        review_hints: Vec::new(),
        raw: build_import_row_raw(row),
    }
    .into()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_datasource_import_dry_run_review_projection(
    report: &DatasourceImportDryRunReport,
) -> DatasourceImportDryRunReviewProjection {
    let mut actions = report
        .rows
        .iter()
        .map(|row| import_row_to_review_action(row))
        .collect::<Vec<_>>();
    actions.sort_by(|left, right| {
        left.kind_order
            .cmp(&right.kind_order)
            .then_with(|| review_action_rank(&left.action).cmp(&review_action_rank(&right.action)))
            .then_with(|| left.identity.cmp(&right.identity))
            .then_with(|| left.action_id.cmp(&right.action_id))
    });
    DatasourceImportDryRunReviewProjection {
        domains: vec!["datasource"],
        actions,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_datasource_import_dry_run_review_envelope(
    report: &DatasourceImportDryRunReport,
) -> ReviewMutationEnvelope {
    let projection = build_datasource_import_dry_run_review_projection(report);
    build_review_mutation_envelope(projection.actions, &projection.domains)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datasource::datasource_import_export::datasource_import_dry_run_output;
    use crate::datasource::{DatasourceImportDryRunReport, DatasourceImportInputFormat};
    use crate::review_contract::{
        build_review_mutation_summary_rows, REVIEW_ACTION_BLOCKED_READ_ONLY,
        REVIEW_ACTION_WOULD_UPDATE, REVIEW_REASON_TARGET_READ_ONLY, REVIEW_STATUS_BLOCKED,
        REVIEW_STATUS_READY,
    };
    use serde_json::json;
    use std::path::PathBuf;

    #[test]
    fn datasource_import_dry_run_review_projection_and_envelope_preserve_row_evidence() {
        let report = DatasourceImportDryRunReport {
            mode: "create-or-update".to_string(),
            input_dir: PathBuf::from("/tmp/datasources"),
            input_format: DatasourceImportInputFormat::Inventory,
            source_org_id: "1".to_string(),
            target_org_id: "7".to_string(),
            rows: vec![
                vec![
                    "prom-main".to_string(),
                    "Prometheus Main".to_string(),
                    "prometheus".to_string(),
                    "uid".to_string(),
                    "exists-uid".to_string(),
                    "would-update".to_string(),
                    "7".to_string(),
                    "datasources.json#0".to_string(),
                    "prom-main".to_string(),
                    "12".to_string(),
                    "false".to_string(),
                    String::new(),
                ],
                vec![
                    "loki-main".to_string(),
                    "Loki Main".to_string(),
                    "loki".to_string(),
                    "name".to_string(),
                    "exists-name".to_string(),
                    "blocked-read-only".to_string(),
                    "7".to_string(),
                    "datasources.json#1".to_string(),
                    "loki-live".to_string(),
                    "4".to_string(),
                    "true".to_string(),
                    "blocked-read-only".to_string(),
                ],
            ],
            datasource_count: 2,
            would_create: 0,
            would_update: 1,
            would_skip: 0,
            would_block: 1,
        };

        let projection = build_datasource_import_dry_run_review_projection(&report);

        assert_eq!(projection.domains, vec!["datasource"]);
        assert_eq!(projection.actions.len(), 2);

        let ready = &projection.actions[0];
        assert_eq!(
            ready.action_id,
            "datasource-import-dry-run:org:7:file:datasources.json#0:uid:prom-main"
        );
        assert_eq!(ready.action, REVIEW_ACTION_WOULD_UPDATE);
        assert_eq!(ready.status, REVIEW_STATUS_READY);
        assert_eq!(ready.identity, "prom-main");
        assert_eq!(ready.blocked_reason, None);
        assert_eq!(
            ready.details.as_deref(),
            Some("matchBasis=uid destination=exists-uid file=datasources.json#0 targetUid=prom-main targetVersion=12 targetReadOnly=false")
        );
        assert_eq!(
            ready.raw,
            json!({
                "uid": "prom-main",
                "name": "Prometheus Main",
                "type": "prometheus",
                "matchBasis": "uid",
                "destination": "exists-uid",
                "action": "would-update",
                "orgId": "7",
                "file": "datasources.json#0",
                "targetUid": "prom-main",
                "targetVersion": 12,
                "targetReadOnly": false,
                "blockedReason": null,
            })
        );

        let blocked = &projection.actions[1];
        assert_eq!(
            blocked.action_id,
            "datasource-import-dry-run:org:7:file:datasources.json#1:uid:loki-main"
        );
        assert_eq!(blocked.action, REVIEW_ACTION_BLOCKED_READ_ONLY);
        assert_eq!(blocked.status, REVIEW_STATUS_BLOCKED);
        assert_eq!(blocked.identity, "loki-main");
        assert_eq!(
            blocked.blocked_reason.as_deref(),
            Some(REVIEW_REASON_TARGET_READ_ONLY)
        );
        assert_eq!(
            blocked.details.as_deref(),
            Some("matchBasis=name destination=exists-name file=datasources.json#1 targetUid=loki-live targetVersion=4 targetReadOnly=true")
        );
        assert_eq!(
            blocked.raw,
            json!({
                "uid": "loki-main",
                "name": "Loki Main",
                "type": "loki",
                "matchBasis": "name",
                "destination": "exists-name",
                "action": "blocked-read-only",
                "orgId": "7",
                "file": "datasources.json#1",
                "targetUid": "loki-live",
                "targetVersion": 4,
                "targetReadOnly": true,
                "blockedReason": "blocked-read-only",
            })
        );

        let envelope = build_datasource_import_dry_run_review_envelope(&report);
        assert_eq!(envelope.actions, projection.actions);
        assert_eq!(envelope.summary.action_count, 2);
        assert_eq!(envelope.summary.blocked_count, 1);
        assert_eq!(envelope.domains.len(), 1);
        assert_eq!(envelope.domains[0].id, "datasource");
        assert_eq!(envelope.domains[0].update, 1);
        assert_eq!(envelope.domains[0].blocked, 1);
        assert_eq!(
            envelope.blocked_reasons,
            vec![REVIEW_REASON_TARGET_READ_ONLY.to_string()]
        );
    }

    #[test]
    fn datasource_import_dry_run_review_envelope_feeds_shared_summary_rows_without_json_drift() {
        let report = DatasourceImportDryRunReport {
            mode: "create-or-update".to_string(),
            input_dir: PathBuf::from("/tmp/datasources"),
            input_format: DatasourceImportInputFormat::Inventory,
            source_org_id: "1".to_string(),
            target_org_id: "7".to_string(),
            rows: vec![vec![
                "prom-main".to_string(),
                "Prometheus Main".to_string(),
                "prometheus".to_string(),
                "uid".to_string(),
                "exists-uid".to_string(),
                "would-update".to_string(),
                "7".to_string(),
                "datasources.json#0".to_string(),
                "prom-main".to_string(),
                "12".to_string(),
                "false".to_string(),
                String::new(),
            ]],
            datasource_count: 1,
            would_create: 0,
            would_update: 1,
            would_skip: 0,
            would_block: 0,
        };
        let public_json_before =
            datasource_import_dry_run_output::build_datasource_import_dry_run_json_value(&report);

        let envelope = build_datasource_import_dry_run_review_envelope(&report);
        let summary_rows = build_review_mutation_summary_rows(&envelope);

        assert_eq!(summary_rows.len(), 1);
        assert_eq!(summary_rows[0].domain, "datasource");
        assert_eq!(summary_rows[0].resource_kind, "datasource");
        assert_eq!(summary_rows[0].identity, "prom-main");
        assert_eq!(summary_rows[0].action, REVIEW_ACTION_WOULD_UPDATE);
        assert_eq!(summary_rows[0].status, REVIEW_STATUS_READY);
        assert_eq!(
            summary_rows[0].details.as_deref(),
            Some("matchBasis=uid destination=exists-uid file=datasources.json#0 targetUid=prom-main targetVersion=12 targetReadOnly=false")
        );
        assert_eq!(summary_rows[0].action_count, 1);
        assert_eq!(summary_rows[0].domain_count, 1);
        assert_eq!(summary_rows[0].blocked_count, 0);
        assert_eq!(summary_rows[0].warning_count, 0);
        assert!(summary_rows[0].blocked_reasons.is_empty());
        assert_eq!(
            datasource_import_dry_run_output::build_datasource_import_dry_run_json_value(&report),
            public_json_before
        );
        assert_eq!(
            public_json_before["datasources"][0],
            json!({
                "uid": "prom-main",
                "name": "Prometheus Main",
                "type": "prometheus",
                "matchBasis": "uid",
                "destination": "exists-uid",
                "action": "would-update",
                "orgId": "7",
                "file": "datasources.json#0",
                "targetUid": "prom-main",
                "targetVersion": 12,
                "targetReadOnly": false,
                "blockedReason": null,
            })
        );
    }
}
