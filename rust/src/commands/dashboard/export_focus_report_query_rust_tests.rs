//! Dashboard domain test suite.
//! Covers parser surfaces, formatter/output contracts, and export/import/inspect/list/diff
//! behavior with in-memory/mocked request fixtures.
#![allow(unused_imports)]

use super::test_support;
use super::test_support::{
    attach_dashboard_folder_paths_with_request, build_export_metadata, build_export_variant_dirs,
    build_external_export_document, build_folder_inventory_status, build_folder_path,
    build_governance_gate_tui_groups, build_governance_gate_tui_items, build_impact_browser_items,
    build_impact_document, build_impact_tui_groups, build_import_auth_context,
    build_import_payload, build_output_path, build_preserved_web_import_document,
    build_root_export_index, build_topology_document, build_topology_tui_groups,
    diff_dashboards_with_request, discover_dashboard_files, export_dashboards_with_request,
    extract_dashboard_variables, filter_impact_tui_items, filter_topology_tui_items,
    format_dashboard_summary_line, format_export_progress_line, format_export_verbose_line,
    format_folder_inventory_status_line, format_import_progress_line, format_import_verbose_line,
    import_dashboards_with_org_clients, import_dashboards_with_request,
    list_dashboards_with_request, parse_cli_from, render_dashboard_governance_gate_result,
    render_dashboard_summary_csv, render_dashboard_summary_json, render_dashboard_summary_table,
    render_impact_text, render_import_dry_run_json, render_import_dry_run_table,
    render_topology_dot, render_topology_mermaid, CommonCliArgs, DashboardCliArgs,
    DashboardCommand, DashboardGovernanceGateFinding, DashboardGovernanceGateResult,
    DashboardGovernanceGateSummary, DiffArgs, ExportArgs, FolderInventoryStatusKind,
    GovernanceGateArgs, GovernanceGateOutputFormat, ImpactAlertResource, ImpactDashboard,
    ImpactDocument, ImpactOutputFormat, ImpactSummary, ImportArgs, InspectExportArgs,
    InspectExportReportFormat, InspectLiveArgs, InspectOutputFormat, ListArgs, SimpleOutputFormat,
    TopologyDocument, TopologyOutputFormat, ValidationOutputFormat,
    DASHBOARD_PERMISSION_BUNDLE_FILENAME, DATASOURCE_INVENTORY_FILENAME, EXPORT_METADATA_FILENAME,
    FOLDER_INVENTORY_FILENAME, TOOL_SCHEMA_VERSION,
};
use super::{
    assert_all_orgs_export_live_documents_match, assert_governance_documents_match,
    export_query_row, load_inspection_analyzer_cases, load_prompt_export_cases,
    make_basic_common_args, make_common_args, make_import_args, sample_topology_tui_document,
    with_dashboard_import_live_preflight, write_basic_raw_export,
    write_combined_export_root_metadata,
};
use crate::common::api_response;
use crate::dashboard::inspect::{
    dispatch_query_analysis, extract_query_field_and_text, resolve_query_analyzer_family,
    QueryAnalysis, QueryExtractionContext,
};
use crate::dashboard::inspect_governance::governance_risk_spec;
use clap::{CommandFactory, Parser};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[cfg(test)]
#[path = "export_focus_report_query_family_rust_tests.rs"]
mod export_focus_report_query_family_rust_tests;

#[cfg(test)]
#[path = "export_focus_report_query_presentation_rust_tests.rs"]
mod export_focus_report_query_presentation_rust_tests;

#[allow(clippy::too_many_arguments)]
pub(super) fn make_core_family_report_row(
    dashboard_uid: &str,
    panel_id: &str,
    ref_id: &str,
    datasource_uid: &str,
    datasource_name: &str,
    datasource_type: &str,
    datasource_family: &str,
    query_text: &str,
    measurements: &[&str],
) -> test_support::ExportInspectionQueryRow {
    test_support::ExportInspectionQueryRow {
        org: "Main Org.".to_string(),
        org_id: "1".to_string(),
        dashboard_uid: dashboard_uid.to_string(),
        dashboard_title: format!("{dashboard_uid} Dashboard"),
        dashboard_tags: Vec::new(),
        folder_path: "General".to_string(),
        folder_full_path: "/".to_string(),
        folder_level: "1".to_string(),
        folder_uid: "general".to_string(),
        parent_folder_uid: String::new(),
        panel_id: panel_id.to_string(),
        panel_title: "Query".to_string(),
        panel_type: "table".to_string(),
        panel_target_count: 1,
        panel_query_count: 1,
        panel_datasource_count: 0,
        panel_variables: Vec::new(),
        ref_id: ref_id.to_string(),
        datasource: datasource_name.to_string(),
        datasource_name: datasource_name.to_string(),
        datasource_uid: datasource_uid.to_string(),
        datasource_org: String::new(),
        datasource_org_id: String::new(),
        datasource_database: String::new(),
        datasource_bucket: String::new(),
        datasource_organization: String::new(),
        datasource_index_pattern: String::new(),
        datasource_type: datasource_type.to_string(),
        datasource_family: datasource_family.to_string(),
        query_field: "query".to_string(),
        target_hidden: "false".to_string(),
        target_disabled: "false".to_string(),
        query_text: query_text.to_string(),
        query_variables: Vec::new(),
        metrics: Vec::new(),
        functions: Vec::new(),
        measurements: measurements.iter().map(|value| value.to_string()).collect(),
        buckets: Vec::new(),
        file_path: format!("/tmp/raw/{dashboard_uid}.json"),
    }
}

#[test]
fn resolve_report_column_ids_for_format_defaults_csv_to_supported_columns() {
    let csv_columns = test_support::resolve_report_column_ids_for_format(
        Some(InspectExportReportFormat::Csv),
        &[],
    )
    .unwrap();
    assert!(csv_columns.iter().any(|value| value == "datasource_uid"));
    assert!(csv_columns
        .iter()
        .any(|value| value == "panel_target_count"));
    assert!(csv_columns.iter().any(|value| value == "target_hidden"));
    assert!(csv_columns.iter().any(|value| value == "target_disabled"));
    assert_eq!(
        csv_columns.len(),
        test_support::SUPPORTED_REPORT_COLUMN_IDS.len()
    );

    let table_columns = test_support::resolve_report_column_ids_for_format(
        Some(InspectExportReportFormat::Table),
        &[],
    )
    .unwrap();
    assert_eq!(
        table_columns,
        test_support::DEFAULT_REPORT_COLUMN_IDS
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
}

#[test]
fn report_format_supports_columns_matches_inspection_contract() {
    assert!(test_support::report_format_supports_columns(
        InspectExportReportFormat::Table
    ));
    assert!(test_support::report_format_supports_columns(
        InspectExportReportFormat::Csv
    ));
    assert!(test_support::report_format_supports_columns(
        InspectExportReportFormat::TreeTable
    ));
    assert!(!test_support::report_format_supports_columns(
        InspectExportReportFormat::QueriesJson
    ));
    assert!(!test_support::report_format_supports_columns(
        InspectExportReportFormat::Tree
    ));
}
