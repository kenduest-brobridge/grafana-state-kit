use reqwest::Method;
use serde_json::json;

use super::*;
use crate::dashboard::history::{
    fetch_dashboard_history_version_data_with_request, list_dashboard_history_versions_with_request,
};

#[test]
fn classic_request_facades_encode_reserved_uid_delimiters() {
    let uid = "cpu/main?orgId=2#frag %one";
    let mut calls = Vec::new();
    let mut request = |method: Method, path: &str, _: &[(String, String)], _: Option<&Value>| {
        calls.push((method, path.to_string()));
        Ok(Some(if path.ends_with("/versions/3") {
            json!({"data": {"uid": uid}})
        } else if path.ends_with("/versions") || path.ends_with("/permissions") {
            json!([])
        } else {
            json!({"dashboard": {"uid": uid}, "uid": uid})
        }))
    };
    assert_eq!(
        fetch_dashboard_with_request(&mut request, uid).unwrap()["dashboard"]["uid"],
        uid
    );
    fetch_folder_if_exists_with_request(&mut request, uid).unwrap();
    fetch_dashboard_permissions_with_request(&mut request, uid).unwrap();
    fetch_folder_permissions_with_request(&mut request, uid).unwrap();
    delete_dashboard_request_with_request(&mut request, uid).unwrap();
    delete_folder_request_with_request(&mut request, uid).unwrap();
    list_dashboard_history_versions_with_request(&mut request, uid, 1).unwrap();
    assert_eq!(
        fetch_dashboard_history_version_data_with_request(&mut request, uid, 3).unwrap()["uid"],
        uid
    );

    let encoded_uid = "cpu%2Fmain%3ForgId%3D2%23frag%20%25one";
    assert_eq!(
        calls,
        vec![
            (Method::GET, format!("/api/dashboards/uid/{encoded_uid}")),
            (Method::GET, format!("/api/folders/{encoded_uid}")),
            (
                Method::GET,
                format!("/api/dashboards/uid/{encoded_uid}/permissions")
            ),
            (
                Method::GET,
                format!("/api/folders/{encoded_uid}/permissions")
            ),
            (Method::DELETE, format!("/api/dashboards/uid/{encoded_uid}")),
            (Method::DELETE, format!("/api/folders/{encoded_uid}")),
            (
                Method::GET,
                format!("/api/dashboards/uid/{encoded_uid}/versions")
            ),
            (
                Method::GET,
                format!("/api/dashboards/uid/{encoded_uid}/versions/3")
            ),
        ]
    );
}
