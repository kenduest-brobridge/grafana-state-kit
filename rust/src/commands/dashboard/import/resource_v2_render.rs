use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ResourceV2DryRunAction {
    Create,
    Update,
    NoChange,
    Skip,
    Blocked,
}

impl ResourceV2DryRunAction {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::NoChange => "no_change",
            Self::Skip => "skip",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResourceV2DryRunRow {
    pub(crate) api_version: String,
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) source_path: String,
    pub(crate) action: ResourceV2DryRunAction,
    pub(crate) folder: Option<String>,
    pub(crate) ownership: Option<String>,
    pub(crate) blocker: Option<String>,
    pub(crate) changed_fields: Vec<String>,
}

impl ResourceV2DryRunRow {
    pub(crate) fn new(
        api_version: impl Into<String>,
        namespace: impl Into<String>,
        name: impl Into<String>,
        source_path: impl Into<String>,
        action: ResourceV2DryRunAction,
    ) -> Self {
        Self {
            api_version: api_version.into(),
            namespace: namespace.into(),
            name: name.into(),
            source_path: source_path.into(),
            action,
            folder: None,
            ownership: None,
            blocker: None,
            changed_fields: Vec::new(),
        }
    }

    fn sanitized(&self) -> Self {
        Self {
            api_version: redact_resource_v2_text(&self.api_version),
            namespace: redact_resource_v2_text(&self.namespace),
            name: redact_resource_v2_text(&self.name),
            source_path: redact_resource_v2_text(&self.source_path),
            action: self.action,
            folder: self.folder.as_deref().map(redact_resource_v2_text),
            ownership: self.ownership.as_deref().map(redact_resource_v2_text),
            blocker: self.blocker.as_deref().map(redact_resource_v2_text),
            changed_fields: self
                .changed_fields
                .iter()
                .map(|field| redact_resource_v2_text(field))
                .collect(),
        }
    }
}

pub(crate) fn render_resource_v2_dry_run_json(
    rows: &[ResourceV2DryRunRow],
) -> Result<String, serde_json::Error> {
    let dashboards = rows
        .iter()
        .map(ResourceV2DryRunRow::sanitized)
        .collect::<Vec<_>>();
    let count = |action| dashboards.iter().filter(|row| row.action == action).count();
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Summary {
        total: usize,
        create: usize,
        update: usize,
        no_change: usize,
        skip: usize,
        blocked: usize,
    }
    #[derive(Serialize)]
    struct Document {
        kind: &'static str,
        mode: &'static str,
        dashboards: Vec<ResourceV2DryRunRow>,
        summary: Summary,
    }
    let document = Document {
        kind: "grafana-util-dashboard-resource-v2-import-dry-run",
        mode: "resource-v2",
        summary: Summary {
            total: dashboards.len(),
            create: count(ResourceV2DryRunAction::Create),
            update: count(ResourceV2DryRunAction::Update),
            no_change: count(ResourceV2DryRunAction::NoChange),
            skip: count(ResourceV2DryRunAction::Skip),
            blocked: count(ResourceV2DryRunAction::Blocked),
        },
        dashboards,
    };
    let mut rendered = serde_json::to_string_pretty(&document)?;
    rendered.push('\n');
    Ok(rendered)
}

pub(crate) fn render_resource_v2_dry_run_text(rows: &[ResourceV2DryRunRow]) -> String {
    rows.iter()
        .map(ResourceV2DryRunRow::sanitized)
        .map(|row| {
            let mut fields = vec![
                row.action.as_str().to_string(),
                format!("{}/{}", row.namespace, row.name),
                format!("apiVersion={}", row.api_version),
            ];
            if let Some(folder) = row.folder {
                fields.push(format!("folder={folder}"));
            }
            if let Some(ownership) = row.ownership {
                fields.push(format!("ownership={ownership}"));
            }
            if let Some(blocker) = row.blocker {
                fields.push(format!("blocker={blocker}"));
            }
            if !row.changed_fields.is_empty() {
                fields.push(format!("changedFields={}", row.changed_fields.join(",")));
            }
            fields.push(format!("sourcePath={}", row.source_path));
            fields.join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputColumn {
    ApiVersion,
    Namespace,
    Name,
    SourcePath,
    Action,
    Folder,
    Ownership,
    Blocker,
    ChangedFields,
}

impl OutputColumn {
    const ALL: [Self; 9] = [
        Self::ApiVersion,
        Self::Namespace,
        Self::Name,
        Self::SourcePath,
        Self::Action,
        Self::Folder,
        Self::Ownership,
        Self::Blocker,
        Self::ChangedFields,
    ];

    const fn header(self) -> &'static str {
        match self {
            Self::ApiVersion => "API_VERSION",
            Self::Namespace => "NAMESPACE",
            Self::Name => "NAME",
            Self::SourcePath => "SOURCE_PATH",
            Self::Action => "ACTION",
            Self::Folder => "FOLDER",
            Self::Ownership => "OWNERSHIP",
            Self::Blocker => "BLOCKER",
            Self::ChangedFields => "CHANGED_FIELDS",
        }
    }

    fn value(self, row: &ResourceV2DryRunRow) -> String {
        match self {
            Self::ApiVersion => row.api_version.clone(),
            Self::Namespace => row.namespace.clone(),
            Self::Name => row.name.clone(),
            Self::SourcePath => row.source_path.clone(),
            Self::Action => row.action.as_str().to_string(),
            Self::Folder => row.folder.clone().unwrap_or_default(),
            Self::Ownership => row.ownership.clone().unwrap_or_default(),
            Self::Blocker => row.blocker.clone().unwrap_or_default(),
            Self::ChangedFields => row.changed_fields.join(","),
        }
    }
}

pub(crate) fn render_resource_v2_dry_run_table(
    rows: &[ResourceV2DryRunRow],
    include_header: bool,
    selected_columns: Option<&[String]>,
) -> Vec<String> {
    let columns = resolve_columns(selected_columns);
    if columns.is_empty() {
        return Vec::new();
    }
    let rows = rows
        .iter()
        .map(ResourceV2DryRunRow::sanitized)
        .collect::<Vec<_>>();
    let values = rows
        .iter()
        .map(|row| {
            columns
                .iter()
                .map(|column| column.value(row))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let widths = columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            values
                .iter()
                .map(|row| row[index].len())
                .max()
                .unwrap_or_default()
                .max(column.header().len())
        })
        .collect::<Vec<_>>();
    let format_row = |row: &[String]| {
        row.iter()
            .enumerate()
            .map(|(index, value)| format!("{value:<width$}", width = widths[index]))
            .collect::<Vec<_>>()
            .join("  ")
    };
    let mut lines = Vec::new();
    if include_header {
        lines.push(format_row(
            &columns
                .iter()
                .map(|column| column.header().to_string())
                .collect::<Vec<_>>(),
        ));
        lines.push(format_row(
            &widths
                .iter()
                .map(|width| "-".repeat(*width))
                .collect::<Vec<_>>(),
        ));
    }
    lines.extend(values.iter().map(|row| format_row(row)));
    lines
}

fn resolve_columns(selected_columns: Option<&[String]>) -> Vec<OutputColumn> {
    let Some(selected_columns) = selected_columns else {
        return OutputColumn::ALL.to_vec();
    };
    if selected_columns
        .iter()
        .any(|column| normalize_column(column) == "all")
    {
        return OutputColumn::ALL.to_vec();
    }
    selected_columns
        .iter()
        .filter_map(|column| match normalize_column(column).as_str() {
            "apiversion" => Some(OutputColumn::ApiVersion),
            "namespace" => Some(OutputColumn::Namespace),
            "name" => Some(OutputColumn::Name),
            "sourcepath" | "source" | "file" => Some(OutputColumn::SourcePath),
            "action" => Some(OutputColumn::Action),
            "folder" => Some(OutputColumn::Folder),
            "ownership" | "owner" => Some(OutputColumn::Ownership),
            "blocker" | "reason" => Some(OutputColumn::Blocker),
            "changedfields" | "changes" => Some(OutputColumn::ChangedFields),
            _ => None,
        })
        .collect()
}

fn normalize_column(column: &str) -> String {
    column
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub(crate) fn redact_resource_v2_text(value: &str) -> String {
    let value = redact_bearer_tokens(value);
    let mut rendered = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut search = 0usize;
    while let Some((operator, operator_length)) = next_assignment_operator(&value, search) {
        let key_start = value[..operator]
            .char_indices()
            .rev()
            .find(|(_, character)| {
                character.is_whitespace() || matches!(character, ',' | ';' | '{' | '[' | '(')
            })
            .map_or(0, |(index, character)| index + character.len_utf8());
        if !is_secret_key(&value[key_start..operator]) {
            search = operator + operator_length;
            continue;
        }
        rendered.push_str(&value[cursor..operator + operator_length]);
        let mut value_start = operator + operator_length;
        while value[value_start..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            let character = value[value_start..]
                .chars()
                .next()
                .expect("character exists");
            rendered.push(character);
            value_start += character.len_utf8();
        }
        let quote = value[value_start..]
            .chars()
            .next()
            .filter(|character| matches!(character, '\'' | '"'));
        if let Some(quote) = quote {
            rendered.push(quote);
            value_start += quote.len_utf8();
        }
        rendered.push_str("<redacted>");
        let value_end = value[value_start..]
            .char_indices()
            .find(|(_, character)| {
                quote.map_or_else(
                    || character.is_whitespace() || matches!(character, ',' | ';' | ']' | '}'),
                    |quote| *character == quote,
                )
            })
            .map_or(value.len(), |(index, _)| value_start + index);
        if quote.is_some() && value_end < value.len() {
            rendered.push(value[value_end..].chars().next().expect("quote exists"));
            cursor = value_end
                + value[value_end..]
                    .chars()
                    .next()
                    .expect("quote exists")
                    .len_utf8();
        } else {
            cursor = value_end;
        }
        search = cursor;
    }
    rendered.push_str(&value[cursor..]);
    rendered
}

fn next_assignment_operator(value: &str, search: usize) -> Option<(usize, usize)> {
    value[search..]
        .char_indices()
        .find(|(_, character)| matches!(character, '=' | ':'))
        .map(|(index, character)| (search + index, character.len_utf8()))
}

fn redact_bearer_tokens(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    let mut rendered = String::with_capacity(value.len());
    let mut cursor = 0;
    while let Some(relative) = lower[cursor..].find("authorization: bearer ") {
        let token_start = cursor + relative + "authorization: bearer ".len();
        rendered.push_str(&value[cursor..token_start]);
        rendered.push_str("<redacted>");
        let token_end = value[token_start..]
            .char_indices()
            .find(|(_, character)| character.is_whitespace() || matches!(character, ',' | ';'))
            .map_or(value.len(), |(index, _)| token_start + index);
        cursor = token_end;
    }
    rendered.push_str(&value[cursor..]);
    rendered
}

fn is_secret_key(key: &str) -> bool {
    let normalized = key
        .trim_matches(|character: char| !character.is_ascii_alphanumeric())
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    [
        "password",
        "passwd",
        "token",
        "secret",
        "apikey",
        "authorization",
        "credential",
        "privatekey",
        "cookie",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

#[cfg(test)]
#[path = "resource_v2_render_tests.rs"]
mod tests;
