//! Datasource import dry-run report collection helpers.

use crate::common::{message, Result};
use crate::dashboard::DEFAULT_ORG_ID;
use crate::datasource::resolve_match;
use crate::grafana_api::DatasourceResourceClient;

use super::datasource_import_export_support::DatasourceImportDryRunReport;
use super::datasource_import_plan::fetch_update_target_evidence;
use super::{
    describe_datasource_import_mode, fetch_current_org, load_import_records,
    validate_matching_export_org, DatasourceImportArgs,
};

pub(crate) fn collect_datasource_import_dry_run_report(
    client: &crate::http::JsonHttpClient,
    args: &DatasourceImportArgs,
) -> Result<DatasourceImportDryRunReport> {
    let replace_existing = args.replace_existing || args.update_existing_only;
    let input_dir = args
        .input_dir
        .as_ref()
        .ok_or_else(|| message("Datasource import dry-run requires --input-dir or --local."))?;
    let (metadata, records) = load_import_records(input_dir, args.input_format)?;
    validate_matching_export_org(client, args, &records)?;
    let live = DatasourceResourceClient::new(client).list_datasources()?;
    let target_org = fetch_current_org(client)?;
    let target_org_id = target_org
        .get("id")
        .map(|value| value.to_string())
        .unwrap_or_else(|| DEFAULT_ORG_ID.to_string());
    let mode = describe_datasource_import_mode(args.replace_existing, args.update_existing_only);
    let mut rows = Vec::new();
    let mut created = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;
    let mut blocked = 0usize;
    for (index, record) in records.iter().enumerate() {
        let matching = resolve_match(record, &live, replace_existing, args.update_existing_only);
        let file_ref = format!("{}#{}", metadata.datasources_file, index);
        let mut action = matching.action.to_string();
        let mut target_uid = matching.target_uid.clone();
        let mut target_version = String::new();
        let mut target_read_only = String::new();
        let mut blocked_reason = String::new();
        if matching.action == "would-update" {
            let identity = if record.uid.is_empty() {
                record.name.as_str()
            } else {
                record.uid.as_str()
            };
            match fetch_update_target_evidence(client, &matching.target_uid, identity) {
                Ok(target) => {
                    target_uid = target.uid;
                    if let Some(version) = target.version {
                        target_version = version.to_string();
                    }
                    target_read_only = target.read_only.to_string();
                    if target.read_only {
                        action = "blocked-read-only".to_string();
                        blocked_reason = "blocked-read-only".to_string();
                    }
                }
                Err(error) => {
                    action = "blocked-target-evidence".to_string();
                    blocked_reason = error.to_string();
                }
            }
        }
        rows.push(vec![
            record.uid.clone(),
            record.name.clone(),
            record.datasource_type.clone(),
            matching.match_basis.to_string(),
            matching.destination.to_string(),
            action.clone(),
            target_org_id.clone(),
            file_ref,
            target_uid,
            target_version,
            target_read_only,
            blocked_reason,
        ]);
        match action.as_str() {
            "would-create" => created += 1,
            "would-update" => updated += 1,
            "would-skip-missing" => skipped += 1,
            _ => blocked += 1,
        }
    }
    Ok(DatasourceImportDryRunReport {
        mode: mode.to_string(),
        input_dir: input_dir.clone(),
        input_format: args.input_format,
        source_org_id: records
            .iter()
            .find(|item| !item.org_id.is_empty())
            .map(|item| item.org_id.clone())
            .unwrap_or_default(),
        target_org_id,
        rows,
        datasource_count: records.len(),
        would_create: created,
        would_update: updated,
        would_skip: skipped,
        would_block: blocked,
    })
}
