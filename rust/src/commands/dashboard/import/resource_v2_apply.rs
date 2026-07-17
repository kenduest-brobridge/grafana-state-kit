#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResourceV2ApplyAction {
    Create,
    Update,
    NoOp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResourceV2ApplyOperation<P> {
    pub(crate) identity: String,
    pub(crate) action: ResourceV2ApplyAction,
    pub(crate) payload: P,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResourceV2ApplySummary {
    pub(crate) identity: String,
    pub(crate) action: ResourceV2ApplyAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResourceV2ApplyReport {
    pub(crate) completed: Vec<ResourceV2ApplySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResourceV2PartialApplyError<E> {
    pub(crate) completed: Vec<ResourceV2ApplySummary>,
    pub(crate) failed: ResourceV2ApplySummary,
    pub(crate) not_attempted: Vec<ResourceV2ApplySummary>,
    pub(crate) write_error: E,
}

impl<P> ResourceV2ApplyOperation<P> {
    fn summary(&self) -> ResourceV2ApplySummary {
        ResourceV2ApplySummary {
            identity: self.identity.clone(),
            action: self.action,
        }
    }
}

pub(crate) fn execute_resource_v2_apply<P, E, I, F>(
    operations: I,
    mut write: F,
) -> Result<ResourceV2ApplyReport, ResourceV2PartialApplyError<E>>
where
    I: IntoIterator<Item = ResourceV2ApplyOperation<P>>,
    F: FnMut(&ResourceV2ApplyOperation<P>) -> Result<(), E>,
{
    let operations = operations.into_iter().collect::<Vec<_>>();
    let mut completed = Vec::with_capacity(operations.len());

    for (index, operation) in operations.iter().enumerate() {
        if operation.action != ResourceV2ApplyAction::NoOp {
            if let Err(write_error) = write(operation) {
                return Err(ResourceV2PartialApplyError {
                    completed,
                    failed: operation.summary(),
                    not_attempted: operations[index + 1..]
                        .iter()
                        .map(ResourceV2ApplyOperation::summary)
                        .collect(),
                    write_error,
                });
            }
        }
        completed.push(operation.summary());
    }

    Ok(ResourceV2ApplyReport { completed })
}
