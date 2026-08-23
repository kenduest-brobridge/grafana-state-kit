//! Dashboard resource API export helpers.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{
    encode_resource_path_segment, message, sanitize_path_component, string_field, Result,
};
use crate::grafana_api::DashboardResourceApiVersion;

use super::super::{
    build_export_metadata, DashboardResourceFormat, DashboardResourceIndexEntry, ExportMetadata,
    RESOURCE_V1_EXPORT_SUBDIR, RESOURCE_V2_EXPORT_SUBDIR,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceExportLane {
    pub(crate) version: DashboardResourceApiVersion,
    pub(crate) dir_name: &'static str,
    pub(crate) format_name: &'static str,
    pub(crate) api_version: &'static str,
}

impl ResourceExportLane {
    pub(crate) fn for_version(version: DashboardResourceApiVersion) -> Result<Self> {
        match version {
            DashboardResourceApiVersion::V1 => Ok(Self {
                version,
                dir_name: RESOURCE_V1_EXPORT_SUBDIR,
                format_name: "grafana-dashboard-resource-v1",
                api_version: "dashboard.grafana.app/v1",
            }),
            DashboardResourceApiVersion::V2 => Ok(Self {
                version,
                dir_name: RESOURCE_V2_EXPORT_SUBDIR,
                format_name: "grafana-dashboard-resource-v2",
                api_version: "dashboard.grafana.app/v2",
            }),
        }
    }
}

impl ResourceExportLane {
    fn version_segment(self) -> &'static str {
        match self.version {
            DashboardResourceApiVersion::V1 => "v1",
            DashboardResourceApiVersion::V2 => "v2",
        }
    }

    fn collection_path(self, namespace: &str) -> String {
        format!(
            "/apis/dashboard.grafana.app/{}/namespaces/{}/dashboards",
            self.version_segment(),
            encode_resource_path_segment(namespace)
        )
    }
}

pub(crate) fn discover_resource_lanes_with_request<F>(
    mut request_json: F,
    format: DashboardResourceFormat,
    namespace: &str,
) -> Result<Vec<ResourceExportLane>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let lanes = selected_resource_lanes(format)?;
    let mut served = Vec::new();
    for lane in lanes {
        let path = lane.collection_path(namespace);
        let params = [("limit".to_string(), "1".to_string())];
        match request_json(Method::GET, &path, &params, None) {
            Ok(value) => {
                dashboard_resource_collection_object(value, lane)?;
                served.push(lane);
            }
            Err(error)
                if error.status_code() == Some(404) && format == DashboardResourceFormat::All => {}
            Err(error) if error.status_code() == Some(404) => {
                return Err(message(format!(
                    "Dashboard resource API {} is not served by this Grafana instance.",
                    lane.api_version
                )))
            }
            Err(error) => return Err(error),
        }
    }
    Ok(served)
}

pub(crate) fn list_dashboard_resources_with_request<F>(
    mut request_json: F,
    format: DashboardResourceFormat,
    namespace: &str,
    page_size: usize,
) -> Result<Vec<(ResourceExportLane, Vec<Value>)>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut exports = Vec::new();
    for lane in selected_resource_lanes(format)? {
        match list_dashboard_resources_for_lane_with_request(
            &mut request_json,
            lane,
            namespace,
            page_size,
        ) {
            Ok(resources) => exports.push((lane, resources)),
            Err(error)
                if error.status_code() == Some(404) && format == DashboardResourceFormat::All => {}
            Err(error) if error.status_code() == Some(404) => {
                return Err(message(format!(
                    "Dashboard resource API {} is not served by this Grafana instance.",
                    lane.api_version
                )))
            }
            Err(error) => return Err(error),
        }
    }
    Ok(exports)
}

fn list_dashboard_resources_for_lane_with_request<F>(
    request_json: &mut F,
    lane: ResourceExportLane,
    namespace: &str,
    page_size: usize,
) -> Result<Vec<Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let path = lane.collection_path(namespace);
    let mut resources = Vec::new();
    let mut continuation: Option<String> = None;
    let mut seen_tokens = std::collections::BTreeSet::new();
    loop {
        let mut params = vec![("limit".to_string(), page_size.max(1).to_string())];
        if let Some(token) = continuation.as_ref() {
            params.push(("continue".to_string(), token.clone()));
        }
        let object = dashboard_resource_collection_object(
            request_json(Method::GET, &path, &params, None)?,
            lane,
        )?;
        let items = object
            .get("items")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                message(format!(
                    "Dashboard resource collection for {} is missing an items array.",
                    lane.api_version
                ))
            })?;
        resources.extend(items.iter().cloned());
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
                "Dashboard resource collection repeated continue token `{token}`."
            )));
        }
    }
    Ok(resources)
}

fn dashboard_resource_collection_object(
    value: Option<Value>,
    lane: ResourceExportLane,
) -> Result<Map<String, Value>> {
    let Some(Value::Object(object)) = value else {
        return Err(message(format!(
            "Unexpected empty or non-object dashboard resource collection payload for {}.",
            lane.api_version
        )));
    };
    if object.get("apiVersion").and_then(Value::as_str) != Some(lane.api_version)
        || object.get("kind").and_then(Value::as_str) != Some("DashboardList")
    {
        return Err(message(format!(
            "Unexpected dashboard resource collection identity for {}.",
            lane.api_version
        )));
    }
    if object.get("items").and_then(Value::as_array).is_none() {
        return Err(message(format!(
            "Dashboard resource collection payload for {} is missing an items array.",
            lane.api_version
        )));
    }
    Ok(object)
}

pub(crate) fn selected_resource_lanes(
    format: DashboardResourceFormat,
) -> Result<Vec<ResourceExportLane>> {
    match format {
        DashboardResourceFormat::None => Ok(Vec::new()),
        DashboardResourceFormat::V1 => Ok(vec![ResourceExportLane::for_version(
            DashboardResourceApiVersion::V1,
        )?]),
        DashboardResourceFormat::V2 => Ok(vec![ResourceExportLane::for_version(
            DashboardResourceApiVersion::V2,
        )?]),
        DashboardResourceFormat::All => Ok(vec![
            ResourceExportLane::for_version(DashboardResourceApiVersion::V1)?,
            ResourceExportLane::for_version(DashboardResourceApiVersion::V2)?,
        ]),
    }
}

pub(crate) fn dashboard_resource_namespace(org_id: &str) -> String {
    if org_id == "1" {
        "default".to_string()
    } else {
        format!("org-{org_id}")
    }
}

pub(crate) fn fetch_dashboard_resource_with_request<F>(
    request_json: &mut F,
    lane: ResourceExportLane,
    namespace: &str,
    name: &str,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let version = lane.version_segment();
    let path = format!(
        "/apis/dashboard.grafana.app/{version}/namespaces/{}/dashboards/{}",
        encode_resource_path_segment(namespace),
        encode_resource_path_segment(name)
    );
    match request_json(Method::GET, &path, &[], None)? {
        Some(Value::Object(object)) => {
            let response_name = object
                .get("metadata")
                .and_then(Value::as_object)
                .and_then(|metadata| metadata.get("name"))
                .and_then(Value::as_str);
            if response_name != Some(name) {
                return Err(message(format!(
                    "Dashboard resource response belongs to `{}`; expected `{name}`.",
                    response_name.unwrap_or("<missing name>")
                )));
            }
            if let Some(response_namespace) = object
                .get("metadata")
                .and_then(Value::as_object)
                .and_then(|metadata| metadata.get("namespace"))
                .and_then(Value::as_str)
            {
                if response_namespace != namespace {
                    return Err(message(format!(
                        "Dashboard resource `{name}` belongs to namespace `{response_namespace}`; expected `{namespace}`."
                    )));
                }
            }
            if let Some(response_api_version) = object.get("apiVersion").and_then(Value::as_str) {
                if response_api_version != lane.api_version {
                    return Err(message(format!(
                        "Dashboard resource `{name}` has API version `{response_api_version}`; expected `{}`.",
                        lane.api_version
                    )));
                }
            }
            if let Some(response_kind) = object.get("kind").and_then(Value::as_str) {
                if response_kind != "Dashboard" {
                    return Err(message(format!(
                        "Dashboard resource `{name}` has kind `{response_kind}`; expected `Dashboard`."
                    )));
                }
            }
            Ok(Value::Object(object))
        }
        Some(_) => Err(message(format!(
            "Unexpected dashboard resource payload for {namespace}/{name}."
        ))),
        None => Err(message(format!(
            "Unexpected empty dashboard resource payload for {namespace}/{name}."
        ))),
    }
}

pub(crate) fn build_resource_output_path(
    objects_dir: &Path,
    resource: &Value,
    fallback_name: &str,
    folder_paths_by_key: &BTreeMap<String, String>,
    org_id: &str,
) -> (PathBuf, String, String, String) {
    let metadata = resource.get("metadata").and_then(Value::as_object);
    let name = metadata
        .map(|object| string_field(object, "name", fallback_name))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback_name.to_string());
    let title = resource
        .get("spec")
        .and_then(Value::as_object)
        .map(|object| string_field(object, "title", &name))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| name.clone());
    let folder_uid = metadata
        .and_then(|object| object.get("annotations"))
        .and_then(Value::as_object)
        .and_then(|annotations| annotations.get("grafana.app/folder"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let folder_path = if folder_uid.is_empty() {
        String::new()
    } else {
        folder_paths_by_key
            .get(&format!("{org_id}:{folder_uid}"))
            .cloned()
            .unwrap_or_default()
    };
    let file_name = format!("{}.json", sanitize_path_component(&name));
    let path = if folder_path.trim().is_empty() {
        objects_dir.join(&file_name)
    } else {
        folder_path
            .split(" / ")
            .filter(|segment| !segment.trim().is_empty())
            .fold(objects_dir.to_path_buf(), |path, segment| {
                path.join(sanitize_path_component(segment.trim()))
            })
            .join(file_name)
    };
    (path, name, title, folder_path)
}

pub(crate) fn build_resource_index_entry(
    lane: ResourceExportLane,
    name: String,
    title: String,
    path: &Path,
    folder_path: String,
    org: &str,
    org_id: &str,
) -> DashboardResourceIndexEntry {
    DashboardResourceIndexEntry {
        name,
        title,
        path: path.display().to_string(),
        format: lane.format_name.to_string(),
        api_version: lane.api_version.to_string(),
        folder_path,
        org: org.to_string(),
        org_id: org_id.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_resource_export_metadata(
    lane: ResourceExportLane,
    dashboard_count: usize,
    org_name: &str,
    org_id: &str,
    source_url: &str,
    source_profile: Option<&str>,
    artifact_path: &Path,
    metadata_path: &Path,
) -> ExportMetadata {
    let mut metadata = build_export_metadata(
        lane.dir_name,
        dashboard_count,
        Some(lane.format_name),
        None,
        None,
        None,
        Some(org_name),
        Some(org_id),
        None,
        "live",
        Some(source_url),
        None,
        source_profile,
        artifact_path,
        metadata_path,
    );
    metadata.resource_api_version = Some(lane.api_version.to_string());
    metadata.serialization = Some("json-pretty".to_string());
    metadata.ui_import_compatible = Some(false);
    metadata
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dashboard::DashboardResourceFormat;
    use serde_json::json;

    #[test]
    fn v2_lane_uses_public_grafana_13_resource_contract() {
        let lane = ResourceExportLane::for_version(DashboardResourceApiVersion::V2).unwrap();

        assert_eq!(lane.dir_name, "resource-v2");
        assert_eq!(lane.format_name, "grafana-dashboard-resource-v2");
        assert_eq!(lane.api_version, "dashboard.grafana.app/v2");
    }

    #[test]
    fn all_lanes_skip_only_resource_versions_returning_404() {
        let mut paths = Vec::new();
        let lanes = discover_resource_lanes_with_request(
            |method, path, params, _| {
                assert_eq!(method, Method::GET);
                assert_eq!(params, &[("limit".to_string(), "1".to_string())]);
                paths.push(path.to_string());
                if path.contains("/v2/") {
                    Err(crate::common::api_response(
                        404,
                        path.to_string(),
                        json!({"message":"not found"}).to_string(),
                    ))
                } else {
                    Ok(Some(json!({
                        "apiVersion": "dashboard.grafana.app/v1",
                        "kind": "DashboardList",
                        "items":[],
                        "metadata":{}
                    })))
                }
            },
            DashboardResourceFormat::All,
            "default",
        )
        .unwrap();

        assert_eq!(lanes.len(), 1);
        assert_eq!(lanes[0].version, DashboardResourceApiVersion::V1);
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn discovery_rejects_a_malformed_collection_object() {
        let error = discover_resource_lanes_with_request(
            |_method, _path, _params, _| {
                Ok(Some(json!({
                    "apiVersion": "dashboard.grafana.app/v2",
                    "kind": "DashboardList",
                    "metadata": {}
                })))
            },
            DashboardResourceFormat::V2,
            "default",
        )
        .expect_err("resource discovery must validate the collection contract");

        assert!(error.to_string().contains("items array"));
    }

    #[test]
    fn resource_paths_encode_namespace_and_name_as_segments() {
        let lane = ResourceExportLane::for_version(DashboardResourceApiVersion::V2).unwrap();
        let mut requested_path = String::new();
        fetch_dashboard_resource_with_request(
            &mut |_method, path, _params, _| {
                requested_path = path.to_string();
                Ok(Some(json!({"metadata": {"name": "dash#1%"}})))
            },
            lane,
            "team/ops?filter",
            "dash#1%",
        )
        .unwrap();

        assert_eq!(
            requested_path,
            "/apis/dashboard.grafana.app/v2/namespaces/team%2Fops%3Ffilter/dashboards/dash%231%25"
        );
    }

    #[test]
    fn resource_fetch_rejects_a_mismatched_identity() {
        let lane = ResourceExportLane::for_version(DashboardResourceApiVersion::V2).unwrap();
        let error = fetch_dashboard_resource_with_request(
            &mut |_method, _path, _params, _| {
                Ok(Some(json!({
                    "apiVersion": "dashboard.grafana.app/v2",
                    "kind": "Dashboard",
                    "metadata": {"name": "other-dashboard", "namespace": "default"}
                })))
            },
            lane,
            "default",
            "cpu-main",
        )
        .expect_err("resource export must not attribute another dashboard");

        assert!(error.to_string().contains("cpu-main"));
        assert!(error.to_string().contains("other-dashboard"));
    }

    #[test]
    fn explicit_lane_reports_unavailable_resource_version() {
        let error = discover_resource_lanes_with_request(
            |_method, path, _params, _| {
                Err(crate::common::api_response(
                    404,
                    path.to_string(),
                    json!({"message":"not found"}).to_string(),
                ))
            },
            DashboardResourceFormat::V2,
            "default",
        )
        .expect_err("explicit v2 must not silently disappear");

        assert!(error.to_string().contains("dashboard.grafana.app/v2"));
        assert!(error.to_string().contains("not served"));
    }

    #[test]
    fn resource_collection_follows_continue_tokens() {
        let mut request_count = 0;
        let exports = list_dashboard_resources_with_request(
            |_method, _path, params, _| {
                request_count += 1;
                if params.iter().any(|(key, _)| key == "continue") {
                    Ok(Some(json!({
                        "apiVersion": "dashboard.grafana.app/v2",
                        "kind": "DashboardList",
                        "items": [{"metadata": {"name": "second"}}],
                        "metadata": {}
                    })))
                } else {
                    Ok(Some(json!({
                        "apiVersion": "dashboard.grafana.app/v2",
                        "kind": "DashboardList",
                        "items": [{"metadata": {"name": "first"}}],
                        "metadata": {"continue": "page-2"}
                    })))
                }
            },
            DashboardResourceFormat::V2,
            "default",
            25,
        )
        .unwrap();

        assert_eq!(request_count, 2);
        assert_eq!(exports[0].1.len(), 2);
    }

    #[test]
    fn resource_collection_rejects_repeated_continue_tokens() {
        let error = list_dashboard_resources_with_request(
            |_method, _path, _params, _| {
                Ok(Some(json!({
                    "apiVersion": "dashboard.grafana.app/v2",
                    "kind": "DashboardList",
                    "items": [],
                    "metadata": {"continue": "loop"}
                })))
            },
            DashboardResourceFormat::V2,
            "default",
            25,
        )
        .expect_err("repeated continuation token must terminate pagination");

        assert!(error.to_string().contains("repeated continue token"));
    }
}
