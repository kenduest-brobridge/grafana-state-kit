//! Datasource import dry-run secret placeholder visibility helpers.

use serde_json::{Map, Value};
use std::path::Path;

use crate::datasource_secret::{
    build_secret_placeholder_plan, inline_secret_provider_contract,
    summarize_secret_placeholder_plan, summarize_secret_provider_contract,
};

use super::{load_import_records, DatasourceImportInputFormat};

pub(crate) fn build_import_secret_visibility_entries(
    input_dir: &Path,
    input_format: DatasourceImportInputFormat,
) -> Vec<Value> {
    let Ok((_, records)) = load_import_records(input_dir, input_format) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    for record in records {
        let Some(placeholders) = &record.secure_json_data_placeholders else {
            continue;
        };
        let datasource_spec = Map::from_iter(vec![
            ("uid".to_string(), Value::String(record.uid.clone())),
            ("name".to_string(), Value::String(record.name.clone())),
            (
                "type".to_string(),
                Value::String(record.datasource_type.clone()),
            ),
            (
                "secureJsonDataPlaceholders".to_string(),
                Value::Object(placeholders.clone()),
            ),
        ]);
        match build_secret_placeholder_plan(&datasource_spec) {
            Ok(plan) => entries.push(summarize_secret_placeholder_plan(&plan)),
            Err(error) => entries.push(Value::Object(Map::from_iter(vec![
                (
                    "provider".to_string(),
                    summarize_secret_provider_contract(&inline_secret_provider_contract()),
                ),
                (
                    "datasourceUid".to_string(),
                    Value::String(record.uid.clone()),
                ),
                (
                    "datasourceName".to_string(),
                    Value::String(record.name.clone()),
                ),
                (
                    "datasourceType".to_string(),
                    Value::String(record.datasource_type.clone()),
                ),
                (
                    "providerKind".to_string(),
                    Value::String(inline_secret_provider_contract().kind),
                ),
                (
                    "action".to_string(),
                    Value::String("secret-plan-error".to_string()),
                ),
                ("reviewRequired".to_string(), Value::Bool(true)),
                ("error".to_string(), Value::String(error.to_string())),
            ]))),
        }
    }
    entries
}
