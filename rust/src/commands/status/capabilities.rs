use clap::{Args, ValueEnum};
use reqwest::Method;
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

use crate::common::{message, render_json_value, set_json_color_choice, GrafanaCliError, Result};
use crate::dashboard::{CommonCliArgs, DEFAULT_TIMEOUT};
use crate::grafana_api::{
    dashboard_resource_path, validate_dashboard_resource_v2_list_response, AuthInputs,
    DashboardResourceApiVersion, GrafanaConnection,
};
use crate::profile_config::ConnectionMergeInput;
use crate::tabular_output::render_yaml;

pub(crate) const STATUS_CAPABILITIES_HELP_TEXT: &str = "Examples:\n\n  Check readable dashboard APIs in one namespace:\n    grafana-util status capabilities --profile prod --namespace default --output-format json";

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CapabilityOutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(Debug, Clone, Args)]
pub struct StatusCapabilitiesArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long,
        help = "Explicit Grafana namespace for dashboard resource v1/v2 reads."
    )]
    pub namespace: String,
    #[arg(
        long,
        help = "Grafana organization id for these reads; does not select the namespace."
    )]
    pub org_id: Option<i64>,
    #[arg(long, conflicts_with_all = ["verify_ssl", "ca_cert"], help = "Disable TLS certificate verification explicitly.")]
    pub insecure: bool,
    #[arg(
        long,
        value_name = "PATH",
        help = "PEM bundle to trust for Grafana TLS verification."
    )]
    pub ca_cert: Option<PathBuf>,
    #[arg(
        long,
        value_enum,
        default_value = "text",
        help = "Render dashboard API read capability observations as text, json, or yaml."
    )]
    pub output_format: CapabilityOutputFormat,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardCapabilities {
    kind: &'static str,
    schema_version: u8,
    namespace: String,
    probes: Vec<CapabilityProbe>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CapabilityProbe {
    api: &'static str,
    path: String,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    http_status: Option<u16>,
    detail: &'static str,
}

fn classify_error(error: &GrafanaCliError) -> (&'static str, &'static str) {
    match error.status_code() {
        Some(401) => (
            "unauthorized",
            "Authentication required; API support is undetermined.",
        ),
        Some(403) => (
            "forbidden",
            "Read access denied; API support is undetermined.",
        ),
        Some(404) => (
            "not-served",
            "Requested route or namespace was not found; global API support is undetermined.",
        ),
        Some(_) => (
            "failed",
            "HTTP request failed; API support is undetermined.",
        ),
        None => match error {
            GrafanaCliError::Json(_) => ("malformed-response", "Response was not valid JSON."),
            _ => (
                "failed",
                "Transport or request failure; API support is undetermined.",
            ),
        },
    }
}

fn valid_list(api: &str, value: Option<Value>, namespace: &str) -> bool {
    match api {
        "classic" => value
            .as_ref()
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().all(|item| {
                    item.get("uid")
                        .and_then(Value::as_str)
                        .is_some_and(|uid| !uid.is_empty())
                        && item.get("type").and_then(Value::as_str) == Some("dash-db")
                })
            }),
        "v2" => validate_dashboard_resource_v2_list_response(value, namespace)
            .is_ok_and(|value| valid_resource_items(&value, api, namespace)),
        "v1" => value.as_ref().is_some_and(|value| {
            value.get("apiVersion").and_then(Value::as_str) == Some("dashboard.grafana.app/v1")
                && value.get("kind").and_then(Value::as_str) == Some("DashboardList")
                && valid_resource_items(value, api, namespace)
        }),
        _ => false,
    }
}

fn valid_resource_items(value: &Value, api: &str, namespace: &str) -> bool {
    let api_version = format!("dashboard.grafana.app/{api}");
    value
        .get("items")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items.iter().all(|item| {
                item.get("apiVersion").and_then(Value::as_str) == Some(api_version.as_str())
                    && item.get("kind").and_then(Value::as_str) == Some("Dashboard")
                    && item
                        .pointer("/metadata/name")
                        .and_then(Value::as_str)
                        .is_some_and(|name| !name.trim().is_empty())
                    && item.pointer("/metadata/namespace").and_then(Value::as_str)
                        == Some(namespace)
                    && item.get("spec").and_then(Value::as_object).is_some()
            })
        })
}

fn build_capabilities<F>(namespace: &str, request: &mut F) -> Result<DashboardCapabilities>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if namespace.trim().is_empty() || namespace != namespace.trim() {
        return Err(message(
            "--namespace must be non-empty and have no surrounding whitespace.",
        ));
    }
    let mut probes = Vec::new();
    for (api, path, params) in [
        (
            "classic",
            "/api/search".to_string(),
            vec![
                ("type".to_string(), "dash-db".to_string()),
                ("limit".to_string(), "1".to_string()),
                ("page".to_string(), "1".to_string()),
            ],
        ),
        (
            "v1",
            dashboard_resource_path(DashboardResourceApiVersion::V1, namespace)?,
            vec![("limit".to_string(), "1".to_string())],
        ),
        (
            "v2",
            dashboard_resource_path(DashboardResourceApiVersion::V2, namespace)?,
            vec![("limit".to_string(), "1".to_string())],
        ),
    ] {
        let (status, http_status, detail) = match request(Method::GET, &path, &params, None) {
            Ok(value) => {
                if valid_list(api, value, namespace) {
                    (
                        "available",
                        None,
                        "List read succeeded; write permissions and other operations are unverified.",
                    )
                } else {
                    (
                        "malformed-response",
                        None,
                        "Successful response did not match the expected dashboard list contract.",
                    )
                }
            }
            Err(error) => {
                let (status, detail) = classify_error(&error);
                (status, error.status_code(), detail)
            }
        };
        probes.push(CapabilityProbe {
            api,
            path,
            status,
            http_status,
            detail,
        });
    }
    Ok(DashboardCapabilities {
        kind: "grafana-dashboard-api-capabilities",
        schema_version: 1,
        namespace: namespace.to_string(),
        probes,
    })
}

fn render_text(report: &DashboardCapabilities) -> Vec<String> {
    let mut lines = vec![format!(
        "Dashboard API read capabilities (namespace={})",
        report.namespace
    )];
    for probe in &report.probes {
        let status = probe
            .http_status
            .map(|status| format!(" HTTP={status}"))
            .unwrap_or_default();
        lines.push(format!(
            "- {}: {}{} — {}",
            probe.api, probe.status, status, probe.detail
        ));
    }
    lines
}

/// Probe the three dashboard list APIs once without modifying Grafana.
pub fn run_status_capabilities(args: StatusCapabilitiesArgs) -> Result<()> {
    // Validate before resolving credentials or opening a connection.
    if args.namespace.trim().is_empty() || args.namespace != args.namespace.trim() {
        return Err(message(
            "--namespace must be non-empty and have no surrounding whitespace.",
        ));
    }
    let common = &args.common;
    let connection = GrafanaConnection::resolve(
        common.profile.as_deref(),
        ConnectionMergeInput {
            url: &common.url,
            url_default: "",
            api_token: common.api_token.as_deref(),
            username: common.username.as_deref(),
            password: common.password.as_deref(),
            org_id: args.org_id,
            timeout: common.timeout,
            timeout_default: DEFAULT_TIMEOUT,
            verify_ssl: common.verify_ssl,
            insecure: args.insecure,
            ca_cert: args.ca_cert.as_deref(),
        },
        AuthInputs {
            api_token: common.api_token.as_deref(),
            username: common.username.as_deref(),
            password: common.password.as_deref(),
            prompt_password: common.prompt_password,
            prompt_token: common.prompt_token,
        },
        true,
    )?;
    let client = connection.build_http_client()?;
    let report = build_capabilities(&args.namespace, &mut |method, path, params, payload| {
        client.request_json(method, path, params, payload)
    })?;
    set_json_color_choice(common.color);
    match args.output_format {
        CapabilityOutputFormat::Text => {
            for line in render_text(&report) {
                println!("{line}");
            }
        }
        CapabilityOutputFormat::Json => println!("{}", render_json_value(&report)?),
        CapabilityOutputFormat::Yaml => println!("{}", render_yaml(&report)?),
    }
    Ok(())
}

#[cfg(test)]
#[path = "capabilities_tests.rs"]
mod tests;
