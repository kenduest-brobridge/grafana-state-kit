//! Opt-in portable native-v2 export policy.

use super::ResourceExportLane;
use crate::common::{message, Result};
use crate::dashboard::{DashboardResourceFormat, ExportArgs};
use crate::grafana_api::DashboardResourceApiVersion;
use serde_json::Value;
use std::borrow::Cow;

pub(crate) fn validate_portable_v2_export_args(args: &ExportArgs) -> Result<()> {
    if args.portable_v2
        && !matches!(
            args.resource_format,
            DashboardResourceFormat::V2 | DashboardResourceFormat::All
        )
    {
        return Err(message(
            "--portable-v2 requires --resource-format v2 or all, or --native.",
        ));
    }
    Ok(())
}

pub(crate) fn validate_portable_v2_served_lanes(
    portable_v2: bool,
    lanes: &[ResourceExportLane],
) -> Result<()> {
    if portable_v2
        && !lanes
            .iter()
            .any(|lane| lane.version == DashboardResourceApiVersion::V2)
    {
        return Err(message("--portable-v2 requires the dashboard.grafana.app/v2 API to be served by this Grafana instance."));
    }
    Ok(())
}

pub(crate) fn dashboard_resource_export_document<'a>(
    resource: &'a Value,
    lane: ResourceExportLane,
    namespace: &str,
    portable_v2: bool,
) -> Result<Cow<'a, Value>> {
    if !portable_v2 || lane.version != DashboardResourceApiVersion::V2 {
        return Ok(Cow::Borrowed(resource));
    }
    if resource.get("apiVersion").and_then(Value::as_str) != Some(lane.api_version)
        || resource.get("kind").and_then(Value::as_str) != Some("Dashboard")
        || resource.get("spec").and_then(Value::as_object).is_none()
    {
        return Err(message(
            "Portable v2 export requires a dashboard.grafana.app/v2 Dashboard with an object spec.",
        ));
    }
    let metadata = resource
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| message("Portable v2 export requires object metadata."))?;
    if metadata
        .get("name")
        .and_then(Value::as_str)
        .is_none_or(|name| name.trim().is_empty())
        || metadata.contains_key("generateName")
    {
        return Err(message(
            "Portable v2 export requires a stable, non-empty metadata.name without generateName.",
        ));
    }
    if namespace.trim().is_empty()
        || metadata.get("namespace").and_then(Value::as_str) != Some(namespace)
    {
        return Err(message(format!("Portable v2 export requires metadata.namespace to match the requested namespace `{namespace}`.")));
    }
    let mut document = resource.clone();
    let metadata = document
        .get_mut("metadata")
        .and_then(Value::as_object_mut)
        .unwrap();
    // Kubernetes ObjectMeta read-only/server fields; retain ownership and unknown metadata.
    // https://github.com/kubernetes/apimachinery/blob/v0.34.1/pkg/apis/meta/v1/types.go
    for key in [
        "uid",
        "resourceVersion",
        "generation",
        "creationTimestamp",
        "deletionTimestamp",
        "deletionGracePeriodSeconds",
        "managedFields",
        "selfLink",
    ] {
        metadata.remove(key);
    }
    // Grafana's pull processor removes these exact server annotations and internal ID label.
    // https://github.com/grafana/gcx/blob/main/internal/resources/process/serverfields.go
    if let Some(annotations) = metadata
        .get_mut("annotations")
        .and_then(Value::as_object_mut)
    {
        for key in [
            "grafana.app/createdBy",
            "grafana.app/updatedBy",
            "grafana.app/updatedTimestamp",
        ] {
            annotations.remove(key);
        }
    }
    if let Some(labels) = metadata.get_mut("labels").and_then(Value::as_object_mut) {
        labels.remove("grafana.app/deprecatedInternalID");
    }
    document.as_object_mut().unwrap().remove("status");
    Ok(Cow::Owned(document))
}

#[cfg(test)]
mod portable_v2_tests {
    use super::*;
    use serde_json::json;

    fn source() -> Value {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../../tests/fixtures/dashboard_resource_v2_import_golden.json"
        ))
        .unwrap();
        let mut source = fixture["source"].clone();
        source["metadata"]["labels"]["grafana.app/deprecatedInternalID"] = json!("42");
        source["metadata"]["annotations"]["grafana.app/managedBy"] = json!("repo");
        source["metadata"]["annotations"]["grafana.app/sourcePath"] = json!("dashboards/cpu.json");
        source["metadata"]["annotations"]["grafana.app/futureAnnotation"] = json!("retain");
        source["metadata"]["ownerReferences"] = json!([{"uid":"controller-uid"}]);
        source["metadata"]["finalizers"] = json!(["custom.example/finalizer"]);
        source["metadata"]["selfLink"] = json!("/server/path");
        source["metadata"]["deletionTimestamp"] = json!("2026-07-03T00:00:00Z");
        source["metadata"]["deletionGracePeriodSeconds"] = json!(30);
        source["status"] = json!({"observedGeneration":7});
        source["futureTopLevel"] = json!({"opaque":true});
        source
    }

    #[test]
    fn portable_v2_removes_only_verified_server_fields_and_preserves_ownership() {
        let source = source();
        let lane = ResourceExportLane::for_version(DashboardResourceApiVersion::V2).unwrap();
        let portable =
            dashboard_resource_export_document(&source, lane, "source-namespace", true).unwrap();
        let mut expected = source.clone();
        let metadata = expected["metadata"].as_object_mut().unwrap();
        for key in [
            "uid",
            "resourceVersion",
            "generation",
            "creationTimestamp",
            "managedFields",
            "selfLink",
            "deletionTimestamp",
            "deletionGracePeriodSeconds",
        ] {
            metadata.remove(key);
        }
        let annotations = metadata["annotations"].as_object_mut().unwrap();
        for key in [
            "grafana.app/createdBy",
            "grafana.app/updatedBy",
            "grafana.app/updatedTimestamp",
        ] {
            annotations.remove(key);
        }
        metadata["labels"]
            .as_object_mut()
            .unwrap()
            .remove("grafana.app/deprecatedInternalID");
        expected.as_object_mut().unwrap().remove("status");
        assert_eq!(*portable, expected);
        assert_eq!(portable["spec"], source["spec"]);
        assert_eq!(
            portable["metadata"]["updatedTimestamp"],
            source["metadata"]["updatedTimestamp"]
        );
        assert_eq!(portable["futureTopLevel"], source["futureTopLevel"]);
        assert_eq!(
            crate::dashboard::import::dashboard_v2_managed_source(&portable),
            Some("Git Sync-managed")
        );
    }

    #[test]
    fn portable_v2_keeps_archival_and_v1_exports_unchanged() {
        let source = source();
        for (version, enabled) in [
            (DashboardResourceApiVersion::V2, false),
            (DashboardResourceApiVersion::V1, true),
        ] {
            let lane = ResourceExportLane::for_version(version).unwrap();
            let exported =
                dashboard_resource_export_document(&source, lane, "source-namespace", enabled)
                    .unwrap();
            assert!(matches!(exported, Cow::Borrowed(_)));
            assert_eq!(*exported, source);
        }
    }

    #[test]
    fn portable_v2_rejects_wrong_or_missing_identity_and_namespace() {
        let lane = ResourceExportLane::for_version(DashboardResourceApiVersion::V2).unwrap();
        for (pointer, value) in [
            ("/apiVersion", json!("dashboard.grafana.app/v2beta1")),
            ("/kind", json!("Folder")),
            ("/spec", Value::Null),
            ("/metadata/name", json!("")),
            ("/metadata/namespace", json!("target-namespace")),
            ("/metadata/namespace", Value::Null),
        ] {
            let mut invalid = source();
            *invalid.pointer_mut(pointer).unwrap() = value;
            assert!(
                dashboard_resource_export_document(&invalid, lane, "source-namespace", true)
                    .is_err(),
                "{pointer}"
            );
        }
        let mut invalid = source();
        invalid["metadata"]
            .as_object_mut()
            .unwrap()
            .remove("namespace");
        assert!(
            dashboard_resource_export_document(&invalid, lane, "source-namespace", true).is_err()
        );
        let mut invalid = source();
        invalid["metadata"]["generateName"] = json!("prefix");
        assert!(
            dashboard_resource_export_document(&invalid, lane, "source-namespace", true).is_err()
        );
    }

    #[test]
    fn portable_v2_requires_a_served_v2_lane() {
        let v1 = ResourceExportLane::for_version(DashboardResourceApiVersion::V1).unwrap();
        let v2 = ResourceExportLane::for_version(DashboardResourceApiVersion::V2).unwrap();
        assert!(validate_portable_v2_served_lanes(true, &[v1]).is_err());
        assert!(validate_portable_v2_served_lanes(true, &[]).is_err());
        assert!(validate_portable_v2_served_lanes(true, &[v1, v2]).is_ok());
        assert!(validate_portable_v2_served_lanes(false, &[v1]).is_ok());
    }
}
