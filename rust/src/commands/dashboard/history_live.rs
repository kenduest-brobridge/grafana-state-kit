use std::collections::BTreeSet;

use crate::common::{encode_resource_path_segment, message, value_as_object, Result};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use reqwest::Method;
use serde_json::{Map, Value};

use super::history_artifacts::{
    build_dashboard_history_list_document_from_export_document,
    ensure_history_artifact_uid_matches, load_dashboard_history_export_document,
    run_dashboard_history_list_from_import_dir,
};
use super::history_render::render_dashboard_history_list_output;
use super::history_types::{
    DashboardHistoryExportDocument, DashboardHistoryExportVersion, DashboardHistoryListDocument,
    DashboardHistoryVersion, DashboardRestorePreview,
};
use super::{
    fetch_dashboard_with_request, import_dashboard_request_with_request, string_field,
    tool_version, write_json_document, DashboardHistorySource, HistoryExportArgs, HistoryListArgs,
    DEFAULT_DASHBOARD_TITLE, DEFAULT_FOLDER_UID, TOOL_SCHEMA_VERSION,
};

const HISTORY_VERSION_FETCH_CONCURRENCY: usize = 8;

fn display_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        _ => value.to_string(),
    }
}

pub(crate) fn list_dashboard_history_versions_with_request<F>(
    mut request_json: F,
    uid: &str,
    limit: usize,
) -> Result<Vec<DashboardHistoryVersion>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let path = format!(
        "/api/dashboards/uid/{}/versions",
        encode_resource_path_segment(uid)
    );
    let params = vec![("limit".to_string(), limit.to_string())];
    let response = request_json(Method::GET, &path, &params, None)?;
    let Some(value) = response else {
        return Ok(Vec::new());
    };
    let versions = match value {
        Value::Array(items) => items,
        Value::Object(object) => object
            .get("versions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => {
            return Err(message(
                "Unexpected dashboard versions payload from Grafana.",
            ))
        }
    };
    Ok(versions
        .into_iter()
        .filter_map(|item| item.as_object().cloned())
        .map(|item| DashboardHistoryVersion {
            version: item
                .get("version")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
            created: item
                .get("created")
                .map(display_value)
                .unwrap_or_else(|| "-".to_string()),
            created_by: string_field(&item, "createdBy", "-"),
            message: string_field(&item, "message", ""),
        })
        .collect())
}

pub(crate) fn build_dashboard_history_list_document_with_request<F>(
    mut request_json: F,
    uid: &str,
    limit: usize,
) -> Result<DashboardHistoryListDocument>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let versions = list_dashboard_history_versions_with_request(&mut request_json, uid, limit)?;
    Ok(DashboardHistoryListDocument {
        kind: super::history_types::DASHBOARD_HISTORY_LIST_KIND.to_string(),
        schema_version: TOOL_SCHEMA_VERSION,
        tool_version: tool_version().to_string(),
        dashboard_uid: uid.to_string(),
        version_count: versions.len(),
        versions,
    })
}

fn resource_history_version_segment(source: DashboardHistorySource) -> Result<&'static str> {
    match source {
        DashboardHistorySource::V1 => Ok("v1"),
        DashboardHistorySource::V2 => Ok("v2"),
        DashboardHistorySource::Classic => Err(message(
            "Classic dashboard history does not use the App Platform resource API.",
        )),
    }
}

fn resource_history_version(resource: &Value) -> Result<i64> {
    resource
        .get("metadata")
        .and_then(Value::as_object)
        .and_then(|metadata| metadata.get("generation"))
        .and_then(Value::as_i64)
        .ok_or_else(|| message("Dashboard resource history item is missing numeric generation."))
}

fn resource_history_metadata(resource: &Value, key: &str, fallback: &str) -> String {
    let metadata = resource.get("metadata").and_then(Value::as_object);
    if key == "created" {
        return metadata
            .and_then(|value| value.get("creationTimestamp"))
            .and_then(Value::as_str)
            .unwrap_or(fallback)
            .to_string();
    }
    metadata
        .and_then(|value| value.get("annotations"))
        .and_then(Value::as_object)
        .and_then(|annotations| annotations.get(key))
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

fn validate_dashboard_resource_history_item(
    resource: &Value,
    source: DashboardHistorySource,
    expected_namespace: &str,
    expected_uid: &str,
) -> Result<()> {
    let object = resource
        .as_object()
        .ok_or_else(|| message("Dashboard resource history item must be a JSON object."))?;
    let metadata = object
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| message("Dashboard resource history item is missing metadata."))?;
    let actual_uid = metadata
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| message("Dashboard resource history item is missing metadata.name."))?;
    if actual_uid != expected_uid {
        return Err(message(format!(
            "Dashboard resource history item belongs to `{actual_uid}`, expected `{expected_uid}`."
        )));
    }
    if let Some(actual_namespace) = metadata.get("namespace").and_then(Value::as_str) {
        if actual_namespace != expected_namespace {
            return Err(message(format!(
                "Dashboard resource history item `{expected_uid}` belongs to namespace `{actual_namespace}`, expected `{expected_namespace}`."
            )));
        }
    }
    let expected_api_version = format!(
        "dashboard.grafana.app/{}",
        resource_history_version_segment(source)?
    );
    if let Some(actual_api_version) = object.get("apiVersion").and_then(Value::as_str) {
        if actual_api_version != expected_api_version {
            return Err(message(format!(
                "Dashboard resource history item `{expected_uid}` has API version `{actual_api_version}`, expected `{expected_api_version}`."
            )));
        }
    }
    if let Some(actual_kind) = object.get("kind").and_then(Value::as_str) {
        if actual_kind != "Dashboard" {
            return Err(message(format!(
                "Dashboard resource history item `{expected_uid}` has kind `{actual_kind}`, expected `Dashboard`."
            )));
        }
    }
    Ok(())
}

fn list_dashboard_resource_history_values_with_request<F>(
    request_json: &mut F,
    source: DashboardHistorySource,
    namespace: &str,
    uid: &str,
    limit: usize,
) -> Result<Vec<Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let version = resource_history_version_segment(source)?;
    let path = format!(
        "/apis/dashboard.grafana.app/{version}/namespaces/{}/dashboards",
        encode_resource_path_segment(namespace)
    );
    let mut resources = Vec::new();
    let mut continuation: Option<String> = None;
    let mut seen_tokens = BTreeSet::new();
    loop {
        let mut params = vec![
            ("fieldSelector".to_string(), format!("metadata.name={uid}")),
            (
                "labelSelector".to_string(),
                "grafana.app/get-history=true".to_string(),
            ),
            ("limit".to_string(), limit.to_string()),
        ];
        if let Some(token) = continuation.as_ref() {
            params.push(("continue".to_string(), token.clone()));
        }
        let response = request_json(Method::GET, &path, &params, None)?;
        let object = match response {
            Some(Value::Object(object)) => object,
            Some(_) => {
                return Err(message(
                    "Unexpected dashboard resource history collection payload.",
                ))
            }
            None => {
                return Err(message(
                    "Unexpected empty dashboard resource history collection payload.",
                ))
            }
        };
        let items = object
            .get("items")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                message("Dashboard resource history collection is missing an items array.")
            })?;
        for resource in items {
            validate_dashboard_resource_history_item(resource, source, namespace, uid)?;
        }
        resources.extend(items.iter().cloned());
        if resources.len() >= limit {
            resources.truncate(limit);
            break;
        }
        continuation = object
            .get("metadata")
            .and_then(Value::as_object)
            .and_then(|metadata| metadata.get("continue"))
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .map(str::to_string);
        let Some(token) = continuation.as_ref() else {
            break;
        };
        if !seen_tokens.insert(token.clone()) {
            return Err(message(format!(
                "Dashboard resource history repeated continue token `{token}`."
            )));
        }
    }
    Ok(resources)
}

pub(crate) fn build_dashboard_resource_history_list_document_with_request<F>(
    mut request_json: F,
    source: DashboardHistorySource,
    namespace: &str,
    uid: &str,
    limit: usize,
) -> Result<DashboardHistoryListDocument>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let resources = list_dashboard_resource_history_values_with_request(
        &mut request_json,
        source,
        namespace,
        uid,
        limit,
    )?;
    let versions = resources
        .iter()
        .map(|resource| {
            Ok(DashboardHistoryVersion {
                version: resource_history_version(resource)?,
                created: resource_history_metadata(resource, "created", "-"),
                created_by: resource_history_metadata(resource, "grafana.app/updatedBy", "-"),
                message: resource_history_metadata(resource, "grafana.app/message", ""),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(DashboardHistoryListDocument {
        kind: super::history_types::DASHBOARD_HISTORY_LIST_KIND.to_string(),
        schema_version: TOOL_SCHEMA_VERSION,
        tool_version: tool_version().to_string(),
        dashboard_uid: uid.to_string(),
        version_count: versions.len(),
        versions,
    })
}

fn build_dashboard_resource_history_export_document_with_request<F>(
    mut request_json: F,
    source: DashboardHistorySource,
    namespace: &str,
    uid: &str,
    limit: usize,
) -> Result<DashboardHistoryExportDocument>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let version = resource_history_version_segment(source)?;
    let path = format!(
        "/apis/dashboard.grafana.app/{version}/namespaces/{}/dashboards/{}",
        encode_resource_path_segment(namespace),
        encode_resource_path_segment(uid)
    );
    let current = request_json(Method::GET, &path, &[], None)?
        .ok_or_else(|| message("Current dashboard resource was not returned."))?;
    validate_dashboard_resource_history_item(&current, source, namespace, uid)?;
    let current_version = resource_history_version(&current)?;
    let current_title = current
        .get("spec")
        .and_then(Value::as_object)
        .and_then(|spec| spec.get("title"))
        .and_then(Value::as_str)
        .unwrap_or(uid)
        .to_string();
    let resources = list_dashboard_resource_history_values_with_request(
        &mut request_json,
        source,
        namespace,
        uid,
        limit,
    )?;
    let versions = resources
        .into_iter()
        .map(|resource| {
            Ok(DashboardHistoryExportVersion {
                version: resource_history_version(&resource)?,
                created: resource_history_metadata(&resource, "created", "-"),
                created_by: resource_history_metadata(&resource, "grafana.app/updatedBy", "-"),
                message: resource_history_metadata(&resource, "grafana.app/message", ""),
                dashboard: resource,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(build_dashboard_history_export_document(
        uid,
        current_version,
        current_title,
        versions,
    ))
}

pub(crate) fn fetch_dashboard_history_version_data_with_request<F>(
    mut request_json: F,
    uid: &str,
    version: i64,
) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let version_path = format!(
        "/api/dashboards/uid/{}/versions/{version}",
        encode_resource_path_segment(uid)
    );
    let version_payload =
        request_json(Method::GET, &version_path, &[], None)?.ok_or_else(|| {
            message(format!(
                "Dashboard history version {version} was not returned."
            ))
        })?;
    let version_object = value_as_object(
        &version_payload,
        "Unexpected dashboard history version payload from Grafana.",
    )?;
    version_object
        .get("data")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| message("Dashboard history version payload did not include dashboard data."))
}

pub(crate) fn build_dashboard_restore_preview_with_request<F>(
    mut request_json: F,
    uid: &str,
    version: i64,
) -> Result<DashboardRestorePreview>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let current_payload = fetch_dashboard_with_request(&mut request_json, uid)?;
    super::history_restore::validate_dashboard_history_restore_target(&current_payload, uid)?;
    let current_object = value_as_object(
        &current_payload,
        "Unexpected current dashboard payload for history restore.",
    )?;
    let current_dashboard = current_object
        .get("dashboard")
        .and_then(Value::as_object)
        .ok_or_else(|| message("Current dashboard payload did not include dashboard data."))?;
    let current_version = current_dashboard
        .get("version")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let current_title = string_field(current_dashboard, "title", DEFAULT_DASHBOARD_TITLE);
    let target_folder_uid = current_object
        .get("meta")
        .and_then(Value::as_object)
        .map(|meta| string_field(meta, "folderUid", DEFAULT_FOLDER_UID))
        .filter(|value| !value.is_empty() && value != DEFAULT_FOLDER_UID);
    let restored_dashboard =
        fetch_dashboard_history_version_data_with_request(&mut request_json, uid, version)?;
    let restored_title = string_field(&restored_dashboard, "title", DEFAULT_DASHBOARD_TITLE);
    Ok(DashboardRestorePreview {
        current_version,
        current_title,
        restored_title,
        target_folder_uid,
    })
}

pub(crate) fn build_dashboard_history_export_document_with_request<F>(
    mut request_json: F,
    uid: &str,
    limit: usize,
) -> Result<DashboardHistoryExportDocument>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let current_payload = fetch_dashboard_with_request(&mut request_json, uid)?;
    build_dashboard_history_export_document_from_current_with_request(
        &mut request_json,
        uid,
        limit,
        &current_payload,
    )
}

pub(crate) fn build_dashboard_history_export_document_from_current_with_request<F>(
    mut request_json: F,
    uid: &str,
    limit: usize,
    current_payload: &Value,
) -> Result<DashboardHistoryExportDocument>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let (current_version, current_title) = current_dashboard_export_metadata(current_payload)?;
    let versions = list_dashboard_history_versions_with_request(&mut request_json, uid, limit)?;
    let versions = versions
        .into_iter()
        .map(|version| {
            let dashboard = Value::Object(fetch_dashboard_history_version_data_with_request(
                &mut request_json,
                uid,
                version.version,
            )?);
            Ok(DashboardHistoryExportVersion {
                version: version.version,
                created: version.created,
                created_by: version.created_by,
                message: version.message,
                dashboard,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(build_dashboard_history_export_document(
        uid,
        current_version,
        current_title,
        versions,
    ))
}

pub(crate) fn build_dashboard_history_export_document_from_current_with_version_fetcher<F, G>(
    mut request_json: F,
    uid: &str,
    limit: usize,
    current_payload: &Value,
    fetch_version: G,
    concurrency: usize,
) -> Result<DashboardHistoryExportDocument>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
    G: Fn(i64) -> Result<Map<String, Value>> + Sync,
{
    let (current_version, current_title) = current_dashboard_export_metadata(current_payload)?;
    let versions = list_dashboard_history_versions_with_request(&mut request_json, uid, limit)?;
    let versions = build_dashboard_history_export_versions_with_fetcher(
        versions,
        &fetch_version,
        concurrency,
    )?;
    Ok(build_dashboard_history_export_document(
        uid,
        current_version,
        current_title,
        versions,
    ))
}

fn current_dashboard_export_metadata(current_payload: &Value) -> Result<(i64, String)> {
    let current_object = value_as_object(
        current_payload,
        "Unexpected current dashboard payload for history export.",
    )?;
    let current_dashboard = current_object
        .get("dashboard")
        .and_then(Value::as_object)
        .ok_or_else(|| message("Current dashboard payload did not include dashboard data."))?;
    let current_version = current_dashboard
        .get("version")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let current_title = string_field(current_dashboard, "title", DEFAULT_DASHBOARD_TITLE);
    Ok((current_version, current_title))
}

fn build_dashboard_history_export_versions_with_fetcher<F>(
    versions: Vec<DashboardHistoryVersion>,
    fetch_version: &F,
    concurrency: usize,
) -> Result<Vec<DashboardHistoryExportVersion>>
where
    F: Fn(i64) -> Result<Map<String, Value>> + Sync,
{
    if versions.is_empty() {
        return Ok(Vec::new());
    }
    let worker_count = concurrency
        .clamp(1, HISTORY_VERSION_FETCH_CONCURRENCY)
        .min(versions.len());
    if worker_count <= 1 {
        return versions
            .into_iter()
            .map(|version| {
                let dashboard = Value::Object(fetch_version(version.version)?);
                Ok(DashboardHistoryExportVersion {
                    version: version.version,
                    created: version.created,
                    created_by: version.created_by,
                    message: version.message,
                    dashboard,
                })
            })
            .collect();
    }

    let pool = ThreadPoolBuilder::new()
        .num_threads(worker_count)
        .build()
        .map_err(|error| {
            message(format!(
                "Failed to build dashboard history read worker pool: {error}"
            ))
        })?;
    let reads = pool.install(|| {
        versions
            .par_iter()
            .map(|version| (version.clone(), fetch_version(version.version)))
            .collect::<Vec<_>>()
    });

    reads
        .into_iter()
        .map(|(version, dashboard)| {
            Ok(DashboardHistoryExportVersion {
                version: version.version,
                created: version.created,
                created_by: version.created_by,
                message: version.message,
                dashboard: Value::Object(dashboard?),
            })
        })
        .collect()
}

fn build_dashboard_history_export_document(
    uid: &str,
    current_version: i64,
    current_title: String,
    versions: Vec<DashboardHistoryExportVersion>,
) -> DashboardHistoryExportDocument {
    DashboardHistoryExportDocument {
        kind: super::history_types::DASHBOARD_HISTORY_EXPORT_KIND.to_string(),
        schema_version: TOOL_SCHEMA_VERSION,
        tool_version: tool_version().to_string(),
        dashboard_uid: uid.to_string(),
        current_version,
        current_title,
        version_count: versions.len(),
        versions,
    }
}

pub(crate) fn export_dashboard_history_with_request<F>(
    mut request_json: F,
    args: &HistoryExportArgs,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if args.output.exists() && !args.overwrite {
        return Err(message(format!(
            "Refusing to overwrite existing file: {}. Use --overwrite.",
            args.output.display()
        )));
    }
    let document = match args.history_source {
        DashboardHistorySource::Classic => build_dashboard_history_export_document_with_request(
            &mut request_json,
            &args.dashboard_uid,
            args.limit,
        )?,
        source => build_dashboard_resource_history_export_document_with_request(
            &mut request_json,
            source,
            &args.resource_namespace,
            &args.dashboard_uid,
            args.limit,
        )?,
    };
    write_json_document(&document, &args.output)?;
    Ok(())
}

pub(crate) fn run_dashboard_history_list<F>(
    mut request_json: F,
    args: &HistoryListArgs,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if let Some(input_path) = &args.input {
        let document = load_dashboard_history_export_document(input_path)?;
        if let Some(uid) = &args.dashboard_uid {
            ensure_history_artifact_uid_matches(uid, &document, input_path)?;
        }
        let list_document = build_dashboard_history_list_document_from_export_document(&document);
        return render_dashboard_history_list_output(&list_document, args.output_format);
    }

    if let Some(input_dir) = &args.input_dir {
        return run_dashboard_history_list_from_import_dir(input_dir, args);
    }

    let dashboard_uid = args.dashboard_uid.as_deref().ok_or_else(|| {
        message(
            "Dashboard history list requires --dashboard-uid unless --input or --input-dir is set.",
        )
    })?;
    let document = match args.history_source {
        DashboardHistorySource::Classic => build_dashboard_history_list_document_with_request(
            &mut request_json,
            dashboard_uid,
            args.limit,
        )?,
        source => build_dashboard_resource_history_list_document_with_request(
            &mut request_json,
            source,
            &args.resource_namespace,
            dashboard_uid,
            args.limit,
        )?,
    };
    render_dashboard_history_list_output(&document, args.output_format)
}

#[cfg(test)]
mod resource_history_tests {
    use super::*;
    use crate::dashboard::DashboardHistorySource;
    use serde_json::json;

    #[test]
    fn resource_history_list_uses_generation_actor_and_total_limit() {
        let mut calls = Vec::new();
        let document = build_dashboard_resource_history_list_document_with_request(
            |method, path, params, _| {
                calls.push((method, path.to_string(), params.to_vec()));
                if params.iter().any(|(key, _)| key == "continue") {
                    Ok(Some(json!({
                        "items": [{
                            "metadata": {
                                "name": "cpu-main",
                                "generation": 7,
                                "resourceVersion": "8",
                                "creationTimestamp": "2026-07-01T01:00:00Z",
                                "annotations": {"grafana.app/updatedBy": "sre"}
                            }
                        }],
                        "metadata": {}
                    })))
                } else {
                    Ok(Some(json!({
                        "items": [{
                            "metadata": {
                                "name": "cpu-main",
                                "generation": 8,
                                "resourceVersion": "9",
                                "creationTimestamp": "2026-07-02T01:00:00Z",
                                "annotations": {"grafana.app/updatedBy": "ops"}
                            }
                        }],
                        "metadata": {"continue": "next page"}
                    })))
                }
            },
            DashboardHistorySource::V2,
            "default",
            "cpu-main",
            1,
        )
        .unwrap();

        assert_eq!(document.version_count, 1);
        assert_eq!(document.versions[0].version, 8);
        assert_eq!(document.versions[0].created_by, "ops");
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0].1,
            "/apis/dashboard.grafana.app/v2/namespaces/default/dashboards"
        );
        assert_eq!(
            calls[0].2,
            vec![
                (
                    "fieldSelector".to_string(),
                    "metadata.name=cpu-main".to_string()
                ),
                (
                    "labelSelector".to_string(),
                    "grafana.app/get-history=true".to_string()
                ),
                ("limit".to_string(), "1".to_string())
            ]
        );
    }

    #[test]
    fn resource_history_list_truncates_continuation_to_total_limit_in_order() {
        let mut calls = Vec::new();
        let document = build_dashboard_resource_history_list_document_with_request(
            |method, path, params, _| {
                calls.push((method, path.to_string(), params.to_vec()));
                if params.iter().any(|(key, _)| key == "continue") {
                    Ok(Some(json!({
                        "items": [
                            {
                                "metadata": {
                                    "name": "cpu-main",
                                    "generation": 7,
                                    "annotations": {"grafana.app/updatedBy": "sre"}
                                }
                            },
                            {
                                "metadata": {
                                    "name": "cpu-main",
                                    "generation": 6,
                                    "annotations": {"grafana.app/updatedBy": "admin"}
                                }
                            }
                        ],
                        "metadata": {"continue": "third page"}
                    })))
                } else {
                    Ok(Some(json!({
                        "items": [{
                            "metadata": {
                                "name": "cpu-main",
                                "generation": 8,
                                "annotations": {"grafana.app/updatedBy": "ops"}
                            }
                        }],
                        "metadata": {"continue": "second page"}
                    })))
                }
            },
            DashboardHistorySource::V2,
            "default",
            "cpu-main",
            2,
        )
        .unwrap();

        assert_eq!(calls.len(), 2);
        assert_eq!(
            document
                .versions
                .iter()
                .map(|version| version.version)
                .collect::<Vec<_>>(),
            vec![8, 7]
        );
        assert_eq!(
            document
                .versions
                .iter()
                .map(|version| version.created_by.as_str())
                .collect::<Vec<_>>(),
            vec!["ops", "sre"]
        );
    }

    #[test]
    fn resource_history_rejects_an_item_for_a_different_dashboard() {
        let error = build_dashboard_resource_history_list_document_with_request(
            |_method, _path, _params, _| {
                Ok(Some(json!({
                    "items": [{"metadata": {"name": "other-dashboard", "generation": 1}}],
                    "metadata": {}
                })))
            },
            DashboardHistorySource::V2,
            "default",
            "cpu-main",
            20,
        )
        .expect_err("history must not attribute another dashboard's revision");

        assert!(error.to_string().contains("cpu-main"));
        assert!(error.to_string().contains("other-dashboard"));
    }

    #[test]
    fn resource_history_export_preserves_v2_elements() {
        let document = build_dashboard_resource_history_export_document_with_request(
            |_method, path, _params, _| {
                if path.ends_with("/dashboards/cpu-main") {
                    Ok(Some(json!({
                        "metadata": {"name": "cpu-main", "generation": 12, "resourceVersion": "9"},
                        "spec": {"title": "CPU Main", "elements": {"panel-1": {"kind": "Panel"}}}
                    })))
                } else {
                    Ok(Some(json!({
                        "items": [{
                            "metadata": {
                                "name": "cpu-main",
                                "generation": 11,
                                "resourceVersion": "8",
                                "annotations": {"grafana.app/updatedBy": "ops"}
                            },
                            "spec": {"title": "CPU Main", "elements": {"panel-1": {"kind": "Panel"}}}
                        }],
                        "metadata": {}
                    })))
                }
            },
            DashboardHistorySource::V2,
            "default",
            "cpu-main",
            20,
        )
        .unwrap();

        assert_eq!(document.current_version, 12);
        assert_eq!(document.versions[0].version, 11);
        assert_eq!(document.versions[0].created_by, "ops");
        assert_eq!(
            document.versions[0].dashboard["spec"]["elements"]["panel-1"]["kind"],
            "Panel"
        );
        assert!(document.versions[0].dashboard["spec"]
            .get("panels")
            .is_none());
    }
}
