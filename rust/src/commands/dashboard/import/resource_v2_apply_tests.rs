use super::resource_v2_apply::{
    execute_resource_v2_apply, ResourceV2ApplyAction, ResourceV2ApplyOperation,
    ResourceV2ApplySummary,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct HttpWriteError {
    status: u16,
    message: String,
}

fn operation(identity: &str, action: ResourceV2ApplyAction) -> ResourceV2ApplyOperation<String> {
    ResourceV2ApplyOperation {
        identity: identity.to_string(),
        action,
        payload: format!("payload-{identity}"),
    }
}

fn summary(identity: &str, action: ResourceV2ApplyAction) -> ResourceV2ApplySummary {
    ResourceV2ApplySummary {
        identity: identity.to_string(),
        action,
    }
}

#[test]
fn applies_create_and_update_operations_in_plan_order() {
    let operations = vec![
        operation("new-dashboard", ResourceV2ApplyAction::Create),
        operation("existing-dashboard", ResourceV2ApplyAction::Update),
    ];
    let mut writes = Vec::new();

    let report = execute_resource_v2_apply(operations, |operation| {
        writes.push((operation.identity.clone(), operation.action));
        Ok::<_, HttpWriteError>(())
    })
    .expect("both writes should succeed");

    assert_eq!(
        writes,
        vec![
            ("new-dashboard".to_string(), ResourceV2ApplyAction::Create),
            (
                "existing-dashboard".to_string(),
                ResourceV2ApplyAction::Update,
            ),
        ]
    );
    assert_eq!(
        report.completed,
        vec![
            summary("new-dashboard", ResourceV2ApplyAction::Create),
            summary("existing-dashboard", ResourceV2ApplyAction::Update),
        ]
    );
}

#[test]
fn records_no_op_as_completed_without_issuing_a_write() {
    let operations = vec![operation("unchanged", ResourceV2ApplyAction::NoOp)];
    let mut write_count = 0;

    let report = execute_resource_v2_apply(operations, |_| {
        write_count += 1;
        Ok::<_, HttpWriteError>(())
    })
    .expect("a no-op plan should succeed");

    assert_eq!(write_count, 0);
    assert_eq!(
        report.completed,
        vec![summary("unchanged", ResourceV2ApplyAction::NoOp)]
    );
}

#[test]
fn failure_at_n_reports_completed_failed_and_not_attempted_operations() {
    let operations = vec![
        operation("created", ResourceV2ApplyAction::Create),
        operation("conflicting", ResourceV2ApplyAction::Update),
        operation("later-create", ResourceV2ApplyAction::Create),
        operation("later-no-op", ResourceV2ApplyAction::NoOp),
    ];

    let failure = execute_resource_v2_apply(operations, |operation| {
        if operation.identity == "conflicting" {
            Err(HttpWriteError {
                status: 500,
                message: "write failed".to_string(),
            })
        } else {
            Ok(())
        }
    })
    .expect_err("the second write should fail");

    assert_eq!(
        failure.completed,
        vec![summary("created", ResourceV2ApplyAction::Create)]
    );
    assert_eq!(
        failure.failed,
        summary("conflicting", ResourceV2ApplyAction::Update)
    );
    assert_eq!(
        failure.not_attempted,
        vec![
            summary("later-create", ResourceV2ApplyAction::Create),
            summary("later-no-op", ResourceV2ApplyAction::NoOp),
        ]
    );
}

#[test]
fn preserves_conflict_status_and_message_without_claiming_rollback() {
    let conflict = HttpWriteError {
        status: 409,
        message: "resourceVersion conflict".to_string(),
    };

    let failure = execute_resource_v2_apply(
        vec![operation("conflicting", ResourceV2ApplyAction::Update)],
        |_| Err(conflict.clone()),
    )
    .expect_err("the conflict should be returned");

    assert_eq!(failure.write_error.status, 409);
    assert_eq!(failure.write_error.message, "resourceVersion conflict");
    assert!(failure.completed.is_empty());
}

#[test]
fn does_not_issue_any_writes_after_the_first_failure() {
    let operations = vec![
        operation("first", ResourceV2ApplyAction::Create),
        operation("fails", ResourceV2ApplyAction::Update),
        operation("must-not-run", ResourceV2ApplyAction::Create),
    ];
    let mut attempted = Vec::new();

    let failure = execute_resource_v2_apply(operations, |operation| {
        attempted.push(operation.identity.clone());
        if operation.identity == "fails" {
            Err(HttpWriteError {
                status: 503,
                message: "unavailable".to_string(),
            })
        } else {
            Ok(())
        }
    })
    .expect_err("the second operation should stop execution");

    assert_eq!(attempted, vec!["first", "fails"]);
    assert_eq!(
        failure.not_attempted,
        vec![summary("must-not-run", ResourceV2ApplyAction::Create)]
    );
}
