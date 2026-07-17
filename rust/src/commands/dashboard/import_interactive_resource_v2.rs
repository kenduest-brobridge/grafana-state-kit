use std::path::PathBuf;
use std::sync::OnceLock;

use regex::Regex;

#[cfg(feature = "tui")]
use crate::common::{message, Result};
#[cfg(feature = "tui")]
use crossterm::event::{self, Event, KeyCode};
#[cfg(feature = "tui")]
use ratatui::layout::{Constraint, Direction, Layout};
#[cfg(feature = "tui")]
use ratatui::style::{Color, Modifier, Style};
#[cfg(feature = "tui")]
use ratatui::text::Line;
#[cfg(feature = "tui")]
use ratatui::widgets::{List, ListItem, Paragraph, Wrap};
#[cfg(feature = "tui")]
use std::io::{self, IsTerminal};
#[cfg(feature = "tui")]
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResourceV2ReviewRow {
    pub(crate) source_path: PathBuf,
    pub(crate) namespace: String,
    pub(crate) api_version: String,
    pub(crate) name: String,
    pub(crate) folder: Option<String>,
    pub(crate) ownership: ResourceV2Ownership,
    pub(crate) action: ResourceV2ReviewAction,
    pub(crate) blocker: Option<String>,
    pub(crate) changed_fields: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ResourceV2Ownership {
    ApiManaged,
    FileProvisioned,
    GitSyncManaged,
    ManagedUnknown,
    Unknown(String),
}

impl ResourceV2Ownership {
    pub(crate) fn label(&self) -> &str {
        match self {
            Self::ApiManaged => "api-managed",
            Self::FileProvisioned => "file-provisioned",
            Self::GitSyncManaged => "git-sync-managed",
            Self::ManagedUnknown => "managed-unknown",
            Self::Unknown(label) => label,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResourceV2ReviewAction {
    Create,
    Update,
    NoChange,
    Skip,
    Blocked,
}

impl ResourceV2ReviewAction {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::NoChange => "no-change",
            Self::Skip => "skip",
            Self::Blocked => "blocked",
        }
    }

    fn is_executable(self) -> bool {
        matches!(self, Self::Create | Self::Update)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResourceV2InteractiveItem {
    pub(crate) source_path: PathBuf,
    pub(crate) namespace: String,
    pub(crate) api_version: String,
    pub(crate) name: String,
    pub(crate) folder: Option<String>,
    pub(crate) ownership: ResourceV2Ownership,
    pub(crate) action: ResourceV2ReviewAction,
    pub(crate) blocker: Option<String>,
    pub(crate) changed_fields: Vec<String>,
}

impl From<ResourceV2ReviewRow> for ResourceV2InteractiveItem {
    fn from(row: ResourceV2ReviewRow) -> Self {
        Self {
            source_path: row.source_path,
            namespace: row.namespace,
            api_version: row.api_version,
            name: row.name,
            folder: row.folder,
            ownership: row.ownership,
            action: row.action,
            blocker: row.blocker.map(|value| redact_review_text(&value)),
            changed_fields: row
                .changed_fields
                .into_iter()
                .map(|value| redact_review_text(&value))
                .collect(),
        }
    }
}

impl ResourceV2InteractiveItem {
    pub(crate) fn list_line(&self) -> String {
        format!(
            "{:8} {:18} {}/{}",
            self.action.label(),
            self.ownership.label(),
            redact_review_text(&self.namespace),
            redact_review_text(&self.name)
        )
    }

    pub(crate) fn detail_lines(&self) -> Vec<String> {
        let folder = self.folder.as_deref().unwrap_or("-");
        let blocker = self.blocker.as_deref().unwrap_or("-");
        let changed_fields = if self.changed_fields.is_empty() {
            "-".to_string()
        } else {
            self.changed_fields.join(", ")
        };
        vec![
            format!(
                "Source: {}",
                redact_review_text(&self.source_path.display().to_string())
            ),
            format!("Namespace: {}", redact_review_text(&self.namespace)),
            format!("API version: {}", redact_review_text(&self.api_version)),
            format!("Name: {}", redact_review_text(&self.name)),
            format!("Folder: {}", redact_review_text(folder)),
            format!("Ownership: {}", self.ownership.label()),
            format!("Action: {}", self.action.label()),
            format!("Blocker: {blocker}"),
            format!("Changed fields: {changed_fields}"),
        ]
    }

    fn matches(&self, query: &str) -> bool {
        let haystack = format!(
            "{} {} {} {} {} {} {} {} {}",
            self.source_path.display(),
            self.namespace,
            self.api_version,
            self.name,
            self.folder.as_deref().unwrap_or_default(),
            self.ownership.label(),
            self.action.label(),
            self.blocker.as_deref().unwrap_or_default(),
            self.changed_fields.join(" ")
        )
        .to_lowercase();
        haystack.contains(query)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ResourceV2InteractiveAction {
    Continue,
    Confirm(Vec<PathBuf>),
    Cancel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum InteractiveModeDecision {
    Launch,
    UseNonInteractive { reason: String },
}

pub(crate) fn decide_interactive_mode(
    requested: bool,
    stdin_is_tty: bool,
    stdout_is_tty: bool,
) -> InteractiveModeDecision {
    if !requested {
        return InteractiveModeDecision::UseNonInteractive {
            reason: "interactive review was not requested".to_string(),
        };
    }
    if !stdin_is_tty || !stdout_is_tty {
        return InteractiveModeDecision::UseNonInteractive {
            reason: "resource-v2 interactive review requires stdin and stdout TTYs".to_string(),
        };
    }
    InteractiveModeDecision::Launch
}

pub(crate) struct ResourceV2InteractiveState {
    items: Vec<ResourceV2InteractiveItem>,
    focused: Option<usize>,
    last_query: Option<String>,
    detail_scroll: usize,
    detail_line_count: usize,
    detail_viewport_height: usize,
    confirmation_pending: bool,
    dry_run: bool,
    status: String,
}

impl ResourceV2InteractiveState {
    #[cfg(test)]
    pub(crate) fn new(rows: Vec<ResourceV2ReviewRow>) -> Self {
        Self::new_for_mode(rows, false)
    }

    pub(crate) fn new_for_mode(rows: Vec<ResourceV2ReviewRow>, dry_run: bool) -> Self {
        let items: Vec<_> = rows.into_iter().map(Into::into).collect();
        Self {
            focused: (!items.is_empty()).then_some(0),
            items,
            last_query: None,
            detail_scroll: 0,
            detail_line_count: 0,
            detail_viewport_height: 0,
            confirmation_pending: false,
            dry_run,
            status: "Review resource-v2 import actions before confirmation.".to_string(),
        }
    }

    pub(crate) fn focused_item(&self) -> Option<&ResourceV2InteractiveItem> {
        self.focused.and_then(|index| self.items.get(index))
    }

    pub(crate) fn status_text(&self) -> &str {
        &self.status
    }

    #[cfg(feature = "tui")]
    fn move_focus(&mut self, delta: isize) {
        if self.items.is_empty() {
            return;
        }
        let current = self.focused.unwrap_or(0);
        let next = current
            .saturating_add_signed(delta)
            .min(self.items.len().saturating_sub(1));
        self.focused = Some(next);
        self.detail_scroll = 0;
        self.confirmation_pending = false;
        self.status = format!("Focused {}.", redact_review_text(&self.items[next].name));
    }

    pub(crate) fn search(&mut self, query: &str) -> bool {
        let normalized = query.trim().to_lowercase();
        self.last_query = (!normalized.is_empty()).then_some(normalized.clone());
        let found = self.items.iter().position(|item| item.matches(&normalized));
        self.apply_search_result(found, &normalized)
    }

    pub(crate) fn repeat_search(&mut self, direction: SearchDirection) -> bool {
        let Some(query) = self.last_query.clone() else {
            self.status = "Start a search with / before repeating it.".to_string();
            return false;
        };
        if self.items.is_empty() {
            return false;
        }
        let current = self.focused.unwrap_or(0);
        let found = match direction {
            SearchDirection::Forward => (1..=self.items.len())
                .map(|offset| (current + offset) % self.items.len())
                .find(|index| self.items[*index].matches(&query)),
            SearchDirection::Backward => (1..=self.items.len())
                .map(|offset| (current + self.items.len() - offset) % self.items.len())
                .find(|index| self.items[*index].matches(&query)),
        };
        self.apply_search_result(found, &query)
    }

    fn apply_search_result(&mut self, found: Option<usize>, query: &str) -> bool {
        let Some(index) = found else {
            self.status = format!(
                "No resource-v2 review row matches {:?}.",
                redact_review_text(query)
            );
            return false;
        };
        self.focused = Some(index);
        self.detail_scroll = 0;
        self.status = format!("Focused {}.", redact_review_text(&self.items[index].name));
        true
    }

    pub(crate) fn set_detail_layout(&mut self, line_count: usize, viewport_height: usize) {
        self.detail_line_count = line_count;
        self.detail_viewport_height = viewport_height;
        self.clamp_detail_scroll();
    }

    pub(crate) fn scroll_detail(&mut self, delta: isize) {
        let next = self.detail_scroll.saturating_add_signed(delta);
        self.detail_scroll = next;
        self.clamp_detail_scroll();
    }

    pub(crate) fn detail_scroll(&self) -> usize {
        self.detail_scroll
    }

    fn clamp_detail_scroll(&mut self) {
        let max_scroll = self
            .detail_line_count
            .saturating_sub(self.detail_viewport_height);
        self.detail_scroll = self.detail_scroll.min(max_scroll);
    }

    pub(crate) fn footer_hints(&self) -> &'static str {
        if self.confirmation_pending {
            "y confirm  n/Esc cancel"
        } else {
            "/ search  n/N repeat  j/k detail  Enter confirm  Esc cancel"
        }
    }

    pub(crate) fn request_confirmation(&mut self) -> ResourceV2InteractiveAction {
        if self.confirmable_paths().is_empty() {
            self.status = "No resource-v2 actions are available to confirm.".to_string();
            return ResourceV2InteractiveAction::Continue;
        }
        self.confirmation_pending = true;
        self.status = "Confirm the reviewed create and update actions.".to_string();
        ResourceV2InteractiveAction::Continue
    }

    pub(crate) fn confirm(&mut self) -> ResourceV2InteractiveAction {
        if !self.confirmation_pending {
            self.status = "Request confirmation before applying resource-v2 actions.".to_string();
            return ResourceV2InteractiveAction::Continue;
        }
        ResourceV2InteractiveAction::Confirm(self.confirmable_paths())
    }

    pub(crate) fn cancel(&mut self) -> ResourceV2InteractiveAction {
        self.confirmation_pending = false;
        self.status = "Resource-v2 interactive review cancelled.".to_string();
        ResourceV2InteractiveAction::Cancel
    }

    fn confirmable_paths(&self) -> Vec<PathBuf> {
        self.items
            .iter()
            .filter(|item| self.dry_run || item.action.is_executable())
            .map(|item| item.source_path.clone())
            .collect()
    }
}

pub(crate) fn handle_confirmation_choice(
    state: &mut ResourceV2InteractiveState,
    confirmed: bool,
) -> ResourceV2InteractiveAction {
    if confirmed {
        state.confirm()
    } else {
        state.cancel()
    }
}

#[cfg(feature = "tui")]
pub(crate) fn run_resource_v2_interactive_review(
    rows: Vec<ResourceV2ReviewRow>,
    dry_run: bool,
) -> Result<Option<Vec<PathBuf>>> {
    match decide_interactive_mode(true, io::stdin().is_terminal(), io::stdout().is_terminal()) {
        InteractiveModeDecision::Launch => {}
        InteractiveModeDecision::UseNonInteractive { reason } => return Err(message(reason)),
    }
    let mut session = super::browse_terminal::TerminalSession::enter()?;
    session.terminal.hide_cursor()?;
    let mut state = ResourceV2InteractiveState::new_for_mode(rows, dry_run);
    let mut search_input: Option<String> = None;

    loop {
        session.terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(8),
                    Constraint::Length(3),
                ])
                .split(area);
            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
                .split(chunks[1]);
            let header = Paragraph::new(vec![
                Line::from("Resource-v2 Dashboard Import Review"),
                Line::from(format!(
                    "Resources={}  mutation requires explicit confirmation",
                    state.items.len()
                )),
            ]);
            frame.render_widget(header, chunks[0]);

            let items = state
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let line = item.list_line();
                    let style = if state.focused == Some(index) {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::LightBlue)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(line).style(style)
                })
                .collect::<Vec<_>>();
            frame.render_widget(List::new(items), body[0]);

            let detail_lines = state
                .focused_item()
                .map(ResourceV2InteractiveItem::detail_lines)
                .unwrap_or_default();
            let viewport_height = body[1].height.saturating_sub(2) as usize;
            state.set_detail_layout(detail_lines.len(), viewport_height);
            let detail = Paragraph::new(
                detail_lines
                    .into_iter()
                    .skip(state.detail_scroll())
                    .map(Line::from)
                    .collect::<Vec<_>>(),
            )
            .wrap(Wrap { trim: false });
            frame.render_widget(detail, body[1]);

            let prompt = search_input
                .as_ref()
                .map(|query| format!("Search: /{query}"))
                .unwrap_or_else(|| state.status_text().to_string());
            frame.render_widget(
                Paragraph::new(vec![Line::from(state.footer_hints()), Line::from(prompt)]),
                chunks[2],
            );
        })?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if let Some(query) = search_input.as_mut() {
            match key.code {
                KeyCode::Esc => search_input = None,
                KeyCode::Enter => {
                    let query = search_input.take().unwrap_or_default();
                    state.search(&query);
                }
                KeyCode::Backspace => {
                    query.pop();
                }
                KeyCode::Char(character) => query.push(character),
                _ => {}
            }
            continue;
        }
        match key.code {
            KeyCode::Char('/') => search_input = Some(String::new()),
            KeyCode::Char('n') if !state.confirmation_pending => {
                state.repeat_search(SearchDirection::Forward);
            }
            KeyCode::Char('N') if !state.confirmation_pending => {
                state.repeat_search(SearchDirection::Backward);
            }
            KeyCode::Down => state.move_focus(1),
            KeyCode::Up => state.move_focus(-1),
            KeyCode::Char('j') => state.scroll_detail(1),
            KeyCode::Char('k') => state.scroll_detail(-1),
            KeyCode::Enter => {
                state.request_confirmation();
            }
            KeyCode::Char('y') if state.confirmation_pending => {
                if let ResourceV2InteractiveAction::Confirm(paths) =
                    handle_confirmation_choice(&mut state, true)
                {
                    return Ok(Some(paths));
                }
            }
            KeyCode::Char('n') | KeyCode::Esc if state.confirmation_pending => {
                if handle_confirmation_choice(&mut state, false)
                    == ResourceV2InteractiveAction::Cancel
                {
                    return Ok(None);
                }
            }
            KeyCode::Esc => return Ok(None),
            _ => {}
        }
    }
}

fn redact_review_text(value: &str) -> String {
    static AUTHORIZATION: OnceLock<Regex> = OnceLock::new();
    static KEY_VALUE: OnceLock<Regex> = OnceLock::new();
    let authorization = AUTHORIZATION.get_or_init(|| {
        Regex::new(r"(?i)(authorization\s*:\s*bearer\s+)[^\s,;]+")
            .expect("valid authorization redaction regex")
    });
    let key_value = KEY_VALUE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(password|passwd|token|access_token|refresh_token|api_key|apikey|client_secret)=([^&\s,;]+)",
        )
        .expect("valid credential redaction regex")
    });
    let redacted = authorization.replace_all(value, "${1}[REDACTED]");
    key_value
        .replace_all(&redacted, "${1}=[REDACTED]")
        .into_owned()
}
