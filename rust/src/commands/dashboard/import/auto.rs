use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use reqwest::Method;
use serde_json::Value;

use crate::common::{message, Result};
use crate::dashboard::{DashboardImportInputFormat, ImportArgs};
use crate::grafana_api::{
    dashboard_resource_path, validate_dashboard_resource_v2_list_response,
    DashboardResourceApiVersion,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DashboardAutoRoutingDecision {
    selected: &'static str,
    namespace: Option<String>,
    api: String,
    fallback: &'static str,
    reason: String,
}

impl DashboardAutoRoutingDecision {
    pub(super) fn stderr_line(&self) -> String {
        let namespace = self
            .namespace
            .as_deref()
            .map(|value| format!(" namespace={value}"))
            .unwrap_or_default();
        format!(
            "Dashboard import routing: input=auto selected={}{} api={} fallback={} reason={}",
            self.selected, namespace, self.api, self.fallback, self.reason
        )
    }
}

#[derive(Debug, Clone)]
pub(super) struct ResolvedAutoImport {
    pub(super) args: ImportArgs,
    pub(super) resource_v2_preflight_complete: bool,
    pub(super) decision: Option<DashboardAutoRoutingDecision>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct DashboardImportInventory {
    pub(super) raw: bool,
    pub(super) provisioning: bool,
    pub(super) resource_v1: bool,
    pub(super) resource_v2: bool,
    resource_v2_namespaces: BTreeSet<String>,
}

impl DashboardImportInventory {
    pub(super) fn resource_v2_namespace(&self, explicit: Option<&str>) -> Result<String> {
        if let Some(explicit) = explicit {
            let explicit = explicit.trim();
            if explicit.is_empty() {
                return Err(message("--resource-namespace must be non-empty."));
            }
            return Ok(explicit.to_string());
        }
        match self.resource_v2_namespaces.len() {
            0 => Ok("default".to_string()),
            1 => Ok(self.resource_v2_namespaces.iter().next().unwrap().clone()),
            _ => Err(message(format!(
                "Dashboard resource-v2 input contains multiple namespaces ({}); pass --resource-namespace to select the destination explicitly.",
                self.resource_v2_namespaces
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }

    fn has_any_lane(&self) -> bool {
        self.raw || self.provisioning || self.resource_v1 || self.resource_v2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IndexedLane {
    Raw,
    Provisioning,
    ResourceV1,
    ResourceV2,
}

impl IndexedLane {
    fn field_name(self) -> &'static str {
        match self {
            Self::Raw => "raw_path",
            Self::Provisioning => "provisioning_path",
            Self::ResourceV1 => "resource_v1_path",
            Self::ResourceV2 => "resource_v2_path",
        }
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path).map_err(|error| {
        message(format!(
            "Unable to read dashboard input {}: {error}",
            path.display()
        ))
    })?;
    serde_json::from_str(&text).map_err(|error| {
        message(format!(
            "Invalid dashboard JSON in {}: {error}",
            path.display()
        ))
    })
}

fn resource_api_version(document: &Value) -> Option<&str> {
    let object = document.as_object()?;
    if object.get("kind").and_then(Value::as_str) != Some("Dashboard") {
        return None;
    }
    object
        .get("apiVersion")
        .and_then(Value::as_str)
        .filter(|version| version.starts_with("dashboard.grafana.app/"))
}

fn classify_document(
    inventory: &mut DashboardImportInventory,
    path: &Path,
    expected_lane: Option<IndexedLane>,
) -> Result<()> {
    let document = read_json(path)?;
    if document
        .as_object()
        .is_some_and(|object| object.contains_key("__inputs") || object.contains_key("__requires"))
    {
        return Err(message(format!(
            "Dashboard input {} is a Grafana UI prompt document; use the UI import flow or convert it to a Classic raw export before dashboard import.",
            path.display()
        )));
    }
    let version = resource_api_version(&document);
    match version {
        Some("dashboard.grafana.app/v1") => {
            if matches!(
                expected_lane,
                Some(IndexedLane::Raw | IndexedLane::Provisioning)
            ) {
                return Err(message(format!(
                    "Dashboard input {} is a resource-v1 object but its export index classifies it as Classic.",
                    path.display()
                )));
            }
            inventory.resource_v1 = true;
        }
        Some("dashboard.grafana.app/v2") => {
            if matches!(
                expected_lane,
                Some(IndexedLane::Raw | IndexedLane::Provisioning)
            ) {
                return Err(message(format!(
                    "Dashboard input {} is a resource-v2 object but its export index classifies it as Classic.",
                    path.display()
                )));
            }
            let has_name = document
                .pointer("/metadata/name")
                .and_then(Value::as_str)
                .is_some_and(|name| !name.trim().is_empty());
            let has_spec = document.get("spec").and_then(Value::as_object).is_some();
            if !has_name || !has_spec {
                return Err(message(format!(
                    "Dashboard resource-v2 input {} requires a non-empty metadata.name and object-valued spec.",
                    path.display()
                )));
            }
            inventory.resource_v2 = true;
            let namespace = document
                .pointer("/metadata/namespace")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|namespace| !namespace.is_empty())
                .unwrap_or("default");
            inventory
                .resource_v2_namespaces
                .insert(namespace.to_string());
        }
        Some(other) if other.starts_with("dashboard.grafana.app/") => {
            return Err(message(format!(
                "Dashboard input {} uses unsupported resource API version {other:?}.",
                path.display()
            )));
        }
        _ => match expected_lane {
            Some(IndexedLane::ResourceV1 | IndexedLane::ResourceV2) => {
                return Err(message(format!(
                    "Dashboard input {} does not match the resource lane recorded by its export index.",
                    path.display()
                )));
            }
            Some(IndexedLane::Provisioning) => inventory.provisioning = true,
            Some(IndexedLane::Raw) | None => inventory.raw = true,
        },
    }
    Ok(())
}

fn resolve_indexed_path(root: &Path, canonical_root: &Path, value: &str) -> Result<PathBuf> {
    let indexed_path = PathBuf::from(value);
    reject_indexed_path_symlinks(root, &indexed_path)?;
    let candidate = if indexed_path.is_absolute() {
        indexed_path
    } else {
        root.join(indexed_path)
    };
    let canonical = fs::canonicalize(&candidate).map_err(|error| {
        message(format!(
            "Dashboard export index path cannot be resolved: {}: {error}",
            candidate.display()
        ))
    })?;
    if !canonical.starts_with(canonical_root) {
        return Err(message(format!(
            "Dashboard export index path resolves outside input root: {}",
            candidate.display()
        )));
    }
    if !canonical.is_file() {
        return Err(message(format!(
            "Dashboard export index path is not a file: {}",
            candidate.display()
        )));
    }
    Ok(canonical)
}

fn reject_indexed_path_symlinks(root: &Path, indexed_path: &Path) -> Result<()> {
    let relative = if indexed_path.is_absolute() {
        indexed_path.strip_prefix(root).map_err(|_| {
            message(format!(
                "Dashboard export index path is outside input root: {}",
                indexed_path.display()
            ))
        })?
    } else {
        indexed_path
    };
    let mut current = root.to_path_buf();
    for component in relative.components() {
        match component {
            Component::CurDir => continue,
            Component::Normal(value) => current.push(value),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(message(format!(
                    "Dashboard export index path contains an unsafe component: {}",
                    indexed_path.display()
                )))
            }
        }
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(message(format!(
                    "Dashboard auto-detection does not follow symbolic links: {}",
                    current.display()
                )))
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn inventory_from_export_index(root: &Path) -> Result<Option<DashboardImportInventory>> {
    let index_path = root.join("index.json");
    match fs::symlink_metadata(&index_path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(message(format!(
                "Dashboard auto-detection does not follow symbolic links: {}",
                index_path.display()
            )))
        }
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    let index = read_json(&index_path)?;
    if index.get("kind").and_then(Value::as_str) != Some(crate::dashboard::ROOT_INDEX_KIND) {
        return Ok(None);
    }
    let items = index
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            message(format!(
                "Dashboard export index must contain an items array: {}",
                index_path.display()
            ))
        })?;
    let canonical_root = fs::canonicalize(root)?;
    let lanes = [
        IndexedLane::Raw,
        IndexedLane::Provisioning,
        IndexedLane::ResourceV1,
        IndexedLane::ResourceV2,
    ];
    let mut inventory = DashboardImportInventory::default();
    for item in items {
        for lane in lanes {
            let Some(path) = item.get(lane.field_name()).and_then(Value::as_str) else {
                continue;
            };
            let resolved = resolve_indexed_path(root, &canonical_root, path)?;
            classify_document(&mut inventory, &resolved, Some(lane))?;
        }
    }
    if !inventory.has_any_lane() {
        return Err(message(format!(
            "Dashboard export index contains no importable dashboard paths: {}",
            index_path.display()
        )));
    }
    Ok(Some(inventory))
}

fn collect_json_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_symlink() {
            return Err(message(format!(
                "Dashboard auto-detection does not follow symbolic links: {}",
                path.display()
            )));
        }
        if file_type.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) != Some("history") {
                collect_json_files(&path, files)?;
            }
        } else if path.extension().and_then(|value| value.to_str()) == Some("json") {
            let name = path.file_name().and_then(|value| value.to_str());
            if !matches!(
                name,
                Some(
                    "index.json"
                        | "export-metadata.json"
                        | "folders.json"
                        | "datasources.json"
                        | "permissions.json"
                        | "_folder.json"
                )
            ) {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn inventory_known_lane_directories(root: &Path) -> Result<Option<DashboardImportInventory>> {
    let lane_dirs = [
        (IndexedLane::Raw, root.join("classic/raw")),
        (IndexedLane::Raw, root.join("raw")),
        (
            IndexedLane::Provisioning,
            root.join("classic/provisioning/dashboards"),
        ),
        (
            IndexedLane::Provisioning,
            root.join("provisioning/dashboards"),
        ),
        (IndexedLane::ResourceV1, root.join("resource-v1/objects")),
        (IndexedLane::ResourceV2, root.join("resource-v2/objects")),
    ];
    let mut inventory = DashboardImportInventory::default();
    let mut found = false;
    for (lane, directory) in lane_dirs {
        let metadata = match fs::symlink_metadata(&directory) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        if metadata.file_type().is_symlink() {
            return Err(message(format!(
                "Dashboard auto-detection does not follow symbolic links: {}",
                directory.display()
            )));
        }
        if !metadata.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        collect_json_files(&directory, &mut files)?;
        if files.is_empty() {
            continue;
        }
        found = true;
        for path in files {
            classify_document(&mut inventory, &path, Some(lane))?;
        }
    }
    Ok(found.then_some(inventory))
}

fn is_provisioning_root(root: &Path) -> bool {
    root.join("dashboards").is_dir()
        && (root.join("provisioning/dashboards.yaml").is_file()
            || root.file_name().and_then(|name| name.to_str()) == Some("provisioning"))
}

pub(super) fn inventory_dashboard_import_source(
    input_dir: &Path,
) -> Result<DashboardImportInventory> {
    match fs::symlink_metadata(input_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(message(format!(
                "Dashboard auto-detection does not follow symbolic links: {}",
                input_dir.display()
            )))
        }
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => {
            return Err(message(format!(
                "Import path is not a directory: {}",
                input_dir.display()
            )))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(message(format!(
                "Import directory does not exist: {}",
                input_dir.display()
            )))
        }
        Err(error) => return Err(error.into()),
    }
    if let Some(inventory) = inventory_from_export_index(input_dir)? {
        return Ok(inventory);
    }
    if let Some(inventory) = inventory_known_lane_directories(input_dir)? {
        return Ok(inventory);
    }
    if is_provisioning_root(input_dir) {
        let mut files = Vec::new();
        collect_json_files(&input_dir.join("dashboards"), &mut files)?;
        if files.is_empty() {
            return Err(message(format!(
                "No dashboard JSON files found in {}",
                input_dir.join("dashboards").display()
            )));
        }
        let mut inventory = DashboardImportInventory::default();
        for path in files {
            classify_document(&mut inventory, &path, Some(IndexedLane::Provisioning))?;
        }
        return Ok(inventory);
    }

    let mut files = Vec::new();
    collect_json_files(input_dir, &mut files)?;
    if files.is_empty() {
        return Err(message(format!(
            "No dashboard JSON files found in {}",
            input_dir.display()
        )));
    }
    let mut inventory = DashboardImportInventory::default();
    for path in files {
        classify_document(&mut inventory, &path, None)?;
    }
    let resource_lanes = usize::from(inventory.resource_v1) + usize::from(inventory.resource_v2);
    if (inventory.raw && resource_lanes > 0) || resource_lanes > 1 {
        return Err(message(format!(
            "Dashboard auto-detection found mixed dashboard schemas in {}; use a generated combined export root or select one lane explicitly.",
            input_dir.display()
        )));
    }
    Ok(inventory)
}

fn first_classic_only_option(args: &ImportArgs) -> Option<&'static str> {
    [
        (args.org_id.is_some(), "--org-id"),
        (args.use_export_org, "--use-export-org"),
        (!args.only_org_id.is_empty(), "--only-org-id"),
        (args.create_missing_orgs, "--create-missing-orgs"),
        (
            args.require_matching_export_org,
            "--require-matching-export-org",
        ),
        (
            args.require_matching_folder_path,
            "--require-matching-folder-path",
        ),
        (args.ensure_folders, "--ensure-folders"),
        (args.strict_schema, "--strict-schema"),
        (
            args.target_schema_version.is_some(),
            "--target-schema-version",
        ),
    ]
    .into_iter()
    .find_map(|(enabled, flag)| enabled.then_some(flag))
}

fn classic_only_options_requested(args: &ImportArgs) -> bool {
    first_classic_only_option(args).is_some()
}

fn classic_decision(
    selected: DashboardImportInputFormat,
    fallback: &'static str,
    reason: impl Into<String>,
) -> DashboardAutoRoutingDecision {
    DashboardAutoRoutingDecision {
        selected: match selected {
            DashboardImportInputFormat::Raw => "raw",
            DashboardImportInputFormat::Provisioning => "provisioning",
            DashboardImportInputFormat::Auto | DashboardImportInputFormat::ResourceV2 => {
                unreachable!("classic decision requires a Classic input lane")
            }
        },
        namespace: None,
        api: "/api/dashboards/db".to_string(),
        fallback,
        reason: reason.into(),
    }
}

fn resolved_classic(
    args: &ImportArgs,
    selected: DashboardImportInputFormat,
    fallback: &'static str,
    reason: impl Into<String>,
) -> ResolvedAutoImport {
    let mut resolved_args = args.clone();
    resolved_args.input_format = selected;
    ResolvedAutoImport {
        args: resolved_args,
        resource_v2_preflight_complete: false,
        decision: Some(classic_decision(selected, fallback, reason)),
    }
}

pub(super) fn resolve_auto_import_with_request<F>(
    request: &mut F,
    args: &ImportArgs,
) -> Result<ResolvedAutoImport>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if args.input_format != DashboardImportInputFormat::Auto {
        return Ok(ResolvedAutoImport {
            args: args.clone(),
            resource_v2_preflight_complete: false,
            decision: None,
        });
    }

    let inventory = inventory_dashboard_import_source(&args.input_dir)?;
    if inventory.raw && classic_only_options_requested(args) {
        return Ok(resolved_classic(
            args,
            DashboardImportInputFormat::Raw,
            "not-needed",
            "classic-only options requested; using the independent classic raw artifact",
        ));
    }

    if inventory.resource_v2 {
        if let Some(flag) = first_classic_only_option(args) {
            return Err(message(format!(
                "{flag} is not supported with auto-selected resource-v2 input; resource-v2 import is single-namespace and uses native schema validation."
            )));
        }
        let namespace = inventory.resource_v2_namespace(args.resource_namespace.as_deref())?;
        let collection = dashboard_resource_path(DashboardResourceApiVersion::V2, &namespace)?;
        let params = [("limit".to_string(), "1".to_string())];
        match request(Method::GET, &collection, &params, None) {
            Ok(value) => {
                validate_dashboard_resource_v2_list_response(value, &namespace)?;
                let mut resolved_args = args.clone();
                resolved_args.input_format = DashboardImportInputFormat::ResourceV2;
                resolved_args.resource_namespace = Some(namespace.clone());
                return Ok(ResolvedAutoImport {
                    args: resolved_args,
                    resource_v2_preflight_complete: true,
                    decision: Some(DashboardAutoRoutingDecision {
                        selected: "resource-v2",
                        namespace: Some(namespace),
                        api: collection,
                        fallback: if inventory.raw {
                            "classic-raw-on-404"
                        } else {
                            "disabled"
                        },
                        reason:
                            "exact dashboard.grafana.app/v2 DashboardList endpoint is available"
                                .to_string(),
                    }),
                });
            }
            Err(error) if error.status_code() == Some(404) && inventory.raw => {
                return Ok(resolved_classic(
                    args,
                    DashboardImportInputFormat::Raw,
                    "used",
                    "resource-v2 unavailable (404); using the independent classic raw artifact",
                ));
            }
            Err(error) if error.status_code() == Some(404) => {
                return Err(message(format!(
                    "Grafana does not serve dashboard.grafana.app/v2 in namespace {namespace}, and this input contains no Classic raw artifact; no lossy fallback or schema conversion was attempted."
                )));
            }
            Err(error) => return Err(error),
        }
    }

    if inventory.raw {
        return Ok(resolved_classic(
            args,
            DashboardImportInputFormat::Raw,
            "not-needed",
            "classic raw artifact detected",
        ));
    }
    if inventory.provisioning {
        return Ok(resolved_classic(
            args,
            DashboardImportInputFormat::Provisioning,
            "not-needed",
            "Classic provisioning artifact detected",
        ));
    }
    if inventory.resource_v1 {
        return Err(message(
            "Dashboard auto-detection found only dashboard.grafana.app/v1 resources. Native resource-v1 import is not supported; re-export with Classic raw included or select a resource-v2 export from a compatible Grafana instance.",
        ));
    }
    Err(message(format!(
        "No supported dashboard import lane was found in {}.",
        args.input_dir.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use reqwest::Method;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn write_json(path: &std::path::Path, value: &serde_json::Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(
            path,
            format!("{}\n", serde_json::to_string_pretty(value).unwrap()),
        )
        .unwrap();
    }

    fn classic_dashboard(uid: &str) -> serde_json::Value {
        json!({
            "meta": {"folderUid": "general"},
            "dashboard": {"id": null, "uid": uid, "title": "CPU", "panels": []}
        })
    }

    fn resource_v2(name: &str, namespace: Option<&str>) -> serde_json::Value {
        let mut value = json!({
            "apiVersion": "dashboard.grafana.app/v2",
            "kind": "Dashboard",
            "metadata": {"name": name},
            "spec": {"title": "CPU", "elements": {}}
        });
        if let Some(namespace) = namespace {
            value["metadata"]["namespace"] = json!(namespace);
        }
        value
    }

    fn import_args(input_dir: &std::path::Path) -> crate::dashboard::ImportArgs {
        match crate::dashboard::DashboardCliArgs::try_parse_from([
            "grafana-util",
            "import",
            "--input-dir",
            input_dir.to_str().unwrap(),
            "--dry-run",
        ])
        .unwrap()
        .command
        {
            crate::dashboard::DashboardCommand::Import(args) => args,
            _ => panic!("expected dashboard import"),
        }
    }

    fn write_combined_root(root: &std::path::Path) {
        write_json(
            &root.join("classic/raw/cpu.json"),
            &classic_dashboard("cpu"),
        );
        write_json(
            &root.join("resource-v2/objects/cpu.json"),
            &resource_v2("cpu", Some("team-ops")),
        );
        write_json(
            &root.join("index.json"),
            &json!({
                "kind": crate::dashboard::ROOT_INDEX_KIND,
                "items": [{
                    "raw_path": "classic/raw/cpu.json",
                    "resource_v2_path": "resource-v2/objects/cpu.json"
                }]
            }),
        );
    }

    #[test]
    fn inventory_classifies_a_direct_classic_raw_tree() {
        let temp = tempdir().unwrap();
        write_json(&temp.path().join("cpu.json"), &classic_dashboard("cpu"));

        let inventory = inventory_dashboard_import_source(temp.path()).unwrap();

        assert!(inventory.raw);
        assert!(!inventory.provisioning);
        assert!(!inventory.resource_v1);
        assert!(!inventory.resource_v2);
    }

    #[test]
    fn inventory_classifies_resource_v2_and_infers_its_namespace() {
        let temp = tempdir().unwrap();
        write_json(
            &temp.path().join("objects/cpu.json"),
            &resource_v2("cpu", Some("team-ops")),
        );

        let inventory = inventory_dashboard_import_source(temp.path()).unwrap();

        assert!(inventory.resource_v2);
        assert!(!inventory.raw);
        assert_eq!(inventory.resource_v2_namespace(None).unwrap(), "team-ops");
        assert_eq!(
            inventory
                .resource_v2_namespace(Some("destination"))
                .unwrap(),
            "destination"
        );
    }

    #[test]
    fn inventory_rejects_malformed_resource_v2_documents_as_resource_input() {
        let temp = tempdir().unwrap();
        write_json(
            &temp.path().join("objects/cpu.json"),
            &json!({
                "apiVersion": "dashboard.grafana.app/v2",
                "kind": "Dashboard"
            }),
        );

        let error = inventory_dashboard_import_source(temp.path()).unwrap_err();

        assert!(error.to_string().contains("resource-v2"));
        assert!(error.to_string().contains("metadata.name"));
    }

    #[test]
    fn resource_v2_namespace_defaults_only_when_documents_omit_it() {
        let temp = tempdir().unwrap();
        write_json(
            &temp.path().join("objects/cpu.json"),
            &resource_v2("cpu", None),
        );

        let inventory = inventory_dashboard_import_source(temp.path()).unwrap();

        assert_eq!(inventory.resource_v2_namespace(None).unwrap(), "default");
    }

    #[test]
    fn resource_v2_namespace_treats_omitted_documents_as_default_for_conflict_checks() {
        let temp = tempdir().unwrap();
        write_json(
            &temp.path().join("objects/explicit.json"),
            &resource_v2("explicit", Some("team-a")),
        );
        write_json(
            &temp.path().join("objects/implicit.json"),
            &resource_v2("implicit", None),
        );

        let inventory = inventory_dashboard_import_source(temp.path()).unwrap();
        let error = inventory.resource_v2_namespace(None).unwrap_err();

        assert!(error.to_string().contains("multiple namespaces"));
        assert!(error.to_string().contains("default"));
        assert!(error.to_string().contains("team-a"));
    }

    #[test]
    fn resource_v2_namespace_conflict_requires_an_explicit_override() {
        let temp = tempdir().unwrap();
        write_json(
            &temp.path().join("objects/cpu.json"),
            &resource_v2("cpu", Some("team-a")),
        );
        write_json(
            &temp.path().join("objects/memory.json"),
            &resource_v2("memory", Some("team-b")),
        );

        let inventory = inventory_dashboard_import_source(temp.path()).unwrap();
        let error = inventory.resource_v2_namespace(None).unwrap_err();

        assert!(error.to_string().contains("multiple namespaces"));
        assert!(error.to_string().contains("--resource-namespace"));
    }

    #[test]
    fn inventory_uses_a_combined_export_index_for_independent_raw_and_v2_lanes() {
        let temp = tempdir().unwrap();
        write_json(
            &temp.path().join("classic/raw/cpu.json"),
            &classic_dashboard("cpu"),
        );
        write_json(
            &temp.path().join("resource-v2/objects/cpu.json"),
            &resource_v2("cpu", Some("default")),
        );
        write_json(
            &temp.path().join("index.json"),
            &json!({
                "kind": crate::dashboard::ROOT_INDEX_KIND,
                "items": [{
                    "raw_path": "classic/raw/cpu.json",
                    "resource_v2_path": "resource-v2/objects/cpu.json"
                }]
            }),
        );

        let inventory = inventory_dashboard_import_source(temp.path()).unwrap();

        assert!(inventory.raw);
        assert!(inventory.resource_v2);
        assert_eq!(inventory.resource_v2_namespace(None).unwrap(), "default");
    }

    #[test]
    fn inventory_rejects_an_export_index_path_that_escapes_the_input_root() {
        let temp = tempdir().unwrap();
        let outside = tempdir().unwrap();
        write_json(&outside.path().join("cpu.json"), &classic_dashboard("cpu"));
        write_json(
            &temp.path().join("index.json"),
            &json!({
                "kind": crate::dashboard::ROOT_INDEX_KIND,
                "items": [{"raw_path": outside.path().join("cpu.json")}]
            }),
        );

        let error = inventory_dashboard_import_source(temp.path()).unwrap_err();

        assert!(error.to_string().contains("outside input root"));
    }

    #[test]
    fn inventory_rejects_mixed_classic_and_resource_documents_without_an_export_index() {
        let temp = tempdir().unwrap();
        write_json(&temp.path().join("cpu.json"), &classic_dashboard("cpu"));
        write_json(
            &temp.path().join("memory.json"),
            &resource_v2("memory", Some("default")),
        );

        let error = inventory_dashboard_import_source(temp.path()).unwrap_err();

        assert!(error.to_string().contains("mixed dashboard schemas"));
    }

    #[test]
    fn auto_routing_rejects_grafana_ui_prompt_documents() {
        let temp = tempdir().unwrap();
        write_json(
            &temp.path().join("cpu.json"),
            &json!({
                "__inputs": [],
                "__requires": [],
                "dashboard": {"uid": "cpu", "title": "CPU", "panels": []}
            }),
        );
        let args = import_args(temp.path());

        let error = resolve_auto_import_with_request(
            &mut |_method, _path, _params, _payload| Ok(None),
            &args,
        )
        .unwrap_err();

        assert!(error.to_string().contains("Grafana UI prompt"));
    }

    #[test]
    fn inventory_recognizes_a_provisioning_root() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join("provisioning")).unwrap();
        fs::write(
            temp.path().join("provisioning/dashboards.yaml"),
            "apiVersion: 1\nproviders: []\n",
        )
        .unwrap();
        write_json(
            &temp.path().join("dashboards/cpu.json"),
            &classic_dashboard("cpu"),
        );

        let inventory = inventory_dashboard_import_source(temp.path()).unwrap();

        assert!(inventory.provisioning);
        assert!(!inventory.raw);
    }

    #[cfg(unix)]
    #[test]
    fn inventory_rejects_a_symlinked_known_lane_directory() {
        let temp = tempdir().unwrap();
        let outside = tempdir().unwrap();
        write_json(&outside.path().join("cpu.json"), &resource_v2("cpu", None));
        fs::create_dir_all(temp.path().join("resource-v2")).unwrap();
        std::os::unix::fs::symlink(outside.path(), temp.path().join("resource-v2/objects"))
            .unwrap();

        let error = inventory_dashboard_import_source(temp.path()).unwrap_err();

        assert!(error.to_string().contains("symbolic links"));
    }

    #[cfg(unix)]
    #[test]
    fn inventory_rejects_a_symlinked_export_index() {
        let temp = tempdir().unwrap();
        let outside = tempdir().unwrap();
        write_json(&temp.path().join("cpu.json"), &classic_dashboard("cpu"));
        write_json(
            &outside.path().join("index.json"),
            &json!({
                "kind": crate::dashboard::ROOT_INDEX_KIND,
                "items": [{"raw_path": "cpu.json"}]
            }),
        );
        std::os::unix::fs::symlink(
            outside.path().join("index.json"),
            temp.path().join("index.json"),
        )
        .unwrap();

        let error = inventory_dashboard_import_source(temp.path()).unwrap_err();

        assert!(error.to_string().contains("symbolic links"));
        assert!(error.to_string().contains("index.json"));
    }

    #[cfg(unix)]
    #[test]
    fn inventory_rejects_an_indexed_file_symlink_inside_the_input_root() {
        let temp = tempdir().unwrap();
        write_json(&temp.path().join("cpu.json"), &classic_dashboard("cpu"));
        std::os::unix::fs::symlink("cpu.json", temp.path().join("linked.json")).unwrap();
        write_json(
            &temp.path().join("index.json"),
            &json!({
                "kind": crate::dashboard::ROOT_INDEX_KIND,
                "items": [{"raw_path": "linked.json"}]
            }),
        );

        let error = inventory_dashboard_import_source(temp.path()).unwrap_err();

        assert!(error.to_string().contains("symbolic links"));
        assert!(error.to_string().contains("linked.json"));
    }

    #[test]
    fn auto_routing_selects_direct_classic_input_without_a_resource_probe() {
        let temp = tempdir().unwrap();
        write_json(&temp.path().join("cpu.json"), &classic_dashboard("cpu"));
        let args = import_args(temp.path());
        let mut requested = false;

        let resolved = resolve_auto_import_with_request(
            &mut |_method, _path, _params, _payload| {
                requested = true;
                Ok(None)
            },
            &args,
        )
        .unwrap();

        assert_eq!(
            resolved.args.input_format,
            crate::dashboard::DashboardImportInputFormat::Raw
        );
        assert!(!requested);
        assert!(!resolved.resource_v2_preflight_complete);
        assert!(resolved
            .decision
            .unwrap()
            .stderr_line()
            .contains("/api/dashboards/db"));
    }

    #[test]
    fn auto_routing_prefers_resource_v2_when_the_exact_collection_is_served() {
        let temp = tempdir().unwrap();
        write_combined_root(temp.path());
        let args = import_args(temp.path());
        let mut captured = None;

        let resolved = resolve_auto_import_with_request(
            &mut |method, path, params, _payload| {
                captured = Some((method, path.to_string(), params.to_vec()));
                Ok(Some(json!({
                    "apiVersion": "dashboard.grafana.app/v2",
                    "kind": "DashboardList",
                    "items": [],
                    "metadata": {}
                })))
            },
            &args,
        )
        .unwrap();

        assert_eq!(
            resolved.args.input_format,
            crate::dashboard::DashboardImportInputFormat::ResourceV2
        );
        assert_eq!(
            resolved.args.resource_namespace.as_deref(),
            Some("team-ops")
        );
        assert!(resolved.resource_v2_preflight_complete);
        let (method, path, params) = captured.unwrap();
        assert_eq!(method, Method::GET);
        assert_eq!(
            path,
            "/apis/dashboard.grafana.app/v2/namespaces/team-ops/dashboards"
        );
        assert_eq!(params, vec![("limit".to_string(), "1".to_string())]);
    }

    #[test]
    fn auto_routing_uses_its_own_raw_artifact_only_after_a_v2_404() {
        let temp = tempdir().unwrap();
        write_combined_root(temp.path());
        let args = import_args(temp.path());

        let resolved = resolve_auto_import_with_request(
            &mut |_method, path, _params, _payload| {
                Err(crate::common::api_response(
                    404,
                    path.to_string(),
                    "not found".to_string(),
                ))
            },
            &args,
        )
        .unwrap();

        assert_eq!(
            resolved.args.input_format,
            crate::dashboard::DashboardImportInputFormat::Raw
        );
        assert!(!resolved.resource_v2_preflight_complete);
        let line = resolved.decision.unwrap().stderr_line();
        assert!(line.contains("resource-v2 unavailable (404)"), "{line}");
        assert!(line.contains("classic raw"), "{line}");
    }

    #[test]
    fn auto_routing_rejects_a_malformed_v2_success_without_classic_fallback() {
        let temp = tempdir().unwrap();
        write_combined_root(temp.path());
        let args = import_args(temp.path());

        let error = resolve_auto_import_with_request(
            &mut |_method, _path, _params, _payload| {
                Ok(Some(json!({
                    "apiVersion": "dashboard.grafana.app/v2",
                    "kind": "Dashboard",
                    "items": []
                })))
            },
            &args,
        )
        .unwrap_err();

        assert!(error.to_string().contains("items array"));
        assert!(!error.to_string().contains("classic raw"));
    }

    #[test]
    fn auto_routing_never_treats_resource_authorization_failure_as_unsupported() {
        let temp = tempdir().unwrap();
        write_combined_root(temp.path());
        let args = import_args(temp.path());

        let error = resolve_auto_import_with_request(
            &mut |_method, path, _params, _payload| {
                Err(crate::common::api_response(
                    403,
                    path.to_string(),
                    "forbidden".to_string(),
                ))
            },
            &args,
        )
        .unwrap_err();

        assert_eq!(error.status_code(), Some(403));
    }

    #[test]
    fn auto_routing_rejects_resource_v2_only_input_when_the_api_is_missing() {
        let temp = tempdir().unwrap();
        write_json(
            &temp.path().join("objects/cpu.json"),
            &resource_v2("cpu", Some("default")),
        );
        let args = import_args(temp.path());

        let error = resolve_auto_import_with_request(
            &mut |_method, path, _params, _payload| {
                Err(crate::common::api_response(
                    404,
                    path.to_string(),
                    "not found".to_string(),
                ))
            },
            &args,
        )
        .unwrap_err();

        assert!(error.to_string().contains("no Classic raw artifact"));
        assert!(error.to_string().contains("no lossy fallback"));
    }

    #[test]
    fn auto_routing_rejects_classic_only_flags_before_probing_v2_only_input() {
        let temp = tempdir().unwrap();
        write_json(
            &temp.path().join("objects/cpu.json"),
            &resource_v2("cpu", Some("default")),
        );
        let mut args = import_args(temp.path());
        args.strict_schema = true;
        let mut requests = 0;

        let error = resolve_auto_import_with_request(
            &mut |_method, _path, _params, _payload| {
                requests += 1;
                Ok(Some(json!({
                    "apiVersion": "dashboard.grafana.app/v2",
                    "kind": "DashboardList",
                    "items": [],
                    "metadata": {}
                })))
            },
            &args,
        )
        .unwrap_err();

        assert!(error.to_string().contains("--strict-schema"));
        assert_eq!(requests, 0);
    }

    #[test]
    fn explicit_input_mode_bypasses_auto_inventory_and_capability_probes() {
        let temp = tempdir().unwrap();
        let mut args = import_args(temp.path());
        args.input_format = crate::dashboard::DashboardImportInputFormat::Raw;
        let mut requested = false;

        let resolved = resolve_auto_import_with_request(
            &mut |_method, _path, _params, _payload| {
                requested = true;
                Ok(None)
            },
            &args,
        )
        .unwrap();

        assert_eq!(
            resolved.args.input_format,
            crate::dashboard::DashboardImportInputFormat::Raw
        );
        assert!(resolved.decision.is_none());
        assert!(!requested);
    }

    #[test]
    fn auto_import_reuses_the_successful_v2_preflight_for_apply() {
        let temp = tempdir().unwrap();
        write_combined_root(temp.path());
        let args = import_args(temp.path());
        let collection_path =
            dashboard_resource_path(DashboardResourceApiVersion::V2, "team-ops").unwrap();
        let mut collection_requests = 0;
        let mut classic_requests = 0;

        crate::dashboard::import::import_dashboards_with_request(
            &mut |_method: Method,
                  path: &str,
                  _params: &[(String, String)],
                  _payload: Option<&Value>| {
                if path == collection_path {
                    collection_requests += 1;
                    return Ok(Some(json!({
                        "apiVersion": "dashboard.grafana.app/v2",
                        "kind": "DashboardList",
                        "items": [],
                        "metadata": {}
                    })));
                }
                if path.starts_with("/apis/dashboard.grafana.app/v2/") {
                    return Err(crate::common::api_response(
                        404,
                        path.to_string(),
                        "not found".to_string(),
                    ));
                }
                classic_requests += 1;
                Ok(None)
            },
            &args,
        )
        .unwrap();

        assert_eq!(collection_requests, 1);
        assert_eq!(classic_requests, 0);
    }

    #[test]
    fn auto_import_rejects_invalid_runtime_output_before_capability_probe() {
        let temp = tempdir().unwrap();
        write_json(
            &temp.path().join("objects/cpu.json"),
            &resource_v2("cpu", Some("default")),
        );
        let mut args = import_args(temp.path());
        args.dry_run = false;
        args.table = true;
        let mut requests = 0;

        let error = crate::dashboard::import::import_dashboards_with_request(
            &mut |_method: Method,
                  _path: &str,
                  _params: &[(String, String)],
                  _payload: Option<&Value>| {
                requests += 1;
                Ok(Some(json!({
                    "apiVersion": "dashboard.grafana.app/v2",
                    "kind": "DashboardList",
                    "items": [],
                    "metadata": {}
                })))
            },
            &args,
        )
        .unwrap_err();

        assert!(error.to_string().contains("--table"));
        assert_eq!(requests, 0);
    }
}
