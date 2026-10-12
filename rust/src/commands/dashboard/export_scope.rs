use std::collections::BTreeSet;
use std::fs;

use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, string_field, Result};

use super::super::export_prompt::build_external_export_document_with_library_panels;
use super::super::history::{
    build_dashboard_history_export_document_from_current_with_request,
    build_dashboard_history_export_document_from_current_with_version_fetcher,
    DashboardHistoryExportDocument, DashboardHistoryExportVersion,
};
use super::super::list::{
    attach_dashboard_org_metadata, collect_dashboard_ownership_provenance,
    collect_dashboard_source_metadata, fetch_current_org_with_request,
};
use super::super::{
    build_dashboard_index_item, build_datasource_catalog, build_datasource_inventory_record,
    build_export_metadata, build_preserved_web_import_document,
    build_root_export_index_with_resources, build_variant_index,
    collect_folder_inventory_with_request, fetch_dashboard_with_request,
    list_dashboard_summaries_with_request, list_datasources_with_request, write_dashboard,
    write_json_document, ExportArgs, ExportOrgSummary, RootExportIndex, CLASSIC_EXPORT_SUBDIR,
    DASHBOARD_PERMISSION_BUNDLE_FILENAME, DATASOURCE_INVENTORY_FILENAME, DEFAULT_UNKNOWN_UID,
    EXPORT_METADATA_FILENAME, FOLDER_INVENTORY_FILENAME, PROMPT_EXPORT_SUBDIR,
    PROVISIONING_EXPORT_SUBDIR, RAW_EXPORT_SUBDIR, RESOURCE_OBJECTS_SUBDIR,
    RESOURCE_V1_EXPORT_SUBDIR, RESOURCE_V2_EXPORT_SUBDIR,
};
use super::export_paths::{
    build_export_variant_dirs, build_folder_paths_by_inventory_key,
    build_history_version_output_path, build_output_path, build_output_path_with_folder_path,
    export_folder_inventory_key,
};
use super::export_render::{format_export_progress_line, format_export_verbose_line};
use super::{
    build_all_orgs_output_dir, build_permission_bundle_document, build_provisioning_config,
    build_used_datasource_summaries, collect_library_panel_exports_with_request,
    collect_permission_export_documents, collect_permission_export_documents_with_fetcher,
    write_yaml_document, PermissionExportTarget,
};

#[path = "export_scope/document_write.rs"]
mod document_write;

use document_write::write_history_document;

use super::export_resource::{
    build_resource_export_metadata, build_resource_index_entry, build_resource_output_path,
    dashboard_resource_export_document, dashboard_resource_namespace,
    discover_resource_lanes_with_request, fetch_dashboard_resource_with_request,
    list_dashboard_resources_with_request, selected_resource_lanes,
    validate_portable_v2_export_args, validate_portable_v2_served_lanes, ResourceExportLane,
};

pub(crate) struct ScopeExportResult {
    pub(crate) exported_count: usize,
    pub(crate) root_index: Option<RootExportIndex>,
    pub(crate) org_summary: Option<ExportOrgSummary>,
}

fn single_version_history_document(
    source: &DashboardHistoryExportDocument,
    version: &DashboardHistoryExportVersion,
) -> DashboardHistoryExportDocument {
    DashboardHistoryExportDocument {
        kind: source.kind.clone(),
        schema_version: source.schema_version,
        tool_version: source.tool_version.clone(),
        dashboard_uid: source.dashboard_uid.clone(),
        current_version: source.current_version,
        current_title: source.current_title.clone(),
        version_count: 1,
        versions: vec![version.clone()],
    }
}

fn clean_stale_history_version_files(history_dir: &std::path::Path, uid: &str) -> Result<()> {
    let version_dir = history_dir.join(crate::common::sanitize_path_component(uid));
    if !version_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(version_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with('v') && name.ends_with(".history.json") {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub(crate) fn export_dashboards_in_scope_with_request<F>(
    request_json: &mut F,
    args: &ExportArgs,
    org: Option<&Map<String, Value>>,
    org_id_override: Option<i64>,
) -> Result<ScopeExportResult>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    export_dashboards_in_scope_with_optional_fetchers(
        request_json,
        args,
        org,
        org_id_override,
        Option::<&fn(&PermissionExportTarget) -> Result<Vec<Map<String, Value>>>>::None,
        Option::<&fn(&str, i64) -> Result<Map<String, Value>>>::None,
        1,
    )
}

pub(crate) fn export_dashboards_in_scope_with_fetchers<F, P, H>(
    request_json: &mut F,
    args: &ExportArgs,
    org: Option<&Map<String, Value>>,
    org_id_override: Option<i64>,
    permission_fetcher: &P,
    history_version_fetcher: &H,
    history_concurrency: usize,
) -> Result<ScopeExportResult>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
    P: Fn(&PermissionExportTarget) -> Result<Vec<Map<String, Value>>> + Sync,
    H: Fn(&str, i64) -> Result<Map<String, Value>> + Sync,
{
    export_dashboards_in_scope_with_optional_fetchers(
        request_json,
        args,
        org,
        org_id_override,
        Some(permission_fetcher),
        Some(history_version_fetcher),
        history_concurrency,
    )
}

fn export_dashboards_in_scope_with_optional_fetchers<F, P, H>(
    request_json: &mut F,
    args: &ExportArgs,
    org: Option<&Map<String, Value>>,
    org_id_override: Option<i64>,
    permission_fetcher: Option<&P>,
    history_version_fetcher: Option<&H>,
    history_concurrency: usize,
) -> Result<ScopeExportResult>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
    P: Fn(&PermissionExportTarget) -> Result<Vec<Map<String, Value>>> + Sync,
    H: Fn(&str, i64) -> Result<Map<String, Value>> + Sync,
{
    validate_portable_v2_export_args(args)?;
    let requested_resource_lanes = selected_resource_lanes(args.resource_format)?;
    let has_classic_export = !(args.without_dashboard_raw
        && args.without_dashboard_prompt
        && args.without_dashboard_provisioning);
    if !has_classic_export && requested_resource_lanes.is_empty() {
        return Err(message(
            "Nothing to export. Remove one of --without-raw, --without-prompt, or --without-provisioning, or set --resource-format.",
        ));
    }
    if !has_classic_export && args.resource_namespace.is_none() {
        return Err(message(
            "Resource-only export requires --resource-namespace so it can use App Platform APIs without resolving the current Grafana org.",
        ));
    }
    let mut scoped_request = |method: Method,
                              path: &str,
                              params: &[(String, String)],
                              payload: Option<&Value>|
     -> Result<Option<Value>> {
        let mut scoped_params = params.to_vec();
        if let Some(org_id) = org_id_override {
            scoped_params.push(("orgId".to_string(), org_id.to_string()));
        }
        request_json(method, path, &scoped_params, payload)
    };
    let resource_only_explicit_namespace =
        !has_classic_export && args.resource_namespace.as_deref().is_some();
    let current_org = if resource_only_explicit_namespace {
        org.cloned().unwrap_or_else(|| {
            let mut fallback = Map::new();
            fallback.insert(
                "id".to_string(),
                Value::String(
                    org_id_override
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| DEFAULT_UNKNOWN_UID.to_string()),
                ),
            );
            fallback.insert(
                "name".to_string(),
                Value::String(args.resource_namespace.clone().unwrap_or_default()),
            );
            fallback
        })
    } else {
        match org {
            Some(org) => org.clone(),
            None => fetch_current_org_with_request(&mut scoped_request)?,
        }
    };
    let current_org_name = string_field(&current_org, "name", "org");
    let current_org_id = current_org
        .get("id")
        .map(|value| match value {
            Value::String(text) => text.clone(),
            _ => value.to_string(),
        })
        .unwrap_or_else(|| DEFAULT_UNKNOWN_UID.to_string());
    let scope_output_dir = if args.all_orgs {
        build_all_orgs_output_dir(&args.output_dir, &current_org)
    } else {
        args.output_dir.clone()
    };
    let family_layout = !requested_resource_lanes.is_empty();
    let classic_output_dir = if family_layout {
        scope_output_dir.join(CLASSIC_EXPORT_SUBDIR)
    } else {
        scope_output_dir.clone()
    };
    let (raw_dir, prompt_dir, provisioning_dir) = build_export_variant_dirs(&classic_output_dir);
    let history_dir = classic_output_dir.join("history");
    let resource_v1_dir = scope_output_dir.join(RESOURCE_V1_EXPORT_SUBDIR);
    let resource_v1_objects_dir = resource_v1_dir.join(RESOURCE_OBJECTS_SUBDIR);
    let resource_v2_dir = scope_output_dir.join(RESOURCE_V2_EXPORT_SUBDIR);
    let resource_v2_objects_dir = resource_v2_dir.join(RESOURCE_OBJECTS_SUBDIR);
    let provisioning_dashboards_dir = provisioning_dir.join("dashboards");
    let provisioning_config_dir = provisioning_dir.join("provisioning");
    if !args.dry_run && !args.without_dashboard_raw {
        fs::create_dir_all(&raw_dir)?;
    }
    if !args.dry_run && !args.without_dashboard_prompt {
        fs::create_dir_all(&prompt_dir)?;
    }
    if !args.dry_run && !args.without_dashboard_provisioning {
        fs::create_dir_all(&provisioning_dashboards_dir)?;
        fs::create_dir_all(&provisioning_config_dir)?;
    }
    if !args.dry_run && args.include_history {
        fs::create_dir_all(&history_dir)?;
    }
    let namespace = args
        .resource_namespace
        .clone()
        .unwrap_or_else(|| dashboard_resource_namespace(&current_org_id));
    if !has_classic_export {
        let resource_exports = list_dashboard_resources_with_request(
            &mut scoped_request,
            args.resource_format,
            &namespace,
            args.page_size,
        )?;
        validate_portable_v2_served_lanes(
            args.portable_v2,
            &resource_exports
                .iter()
                .map(|(lane, _)| *lane)
                .collect::<Vec<_>>(),
        )?;
        if resource_exports.is_empty() {
            return Err(message(
                "None of the requested dashboard resource API versions are served by this Grafana instance.",
            ));
        }
        return export_resource_only_dashboards_in_scope(
            args,
            &current_org_name,
            &current_org_id,
            &scope_output_dir,
            resource_exports,
        );
    }
    let resource_lanes = discover_resource_lanes_with_request(
        &mut scoped_request,
        args.resource_format,
        &namespace,
    )?;
    validate_portable_v2_served_lanes(args.portable_v2, &resource_lanes)?;
    if !has_classic_export && resource_lanes.is_empty() {
        return Err(message(
            "None of the requested dashboard resource API versions are served by this Grafana instance.",
        ));
    }
    if !args.dry_run {
        for lane in &resource_lanes {
            fs::create_dir_all(
                scope_output_dir
                    .join(lane.dir_name)
                    .join(RESOURCE_OBJECTS_SUBDIR),
            )?;
        }
    }
    let datasource_list = list_datasources_with_request(&mut scoped_request)?;
    let datasource_inventory = datasource_list
        .iter()
        .map(|datasource| build_datasource_inventory_record(datasource, &current_org))
        .collect::<Vec<_>>();
    let datasource_catalog = build_datasource_catalog(&datasource_list);

    let summaries = list_dashboard_summaries_with_request(&mut scoped_request, args.page_size)?;
    if summaries.is_empty() {
        return Ok(ScopeExportResult {
            exported_count: 0,
            root_index: None,
            org_summary: None,
        });
    }
    let summaries = attach_dashboard_org_metadata(&summaries, &current_org);
    let folder_inventory = collect_folder_inventory_with_request(&mut scoped_request, &summaries)?;
    let folder_paths_by_key = build_folder_paths_by_inventory_key(&folder_inventory);
    let permission_documents = if args.without_dashboard_raw || args.dry_run {
        Vec::new()
    } else if let Some(permission_fetcher) = permission_fetcher {
        collect_permission_export_documents_with_fetcher(
            &summaries,
            &folder_inventory,
            permission_fetcher,
        )?
    } else {
        collect_permission_export_documents(&mut scoped_request, &summaries, &folder_inventory)?
    };

    let mut exported_count = 0;
    let mut index_items = Vec::new();
    let mut resource_v1_index_items = Vec::new();
    let mut resource_v2_index_items = Vec::new();
    let mut used_source_names = BTreeSet::new();
    let mut used_source_uids = BTreeSet::new();
    let total = summaries.len();
    for (index, summary) in summaries.into_iter().enumerate() {
        let uid = string_field(&summary, "uid", "");
        if uid.is_empty() {
            continue;
        }
        if args.verbose {
            // Verbose mode prints per-variant details after each write, which already
            // includes status for this iteration; suppressing progress lines avoids
            // duplicated/noisy output and keeps verbose logs ordered.
        } else if args.progress {
            println!(
                "{}",
                format_export_progress_line(index + 1, total, &uid, args.dry_run)
            );
        }
        let payload = fetch_dashboard_with_request(&mut scoped_request, &uid)?;
        let ownership = collect_dashboard_ownership_provenance(&payload)?;
        let (source_names, source_uids) =
            collect_dashboard_source_metadata(&payload, &datasource_catalog)?;
        used_source_names.extend(source_names);
        used_source_uids.extend(source_uids);
        if args.include_history {
            let history_document = match history_version_fetcher {
                Some(fetcher) => {
                    build_dashboard_history_export_document_from_current_with_version_fetcher(
                        &mut scoped_request,
                        &uid,
                        20,
                        &payload,
                        |version| fetcher(&uid, version),
                        history_concurrency,
                    )?
                }
                None => build_dashboard_history_export_document_from_current_with_request(
                    &mut scoped_request,
                    &uid,
                    20,
                    &payload,
                )?,
            };
            if !args.dry_run && args.overwrite {
                clean_stale_history_version_files(&history_dir, &uid)?;
            }
            for version in &history_document.versions {
                let history_path =
                    build_history_version_output_path(&history_dir, &uid, version.version);
                let single_version_document =
                    single_version_history_document(&history_document, version);
                if !args.dry_run {
                    write_history_document(
                        &single_version_document,
                        &history_path,
                        args.overwrite,
                    )?;
                }
                if args.verbose {
                    println!(
                        "{}",
                        format_export_verbose_line("history", &uid, &history_path, args.dry_run)
                    );
                }
            }
        }
        let mut item = build_dashboard_index_item(&summary, &uid);
        let folder_path = export_folder_inventory_key(&summary)
            .and_then(|key| folder_paths_by_key.get(&key))
            .map(String::as_str);
        item.folder_path = folder_path.unwrap_or("").to_string();
        item.ownership = ownership.ownership;
        item.provenance = ownership.provenance;
        if !args.without_dashboard_raw {
            let raw_document = build_preserved_web_import_document(&payload)?;
            let raw_path =
                build_output_path_with_folder_path(&raw_dir, &summary, folder_path, args.flat);
            if !args.dry_run {
                write_dashboard(&raw_document, &raw_path, args.overwrite)?;
            }
            if args.verbose {
                println!(
                    "{}",
                    format_export_verbose_line("raw", &uid, &raw_path, args.dry_run)
                );
            }
            item.raw_path = Some(raw_path.display().to_string());
        }
        if !args.without_dashboard_prompt {
            let (library_panel_exports, library_panel_warnings) =
                collect_library_panel_exports_with_request(&mut scoped_request, &payload)?;
            for warning in library_panel_warnings {
                eprintln!("Dashboard export prompt warning: {warning}");
            }
            let prompt_document = build_external_export_document_with_library_panels(
                &payload,
                &datasource_catalog,
                Some(&library_panel_exports),
            )?;
            let prompt_path =
                build_output_path_with_folder_path(&prompt_dir, &summary, folder_path, args.flat);
            if !args.dry_run {
                write_dashboard(&prompt_document, &prompt_path, args.overwrite)?;
            }
            if args.verbose {
                println!(
                    "{}",
                    format_export_verbose_line("prompt", &uid, &prompt_path, args.dry_run)
                );
            }
            item.prompt_path = Some(prompt_path.display().to_string());
        }
        if !args.without_dashboard_provisioning {
            let provisioning_document = build_preserved_web_import_document(&payload)?;
            let provisioning_path =
                build_output_path(&provisioning_dashboards_dir, &summary, false);
            if !args.dry_run {
                write_dashboard(&provisioning_document, &provisioning_path, args.overwrite)?;
            }
            if args.verbose {
                println!(
                    "{}",
                    format_export_verbose_line(
                        "provisioning",
                        &uid,
                        &provisioning_path,
                        args.dry_run
                    )
                );
            }
            item.provisioning_path = Some(provisioning_path.display().to_string());
        }
        if !resource_lanes.is_empty() {
            for lane in resource_lanes.iter().copied() {
                let resource = fetch_dashboard_resource_with_request(
                    &mut scoped_request,
                    lane,
                    &namespace,
                    &uid,
                )?;
                let resource = dashboard_resource_export_document(
                    &resource,
                    lane,
                    &namespace,
                    args.portable_v2,
                )?;
                let objects_dir = if lane.dir_name == RESOURCE_V2_EXPORT_SUBDIR {
                    &resource_v2_objects_dir
                } else {
                    &resource_v1_objects_dir
                };
                let (resource_path, resource_name, resource_title, resource_folder_path) =
                    build_resource_output_path(
                        objects_dir,
                        &resource,
                        &uid,
                        &folder_paths_by_key,
                        &current_org_id,
                    );
                if !args.dry_run {
                    write_dashboard(&resource, &resource_path, args.overwrite)?;
                }
                if args.verbose {
                    println!(
                        "{}",
                        format_export_verbose_line(
                            lane.dir_name,
                            &uid,
                            &resource_path,
                            args.dry_run
                        )
                    );
                }
                if lane.dir_name == RESOURCE_V1_EXPORT_SUBDIR {
                    item.resource_v1_path = Some(resource_path.display().to_string());
                    resource_v1_index_items.push(build_resource_index_entry(
                        lane,
                        resource_name,
                        resource_title,
                        &resource_path,
                        resource_folder_path,
                        &current_org_name,
                        &current_org_id,
                    ));
                } else if lane.dir_name == RESOURCE_V2_EXPORT_SUBDIR {
                    item.resource_v2_path = Some(resource_path.display().to_string());
                    resource_v2_index_items.push(build_resource_index_entry(
                        lane,
                        resource_name,
                        resource_title,
                        &resource_path,
                        resource_folder_path,
                        &current_org_name,
                        &current_org_id,
                    ));
                }
            }
        }
        exported_count += 1;
        index_items.push(item);
    }

    let mut raw_index_path = None;
    if !args.without_dashboard_raw {
        let index_path = raw_dir.join("index.json");
        let metadata_path = raw_dir.join(EXPORT_METADATA_FILENAME);
        let folder_inventory_path = raw_dir.join(FOLDER_INVENTORY_FILENAME);
        let datasource_inventory_path = raw_dir.join(DATASOURCE_INVENTORY_FILENAME);
        let permission_bundle_path = raw_dir.join(DASHBOARD_PERMISSION_BUNDLE_FILENAME);
        if !args.dry_run {
            write_json_document(
                &build_variant_index(
                    &index_items,
                    |item| item.raw_path.as_deref(),
                    "grafana-web-import-preserve-uid",
                ),
                &index_path,
            )?;
            write_json_document(
                &build_export_metadata(
                    RAW_EXPORT_SUBDIR,
                    index_items
                        .iter()
                        .filter(|item| item.raw_path.is_some())
                        .count(),
                    Some("grafana-web-import-preserve-uid"),
                    Some(FOLDER_INVENTORY_FILENAME),
                    Some(DATASOURCE_INVENTORY_FILENAME),
                    Some(DASHBOARD_PERMISSION_BUNDLE_FILENAME),
                    Some(&current_org_name),
                    Some(&current_org_id),
                    None,
                    "live",
                    Some(&args.common.url),
                    None,
                    args.common.profile.as_deref(),
                    raw_dir.as_path(),
                    &metadata_path,
                ),
                &metadata_path,
            )?;
            write_json_document(&folder_inventory, &folder_inventory_path)?;
            write_json_document(&datasource_inventory, &datasource_inventory_path)?;
            write_json_document(
                &build_permission_bundle_document(&permission_documents),
                &permission_bundle_path,
            )?;
        }
        raw_index_path = Some(index_path);
    }
    let mut prompt_index_path = None;
    if !args.without_dashboard_prompt {
        let index_path = prompt_dir.join("index.json");
        let metadata_path = prompt_dir.join(EXPORT_METADATA_FILENAME);
        if !args.dry_run {
            write_json_document(
                &build_variant_index(
                    &index_items,
                    |item| item.prompt_path.as_deref(),
                    "grafana-web-import-with-datasource-inputs",
                ),
                &index_path,
            )?;
            write_json_document(
                &build_export_metadata(
                    PROMPT_EXPORT_SUBDIR,
                    index_items
                        .iter()
                        .filter(|item| item.prompt_path.is_some())
                        .count(),
                    Some("grafana-web-import-with-datasource-inputs"),
                    None,
                    None,
                    None,
                    Some(&current_org_name),
                    Some(&current_org_id),
                    None,
                    "live",
                    Some(&args.common.url),
                    None,
                    args.common.profile.as_deref(),
                    prompt_dir.as_path(),
                    &metadata_path,
                ),
                &metadata_path,
            )?;
        }
        prompt_index_path = Some(index_path);
    }
    let mut provisioning_index_path = None;
    if !args.without_dashboard_provisioning {
        let index_path = provisioning_dir.join("index.json");
        let metadata_path = provisioning_dir.join(EXPORT_METADATA_FILENAME);
        let config_path = provisioning_config_dir.join("dashboards.yaml");
        let provisioning_provider_org_id = args
            .provisioning_provider_org_id
            .unwrap_or_else(|| current_org_id.parse::<i64>().unwrap_or(1));
        let provisioning_provider_path = args
            .provisioning_provider_path
            .as_deref()
            .unwrap_or(&provisioning_dashboards_dir);
        if !args.dry_run {
            write_json_document(
                &build_variant_index(
                    &index_items,
                    |item| item.provisioning_path.as_deref(),
                    "grafana-file-provisioning-dashboard",
                ),
                &index_path,
            )?;
            write_json_document(
                &build_export_metadata(
                    PROVISIONING_EXPORT_SUBDIR,
                    index_items
                        .iter()
                        .filter(|item| item.provisioning_path.is_some())
                        .count(),
                    Some("grafana-file-provisioning-dashboard"),
                    None,
                    None,
                    None,
                    Some(&current_org_name),
                    Some(&current_org_id),
                    None,
                    "live",
                    Some(&args.common.url),
                    None,
                    args.common.profile.as_deref(),
                    provisioning_dir.as_path(),
                    &metadata_path,
                ),
                &metadata_path,
            )?;
            write_yaml_document(
                &build_provisioning_config(
                    provisioning_provider_org_id,
                    &args.provisioning_provider_name,
                    provisioning_provider_path,
                    args.provisioning_provider_disable_deletion,
                    args.provisioning_provider_allow_ui_updates,
                    args.provisioning_provider_update_interval_seconds,
                ),
                &config_path,
                args.overwrite,
            )?;
        }
        provisioning_index_path = Some(index_path);
    }
    let mut resource_v1_index_path = None;
    if let Some(lane) = resource_lanes
        .iter()
        .copied()
        .find(|lane| lane.dir_name == RESOURCE_V1_EXPORT_SUBDIR)
    {
        let index_path = resource_v1_dir.join("index.json");
        let metadata_path = resource_v1_dir.join(EXPORT_METADATA_FILENAME);
        if !args.dry_run {
            write_json_document(&resource_v1_index_items, &index_path)?;
            write_json_document(
                &build_resource_export_metadata(
                    lane,
                    resource_v1_index_items.len(),
                    &current_org_name,
                    &current_org_id,
                    &args.common.url,
                    args.common.profile.as_deref(),
                    resource_v1_dir.as_path(),
                    &metadata_path,
                ),
                &metadata_path,
            )?;
        }
        resource_v1_index_path = Some(index_path);
    }
    let mut resource_v2_index_path = None;
    if let Some(lane) = resource_lanes
        .iter()
        .copied()
        .find(|lane| lane.dir_name == RESOURCE_V2_EXPORT_SUBDIR)
    {
        let index_path = resource_v2_dir.join("index.json");
        let metadata_path = resource_v2_dir.join(EXPORT_METADATA_FILENAME);
        if !args.dry_run {
            write_json_document(&resource_v2_index_items, &index_path)?;
            write_json_document(
                &build_resource_export_metadata(
                    lane,
                    resource_v2_index_items.len(),
                    &current_org_name,
                    &current_org_id,
                    &args.common.url,
                    args.common.profile.as_deref(),
                    resource_v2_dir.as_path(),
                    &metadata_path,
                ),
                &metadata_path,
            )?;
        }
        resource_v2_index_path = Some(index_path);
    }
    let root_index = build_root_export_index_with_resources(
        &index_items,
        raw_index_path.as_deref(),
        prompt_index_path.as_deref(),
        provisioning_index_path.as_deref(),
        resource_v1_index_path.as_deref(),
        resource_v2_index_path.as_deref(),
        &folder_inventory,
    );
    let used_datasources = build_used_datasource_summaries(
        &datasource_inventory,
        &used_source_names,
        &used_source_uids,
    );
    let org_summary = ExportOrgSummary {
        org: current_org_name.clone(),
        org_id: current_org_id.clone(),
        dashboard_count: index_items.len() as u64,
        datasource_count: Some(datasource_inventory.len() as u64),
        used_datasource_count: Some(used_datasources.len() as u64),
        used_datasources: Some(used_datasources),
        output_dir: if args.all_orgs {
            Some(scope_output_dir.display().to_string())
        } else {
            None
        },
    };
    if !args.dry_run {
        write_json_document(&root_index, &scope_output_dir.join("index.json"))?;
        write_json_document(
            &build_export_metadata(
                "root",
                index_items.len(),
                None,
                None,
                None,
                None,
                Some(&current_org_name),
                Some(&current_org_id),
                None,
                "live",
                Some(&args.common.url),
                None,
                args.common.profile.as_deref(),
                scope_output_dir.as_path(),
                &scope_output_dir.join(EXPORT_METADATA_FILENAME),
            ),
            &scope_output_dir.join(EXPORT_METADATA_FILENAME),
        )?;
    }
    Ok(ScopeExportResult {
        exported_count,
        root_index: Some(root_index),
        org_summary: Some(org_summary),
    })
}

fn export_resource_only_dashboards_in_scope(
    args: &ExportArgs,
    current_org_name: &str,
    current_org_id: &str,
    scope_output_dir: &std::path::Path,
    resource_exports: Vec<(ResourceExportLane, Vec<Value>)>,
) -> Result<ScopeExportResult> {
    let mut index_items = Vec::new();
    let mut item_positions = std::collections::BTreeMap::new();
    let mut resource_v1_index_items = Vec::new();
    let mut resource_v2_index_items = Vec::new();
    let folder_paths_by_key = std::collections::BTreeMap::new();

    for (lane, resources) in &resource_exports {
        let objects_dir = scope_output_dir
            .join(lane.dir_name)
            .join(RESOURCE_OBJECTS_SUBDIR);
        if !args.dry_run {
            fs::create_dir_all(&objects_dir)?;
        }
        for (resource_index, resource) in resources.iter().enumerate() {
            let resource = dashboard_resource_export_document(
                resource,
                *lane,
                args.resource_namespace.as_deref().unwrap_or_default(),
                args.portable_v2,
            )?;
            let fallback_name = format!("dashboard-{}", resource_index + 1);
            let (resource_path, resource_name, resource_title, resource_folder_path) =
                build_resource_output_path(
                    &objects_dir,
                    &resource,
                    &fallback_name,
                    &folder_paths_by_key,
                    current_org_id,
                );
            if !args.dry_run {
                write_dashboard(&resource, &resource_path, args.overwrite)?;
            }
            if args.verbose {
                println!(
                    "{}",
                    format_export_verbose_line(
                        lane.dir_name,
                        &resource_name,
                        &resource_path,
                        args.dry_run
                    )
                );
            }
            let item_position = match item_positions.get(&resource_name) {
                Some(position) => *position,
                None => {
                    let mut summary = Map::new();
                    summary.insert("uid".to_string(), Value::String(resource_name.clone()));
                    summary.insert("title".to_string(), Value::String(resource_title.clone()));
                    summary.insert(
                        "orgName".to_string(),
                        Value::String(current_org_name.to_string()),
                    );
                    summary.insert(
                        "orgId".to_string(),
                        Value::String(current_org_id.to_string()),
                    );
                    index_items.push(build_dashboard_index_item(&summary, &resource_name));
                    let position = index_items.len() - 1;
                    item_positions.insert(resource_name.clone(), position);
                    position
                }
            };
            let item = &mut index_items[item_position];
            item.folder_path = resource_folder_path.clone();
            let resource_entry = build_resource_index_entry(
                *lane,
                resource_name,
                resource_title,
                &resource_path,
                resource_folder_path,
                current_org_name,
                current_org_id,
            );
            if lane.dir_name == RESOURCE_V1_EXPORT_SUBDIR {
                item.resource_v1_path = Some(resource_path.display().to_string());
                resource_v1_index_items.push(resource_entry);
            } else if lane.dir_name == RESOURCE_V2_EXPORT_SUBDIR {
                item.resource_v2_path = Some(resource_path.display().to_string());
                resource_v2_index_items.push(resource_entry);
            }
        }
    }

    let mut resource_v1_index_path = None;
    let mut resource_v2_index_path = None;
    for (lane, _) in &resource_exports {
        let resource_dir = scope_output_dir.join(lane.dir_name);
        let index_path = resource_dir.join("index.json");
        let metadata_path = resource_dir.join(EXPORT_METADATA_FILENAME);
        let entries = if lane.dir_name == RESOURCE_V1_EXPORT_SUBDIR {
            &resource_v1_index_items
        } else {
            &resource_v2_index_items
        };
        if !args.dry_run {
            write_json_document(entries, &index_path)?;
            write_json_document(
                &build_resource_export_metadata(
                    *lane,
                    entries.len(),
                    current_org_name,
                    current_org_id,
                    &args.common.url,
                    args.common.profile.as_deref(),
                    &resource_dir,
                    &metadata_path,
                ),
                &metadata_path,
            )?;
        }
        if lane.dir_name == RESOURCE_V1_EXPORT_SUBDIR {
            resource_v1_index_path = Some(index_path);
        } else if lane.dir_name == RESOURCE_V2_EXPORT_SUBDIR {
            resource_v2_index_path = Some(index_path);
        }
    }

    let root_index = build_root_export_index_with_resources(
        &index_items,
        None,
        None,
        None,
        resource_v1_index_path.as_deref(),
        resource_v2_index_path.as_deref(),
        &[],
    );
    if !args.dry_run {
        write_json_document(&root_index, &scope_output_dir.join("index.json"))?;
        write_json_document(
            &build_export_metadata(
                "root",
                index_items.len(),
                None,
                None,
                None,
                None,
                Some(current_org_name),
                Some(current_org_id),
                None,
                "live",
                Some(&args.common.url),
                None,
                args.common.profile.as_deref(),
                scope_output_dir,
                &scope_output_dir.join(EXPORT_METADATA_FILENAME),
            ),
            &scope_output_dir.join(EXPORT_METADATA_FILENAME),
        )?;
    }
    Ok(ScopeExportResult {
        exported_count: index_items.len(),
        root_index: Some(root_index),
        org_summary: Some(ExportOrgSummary {
            org: current_org_name.to_string(),
            org_id: current_org_id.to_string(),
            dashboard_count: index_items.len() as u64,
            datasource_count: None,
            used_datasource_count: None,
            used_datasources: None,
            output_dir: if args.all_orgs {
                Some(scope_output_dir.display().to_string())
            } else {
                None
            },
        }),
    })
}
