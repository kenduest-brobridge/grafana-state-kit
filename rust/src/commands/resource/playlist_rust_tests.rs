use super::tests::{build_test_api, http_response, spawn_sequence_server};
use super::*;
use serde_json::json;

fn playlist(name: &str) -> Value {
    json!({
        "apiVersion": "playlist.grafana.app/v1", "kind": "Playlist",
        "metadata": {"name": name, "namespace": "team space"},
        "spec": {"title": "Ops rotation", "interval": "5m",
            "items": [{"type": "dashboard_by_uid", "value": "cpu"}]}
    })
}

#[test]
fn playlist_list_follows_continuation_and_encodes_namespace() {
    let first = json!({"apiVersion":"playlist.grafana.app/v1", "kind":"PlaylistList",
        "metadata":{"continue":"next/token"}, "items":[playlist("first")]});
    let second = json!({"apiVersion":"playlist.grafana.app/v1", "kind":"PlaylistList",
        "metadata":{}, "items":[playlist("second")]});
    let Some((url, requests, handle)) = spawn_sequence_server(vec![
        http_response("200 OK", &first.to_string()),
        http_response("200 OK", &second.to_string()),
    ]) else {
        return;
    };
    let items = list_resource_items(
        &build_test_api(url),
        ResourceKind::Playlists,
        ResourceApiMode::Auto,
        "team space",
    )
    .unwrap();
    handle.join().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[1]["metadata"]["name"], "second");
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests
        .iter()
        .all(|r| r
            .starts_with("GET /apis/playlist.grafana.app/v1/namespaces/team%20space/playlists?")));
    assert!(requests[0].contains("limit=500"));
    assert!(requests[1].contains("continue=next%2Ftoken"));
}

#[test]
fn playlist_get_encodes_uid_and_preserves_full_spec() {
    let expected = playlist("ops/rotation");
    let Some((url, requests, handle)) =
        spawn_sequence_server(vec![http_response("200 OK", &expected.to_string())])
    else {
        return;
    };
    let value = get_resource_item(
        &build_test_api(url),
        &ResourceSelector {
            kind: ResourceKind::Playlists,
            identity: "ops/rotation".to_string(),
        },
        ResourceApiMode::V1,
        "team space",
    )
    .unwrap();
    handle.join().unwrap();
    assert_eq!(value, expected);
    assert!(requests.lock().unwrap()[0].starts_with(
        "GET /apis/playlist.grafana.app/v1/namespaces/team%20space/playlists/ops%2Frotation "
    ));
}

#[test]
fn playlist_get_rejects_wrong_identity_namespace_and_kind() {
    for field in ["name", "namespace", "kind"] {
        let mut invalid = playlist("ops");
        if field == "kind" {
            invalid[field] = json!("Dashboard");
        } else {
            invalid["metadata"][field] = json!("other");
        }
        let Some((url, _, handle)) =
            spawn_sequence_server(vec![http_response("200 OK", &invalid.to_string())])
        else {
            return;
        };
        assert!(get_resource_item(
            &build_test_api(url),
            &ResourceSelector {
                kind: ResourceKind::Playlists,
                identity: "ops".to_string(),
            },
            ResourceApiMode::Auto,
            "team space"
        )
        .is_err());
        handle.join().unwrap();
    }
}

#[test]
fn playlist_http_errors_do_not_fall_back_to_other_apis() {
    for status in [
        "401 Unauthorized",
        "403 Forbidden",
        "404 Not Found",
        "500 Internal Server Error",
    ] {
        let Some((url, requests, handle)) =
            spawn_sequence_server(vec![http_response(status, r#"{"message":"unavailable"}"#)])
        else {
            return;
        };
        let error = list_resource_items(
            &build_test_api(url),
            ResourceKind::Playlists,
            ResourceApiMode::Auto,
            "team space",
        )
        .unwrap_err();
        assert_eq!(error.status_code(), Some(status[..3].parse().unwrap()));
        handle.join().unwrap();
        assert_eq!(requests.lock().unwrap().len(), 1);
    }
}

#[test]
fn playlist_rejects_unsupported_modes_before_requesting() {
    let api = build_test_api("http://127.0.0.1:9".to_string());
    for mode in [
        ResourceApiMode::Legacy,
        ResourceApiMode::V2,
        ResourceApiMode::V1Beta1,
    ] {
        assert!(
            list_resource_items(&api, ResourceKind::Playlists, mode, "default")
                .unwrap_err()
                .to_string()
                .contains("does not support API mode")
        );
        assert!(get_resource_item(
            &api,
            &ResourceSelector {
                kind: ResourceKind::Playlists,
                identity: "ops".to_string(),
            },
            mode,
            "default"
        )
        .is_err());
    }
}

#[test]
fn playlist_list_accepts_empty_collection() {
    let body = json!({"apiVersion":"playlist.grafana.app/v1", "kind":"PlaylistList",
        "metadata":{}, "items":[]});
    let Some((url, requests, handle)) =
        spawn_sequence_server(vec![http_response("200 OK", &body.to_string())])
    else {
        return;
    };
    let items = list_resource_items(
        &build_test_api(url),
        ResourceKind::Playlists,
        ResourceApiMode::Auto,
        "default",
    )
    .unwrap();
    handle.join().unwrap();
    assert!(items.is_empty());
    assert_eq!(requests.lock().unwrap().len(), 1);
}
