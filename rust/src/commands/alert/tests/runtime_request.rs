use crate::common::api_response;
use crate::grafana_api::alert_live::request_optional_object_with_request;
use reqwest::Method;

#[test]
fn contact_point_import_preserves_forbidden_without_retry_or_create_fallback() {
    use crate::grafana_api::alert_live::import_resource_document_with_request;
    use serde_json::json;

    let payload = json!({
        "uid": "cp-main",
        "name": "Main",
        "type": "webhook",
        "settings": {"url": "http://127.0.0.1/hook"}
    });
    for forbidden_method in [Method::GET, Method::PUT, Method::POST] {
        let mut calls = Vec::new();
        let replace_existing = forbidden_method != Method::POST;
        let error = import_resource_document_with_request(
            |method, path, _params, _payload| {
                calls.push((method.clone(), path.to_string()));
                if method == forbidden_method {
                    return Err(api_response(
                        403,
                        format!("http://127.0.0.1:3000{path}"),
                        r#"{"message":"permission denied"}"#,
                    ));
                }
                assert_eq!(method, Method::GET);
                Ok(Some(json!([payload.clone()])))
            },
            super::CONTACT_POINT_KIND,
            payload.as_object().unwrap(),
            replace_existing,
        )
        .unwrap_err();

        assert_eq!(error.status_code(), Some(403));
        assert!(error.to_string().contains("permission denied"));
        let expected = if forbidden_method == Method::PUT {
            vec![
                (
                    Method::GET,
                    "/api/v1/provisioning/contact-points".to_string(),
                ),
                (
                    Method::PUT,
                    "/api/v1/provisioning/contact-points/cp-main".to_string(),
                ),
            ]
        } else {
            vec![(
                forbidden_method,
                "/api/v1/provisioning/contact-points".to_string(),
            )]
        };
        assert_eq!(calls, expected);
    }
}

#[test]
fn request_optional_object_with_request_treats_http_404_as_missing() {
    let result = request_optional_object_with_request(
        |_method, path, _params, _payload| {
            Err(api_response(
                404,
                format!("http://127.0.0.1:3000{path}"),
                "",
            ))
        },
        Method::GET,
        "/api/v1/provisioning/alert-rules/missing-rule",
        None,
    )
    .unwrap();

    assert!(result.is_none());
}
