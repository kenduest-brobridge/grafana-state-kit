//! Task-first `grafana-util workspace` workflow helpers.

use std::path::{Path, PathBuf};

use super::{
    attach_lineage, attach_trace_id, build_sync_plan_document,
    cli::{
        build_sync_review_document, execute_sync_bundle_preflight, execute_sync_plan,
        execute_sync_promotion_preflight, load_sync_live_array, load_sync_merged_availability,
        render_and_emit_sync_command_output,
    },
    input_normalization::{
        build_overview_args, build_status_args, load_preview_desired_specs,
        normalize_change_dashboard_inputs, select_preview_dashboard_sources,
    },
    workspace_discovery::{
        attach_discovery_to_overview, attach_discovery_to_status, attach_discovery_to_sync_output,
        discover_change_staged_inputs, emit_discovery_provenance, DiscoveredChangeInputs,
    },
    ChangeCheckArgs, ChangeInspectArgs, ChangeOutputArgs, ChangePreviewArgs, Result,
    SyncBundlePreflightArgs, SyncCommandOutput, SyncOutputFormat, SyncPlanArgs,
    SyncPromotionPreflightArgs,
};
use crate::common::{emit_plain_output, message};
use crate::overview::{
    build_overview_artifacts_with_inputs, build_overview_document, render_overview_text,
    OverviewArgs, OverviewArtifact, OverviewInputField, OverviewInputOverrides,
    OVERVIEW_ARTIFACT_BUNDLE_PREFLIGHT_KIND, OVERVIEW_ARTIFACT_PROMOTION_PREFLIGHT_KIND,
};
use crate::project_status_command::{render_project_status_text, staged_args_to_overview_args};
use crate::project_status_staged::build_staged_project_status;

fn emit_workspace_output(
    document: &impl serde::Serialize,
    text_lines: &[String],
    output: &ChangeOutputArgs,
) -> Result<()> {
    let persisted = match output.output_format {
        SyncOutputFormat::Json => serde_json::to_string_pretty(document)?,
        SyncOutputFormat::Text => text_lines.join("\n"),
    };
    if let Some(path) = output.output_file.as_deref() {
        emit_plain_output(&persisted, Some(path), false)?;
    }
    if output.output_file.is_none() || output.also_stdout {
        let rendered = match output.output_format {
            SyncOutputFormat::Json => crate::common::render_json_value(document)?,
            SyncOutputFormat::Text => persisted,
        };
        emit_plain_output(&rendered, None, false)?;
    }
    Ok(())
}

fn build_live_check_artifact(
    args: &ChangeCheckArgs,
    inputs: &mut OverviewArgs,
    availability: &serde_json::Value,
) -> Result<Option<OverviewArtifact>> {
    if inputs.target_inventory.is_some() || inputs.mapping_file.is_some() {
        return Ok(None);
    }
    let source = if let Some(path) = inputs.source_bundle.as_ref() {
        super::load_json_value(path, "Workspace package input")?
    } else {
        if inputs.desired_file.is_none()
            && inputs.dashboard_export_dir.is_none()
            && inputs.dashboard_provisioning_dir.is_none()
            && inputs.datasource_provisioning_file.is_none()
            && args.inputs.datasource_export_file.is_none()
            && inputs.alert_export_dir.is_none()
        {
            return Ok(None);
        }
        let desired = load_preview_desired_specs(
            &args.inputs,
            inputs.desired_file.as_ref(),
            None,
            inputs.dashboard_export_dir.as_ref(),
            inputs.dashboard_provisioning_dir.as_ref(),
            inputs.alert_export_dir.as_ref(),
            inputs.datasource_provisioning_file.as_ref(),
        )?;
        let mut dashboards = Vec::new();
        let mut datasources = Vec::new();
        let mut folders = Vec::new();
        let mut alerts = Vec::new();
        for value in desired {
            let kind = super::workbench::normalize_resource_spec(&value)?.kind;
            match kind.as_str() {
                "dashboard" => dashboards.push(value),
                "datasource" => datasources.push(value),
                "folder" => folders.push(value),
                _ => alerts.push(value),
            }
        }
        super::build_sync_source_bundle_document(
            &dashboards,
            &datasources,
            &folders,
            &alerts,
            None,
            None,
        )?
    };
    let document = super::build_sync_bundle_preflight_document(
        &source,
        &serde_json::json!({}),
        Some(availability),
    )?;
    let mut artifact_inputs = Vec::new();
    for (name, path) in [
        ("sourceBundle", inputs.source_bundle.as_deref()),
        ("desiredFile", inputs.desired_file.as_deref()),
        ("dashboardExportDir", inputs.dashboard_export_dir.as_deref()),
        (
            "dashboardProvisioningDir",
            inputs.dashboard_provisioning_dir.as_deref(),
        ),
        (
            "datasourceExportFile",
            args.inputs.datasource_export_file.as_deref(),
        ),
        (
            "datasourceProvisioningFile",
            inputs.datasource_provisioning_file.as_deref(),
        ),
        ("alertExportDir", inputs.alert_export_dir.as_deref()),
        ("availabilityFile", inputs.availability_file.as_deref()),
    ] {
        if let Some(path) = path {
            artifact_inputs.push(OverviewInputField {
                name: name.to_string(),
                value: path.display().to_string(),
            });
        }
    }
    artifact_inputs.push(OverviewInputField {
        name: "availabilitySource".to_string(),
        value: "live".to_string(),
    });
    inputs.source_bundle = None;
    inputs.availability_file = None;
    Ok(Some(OverviewArtifact {
        kind: OVERVIEW_ARTIFACT_BUNDLE_PREFLIGHT_KIND.to_string(),
        title: "Workspace live availability check".to_string(),
        inputs: artifact_inputs,
        document,
    }))
}

fn ensure_any_discovered(discovered: &DiscoveredChangeInputs) -> Result<()> {
    if discovered == &DiscoveredChangeInputs::default() {
        return Err(message(
            "Workspace input discovery did not find staged inputs in the current directory. Provide explicit flags such as --desired-file, --dashboard-export-dir, --access-user-export-dir, or a staged workspace package file.",
        ));
    }
    Ok(())
}

fn emit_preview_output(mut output: SyncCommandOutput, args: &ChangePreviewArgs) -> Result<()> {
    if args.mark_reviewed {
        if output
            .document
            .get("kind")
            .and_then(serde_json::Value::as_str)
            != Some(super::workbench::SYNC_PLAN_KIND)
        {
            return Err(message("Workspace preview review requires a sync plan; package and promotion checks cannot be marked reviewed."));
        }
        let trace_id = super::require_trace_id(&output.document, "Workspace preview document")?;
        let plan = if args.interactive_review {
            super::review_tui::run_sync_review_tui(&output.document)?
        } else {
            output.document
        };
        output.document = build_sync_review_document(
            &plan,
            &args.review_token,
            &trace_id,
            args.reviewed_by.as_deref(),
            args.reviewed_at.as_deref(),
            args.review_note.as_deref(),
        )?;
        output.text_lines = super::render_sync_plan_text(&output.document)?;
    }
    let output_file = args.output.output_file.as_ref();
    let format = args.output.output_format;
    if let Some(path) = output_file {
        let persisted = match format {
            SyncOutputFormat::Json => serde_json::to_string_pretty(&output.document)?,
            SyncOutputFormat::Text => output.text_lines.join("\n"),
        };
        emit_plain_output(&persisted, Some(path.as_path()), false)?;
    }
    if output_file.is_none() || args.output.also_stdout {
        render_and_emit_sync_command_output(output, format)?;
    }
    Ok(())
}

fn build_preview_project_status_inputs(
    source_bundle: &Path,
    target_inventory: &Path,
    mapping_file: Option<&PathBuf>,
    availability_file: Option<&PathBuf>,
) -> Vec<OverviewInputField> {
    let mut inputs = vec![
        OverviewInputField {
            name: "sourceBundle".to_string(),
            value: source_bundle.display().to_string(),
        },
        OverviewInputField {
            name: "targetInventory".to_string(),
            value: target_inventory.display().to_string(),
        },
    ];
    if let Some(path) = mapping_file {
        inputs.push(OverviewInputField {
            name: "mappingFile".to_string(),
            value: path.display().to_string(),
        });
    }
    if let Some(path) = availability_file {
        inputs.push(OverviewInputField {
            name: "availabilityFile".to_string(),
            value: path.display().to_string(),
        });
    }
    inputs
}

fn attach_preview_project_status(
    mut output: SyncCommandOutput,
    artifact_kind: &str,
    title: &str,
    inputs: Vec<OverviewInputField>,
) -> Result<SyncCommandOutput> {
    let status = build_staged_project_status(&[OverviewArtifact {
        kind: artifact_kind.to_string(),
        title: title.to_string(),
        inputs,
        document: output.document.clone(),
    }]);
    if let Some(object) = output.document.as_object_mut() {
        object.insert("projectStatus".to_string(), serde_json::to_value(&status)?);
    }
    let mut status_lines = render_project_status_text(&status);
    status_lines.push(String::new());
    status_lines.extend(output.text_lines);
    output.text_lines = status_lines;
    Ok(output)
}

fn normalize_change_dashboard_args(
    dashboard_export_dir: &mut Option<PathBuf>,
    dashboard_provisioning_dir: &mut Option<PathBuf>,
) -> Result<()> {
    let normalized = normalize_change_dashboard_inputs(
        dashboard_export_dir.as_ref(),
        dashboard_provisioning_dir.as_ref(),
    )?;
    *dashboard_export_dir = normalized.dashboard_export_dir;
    *dashboard_provisioning_dir = normalized.dashboard_provisioning_dir;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn ensure_change_inputs_present(
    discovered: &DiscoveredChangeInputs,
    dashboard_export_dir: Option<&PathBuf>,
    dashboard_provisioning_dir: Option<&PathBuf>,
    datasource_provisioning_file: Option<&PathBuf>,
    access_user_export_dir: Option<&PathBuf>,
    access_team_export_dir: Option<&PathBuf>,
    access_org_export_dir: Option<&PathBuf>,
    access_service_account_export_dir: Option<&PathBuf>,
    alert_export_dir: Option<&PathBuf>,
    desired_file: Option<&PathBuf>,
    source_bundle: Option<&PathBuf>,
) -> Result<()> {
    if dashboard_export_dir.is_none()
        && dashboard_provisioning_dir.is_none()
        && datasource_provisioning_file.is_none()
        && access_user_export_dir.is_none()
        && access_team_export_dir.is_none()
        && access_org_export_dir.is_none()
        && access_service_account_export_dir.is_none()
        && alert_export_dir.is_none()
        && desired_file.is_none()
        && source_bundle.is_none()
    {
        ensure_any_discovered(discovered)?;
    }
    Ok(())
}

pub(crate) fn run_sync_inspect(args: ChangeInspectArgs) -> Result<()> {
    let discovered = discover_change_staged_inputs(Some(args.inputs.workspace.as_path()))?;
    emit_discovery_provenance(&discovered, args.output.output_format);
    let mut merged = build_overview_args(
        &args,
        discovered.dashboard_export_dir.as_ref(),
        discovered.dashboard_provisioning_dir.as_ref(),
        discovered.datasource_provisioning_file.as_ref(),
        discovered.access_user_export_dir.as_ref(),
        discovered.access_team_export_dir.as_ref(),
        discovered.access_org_export_dir.as_ref(),
        discovered.access_service_account_export_dir.as_ref(),
        discovered.desired_file.as_ref(),
        discovered.source_bundle.as_ref(),
        discovered.target_inventory.as_ref(),
        discovered.alert_export_dir.as_ref(),
        discovered.availability_file.as_ref(),
        discovered.mapping_file.as_ref(),
    );
    normalize_change_dashboard_args(
        &mut merged.dashboard_export_dir,
        &mut merged.dashboard_provisioning_dir,
    )?;
    ensure_change_inputs_present(
        &discovered,
        merged.dashboard_export_dir.as_ref(),
        merged.dashboard_provisioning_dir.as_ref(),
        merged.datasource_provisioning_file.as_ref(),
        merged.access_user_export_dir.as_ref(),
        merged.access_team_export_dir.as_ref(),
        merged.access_org_export_dir.as_ref(),
        merged.access_service_account_export_dir.as_ref(),
        merged.alert_export_dir.as_ref(),
        merged.desired_file.as_ref(),
        merged.source_bundle.as_ref(),
    )?;
    let artifacts = build_overview_artifacts_with_inputs(
        &merged,
        &OverviewInputOverrides {
            datasource_export_file: args.inputs.datasource_export_file.as_deref(),
            ..Default::default()
        },
    )?;
    let document = attach_discovery_to_overview(build_overview_document(artifacts)?, &discovered);
    emit_workspace_output(&document, &render_overview_text(&document)?, &args.output)
}

pub(crate) fn run_sync_check(args: ChangeCheckArgs) -> Result<()> {
    let discovered = discover_change_staged_inputs(Some(args.inputs.workspace.as_path()))?;
    emit_discovery_provenance(&discovered, args.output.output_format);
    let mut merged = build_status_args(
        &args,
        discovered.dashboard_export_dir.as_ref(),
        discovered.dashboard_provisioning_dir.as_ref(),
        discovered.datasource_provisioning_file.as_ref(),
        discovered.access_user_export_dir.as_ref(),
        discovered.access_team_export_dir.as_ref(),
        discovered.access_org_export_dir.as_ref(),
        discovered.access_service_account_export_dir.as_ref(),
        discovered.desired_file.as_ref(),
        discovered.source_bundle.as_ref(),
        discovered.target_inventory.as_ref(),
        discovered.alert_export_dir.as_ref(),
        discovered.availability_file.as_ref(),
        discovered.mapping_file.as_ref(),
    );
    normalize_change_dashboard_args(
        &mut merged.dashboard_export_dir,
        &mut merged.dashboard_provisioning_dir,
    )?;
    ensure_change_inputs_present(
        &discovered,
        merged.dashboard_export_dir.as_ref(),
        merged.dashboard_provisioning_dir.as_ref(),
        merged.datasource_provisioning_file.as_ref(),
        merged.access_user_export_dir.as_ref(),
        merged.access_team_export_dir.as_ref(),
        merged.access_org_export_dir.as_ref(),
        merged.access_service_account_export_dir.as_ref(),
        merged.alert_export_dir.as_ref(),
        merged.desired_file.as_ref(),
        merged.source_bundle.as_ref(),
    )?;
    let availability = if args.fetch_live {
        load_sync_merged_availability(
            true,
            &args.common,
            args.org_id,
            merged.availability_file.as_ref(),
        )?
    } else {
        None
    };
    let mut overview_args = staged_args_to_overview_args(&merged);
    let extra_artifacts = match availability.as_ref() {
        Some(availability) => build_live_check_artifact(&args, &mut overview_args, availability)?
            .into_iter()
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };
    let artifacts = build_overview_artifacts_with_inputs(
        &overview_args,
        &OverviewInputOverrides {
            datasource_export_file: args.inputs.datasource_export_file.as_deref(),
            availability: availability.as_ref(),
            extra_artifacts: &extra_artifacts,
        },
    )?;
    let status = attach_discovery_to_status(build_staged_project_status(&artifacts), &discovered);
    emit_workspace_output(&status, &render_project_status_text(&status), &args.output)
}

pub(crate) fn run_sync_preview(args: ChangePreviewArgs) -> Result<()> {
    let discovered = discover_change_staged_inputs(Some(args.inputs.workspace.as_path()))?;
    emit_discovery_provenance(&discovered, args.output.output_format);
    let source_bundle = args
        .inputs
        .source_bundle
        .clone()
        .or(discovered.source_bundle.clone());
    let target_inventory = args
        .target_inventory
        .clone()
        .or(discovered.target_inventory.clone());
    let mapping_file = args
        .mapping_file
        .clone()
        .or(discovered.mapping_file.clone());
    let availability_file = args
        .availability_file
        .clone()
        .or(discovered.availability_file.clone());
    if let (Some(source_bundle), Some(target_inventory), Some(mapping_file)) = (
        source_bundle.clone(),
        target_inventory.clone(),
        mapping_file.clone(),
    ) {
        let output = execute_sync_promotion_preflight(&SyncPromotionPreflightArgs {
            source_bundle,
            target_inventory,
            mapping_file: Some(mapping_file),
            availability_file,
            fetch_live: args.fetch_live,
            common: args.common.clone(),
            org_id: args.org_id,
            output_format: args.output.output_format,
        })?;
        let output = attach_preview_project_status(
            output,
            OVERVIEW_ARTIFACT_PROMOTION_PREFLIGHT_KIND,
            "Workspace promote test",
            build_preview_project_status_inputs(
                &args
                    .inputs
                    .source_bundle
                    .clone()
                    .or(discovered.source_bundle.clone())
                    .expect("promotion preview requires workspace package"),
                &args
                    .target_inventory
                    .clone()
                    .or(discovered.target_inventory.clone())
                    .expect("promotion preview requires target inventory"),
                args.mapping_file
                    .as_ref()
                    .or(discovered.mapping_file.as_ref()),
                args.availability_file
                    .as_ref()
                    .or(discovered.availability_file.as_ref()),
            ),
        )?;
        return emit_preview_output(attach_discovery_to_sync_output(output, &discovered), &args);
    }
    if let (Some(source_bundle), Some(target_inventory)) = (source_bundle, target_inventory) {
        let output = execute_sync_bundle_preflight(&SyncBundlePreflightArgs {
            source_bundle,
            target_inventory,
            availability_file,
            fetch_live: args.fetch_live,
            common: args.common.clone(),
            org_id: args.org_id,
            output_format: args.output.output_format,
        })?;
        let output = attach_preview_project_status(
            output,
            OVERVIEW_ARTIFACT_BUNDLE_PREFLIGHT_KIND,
            "Workspace package test",
            build_preview_project_status_inputs(
                &args
                    .inputs
                    .source_bundle
                    .clone()
                    .or(discovered.source_bundle.clone())
                    .expect("bundle preview requires workspace package"),
                &args
                    .target_inventory
                    .clone()
                    .or(discovered.target_inventory.clone())
                    .expect("bundle preview requires target inventory"),
                None,
                args.availability_file
                    .as_ref()
                    .or(discovered.availability_file.as_ref()),
            ),
        )?;
        return emit_preview_output(attach_discovery_to_sync_output(output, &discovered), &args);
    }
    let output = if let Some(desired_file) = args.inputs.desired_file.clone().or_else(|| {
        args.inputs
            .source_bundle
            .is_none()
            .then(|| discovered.desired_file.clone())
            .flatten()
    }) {
        execute_sync_plan(&SyncPlanArgs {
            desired_file,
            live_file: args.live_file.clone(),
            fetch_live: args.fetch_live,
            common: args.common.clone(),
            org_id: args.org_id,
            page_size: args.page_size,
            allow_prune: args.allow_prune,
            output_format: args.output.output_format,
            trace_id: args.trace_id.clone(),
        })?
    } else {
        let normalized_dashboard_inputs =
            if args.inputs.source_bundle.is_some() || discovered.source_bundle.is_some() {
                Default::default()
            } else {
                let (selected_export, selected_provisioning) = select_preview_dashboard_sources(
                    &args.inputs,
                    discovered.dashboard_export_dir.as_ref(),
                    discovered.dashboard_provisioning_dir.as_ref(),
                )?;
                normalize_change_dashboard_inputs(selected_export, selected_provisioning)?
            };
        let preview_discovered = DiscoveredChangeInputs {
            workspace_root: discovered.workspace_root.clone(),
            dashboard_export_dir: normalized_dashboard_inputs.dashboard_export_dir.clone(),
            dashboard_provisioning_dir: normalized_dashboard_inputs
                .dashboard_provisioning_dir
                .clone(),
            ..discovered.clone()
        };
        let desired = load_preview_desired_specs(
            &args.inputs,
            preview_discovered.desired_file.as_ref(),
            preview_discovered.source_bundle.as_ref(),
            preview_discovered.dashboard_export_dir.as_ref(),
            preview_discovered.dashboard_provisioning_dir.as_ref(),
            preview_discovered.alert_export_dir.as_ref(),
            preview_discovered.datasource_provisioning_file.as_ref(),
        )?;
        let live = load_sync_live_array(
            args.fetch_live,
            &args.common,
            args.org_id,
            args.page_size,
            args.live_file.as_ref(),
            "Workspace preview requires --live-file unless --fetch-live is used.",
        )?;
        let document = attach_lineage(
            &attach_trace_id(
                &build_sync_plan_document(&desired, &live, args.allow_prune)?,
                args.trace_id.as_deref(),
            )?,
            "plan",
            1,
            None,
        )?;
        let document =
            super::workspace_preview_contract::enrich_workspace_preview_document(&document)?;
        SyncCommandOutput {
            text_lines: super::render_sync_plan_text(&document)?,
            document,
        }
    };
    emit_preview_output(attach_discovery_to_sync_output(output, &discovered), &args)
}

#[cfg(test)]
mod task_first_rust_tests {
    use super::*;
    use crate::sync::workspace_discovery::render_discovery_provenance;
    use crate::sync::ChangeStagedInputsArgs;
    use tempfile::tempdir;

    fn staged_inputs(
        dashboard_export_dir: Option<&str>,
        dashboard_provisioning_dir: Option<&str>,
    ) -> ChangeStagedInputsArgs {
        ChangeStagedInputsArgs {
            workspace: PathBuf::from("."),
            desired_file: None,
            source_bundle: None,
            dashboard_export_dir: dashboard_export_dir.map(PathBuf::from),
            dashboard_provisioning_dir: dashboard_provisioning_dir.map(PathBuf::from),
            alert_export_dir: None,
            datasource_export_file: None,
            datasource_provisioning_file: None,
            access_user_export_dir: None,
            access_team_export_dir: None,
            access_org_export_dir: None,
            access_service_account_export_dir: None,
        }
    }

    #[test]
    fn ensure_change_inputs_present_accepts_access_only_inputs() {
        let discovered = DiscoveredChangeInputs {
            workspace_root: Some(PathBuf::from(".")),
            access_user_export_dir: Some(PathBuf::from("./access-users")),
            ..DiscoveredChangeInputs::default()
        };

        assert!(ensure_change_inputs_present(
            &discovered,
            None,
            None,
            None,
            Some(&PathBuf::from("./access-users")),
            None,
            None,
            None,
            None,
            None,
            None
        )
        .is_ok());
    }

    #[test]
    fn select_preview_dashboard_sources_prefers_discovered_export_over_provisioning() {
        let inputs = staged_inputs(None, None);
        let discovered_export = PathBuf::from("./dashboards/raw");
        let discovered_provisioning = PathBuf::from("./dashboards/provisioning");

        let (output_dir, provisioning_dir) = select_preview_dashboard_sources(
            &inputs,
            Some(&discovered_export),
            Some(&discovered_provisioning),
        )
        .unwrap();

        assert_eq!(output_dir, Some(&discovered_export));
        assert!(provisioning_dir.is_none());
    }

    #[test]
    fn discover_change_staged_inputs_accepts_dashboards_tree_as_workspace() {
        let temp = tempdir().unwrap();
        let workspace = temp.path();
        std::fs::create_dir_all(workspace.join("dashboards/raw")).unwrap();
        std::fs::create_dir_all(workspace.join("alerts/raw")).unwrap();
        std::fs::create_dir_all(workspace.join("datasources/provisioning")).unwrap();
        std::fs::write(
            workspace.join("datasources/provisioning/datasources.yaml"),
            "apiVersion: 1\n",
        )
        .unwrap();

        let discovered =
            discover_change_staged_inputs(Some(&workspace.join("dashboards"))).unwrap();

        assert_eq!(
            discovered.dashboard_export_dir,
            Some(workspace.join("dashboards/raw"))
        );
        assert_eq!(
            discovered.alert_export_dir,
            Some(workspace.join("alerts/raw"))
        );
        assert_eq!(
            discovered.datasource_provisioning_file,
            Some(workspace.join("datasources/provisioning/datasources.yaml"))
        );
    }

    #[test]
    fn discover_change_staged_inputs_accepts_dashboard_raw_tree_as_workspace() {
        let temp = tempdir().unwrap();
        let workspace = temp.path();
        std::fs::create_dir_all(workspace.join("dashboards/raw")).unwrap();
        std::fs::create_dir_all(workspace.join("dashboards/provisioning")).unwrap();
        std::fs::create_dir_all(workspace.join("alerts/raw")).unwrap();

        let discovered =
            discover_change_staged_inputs(Some(&workspace.join("dashboards/raw"))).unwrap();

        assert_eq!(
            discovered.dashboard_export_dir,
            Some(workspace.join("dashboards/raw"))
        );
        assert_eq!(
            discovered.dashboard_provisioning_dir,
            Some(workspace.join("dashboards/provisioning"))
        );
        assert_eq!(
            discovered.alert_export_dir,
            Some(workspace.join("alerts/raw"))
        );
    }

    #[test]
    fn discover_change_staged_inputs_recognizes_git_sync_wrapped_dashboard_tree() {
        let temp = tempdir().unwrap();
        let workspace = temp.path();
        std::fs::create_dir_all(workspace.join("dashboards/git-sync/raw")).unwrap();
        std::fs::create_dir_all(workspace.join("dashboards/git-sync/provisioning")).unwrap();

        let discovered = discover_change_staged_inputs(Some(workspace)).unwrap();

        assert_eq!(
            discovered.dashboard_export_dir,
            Some(workspace.join("dashboards/git-sync/raw"))
        );
        assert_eq!(
            discovered.dashboard_provisioning_dir,
            Some(workspace.join("dashboards/git-sync/provisioning"))
        );
    }

    #[test]
    fn discover_change_staged_inputs_accepts_dashboards_git_sync_dir_as_workspace() {
        let temp = tempdir().unwrap();
        let workspace = temp.path();
        std::fs::create_dir_all(workspace.join("dashboards/git-sync/raw")).unwrap();
        std::fs::create_dir_all(workspace.join("dashboards/git-sync/provisioning")).unwrap();

        let discovered =
            discover_change_staged_inputs(Some(&workspace.join("dashboards/git-sync"))).unwrap();

        assert_eq!(
            discovered.dashboard_export_dir,
            Some(workspace.join("dashboards/git-sync/raw"))
        );
        assert_eq!(
            discovered.dashboard_provisioning_dir,
            Some(workspace.join("dashboards/git-sync/provisioning"))
        );
    }

    #[test]
    fn discover_change_staged_inputs_accepts_datasource_provisioning_tree_as_workspace() {
        let temp = tempdir().unwrap();
        let workspace = temp.path();
        std::fs::create_dir_all(workspace.join("dashboards/raw")).unwrap();
        std::fs::create_dir_all(workspace.join("datasources/provisioning")).unwrap();
        std::fs::write(
            workspace.join("datasources/provisioning/datasources.yaml"),
            "apiVersion: 1\n",
        )
        .unwrap();

        let discovered =
            discover_change_staged_inputs(Some(&workspace.join("datasources/provisioning")))
                .unwrap();

        assert_eq!(
            discovered.datasource_provisioning_file,
            Some(workspace.join("datasources/provisioning/datasources.yaml"))
        );
        assert_eq!(
            discovered.dashboard_export_dir,
            Some(workspace.join("dashboards/raw"))
        );
        assert_eq!(discovered.workspace_root, Some(workspace.to_path_buf()));
    }

    #[test]
    fn discovered_mixed_workspace_provenance_still_reports_all_sources() {
        let temp = tempdir().unwrap();
        let workspace = temp.path();
        std::fs::create_dir_all(workspace.join("dashboards/git-sync/raw")).unwrap();
        std::fs::create_dir_all(workspace.join("dashboards/git-sync/provisioning")).unwrap();
        std::fs::create_dir_all(workspace.join("alerts/raw")).unwrap();
        std::fs::create_dir_all(workspace.join("datasources/provisioning")).unwrap();
        std::fs::write(
            workspace.join("datasources/provisioning/datasources.yaml"),
            "apiVersion: 1\n",
        )
        .unwrap();

        let discovered = discover_change_staged_inputs(Some(workspace)).unwrap();
        let provenance = render_discovery_provenance(&discovered).unwrap();

        assert!(provenance.contains("dashboard-export(git-sync)="));
        assert!(provenance.contains("dashboard-provisioning(git-sync)="));
        assert!(provenance.contains("alert-export="));
        assert!(provenance.contains("datasource-provisioning="));
    }

    #[test]
    fn attach_preview_project_status_adds_project_status_document_and_text_block() {
        let output = SyncCommandOutput {
            document: serde_json::json!({
                "kind": "grafana-utils-sync-bundle-preflight",
                "summary": {
                    "resourceCount": 2,
                    "syncBlockingCount": 1,
                    "providerBlockingCount": 0,
                    "secretPlaceholderBlockingCount": 0,
                    "alertArtifactCount": 0,
                    "alertArtifactPlanOnlyCount": 0,
                    "alertArtifactBlockingCount": 0
                }
            }),
            text_lines: vec!["Workspace package test".to_string()],
        };

        let attached = attach_preview_project_status(
            output,
            OVERVIEW_ARTIFACT_BUNDLE_PREFLIGHT_KIND,
            "Workspace package test",
            build_preview_project_status_inputs(
                &PathBuf::from("./bundle.json"),
                &PathBuf::from("./target.json"),
                None,
                None,
            ),
        )
        .unwrap();

        assert_eq!(
            attached.document["projectStatus"]["domains"][0]["id"],
            serde_json::json!("sync")
        );
        assert_eq!(
            attached.document["projectStatus"]["domains"][0]["sourceKinds"],
            serde_json::json!(["package-test"])
        );
        assert_eq!(attached.text_lines[0], "Project status");
        assert!(attached
            .text_lines
            .iter()
            .any(|line| line == "Workspace package test"));
    }
}
