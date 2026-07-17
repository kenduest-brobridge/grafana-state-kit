use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::{api_response, message, GrafanaCliError, Result};
use crate::dashboard::cli_defs::DryRunOutputFormat;
use crate::dashboard::ROOT_INDEX_KIND;
use crate::grafana_api::{
    dashboard_resource_item_path, dashboard_resource_path,
    validate_dashboard_resource_v2_list_response, validate_dashboard_resource_v2_response,
    DashboardResourceApiVersion,
};
use reqwest::Method;

use super::resource_v2_apply::{
    execute_resource_v2_apply, ResourceV2ApplyAction, ResourceV2ApplyOperation,
};
use super::resource_v2_render::{
    redact_resource_v2_text, render_resource_v2_dry_run_json, render_resource_v2_dry_run_table,
    render_resource_v2_dry_run_text, ResourceV2DryRunAction, ResourceV2DryRunRow,
};

const API_VERSION: &str = "dashboard.grafana.app/v2";
const FOLDER_ANNOTATION: &str = "grafana.app/folder";
const MESSAGE_ANNOTATION: &str = "grafana.app/message";
const SERVER_ANNOTATIONS: &[&str] = &[
    "grafana.app/createdBy",
    "grafana.app/updatedBy",
    "grafana.app/updatedTimestamp",
];

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResourceV2ImportDocument {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) document: Value,
}

fn collect_json_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_symlink() {
            return Err(message(format!(
                "Dashboard resource discovery does not follow symbolic links: {}",
                path.display()
            )));
        }
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) != Some("history") {
                collect_json_files(&path, files)?;
            }
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            files.push(path);
        }
    }
    Ok(())
}

fn combined_root_paths(input_dir: &Path) -> Result<Option<Vec<PathBuf>>> {
    let index_path = input_dir.join("index.json");
    if !index_path.is_file() {
        return Ok(None);
    }
    let index: Value = serde_json::from_str(&fs::read_to_string(&index_path)?)?;
    if index.get("kind").and_then(Value::as_str) != Some(ROOT_INDEX_KIND) {
        return Ok(None);
    }
    let items = index
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            message(format!(
                "Dashboard export index must contain items: {}",
                index_path.display()
            ))
        })?;
    let canonical_root = fs::canonicalize(input_dir)?;
    let mut paths = Vec::new();
    for item in items {
        let Some(path_text) = item.get("resource_v2_path").and_then(Value::as_str) else {
            continue;
        };
        let path = PathBuf::from(path_text);
        let resolved = if path.is_absolute() {
            path
        } else {
            input_dir.join(path)
        };
        let canonical = fs::canonicalize(&resolved)?;
        if !canonical.starts_with(&canonical_root) {
            return Err(message(format!(
                "Dashboard resource path resolves outside export root: {}",
                resolved.display()
            )));
        }
        paths.push(canonical);
    }
    paths.sort();
    paths.dedup();
    Ok(Some(paths))
}

fn validate_resource_document(
    path: &Path,
    document: Value,
) -> Result<Option<ResourceV2ImportDocument>> {
    let object = document.as_object().ok_or_else(|| {
        message(format!(
            "Dashboard resource must be a JSON object: {}",
            path.display()
        ))
    })?;
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let api_version = object
        .get("apiVersion")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if kind != "Dashboard" {
        return Ok(None);
    }
    if api_version != API_VERSION {
        return Err(message(format!(
            "Dashboard resource {} uses unsupported API version {:?}; resource-v2 import accepts only {API_VERSION}.",
            path.display(), api_version
        )));
    }
    let metadata = object
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            message(format!(
                "Dashboard resource metadata must be an object: {}",
                path.display()
            ))
        })?;
    if metadata.get("generateName").is_some() {
        return Err(message(format!(
            "Dashboard resource {} uses metadata.generateName; resource-v2 import requires a stable metadata.name.",
            path.display()
        )));
    }
    let name = metadata
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            message(format!(
                "Dashboard resource metadata.name must be non-empty: {}",
                path.display()
            ))
        })?;
    if !object.get("spec").is_some_and(Value::is_object) {
        return Err(message(format!(
            "Dashboard resource spec must be an object: {}",
            path.display()
        )));
    }
    Ok(Some(ResourceV2ImportDocument {
        path: path.to_path_buf(),
        name: name.to_string(),
        document: Value::Object(object.clone()),
    }))
}

pub(crate) fn discover_dashboard_v2_resources(
    input_dir: &Path,
) -> Result<Vec<ResourceV2ImportDocument>> {
    if !input_dir.is_dir() {
        return Err(message(format!(
            "Import directory does not exist or is not a directory: {}",
            input_dir.display()
        )));
    }
    let mut paths = match combined_root_paths(input_dir)? {
        Some(paths) => paths,
        None => {
            let mut paths = Vec::new();
            collect_json_files(input_dir, &mut paths)?;
            paths.retain(|path| {
                !matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("index.json" | "_folder.json" | "export-metadata.json")
                )
            });
            paths.sort();
            paths
        }
    };
    paths.sort();
    let mut resources = Vec::new();
    let mut identities = BTreeMap::<String, PathBuf>::new();
    for path in paths {
        let document: Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
        let Some(resource) = validate_resource_document(&path, document)? else {
            continue;
        };
        if let Some(first_path) = identities.insert(resource.name.clone(), path.clone()) {
            return Err(message(format!(
                "Duplicate dashboard resource metadata.name {:?} in {} and {}.",
                resource.name,
                first_path.display(),
                path.display()
            )));
        }
        resources.push(resource);
    }
    if resources.is_empty() {
        return Err(message(format!(
            "No dashboard.grafana.app/v2 Dashboard resources found in {}",
            input_dir.display()
        )));
    }
    Ok(resources)
}

fn normalized_metadata(
    source: &Map<String, Value>,
    namespace: &str,
    folder_uid: Option<&str>,
    message_text: &str,
) -> Result<Map<String, Value>> {
    let source_metadata = source
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| message("Dashboard resource metadata must be an object."))?;
    let name = source_metadata
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| message("Dashboard resource metadata.name must be non-empty."))?;
    let namespace = namespace.trim();
    if namespace.is_empty() {
        return Err(message("Dashboard resource namespace must be non-empty."));
    }
    let mut metadata = Map::new();
    metadata.insert("name".to_string(), Value::String(name.to_string()));
    metadata.insert(
        "namespace".to_string(),
        Value::String(namespace.to_string()),
    );
    if let Some(labels) = source_metadata.get("labels").and_then(Value::as_object) {
        let mut labels = labels.clone();
        labels.remove("grafana.app/deprecatedInternalID");
        if !labels.is_empty() {
            metadata.insert("labels".to_string(), Value::Object(labels));
        }
    }
    let mut annotations = source_metadata
        .get("annotations")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for key in SERVER_ANNOTATIONS {
        annotations.remove(*key);
    }
    if let Some(folder_uid) = folder_uid {
        annotations.insert(
            FOLDER_ANNOTATION.to_string(),
            Value::String(folder_uid.to_string()),
        );
    }
    if message_text.is_empty() {
        annotations.remove(MESSAGE_ANNOTATION);
    } else {
        annotations.insert(
            MESSAGE_ANNOTATION.to_string(),
            Value::String(message_text.to_string()),
        );
    }
    if !annotations.is_empty() {
        metadata.insert("annotations".to_string(), Value::Object(annotations));
    }
    Ok(metadata)
}

pub(crate) fn normalize_dashboard_v2_create_payload(
    source: &Value,
    namespace: &str,
    folder_uid: Option<&str>,
    message_text: &str,
) -> Result<Value> {
    let source = source
        .as_object()
        .ok_or_else(|| message("Dashboard resource must be a JSON object."))?;
    if source.get("apiVersion").and_then(Value::as_str) != Some(API_VERSION)
        || source.get("kind").and_then(Value::as_str) != Some("Dashboard")
    {
        return Err(message(
            "Resource-v2 payload must be a dashboard.grafana.app/v2 Dashboard.",
        ));
    }
    let spec = source
        .get("spec")
        .and_then(Value::as_object)
        .ok_or_else(|| message("Dashboard resource spec must be an object."))?;
    let mut payload = Map::new();
    payload.insert(
        "apiVersion".to_string(),
        Value::String(API_VERSION.to_string()),
    );
    payload.insert("kind".to_string(), Value::String("Dashboard".to_string()));
    payload.insert(
        "metadata".to_string(),
        Value::Object(normalized_metadata(
            source,
            namespace,
            folder_uid,
            message_text,
        )?),
    );
    payload.insert("spec".to_string(), Value::Object(spec.clone()));
    Ok(Value::Object(payload))
}

pub(crate) fn normalize_dashboard_v2_update_payload(
    source: &Value,
    live: &Value,
    namespace: &str,
    folder_uid: Option<&str>,
    message_text: &str,
) -> Result<Value> {
    let mut payload =
        normalize_dashboard_v2_create_payload(source, namespace, folder_uid, message_text)?;
    let resource_version = live
        .get("metadata")
        .and_then(Value::as_object)
        .and_then(|metadata| metadata.get("resourceVersion"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|version| !version.is_empty())
        .ok_or_else(|| {
            message(
                "Live dashboard resource is missing metadata.resourceVersion required for update.",
            )
        })?;
    payload["metadata"]["resourceVersion"] = Value::String(resource_version.to_string());
    Ok(payload)
}

fn comparable_resource(value: &Value) -> Result<Value> {
    let mut normalized = normalize_dashboard_v2_create_payload(
        value,
        value
            .get("metadata")
            .and_then(Value::as_object)
            .and_then(|m| m.get("namespace"))
            .and_then(Value::as_str)
            .unwrap_or("default"),
        None,
        "",
    )?;
    if let Some(labels) = normalized
        .pointer_mut("/metadata/labels")
        .and_then(Value::as_object_mut)
    {
        labels.remove("grafana.app/deprecatedInternalID");
    }
    if normalized
        .pointer("/metadata/labels")
        .and_then(Value::as_object)
        .is_some_and(Map::is_empty)
    {
        normalized
            .as_object_mut()
            .and_then(|o| o.get_mut("metadata"))
            .and_then(Value::as_object_mut)
            .map(|m| m.remove("labels"));
    }
    strip_grafana_v2_schema_defaults(&mut normalized);
    Ok(normalized)
}

fn strip_grafana_v2_schema_defaults(value: &mut Value) {
    let Some(spec) = value.get_mut("spec").and_then(Value::as_object_mut) else {
        return;
    };
    for (key, default) in [
        ("annotations", Value::Null),
        ("cursorSync", Value::String(String::new())),
        ("links", Value::Null),
        ("preload", Value::Bool(false)),
        ("tags", Value::Null),
        ("variables", Value::Null),
    ] {
        if spec.get(key) == Some(&default) {
            spec.remove(key);
        }
    }
    let default_layout = serde_json::json!({
        "kind": "GridLayout",
        "spec": {"items": null}
    });
    if spec.get("layout") == Some(&default_layout) {
        spec.remove("layout");
    }
    let default_time_settings = serde_json::json!({
        "from": "",
        "to": "",
        "autoRefresh": "",
        "autoRefreshIntervals": null,
        "hideTimepicker": false,
        "fiscalYearStartMonth": 0
    });
    if spec.get("timeSettings") == Some(&default_time_settings) {
        spec.remove("timeSettings");
    }
}

pub(crate) fn dashboard_v2_resources_equal(desired: &Value, live: &Value) -> Result<bool> {
    Ok(comparable_resource(desired)? == comparable_resource(live)?)
}

pub(crate) fn dashboard_v2_managed_source(value: &Value) -> Option<&'static str> {
    let metadata = value.get("metadata")?.as_object()?;
    let annotations = metadata.get("annotations").and_then(Value::as_object);
    let managed_by = metadata
        .get("managedBy")
        .or_else(|| annotations.and_then(|annotations| annotations.get("grafana.app/managedBy")));
    let managed_by_kind = managed_by.and_then(|value| match value {
        Value::String(value) => Some(value.as_str()),
        Value::Object(value) => value.get("kind").and_then(Value::as_str),
        _ => None,
    });
    let has_managed_by = managed_by.is_some_and(|value| match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(value) => !value.is_empty(),
        _ => true,
    });
    let git_sync_managed = managed_by_kind.is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "repo" | "repository" | "git" | "git-sync" | "git sync"
        )
    }) || ["managedRepository", "repository"]
        .iter()
        .any(|key| metadata.get(*key).is_some_and(|value| !value.is_null()));
    if git_sync_managed {
        Some("Git Sync-managed")
    } else if metadata.get("provisioned").and_then(Value::as_bool) == Some(true)
        || metadata
            .get("provisionedExternalId")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
        || annotations
            .and_then(|annotations| annotations.get("grafana.app/provisioned"))
            .and_then(Value::as_str)
            .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    {
        Some("file-provisioned")
    } else if has_managed_by {
        Some("managed-unknown")
    } else {
        None
    }
}

fn resource_ownership(value: Option<&Value>) -> Option<String> {
    value.map(|value| {
        dashboard_v2_managed_source(value)
            .unwrap_or("api-managed")
            .to_string()
    })
}

fn collect_changed_fields(prefix: &str, desired: &Value, live: &Value, fields: &mut Vec<String>) {
    match (desired, live) {
        (Value::Object(desired), Value::Object(live)) => {
            let keys = desired
                .keys()
                .chain(live.keys())
                .collect::<std::collections::BTreeSet<_>>();
            for key in keys {
                let field = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{prefix}.{key}")
                };
                match (desired.get(key), live.get(key)) {
                    (Some(desired), Some(live)) => {
                        collect_changed_fields(&field, desired, live, fields)
                    }
                    _ => fields.push(field),
                }
            }
        }
        _ if desired != live => fields.push(prefix.to_string()),
        _ => {}
    }
}

fn changed_fields(desired: &Value, live: Option<&Value>) -> Result<Vec<String>> {
    let Some(live) = live else {
        return Ok(Vec::new());
    };
    let desired = comparable_resource(desired)?;
    let live = comparable_resource(live)?;
    let mut fields = Vec::new();
    collect_changed_fields("", &desired, &live, &mut fields);
    Ok(fields)
}

#[derive(Debug)]
struct ResourceV2Write {
    method: Method,
    path: String,
    payload: Option<Value>,
}

fn redact_resource_v2_apply_error(error: GrafanaCliError) -> GrafanaCliError {
    let rendered = redact_resource_v2_text(&error.to_string());
    match error.status_code() {
        Some(status_code) => api_response(status_code, "resource-v2 apply", rendered),
        None => message(rendered),
    }
}

pub(crate) fn run_dashboard_v2_import_with_request<F>(
    request: &mut F,
    args: &crate::dashboard::ImportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let resources = discover_dashboard_v2_resources(&args.input_dir)?;
    let namespace = args
        .resource_namespace
        .as_deref()
        .unwrap_or("default")
        .trim();
    if namespace.is_empty() {
        return Err(message("--resource-namespace must be non-empty."));
    }
    let collection = dashboard_resource_path(DashboardResourceApiVersion::V2, namespace)?;
    match request(Method::GET, &collection, &[("limit".into(), "1".into())], None) {
        Ok(value) => {
            validate_dashboard_resource_v2_list_response(value, namespace)?;
        }
        Err(error) if error.status_code() == Some(404) => return Err(message(format!("Grafana does not serve dashboard.grafana.app/v2 in namespace {namespace}; no classic API fallback was attempted."))),
        Err(error) => return Err(error),
    }

    let mut planned = Vec::new();
    for resource in &resources {
        let item_path = dashboard_resource_item_path(
            DashboardResourceApiVersion::V2,
            namespace,
            &resource.name,
        )?;
        let live = match request(Method::GET, &item_path, &[], None) {
            Ok(value) => Some(validate_dashboard_resource_v2_response(
                value,
                namespace,
                &resource.name,
            )?),
            Err(error) if error.status_code() == Some(404) => None,
            Err(error) => return Err(error),
        };
        let desired = normalize_dashboard_v2_create_payload(
            &resource.document,
            namespace,
            args.import_folder_uid.as_deref(),
            &args.import_message,
        )?;
        let (action, blocker) = match live.as_ref() {
            None if args.update_existing_only => ("skip", Some("missing resource".to_string())),
            None => ("create", None),
            Some(value) if dashboard_v2_managed_source(value).is_some() => (
                "blocked",
                Some(format!(
                    "{} resource must be updated at its owning source",
                    dashboard_v2_managed_source(value).unwrap()
                )),
            ),
            Some(value) if dashboard_v2_resources_equal(value, &desired)? => ("no_change", None),
            Some(_) if !args.replace_existing && !args.update_existing_only => (
                "blocked",
                Some("existing resource requires --replace-existing".to_string()),
            ),
            Some(_) => ("update", None),
        };
        planned.push((resource, live, desired, action, blocker));
    }

    let mut rows = planned
        .iter()
        .map(|(resource, live, desired, action, blocker)| {
            let action = match *action {
                "create" => ResourceV2DryRunAction::Create,
                "update" => ResourceV2DryRunAction::Update,
                "no_change" => ResourceV2DryRunAction::NoChange,
                "skip" => ResourceV2DryRunAction::Skip,
                _ => ResourceV2DryRunAction::Blocked,
            };
            let mut row = ResourceV2DryRunRow::new(
                API_VERSION,
                namespace,
                &resource.name,
                resource.path.display().to_string(),
                action,
            );
            row.folder = desired
                .pointer("/metadata/annotations/grafana.app~1folder")
                .and_then(Value::as_str)
                .map(str::to_string);
            row.ownership = resource_ownership(live.as_ref());
            row.blocker = blocker.clone();
            row.changed_fields = changed_fields(desired, live.as_ref())?;
            Ok(row)
        })
        .collect::<Result<Vec<_>>>()?;

    if args.interactive {
        #[cfg(feature = "tui")]
        {
            use crate::dashboard::import_interactive_resource_v2::{
                run_resource_v2_interactive_review, ResourceV2Ownership, ResourceV2ReviewAction,
                ResourceV2ReviewRow,
            };
            let review_rows = rows
                .iter()
                .map(|row| ResourceV2ReviewRow {
                    source_path: PathBuf::from(&row.source_path),
                    namespace: row.namespace.clone(),
                    api_version: row.api_version.clone(),
                    name: row.name.clone(),
                    folder: row.folder.clone(),
                    ownership: match row.ownership.as_deref() {
                        Some("api-managed") => ResourceV2Ownership::ApiManaged,
                        Some("file-provisioned") => ResourceV2Ownership::FileProvisioned,
                        Some("Git Sync-managed" | "git-sync-managed") => {
                            ResourceV2Ownership::GitSyncManaged
                        }
                        Some("managed-unknown") => ResourceV2Ownership::ManagedUnknown,
                        Some(value) => ResourceV2Ownership::Unknown(value.to_string()),
                        None => ResourceV2Ownership::ApiManaged,
                    },
                    action: match row.action {
                        ResourceV2DryRunAction::Create => ResourceV2ReviewAction::Create,
                        ResourceV2DryRunAction::Update => ResourceV2ReviewAction::Update,
                        ResourceV2DryRunAction::NoChange => ResourceV2ReviewAction::NoChange,
                        ResourceV2DryRunAction::Skip => ResourceV2ReviewAction::Skip,
                        ResourceV2DryRunAction::Blocked => ResourceV2ReviewAction::Blocked,
                    },
                    blocker: row.blocker.clone(),
                    changed_fields: row.changed_fields.clone(),
                })
                .collect();
            let Some(selected) = run_resource_v2_interactive_review(review_rows, args.dry_run)?
            else {
                return Ok(0);
            };
            let selected = selected
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>();
            planned.retain(|(resource, _, _, _, _)| selected.contains(&resource.path));
            rows.retain(|row| selected.contains(&PathBuf::from(&row.source_path)));
        }
        #[cfg(not(feature = "tui"))]
        {
            return crate::dashboard::tui_not_built("import --interactive");
        }
    }

    if args.dry_run {
        if args.json || matches!(args.output_format, Some(DryRunOutputFormat::Json)) {
            print!("{}", render_resource_v2_dry_run_json(&rows)?);
        } else if args.table || matches!(args.output_format, Some(DryRunOutputFormat::Table)) {
            for line in render_resource_v2_dry_run_table(
                &rows,
                !args.no_header,
                (!args.output_columns.is_empty()).then_some(args.output_columns.as_slice()),
            ) {
                println!("{line}");
            }
        } else {
            let text = render_resource_v2_dry_run_text(&rows);
            if !text.is_empty() {
                println!("{text}");
            }
        }
        return Ok(planned.len());
    }
    if let Some((resource, _, _, _, blocker)) = planned
        .iter()
        .find(|(_, _, _, action, _)| *action == "blocked")
    {
        return Err(message(format!(
            "Refusing resource-v2 import for {}: {}",
            resource.name,
            blocker.as_deref().unwrap_or("blocked")
        )));
    }
    let operations = planned
        .into_iter()
        .map(|(resource, live, desired, action, _)| {
            let (action, write) = match action {
                "create" => (
                    ResourceV2ApplyAction::Create,
                    ResourceV2Write {
                        method: Method::POST,
                        path: collection.clone(),
                        payload: Some(desired),
                    },
                ),
                "update" => {
                    let payload = normalize_dashboard_v2_update_payload(
                        &resource.document,
                        live.as_ref().unwrap(),
                        namespace,
                        args.import_folder_uid.as_deref(),
                        &args.import_message,
                    )?;
                    (
                        ResourceV2ApplyAction::Update,
                        ResourceV2Write {
                            method: Method::PUT,
                            path: dashboard_resource_item_path(
                                DashboardResourceApiVersion::V2,
                                namespace,
                                &resource.name,
                            )?,
                            payload: Some(payload),
                        },
                    )
                }
                _ => (
                    ResourceV2ApplyAction::NoOp,
                    ResourceV2Write {
                        method: Method::GET,
                        path: String::new(),
                        payload: None,
                    },
                ),
            };
            Ok(ResourceV2ApplyOperation {
                identity: resource.name.clone(),
                action,
                payload: write,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let report = execute_resource_v2_apply(operations, |operation| {
        let write = &operation.payload;
        let response = request(
            write.method.clone(),
            &write.path,
            &[],
            write.payload.as_ref(),
        )?;
        validate_dashboard_resource_v2_response(response, namespace, &operation.identity)
            .map(|_| ())
    })
    .map_err(|failure| {
        let completed = failure
            .completed
            .iter()
            .map(|item| item.identity.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let not_attempted = failure
            .not_attempted
            .iter()
            .map(|item| item.identity.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        redact_resource_v2_apply_error(failure.write_error).with_context(format!(
            "Resource-v2 partial apply: completed=[{completed}]; failed={}; not-attempted=[{not_attempted}]; no rollback was attempted",
            failure.failed.identity
        ))
    })?;
    Ok(report
        .completed
        .iter()
        .filter(|item| item.action != ResourceV2ApplyAction::NoOp)
        .count())
}
