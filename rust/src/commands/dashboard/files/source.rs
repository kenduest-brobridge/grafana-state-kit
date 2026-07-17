use std::path::{Path, PathBuf};

use crate::common::{message, Result};

use super::super::{DashboardImportInputFormat, PROVISIONING_EXPORT_SUBDIR, RAW_EXPORT_SUBDIR};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DashboardSourceKind {
    LiveGrafana,
    RawExport,
    ProvisioningExport,
    ResourceV2Import,
    HistoryArtifact,
}

impl DashboardSourceKind {
    #[allow(dead_code)]
    pub(crate) fn from_import_input_format(input_format: DashboardImportInputFormat) -> Self {
        match input_format {
            DashboardImportInputFormat::Raw => Self::RawExport,
            DashboardImportInputFormat::Provisioning => Self::ProvisioningExport,
            DashboardImportInputFormat::ResourceV2 => Self::ResourceV2Import,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_workspace_dir(path: &Path) -> Option<Self> {
        let name = path.file_name().and_then(|name| name.to_str())?;
        let parent_name = path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str());
        match (parent_name, name) {
            (Some("dashboards"), "raw") => Some(Self::RawExport),
            (Some("dashboards"), "provisioning") => Some(Self::ProvisioningExport),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn expected_variant(self) -> Option<&'static str> {
        match self {
            Self::RawExport => Some(RAW_EXPORT_SUBDIR),
            Self::ProvisioningExport => Some(PROVISIONING_EXPORT_SUBDIR),
            Self::ResourceV2Import => Some(crate::dashboard::RESOURCE_V2_EXPORT_SUBDIR),
            Self::LiveGrafana | Self::HistoryArtifact => None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_expected_variant(expected_variant: &str) -> Option<Self> {
        match expected_variant {
            RAW_EXPORT_SUBDIR => Some(Self::RawExport),
            PROVISIONING_EXPORT_SUBDIR => Some(Self::ProvisioningExport),
            crate::dashboard::RESOURCE_V2_EXPORT_SUBDIR => Some(Self::ResourceV2Import),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn is_file_backed(self) -> bool {
        matches!(
            self,
            Self::RawExport | Self::ProvisioningExport | Self::ResourceV2Import
        )
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DashboardRepoLayoutKind {
    GitSyncRepo,
}

impl DashboardRepoLayoutKind {
    #[allow(dead_code)]
    pub(crate) fn from_root_dir(path: &Path) -> Option<Self> {
        if path.join(".git").exists()
            && (path.join("dashboards").is_dir() || contains_dashboard_resource(path))
        {
            Some(Self::GitSyncRepo)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub(crate) fn is_git_sync_repo(self) -> bool {
        matches!(self, Self::GitSyncRepo)
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_dashboard_variant_root(
        self,
        input_dir: &Path,
        variant_dir_name: &str,
    ) -> Option<PathBuf> {
        if !self.is_git_sync_repo() {
            return None;
        }
        if variant_dir_name == RAW_EXPORT_SUBDIR && contains_dashboard_resource(input_dir) {
            return Some(input_dir.to_path_buf());
        }
        let dashboards_dir =
            if input_dir.file_name().and_then(|name| name.to_str()) == Some("dashboards") {
                input_dir.to_path_buf()
            } else {
                input_dir.join("dashboards")
            };
        let direct_candidate = dashboards_dir.join(variant_dir_name);
        if direct_candidate.is_dir() {
            return Some(direct_candidate);
        }
        let wrapped_candidate = dashboards_dir.join("git-sync").join(variant_dir_name);
        wrapped_candidate.is_dir().then_some(wrapped_candidate)
    }
}

fn contains_dashboard_resource(path: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(path) else {
        return false;
    };
    for entry in entries.flatten() {
        let entry_path = entry.path();
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if contains_dashboard_resource(&entry_path) {
                return true;
            }
            continue;
        }
        if entry_path.extension().and_then(|value| value.to_str()) != Some("json")
            || entry_path.file_name().and_then(|value| value.to_str()) == Some("_folder.json")
        {
            continue;
        }
        let Ok(bytes) = std::fs::read(&entry_path) else {
            continue;
        };
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            continue;
        };
        if value.get("kind").and_then(serde_json::Value::as_str) == Some("Dashboard")
            && value
                .get("apiVersion")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|version| version.starts_with("dashboard.grafana.app/"))
        {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedDashboardImportSource {
    pub source_kind: DashboardSourceKind,
    pub dashboard_dir: PathBuf,
    pub metadata_dir: PathBuf,
}

pub(crate) fn resolve_dashboard_import_source(
    input_dir: &Path,
    input_format: DashboardImportInputFormat,
) -> Result<ResolvedDashboardImportSource> {
    match input_format {
        DashboardImportInputFormat::Raw => Ok(ResolvedDashboardImportSource {
            source_kind: DashboardSourceKind::RawExport,
            dashboard_dir: input_dir.to_path_buf(),
            metadata_dir: input_dir.to_path_buf(),
        }),
        DashboardImportInputFormat::Provisioning => {
            if !input_dir.exists() {
                return Err(message(format!(
                    "Import directory does not exist: {}",
                    input_dir.display()
                )));
            }
            if !input_dir.is_dir() {
                return Err(message(format!(
                    "Import path is not a directory: {}",
                    input_dir.display()
                )));
            }
            let nested_dashboards_dir = input_dir.join("dashboards");
            if nested_dashboards_dir.is_dir() {
                return Ok(ResolvedDashboardImportSource {
                    source_kind: DashboardSourceKind::ProvisioningExport,
                    dashboard_dir: nested_dashboards_dir,
                    metadata_dir: input_dir.to_path_buf(),
                });
            }
            if input_dir.file_name().and_then(|name| name.to_str()) == Some("dashboards") {
                let metadata_dir = input_dir.parent().ok_or_else(|| {
                    message(format!(
                        "Dashboard provisioning import expects a parent provisioning directory for {}.",
                        input_dir.display()
                    ))
                })?;
                return Ok(ResolvedDashboardImportSource {
                    source_kind: DashboardSourceKind::ProvisioningExport,
                    dashboard_dir: input_dir.to_path_buf(),
                    metadata_dir: metadata_dir.to_path_buf(),
                });
            }
            Err(message(format!(
                "Dashboard provisioning import expects --input-dir to point at the {}/ root or its dashboards/ directory: {}",
                PROVISIONING_EXPORT_SUBDIR,
                input_dir.display()
            )))
        }
        DashboardImportInputFormat::ResourceV2 => Ok(ResolvedDashboardImportSource {
            source_kind: DashboardSourceKind::ResourceV2Import,
            dashboard_dir: input_dir.to_path_buf(),
            metadata_dir: input_dir.to_path_buf(),
        }),
    }
}
