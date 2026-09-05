use serde_json::{Map, Value};

use crate::common::Result;

pub(crate) fn resolve_live_datasource_target(
    datasources: &[Map<String, Value>],
    identity: &str,
) -> Result<Option<Map<String, Value>>> {
    for datasource in datasources {
        if datasource.get("uid").and_then(Value::as_str).map(str::trim) == Some(identity) {
            return Ok(Some(datasource.clone()));
        }
    }
    for datasource in datasources {
        if datasource
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            == Some(identity)
        {
            return Ok(Some(datasource.clone()));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn datasource(uid: &str, name: &str, id: Value) -> Map<String, Value> {
        serde_json::from_value(json!({
            "uid": uid,
            "name": name,
            "id": id,
        }))
        .expect("datasource object")
    }

    #[test]
    fn resolves_datasource_target_by_uid_before_name() {
        let datasources = vec![
            datasource("uid-a", "shared-name", Value::from(1)),
            datasource("uid-b", "uid-a", Value::from(2)),
        ];

        let target = resolve_live_datasource_target(&datasources, "uid-a")
            .unwrap()
            .unwrap();

        assert_eq!(target["uid"], json!("uid-a"));
        assert_eq!(target["id"], json!(1));
    }

    #[test]
    fn resolves_datasource_target_by_name_when_uid_does_not_match() {
        let datasources = vec![datasource("uid-a", "shared-name", Value::from(1))];

        let target = resolve_live_datasource_target(&datasources, "shared-name")
            .unwrap()
            .unwrap();

        assert_eq!(target["uid"], json!("uid-a"));
        assert_eq!(target["name"], json!("shared-name"));
    }
}
