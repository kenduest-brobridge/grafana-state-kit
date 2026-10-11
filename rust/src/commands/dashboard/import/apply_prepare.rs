use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::common::{message, Result};
use crate::dashboard::{
    load_export_metadata, load_folder_inventory, ExportMetadata, FolderInventoryItem, ImportArgs,
    FOLDER_INVENTORY_FILENAME,
};

pub(super) struct PreparedImportRun {
    pub(super) resolved_import: super::super::LoadedImportSource,
    pub(super) metadata: Option<ExportMetadata>,
    pub(super) folders_by_uid: BTreeMap<String, FolderInventoryItem>,
    pub(super) dashboard_files: Vec<PathBuf>,
}

pub(super) fn validate_import_args(args: &ImportArgs) -> Result<()> {
    validate_import_common_args(args)?;
    validate_import_output_columns(args)?;
    validate_resource_v2_import_args(args)?;
    Ok(())
}

pub(super) fn validate_import_common_args(args: &ImportArgs) -> Result<()> {
    if args.table && !args.dry_run {
        return Err(message(
            "--table is only supported with --dry-run for import-dashboard.",
        ));
    }
    if args.json && !args.dry_run {
        return Err(message(
            "--json is only supported with --dry-run for import-dashboard.",
        ));
    }
    if args.output_format.is_some() && !args.dry_run {
        return Err(message(
            "--output-format is only supported with --dry-run for import-dashboard.",
        ));
    }
    if args.table && args.json {
        return Err(message(
            "--table and --json are mutually exclusive for import-dashboard.",
        ));
    }
    if args.no_header && !args.table {
        return Err(message(
            "--no-header is only supported with --dry-run --table for import-dashboard.",
        ));
    }
    if !args.output_columns.is_empty() && !args.table {
        return Err(message(
            "--output-columns is only supported with --dry-run --table or table-like --output-format for import-dashboard.",
        ));
    }
    if args.require_matching_folder_path && args.import_folder_uid.is_some() {
        return Err(message(
            "--require-matching-folder-path cannot be combined with --import-folder-uid.",
        ));
    }
    if args.ensure_folders && args.import_folder_uid.is_some() {
        return Err(message(
            "--ensure-folders cannot be combined with --import-folder-uid.",
        ));
    }
    Ok(())
}

pub(crate) fn validate_import_output_columns(args: &ImportArgs) -> Result<()> {
    if args.output_columns.is_empty() {
        return Ok(());
    }
    let (lane, supported): (&str, &[&str]) =
        if args.input_format == crate::dashboard::DashboardImportInputFormat::ResourceV2 {
            (
                "resource-v2",
                &[
                    "all",
                    "api_version",
                    "namespace",
                    "name",
                    "source_path",
                    "action",
                    "folder",
                    "ownership",
                    "blocker",
                    "changed_fields",
                    "file",
                    "reason",
                ],
            )
        } else {
            (
                "classic",
                &[
                    "all",
                    "uid",
                    "destination",
                    "action",
                    "folder_path",
                    "source_folder_path",
                    "destination_folder_path",
                    "reason",
                    "file",
                ],
            )
        };
    if let Some(column) = args
        .output_columns
        .iter()
        .find(|column| !supported.contains(&column.as_str()))
    {
        return Err(message(format!(
            "--output-columns value {column:?} is not supported for {lane} dashboard import."
        )));
    }
    Ok(())
}

pub(crate) fn validate_resource_v2_import_args(args: &ImportArgs) -> Result<()> {
    if args.input_format != crate::dashboard::DashboardImportInputFormat::ResourceV2 {
        return Ok(());
    }
    let incompatible = [
        (args.org_id.is_some(), "--org-id"),
        (!args.only_org_id.is_empty(), "--only-org-id"),
        (args.create_missing_orgs, "--create-missing-orgs"),
        (args.use_export_org, "--use-export-org"),
        (
            args.require_matching_export_org,
            "--require-matching-export-org",
        ),
        (
            args.require_matching_folder_path,
            "--require-matching-folder-path",
        ),
        (args.ensure_folders, "--ensure-folders"),
        (
            args.target_schema_version.is_some(),
            "--target-schema-version",
        ),
        (args.strict_schema, "--strict-schema"),
    ];
    if let Some((_, flag)) = incompatible.into_iter().find(|(enabled, _)| *enabled) {
        return Err(message(format!(
            "{flag} is not supported with --input-format resource-v2; resource-v2 import is single-namespace and uses native schema validation."
        )));
    }
    Ok(())
}

pub(super) fn prepare_import_run<F>(
    select_dashboard_files: &mut F,
    args: &ImportArgs,
) -> Result<PreparedImportRun>
where
    F: FnMut(&super::super::LoadedImportSource, Vec<PathBuf>) -> Result<Option<Vec<PathBuf>>>,
{
    let resolved_import = super::super::resolve_import_source(args)?;
    let metadata = load_export_metadata(
        resolved_import.metadata_dir(),
        Some(super::super::import_metadata_variant(args)),
    )?;
    let folder_inventory =
        if args.ensure_folders || args.require_matching_folder_path || args.dry_run {
            load_folder_inventory(resolved_import.metadata_dir(), metadata.as_ref())?
        } else {
            Vec::new()
        };
    if args.ensure_folders && folder_inventory.is_empty() {
        let folders_file = metadata
            .as_ref()
            .and_then(|item| item.folders_file.as_deref())
            .unwrap_or(FOLDER_INVENTORY_FILENAME);
        return Err(message(format!(
            "Folder inventory file not found for --ensure-folders: {}. Re-export dashboards with raw folder inventory or omit --ensure-folders.",
            resolved_import.metadata_dir().join(folders_file).display()
        )));
    }
    let folders_by_uid = folder_inventory
        .into_iter()
        .map(|item| (item.uid.clone(), item))
        .collect::<BTreeMap<String, FolderInventoryItem>>();
    let discovered_dashboard_files =
        super::super::dashboard_files_for_import(resolved_import.dashboard_dir())?;
    let dashboard_files =
        match select_dashboard_files(&resolved_import, discovered_dashboard_files.clone())? {
            Some(selected) => selected,
            None if args.interactive => {
                println!(
                    "{} cancelled.",
                    if args.dry_run {
                        "Interactive dry-run"
                    } else {
                        "Import"
                    }
                );
                return Ok(PreparedImportRun {
                    resolved_import,
                    metadata,
                    folders_by_uid,
                    dashboard_files: Vec::new(),
                });
            }
            None => discovered_dashboard_files,
        };
    Ok(PreparedImportRun {
        resolved_import,
        metadata,
        folders_by_uid,
        dashboard_files,
    })
}
