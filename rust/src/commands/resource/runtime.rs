use reqwest::Method;
use serde_json::{Map, Value};

use super::catalog::ResourceSelector;
use super::cli_defs::{ResourceApiMode, ResourceCliArgs, ResourceCommand, ResourceKind};
use super::render::{render_describe, render_get, render_kind_catalog, render_list};
use crate::common::{message, set_json_color_choice, Result};
use crate::dashboard::{CommonCliArgs, DEFAULT_TIMEOUT, DEFAULT_URL};
use crate::grafana_api::{
    expect_object, expect_object_list, AppPlatformApiVersion, AppPlatformResource, AuthInputs,
    GrafanaApiClient, GrafanaConnection,
};
use crate::profile_config::ConnectionMergeInput;

pub(crate) fn build_client(common: &CommonCliArgs) -> Result<GrafanaApiClient> {
    let connection = GrafanaConnection::resolve(
        common.profile.as_deref(),
        ConnectionMergeInput {
            url: &common.url,
            url_default: DEFAULT_URL,
            api_token: common.api_token.as_deref(),
            username: common.username.as_deref(),
            password: common.password.as_deref(),
            org_id: None,
            timeout: common.timeout,
            timeout_default: DEFAULT_TIMEOUT,
            verify_ssl: common.verify_ssl,
            insecure: false,
            ca_cert: None,
        },
        AuthInputs {
            api_token: common.api_token.as_deref(),
            username: common.username.as_deref(),
            password: common.password.as_deref(),
            prompt_password: common.prompt_password,
            prompt_token: common.prompt_token,
        },
        false,
    )?;
    GrafanaApiClient::from_connection(connection)
}

pub(crate) fn list_resource_items(
    client: &GrafanaApiClient,
    kind: ResourceKind,
    api_mode: ResourceApiMode,
    namespace: &str,
) -> Result<Vec<Map<String, Value>>> {
    let legacy_dashboards = || client.dashboard().list_dashboard_summaries(500);
    let legacy_folders = || {
        expect_object_list(
            client
                .dashboard()
                .request_json(Method::GET, "/api/folders", &[], None)?,
            "Unexpected folder list response from Grafana.",
        )
    };
    match kind {
        ResourceKind::Dashboards => match api_mode {
            ResourceApiMode::Auto => fallback_on_404(
                &|| list_app_resource(client, kind, AppPlatformApiVersion::V2, namespace),
                &|| {
                    fallback_on_404(
                        &|| list_app_resource(client, kind, AppPlatformApiVersion::V1, namespace),
                        &legacy_dashboards,
                    )
                },
            ),
            ResourceApiMode::Legacy => legacy_dashboards(),
            ResourceApiMode::V1 => {
                list_app_resource(client, kind, AppPlatformApiVersion::V1, namespace)
            }
            ResourceApiMode::V2 => {
                list_app_resource(client, kind, AppPlatformApiVersion::V2, namespace)
            }
            ResourceApiMode::V1Beta1 => unsupported_mode(kind, api_mode),
        },
        ResourceKind::Folders => match api_mode {
            ResourceApiMode::Auto => fallback_on_404(
                &|| list_app_resource(client, kind, AppPlatformApiVersion::V1, namespace),
                &legacy_folders,
            ),
            ResourceApiMode::Legacy => legacy_folders(),
            ResourceApiMode::V1 => {
                list_app_resource(client, kind, AppPlatformApiVersion::V1, namespace)
            }
            ResourceApiMode::V2 | ResourceApiMode::V1Beta1 => unsupported_mode(kind, api_mode),
        },
        ResourceKind::Datasources => {
            ensure_legacy_only_mode(kind, api_mode)?;
            client.datasource().list_datasources()
        }
        ResourceKind::AlertRules => {
            ensure_legacy_only_mode(kind, api_mode)?;
            client.alerting().list_alert_rules()
        }
        ResourceKind::Orgs => {
            ensure_legacy_only_mode(kind, api_mode)?;
            client.access().list_orgs()
        }
        ResourceKind::Receivers
        | ResourceKind::RoutingTrees
        | ResourceKind::TemplateGroups
        | ResourceKind::TimeIntervals
        | ResourceKind::InhibitionRules
        | ResourceKind::SecureValues => {
            ensure_v1beta1_mode(kind, api_mode)?;
            list_app_resource(client, kind, AppPlatformApiVersion::V1Beta1, namespace)
        }
    }
}

pub(crate) fn get_resource_item(
    client: &GrafanaApiClient,
    selector: &ResourceSelector,
    api_mode: ResourceApiMode,
    namespace: &str,
) -> Result<Value> {
    match selector.kind {
        ResourceKind::Dashboards => {
            let legacy = || client.dashboard().fetch_dashboard(&selector.identity);
            match api_mode {
                ResourceApiMode::Auto => fallback_on_404(
                    &|| {
                        get_app_resource(
                            client,
                            selector.kind,
                            AppPlatformApiVersion::V2,
                            namespace,
                            &selector.identity,
                        )
                    },
                    &|| {
                        fallback_on_404(
                            &|| {
                                get_app_resource(
                                    client,
                                    selector.kind,
                                    AppPlatformApiVersion::V1,
                                    namespace,
                                    &selector.identity,
                                )
                            },
                            &legacy,
                        )
                    },
                ),
                ResourceApiMode::Legacy => legacy(),
                ResourceApiMode::V1 => get_app_resource(
                    client,
                    selector.kind,
                    AppPlatformApiVersion::V1,
                    namespace,
                    &selector.identity,
                ),
                ResourceApiMode::V2 => get_app_resource(
                    client,
                    selector.kind,
                    AppPlatformApiVersion::V2,
                    namespace,
                    &selector.identity,
                ),
                ResourceApiMode::V1Beta1 => unsupported_mode(selector.kind, api_mode),
            }
        }
        ResourceKind::Folders => {
            let legacy = || {
                let object = client.dashboard().request_json(
                    Method::GET,
                    &format!("/api/folders/{}", selector.identity),
                    &[],
                    None,
                )?;
                Ok(Value::Object(expect_object(
                    object,
                    "Unexpected folder payload from Grafana.",
                )?))
            };
            match api_mode {
                ResourceApiMode::Auto => fallback_on_404(
                    &|| {
                        get_app_resource(
                            client,
                            selector.kind,
                            AppPlatformApiVersion::V1,
                            namespace,
                            &selector.identity,
                        )
                    },
                    &legacy,
                ),
                ResourceApiMode::Legacy => legacy(),
                ResourceApiMode::V1 => get_app_resource(
                    client,
                    selector.kind,
                    AppPlatformApiVersion::V1,
                    namespace,
                    &selector.identity,
                ),
                ResourceApiMode::V2 | ResourceApiMode::V1Beta1 => {
                    unsupported_mode(selector.kind, api_mode)
                }
            }
        }
        ResourceKind::Datasources => {
            ensure_legacy_only_mode(selector.kind, api_mode)?;
            let legacy_identity_is_numeric = selector.identity.parse::<u64>().is_ok();
            if api_mode == ResourceApiMode::Legacy {
                ensure_legacy_datasource_identity(&selector.identity)?;
            }
            let legacy_request = || {
                client.datasource().request_json(
                    Method::GET,
                    &format!("/api/datasources/{}", selector.identity),
                    &[],
                    None,
                )
            };
            let legacy_request = legacy_identity_is_numeric
                .then_some(&legacy_request as &dyn Fn() -> Result<Option<Value>>);
            let object = resolve_with_api_mode(
                api_mode,
                &|| {
                    client.datasource().request_json(
                        Method::GET,
                        &format!("/api/datasources/uid/{}", selector.identity),
                        &[],
                        None,
                    )
                },
                legacy_request,
            )?;
            Ok(Value::Object(expect_object(
                object,
                "Unexpected datasource payload from Grafana.",
            )?))
        }
        ResourceKind::AlertRules => {
            ensure_legacy_only_mode(selector.kind, api_mode)?;
            Ok(Value::Object(resolve_with_api_mode(
                api_mode,
                &|| client.alerting().get_alert_rule(&selector.identity),
                None,
            )?))
        }
        ResourceKind::Orgs => {
            ensure_legacy_only_mode(selector.kind, api_mode)?;
            let object = resolve_with_api_mode(
                api_mode,
                &|| {
                    client.access().request_json(
                        Method::GET,
                        &format!("/api/orgs/{}", selector.identity),
                        &[],
                        None,
                    )
                },
                None,
            )?;
            Ok(Value::Object(expect_object(
                object,
                "Unexpected org payload from Grafana.",
            )?))
        }
        ResourceKind::Receivers
        | ResourceKind::RoutingTrees
        | ResourceKind::TemplateGroups
        | ResourceKind::TimeIntervals
        | ResourceKind::InhibitionRules
        | ResourceKind::SecureValues => {
            ensure_v1beta1_mode(selector.kind, api_mode)?;
            get_app_resource(
                client,
                selector.kind,
                AppPlatformApiVersion::V1Beta1,
                namespace,
                &selector.identity,
            )
        }
    }
}

fn app_platform_resource(
    kind: ResourceKind,
    version: AppPlatformApiVersion,
) -> Result<AppPlatformResource> {
    let (group, plural, resource_kind) = match kind {
        ResourceKind::Dashboards => ("dashboard.grafana.app", "dashboards", "Dashboard"),
        ResourceKind::Folders => ("folder.grafana.app", "folders", "Folder"),
        ResourceKind::Receivers => (
            "notifications.alerting.grafana.app",
            "receivers",
            "Receiver",
        ),
        ResourceKind::RoutingTrees => (
            "notifications.alerting.grafana.app",
            "routingtrees",
            "RoutingTree",
        ),
        ResourceKind::TemplateGroups => (
            "notifications.alerting.grafana.app",
            "templategroups",
            "TemplateGroup",
        ),
        ResourceKind::TimeIntervals => (
            "notifications.alerting.grafana.app",
            "timeintervals",
            "TimeInterval",
        ),
        ResourceKind::InhibitionRules => (
            "notifications.alerting.grafana.app",
            "inhibitionrules",
            "InhibitionRule",
        ),
        ResourceKind::SecureValues => ("secret.grafana.app", "securevalues", "SecureValue"),
        _ => {
            return Err(message(format!(
                "{} has no App Platform API.",
                kind.as_str()
            )))
        }
    };
    Ok(AppPlatformResource::new(
        group,
        version,
        plural,
        resource_kind,
    ))
}

fn list_app_resource(
    client: &GrafanaApiClient,
    kind: ResourceKind,
    version: AppPlatformApiVersion,
    namespace: &str,
) -> Result<Vec<Map<String, Value>>> {
    client
        .app_platform()
        .list_all(app_platform_resource(kind, version)?, namespace, Some(500))?
        .into_iter()
        .map(|value| {
            redact_secure_value(kind, value)
                .as_object()
                .cloned()
                .ok_or_else(|| message("App Platform resource list item is not an object."))
        })
        .collect()
}

fn get_app_resource(
    client: &GrafanaApiClient,
    kind: ResourceKind,
    version: AppPlatformApiVersion,
    namespace: &str,
    name: &str,
) -> Result<Value> {
    let value =
        client
            .app_platform()
            .get(app_platform_resource(kind, version)?, namespace, name)?;
    Ok(redact_secure_value(kind, value))
}

fn redact_secure_value(kind: ResourceKind, mut value: Value) -> Value {
    if kind == ResourceKind::SecureValues {
        if let Some(spec) = value.get_mut("spec").and_then(Value::as_object_mut) {
            spec.remove("value");
        }
    }
    value
}

fn fallback_on_404<T>(
    primary: &dyn Fn() -> Result<T>,
    fallback: &dyn Fn() -> Result<T>,
) -> Result<T> {
    match primary() {
        Err(error) if error.status_code() == Some(404) => fallback(),
        result => result,
    }
}

fn unsupported_mode<T>(kind: ResourceKind, mode: ResourceApiMode) -> Result<T> {
    Err(message(format!(
        "Resource kind {} does not support API mode {:?}.",
        kind.as_str(),
        mode
    )))
}

fn ensure_legacy_only_mode(kind: ResourceKind, mode: ResourceApiMode) -> Result<()> {
    match mode {
        ResourceApiMode::Auto | ResourceApiMode::Legacy => Ok(()),
        _ => unsupported_mode(kind, mode),
    }
}

fn ensure_v1beta1_mode(kind: ResourceKind, mode: ResourceApiMode) -> Result<()> {
    if mode == ResourceApiMode::V1Beta1 {
        Ok(())
    } else {
        Err(message(format!(
            "Resource kind {} is public preview and requires --api-mode v1beta1.",
            kind.as_str()
        )))
    }
}

fn ensure_legacy_datasource_identity(identity: &str) -> Result<()> {
    if identity.parse::<u64>().is_ok() {
        return Ok(());
    }
    Err(message(format!(
        "Legacy datasource API mode requires a numeric datasource ID, got '{identity}'."
    )))
}

fn resolve_with_api_mode<T>(
    mode: ResourceApiMode,
    modern: &dyn Fn() -> Result<T>,
    legacy: Option<&dyn Fn() -> Result<T>>,
) -> Result<T> {
    match mode {
        ResourceApiMode::Auto => match modern() {
            Ok(value) => Ok(value),
            Err(error) if error.status_code() == Some(404) => {
                if let Some(legacy_attempt) = legacy {
                    legacy_attempt()
                } else {
                    Err(error)
                }
            }
            Err(error) => Err(error),
        },
        ResourceApiMode::Legacy => {
            if let Some(legacy_attempt) = legacy {
                legacy_attempt()
            } else {
                modern()
            }
        }
        ResourceApiMode::V1 | ResourceApiMode::V2 | ResourceApiMode::V1Beta1 => modern(),
    }
}

pub fn run_resource_cli(args: ResourceCliArgs) -> Result<()> {
    // Resource is a pure read-only surface; dispatch only routes to renderers,
    // with output shape determined by each command variant.
    set_json_color_choice(args.color);
    match args.command {
        ResourceCommand::Kinds(inner) => render_kind_catalog(&inner),
        ResourceCommand::Describe(inner) => render_describe(&inner),
        ResourceCommand::List(inner) => render_list(&inner),
        ResourceCommand::Get(inner) => render_get(&inner),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{ErrorKind, Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::super::catalog::ResourceSelector;
    use super::super::cli_defs::ResourceApiMode;
    use super::super::cli_defs::ResourceKind;
    use crate::common::{api_response, message, Result};
    use crate::grafana_api::{GrafanaApiClient, GrafanaConnection};
    use serde_json::Value;

    type SequenceServer = (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>);

    fn http_response(status: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn spawn_sequence_server(responses: Vec<String>) -> Option<SequenceServer> {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(error) if error.kind() == ErrorKind::PermissionDenied => return None,
            Err(error) => panic!("failed to bind test listener: {error}"),
        };
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();

                let mut request = Vec::new();
                let mut buffer = [0_u8; 1024];
                loop {
                    let bytes_read = stream.read(&mut buffer).unwrap();
                    if bytes_read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..bytes_read]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                let request_line = String::from_utf8_lossy(&request)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_string();
                requests_thread.lock().unwrap().push(request_line);

                stream.write_all(response.as_bytes()).unwrap();
                let _ = stream.flush();
            }
        });
        Some((format!("http://{address}"), requests, handle))
    }

    fn build_test_api(base_url: String) -> GrafanaApiClient {
        GrafanaApiClient::from_connection(GrafanaConnection::new(
            base_url,
            vec![("Authorization".to_string(), "Bearer token".to_string())],
            5,
            false,
            None,
            "token".to_string(),
        ))
        .unwrap()
    }

    fn datasource_selector(identity: &str) -> ResourceSelector {
        ResourceSelector {
            kind: ResourceKind::Datasources,
            identity: identity.to_string(),
        }
    }

    fn selector(kind: ResourceKind, identity: &str) -> ResourceSelector {
        ResourceSelector {
            kind,
            identity: identity.to_string(),
        }
    }

    #[test]
    fn dashboard_auto_mode_falls_back_from_v2_to_v1_before_legacy() {
        let responses = vec![
            http_response("404 Not Found", r#"{"message":"not found"}"#),
            http_response(
                "200 OK",
                r#"{"apiVersion":"dashboard.grafana.app/v1","kind":"DashboardList","items":[{"apiVersion":"dashboard.grafana.app/v1","kind":"Dashboard","metadata":{"name":"cpu-main","namespace":"default"},"spec":{"title":"CPU Main"}}],"metadata":{}}"#,
            ),
        ];
        let Some((base_url, requests, handle)) = spawn_sequence_server(responses) else {
            return;
        };
        let api = build_test_api(base_url);

        let items = super::list_resource_items(
            &api,
            ResourceKind::Dashboards,
            ResourceApiMode::Auto,
            "default",
        )
        .unwrap();

        handle.join().unwrap();
        assert_eq!(items[0]["metadata"]["name"], "cpu-main");
        let requests = requests.lock().unwrap().clone();
        assert!(requests[0]
            .starts_with("GET /apis/dashboard.grafana.app/v2/namespaces/default/dashboards"));
        assert!(requests[1]
            .starts_with("GET /apis/dashboard.grafana.app/v1/namespaces/default/dashboards"));
    }

    #[test]
    fn receiver_get_uses_explicit_notification_v1beta1_path() {
        let responses = vec![http_response(
            "200 OK",
            r#"{"apiVersion":"notifications.alerting.grafana.app/v1beta1","kind":"Receiver","metadata":{"name":"oncall","namespace":"default"},"spec":{"integrations":[]}}"#,
        )];
        let Some((base_url, requests, handle)) = spawn_sequence_server(responses) else {
            return;
        };
        let api = build_test_api(base_url);

        let value = super::get_resource_item(
            &api,
            &selector(ResourceKind::Receivers, "oncall"),
            ResourceApiMode::V1Beta1,
            "default",
        )
        .unwrap();

        handle.join().unwrap();
        assert_eq!(value["metadata"]["name"], "oncall");
        let requests = requests.lock().unwrap().clone();
        assert!(requests[0].starts_with(
            "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/default/receivers/oncall"
        ));
    }

    #[test]
    fn preview_resource_kind_requires_explicit_v1beta1_mode() {
        let api = build_test_api("http://127.0.0.1:9".to_string());

        let error = super::list_resource_items(
            &api,
            ResourceKind::RoutingTrees,
            ResourceApiMode::Auto,
            "default",
        )
        .expect_err("preview resources must not be auto-probed");

        assert!(error.to_string().contains("requires --api-mode v1beta1"));
    }

    #[test]
    fn alert_rule_get_rejects_unsupported_explicit_api_modes_before_transport() {
        let api = build_test_api("http://127.0.0.1:9".to_string());
        for mode in [
            ResourceApiMode::V1,
            ResourceApiMode::V2,
            ResourceApiMode::V1Beta1,
        ] {
            let error = super::get_resource_item(
                &api,
                &selector(ResourceKind::AlertRules, "rule-1"),
                mode,
                "default",
            )
            .expect_err("unsupported alert-rule API mode must fail before transport");
            assert!(error
                .to_string()
                .contains("Resource kind alert-rules does not support API mode"));
        }
    }

    #[test]
    fn secure_value_get_never_exposes_spec_value() {
        let hostile =
            include_str!("../../../../tests/fixtures/grafana13/secure-value-hostile.json");
        let responses = vec![http_response("200 OK", hostile)];
        let Some((base_url, _requests, handle)) = spawn_sequence_server(responses) else {
            return;
        };
        let api = build_test_api(base_url);

        let value = super::get_resource_item(
            &api,
            &selector(ResourceKind::SecureValues, "pagerduty-token"),
            ResourceApiMode::V1Beta1,
            "default",
        )
        .unwrap();

        handle.join().unwrap();
        assert!(value["spec"].get("value").is_none());
        assert!(!value.to_string().contains("TOP-SECRET-MUST-NOT-RENDER"));
        assert_eq!(value["spec"]["description"], "PagerDuty integration token");
    }

    #[test]
    fn folder_auto_mode_prefers_v1_then_falls_back_to_legacy() {
        let responses = vec![
            http_response("404 Not Found", r#"{"message":"not found"}"#),
            http_response("200 OK", r#"[{"uid":"infra","title":"Infra"}]"#),
        ];
        let Some((base_url, requests, handle)) = spawn_sequence_server(responses) else {
            return;
        };
        let api = build_test_api(base_url);

        let items = super::list_resource_items(
            &api,
            ResourceKind::Folders,
            ResourceApiMode::Auto,
            "default",
        )
        .unwrap();

        handle.join().unwrap();
        assert_eq!(items[0]["uid"], "infra");
        let requests = requests.lock().unwrap().clone();
        assert!(
            requests[0].starts_with("GET /apis/folder.grafana.app/v1/namespaces/default/folders")
        );
        assert!(requests[1].starts_with("GET /api/folders"));
    }

    #[test]
    fn notification_preview_kinds_use_documented_plural_paths() {
        let kinds = [
            (ResourceKind::Receivers, "receivers"),
            (ResourceKind::RoutingTrees, "routingtrees"),
            (ResourceKind::TemplateGroups, "templategroups"),
            (ResourceKind::TimeIntervals, "timeintervals"),
            (ResourceKind::InhibitionRules, "inhibitionrules"),
        ];
        let responses = kinds
            .iter()
            .map(|(kind, _)| {
                let list_kind = match kind {
                    ResourceKind::Receivers => "ReceiverList",
                    ResourceKind::RoutingTrees => "RoutingTreeList",
                    ResourceKind::TemplateGroups => "TemplateGroupList",
                    ResourceKind::TimeIntervals => "TimeIntervalList",
                    ResourceKind::InhibitionRules => "InhibitionRuleList",
                    _ => unreachable!("notification preview test only covers App Platform kinds"),
                };
                let body = format!(
                    r#"{{"apiVersion":"notifications.alerting.grafana.app/v1beta1","kind":"{list_kind}","items":[],"metadata":{{}}}}"#
                );
                http_response("200 OK", &body)
            })
            .collect();
        let Some((base_url, requests, handle)) = spawn_sequence_server(responses) else {
            return;
        };
        let api = build_test_api(base_url);

        for (kind, _) in kinds {
            let items = super::list_resource_items(&api, kind, ResourceApiMode::V1Beta1, "default")
                .unwrap();
            assert!(items.is_empty());
        }

        handle.join().unwrap();
        let requests = requests.lock().unwrap().clone();
        for (request, (_, plural)) in requests.iter().zip(kinds) {
            assert!(request.starts_with(&format!(
                "GET /apis/notifications.alerting.grafana.app/v1beta1/namespaces/default/{plural}"
            )));
        }
    }

    #[test]
    fn receiver_fixture_round_trips_without_shape_changes() {
        let receiver =
            include_str!("../../../../tests/fixtures/grafana13/notification-receiver.json");
        let responses = vec![http_response("200 OK", receiver)];
        let Some((base_url, _requests, handle)) = spawn_sequence_server(responses) else {
            return;
        };
        let api = build_test_api(base_url);

        let value = super::get_resource_item(
            &api,
            &selector(ResourceKind::Receivers, "oncall"),
            ResourceApiMode::V1Beta1,
            "default",
        )
        .unwrap();

        handle.join().unwrap();
        assert_eq!(
            value,
            serde_json::from_str::<Value>(receiver).expect("fixture should be valid JSON")
        );
    }

    #[test]
    fn legacy_datasource_mode_rejects_nonnumeric_identity_before_transport() {
        let error = super::ensure_legacy_datasource_identity("prom-main").unwrap_err();
        assert_eq!(
            error.to_string(),
            "Legacy datasource API mode requires a numeric datasource ID, got 'prom-main'."
        );
    }

    #[test]
    fn resolve_with_api_mode_auto_uses_legacy_when_modern_404s() {
        let value: Result<i32> = super::resolve_with_api_mode(
            ResourceApiMode::Auto,
            &|| {
                Err(api_response(
                    404,
                    "http://grafana.local/api/datasources/uid/1",
                    "not found",
                ))
            },
            Some(&|| Ok(42)),
        );
        assert_eq!(value.unwrap(), 42);
    }

    #[test]
    fn resolve_with_api_mode_auto_preserves_non_404_legacy_error() {
        let value: Result<i32> = super::resolve_with_api_mode(
            ResourceApiMode::Auto,
            &|| {
                Err(api_response(
                    403,
                    "http://grafana.local/api/datasources/uid/1",
                    "forbidden",
                ))
            },
            Some(&|| unreachable!("legacy path should not be called when modern returns non-404")),
        );
        let error = value.unwrap_err();
        assert_eq!(error.status_code(), Some(403));
    }

    #[test]
    fn resolve_with_api_mode_legacy_does_not_use_modern_path() {
        let value: Result<i32> = super::resolve_with_api_mode(
            ResourceApiMode::Legacy,
            &|| unreachable!("modern path should not be called in legacy mode"),
            Some(&|| Ok(7)),
        );
        assert_eq!(value.unwrap(), 7);
    }

    #[test]
    fn resolve_with_api_mode_legacy_falls_back_to_modern_when_legacy_not_available() {
        let value: Result<i32> =
            super::resolve_with_api_mode(ResourceApiMode::Legacy, &|| Ok(99), None);
        assert_eq!(value.unwrap(), 99);
    }

    #[test]
    fn resolve_with_api_mode_legacy_returns_error_when_legacy_missing() {
        let value: Result<i32> = super::resolve_with_api_mode(
            ResourceApiMode::Legacy,
            &|| Err(message("modern-only")),
            None,
        );
        assert_eq!(
            value.err().map(|error| error.to_string()),
            Some("modern-only".to_string())
        );
    }

    #[test]
    fn get_datasource_auto_falls_back_from_uid_path_to_legacy_path_on_404() {
        let Some((base_url, requests, handle)) = spawn_sequence_server(vec![
            http_response("404 Not Found", r#"{"message":"not found"}"#),
            http_response(
                "200 OK",
                r#"{"id":10,"uid":"prom-main","name":"Prometheus"}"#,
            ),
        ]) else {
            return;
        };
        let client = build_test_api(base_url);
        let value = super::get_resource_item(
            &client,
            &datasource_selector("10"),
            ResourceApiMode::Auto,
            "default",
        )
        .unwrap();

        assert_eq!(value["name"], "Prometheus");
        handle.join().unwrap();
        let requests = requests.lock().unwrap();
        assert_eq!(requests[0], "GET /api/datasources/uid/10 HTTP/1.1");
        assert_eq!(requests[1], "GET /api/datasources/10 HTTP/1.1");
    }

    #[test]
    fn get_datasource_auto_preserves_nonnumeric_uid_404_without_legacy_request() {
        let Some((base_url, requests, handle)) = spawn_sequence_server(vec![http_response(
            "404 Not Found",
            r#"{"message":"not found"}"#,
        )]) else {
            return;
        };
        let client = build_test_api(base_url);

        let error = super::get_resource_item(
            &client,
            &datasource_selector("prom-main"),
            ResourceApiMode::Auto,
            "default",
        )
        .unwrap_err();

        assert_eq!(error.status_code(), Some(404));
        handle.join().unwrap();
        let requests = requests.lock().unwrap();
        assert_eq!(
            requests.as_slice(),
            ["GET /api/datasources/uid/prom-main HTTP/1.1"]
        );
    }

    #[test]
    fn get_datasource_legacy_uses_legacy_path_without_uid_probe() {
        let Some((base_url, requests, handle)) = spawn_sequence_server(vec![http_response(
            "200 OK",
            r#"{"id":10,"uid":"prom-main","name":"Prometheus"}"#,
        )]) else {
            return;
        };
        let client = build_test_api(base_url);
        let value = super::get_resource_item(
            &client,
            &datasource_selector("10"),
            ResourceApiMode::Legacy,
            "default",
        )
        .unwrap();

        assert_eq!(value["uid"], "prom-main");
        handle.join().unwrap();
        let requests = requests.lock().unwrap();
        assert_eq!(requests.as_slice(), ["GET /api/datasources/10 HTTP/1.1"]);
    }
}
