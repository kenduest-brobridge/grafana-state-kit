use super::{
    render_resource_v2_dry_run_json, render_resource_v2_dry_run_table,
    render_resource_v2_dry_run_text, ResourceV2DryRunAction, ResourceV2DryRunRow,
};

fn sample_rows() -> Vec<ResourceV2DryRunRow> {
    vec![
        ResourceV2DryRunRow {
            api_version: "dashboard.grafana.app/v2".to_string(),
            namespace: "team-a".to_string(),
            name: "overview".to_string(),
            source_path: "dashboards/overview.json".to_string(),
            action: ResourceV2DryRunAction::Update,
            folder: Some("operations".to_string()),
            ownership: Some("team-a".to_string()),
            blocker: None,
            changed_fields: vec!["spec.title".to_string(), "spec.tags".to_string()],
        },
        ResourceV2DryRunRow {
            api_version: "dashboard.grafana.app/v2".to_string(),
            namespace: "team-b".to_string(),
            name: "new-dashboard".to_string(),
            source_path: "dashboards/new.json".to_string(),
            action: ResourceV2DryRunAction::Create,
            folder: None,
            ownership: None,
            blocker: None,
            changed_fields: Vec::new(),
        },
    ]
}

#[test]
fn json_rendering_is_stable_and_uses_public_field_names() {
    let rendered = render_resource_v2_dry_run_json(&sample_rows()).unwrap();

    assert_eq!(
        rendered,
        concat!(
            "{\n",
            "  \"kind\": \"grafana-util-dashboard-resource-v2-import-dry-run\",\n",
            "  \"mode\": \"resource-v2\",\n",
            "  \"dashboards\": [\n",
            "    {\n",
            "      \"apiVersion\": \"dashboard.grafana.app/v2\",\n",
            "      \"namespace\": \"team-a\",\n",
            "      \"name\": \"overview\",\n",
            "      \"sourcePath\": \"dashboards/overview.json\",\n",
            "      \"action\": \"update\",\n",
            "      \"folder\": \"operations\",\n",
            "      \"ownership\": \"team-a\",\n",
            "      \"blocker\": null,\n",
            "      \"changedFields\": [\n",
            "        \"spec.title\",\n",
            "        \"spec.tags\"\n",
            "      ]\n",
            "    },\n",
            "    {\n",
            "      \"apiVersion\": \"dashboard.grafana.app/v2\",\n",
            "      \"namespace\": \"team-b\",\n",
            "      \"name\": \"new-dashboard\",\n",
            "      \"sourcePath\": \"dashboards/new.json\",\n",
            "      \"action\": \"create\",\n",
            "      \"folder\": null,\n",
            "      \"ownership\": null,\n",
            "      \"blocker\": null,\n",
            "      \"changedFields\": []\n",
            "    }\n",
            "  ],\n",
            "  \"summary\": {\n",
            "    \"total\": 2,\n",
            "    \"create\": 1,\n",
            "    \"update\": 1,\n",
            "    \"noChange\": 0,\n",
            "    \"skip\": 0,\n",
            "    \"blocked\": 0\n",
            "  }\n",
            "}\n"
        )
    );
}

#[test]
fn table_rendering_supports_selected_columns_without_a_header() {
    let columns = vec![
        "action".to_string(),
        "name".to_string(),
        "changedFields".to_string(),
    ];

    let lines = render_resource_v2_dry_run_table(&sample_rows(), false, Some(&columns));

    assert_eq!(
        lines,
        vec![
            "update  overview       spec.title,spec.tags",
            "create  new-dashboard                      ",
        ]
    );
}

#[test]
fn text_and_table_render_all_resource_v2_actions() {
    let rows = [
        ResourceV2DryRunRow::new(
            "v2",
            "ns",
            "create",
            "create.json",
            ResourceV2DryRunAction::Create,
        ),
        ResourceV2DryRunRow::new(
            "v2",
            "ns",
            "update",
            "update.json",
            ResourceV2DryRunAction::Update,
        ),
        ResourceV2DryRunRow::new(
            "v2",
            "ns",
            "same",
            "same.json",
            ResourceV2DryRunAction::NoChange,
        ),
        ResourceV2DryRunRow::new(
            "v2",
            "ns",
            "skip",
            "skip.json",
            ResourceV2DryRunAction::Skip,
        ),
        ResourceV2DryRunRow::new(
            "v2",
            "ns",
            "blocked",
            "blocked.json",
            ResourceV2DryRunAction::Blocked,
        ),
    ];

    let text = render_resource_v2_dry_run_text(&rows);
    let columns = vec!["action".to_string(), "name".to_string()];
    let table = render_resource_v2_dry_run_table(&rows, true, Some(&columns)).join("\n");

    for action in ["create", "update", "no_change", "skip", "blocked"] {
        assert!(text.contains(action), "missing {action} in {text}");
        assert!(table.contains(action), "missing {action} in {table}");
    }
}

#[test]
fn secret_looking_assignments_are_redacted_in_every_format() {
    let mut row = ResourceV2DryRunRow::new(
        "dashboard.grafana.app/v2",
        "ns",
        "secret-check",
        "dashboards/secret.json",
        ResourceV2DryRunAction::Blocked,
    );
    row.ownership = Some("metadata.labels.owner=platform".to_string());
    row.blocker = Some("metadata.annotations.apiToken=hunter2".to_string());
    row.changed_fields = vec!["metadata.annotations.password=also-secret".to_string()];
    let columns = vec![
        "ownership".to_string(),
        "blocker".to_string(),
        "changedFields".to_string(),
    ];

    let json = render_resource_v2_dry_run_json(&[row.clone()]).unwrap();
    let text = render_resource_v2_dry_run_text(&[row.clone()]);
    let table = render_resource_v2_dry_run_table(&[row], true, Some(&columns)).join("\n");

    for rendered in [&json, &text, &table] {
        assert!(!rendered.contains("hunter2"), "secret leaked: {rendered}");
        assert!(
            !rendered.contains("also-secret"),
            "secret leaked: {rendered}"
        );
        assert!(rendered.contains("<redacted>"));
        assert!(rendered.contains("owner=platform"));
    }
}
