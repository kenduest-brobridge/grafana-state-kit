#![cfg(any(feature = "tui", test))]
#![cfg_attr(test, allow(dead_code))]

#[path = "render.rs"]
mod project_status_tui_render;

use crate::project_status::{
    ProjectDomainStatus, ProjectStatus, ProjectStatusAction, ProjectStatusFinding,
    PROJECT_STATUS_BLOCKED,
};
#[cfg(any(feature = "tui", test))]
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
#[cfg(feature = "tui")]
use crossterm::execute;
#[cfg(feature = "tui")]
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
#[cfg(feature = "tui")]
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ListState;
#[cfg(feature = "tui")]
use ratatui::Terminal;
#[cfg(feature = "tui")]
use std::io::{self, Stdout};
#[cfg(feature = "tui")]
use std::time::Duration;

#[cfg(feature = "tui")]
use crate::common::Result;

pub(crate) use project_status_tui_render::render_project_status_frame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProjectStatusPane {
    Home,
    Domains,
    Details,
    Actions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchTarget {
    Domains,
    Actions,
}

impl SearchDirection {
    fn label(self) -> &'static str {
        match self {
            SearchDirection::Forward => "forward",
            SearchDirection::Backward => "backward",
        }
    }
}

impl SearchTarget {
    fn label(self) -> &'static str {
        match self {
            SearchTarget::Domains => "domains",
            SearchTarget::Actions => "actions",
        }
    }

    fn singular_label(self) -> &'static str {
        match self {
            SearchTarget::Domains => "domain",
            SearchTarget::Actions => "action",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SearchPromptState {
    pub(crate) direction: SearchDirection,
    pub(crate) target: SearchTarget,
    pub(crate) query: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SearchState {
    pub(crate) direction: SearchDirection,
    pub(crate) target: SearchTarget,
    pub(crate) query: String,
}

pub(crate) struct ProjectStatusTuiState {
    document: ProjectStatus,
    domain_state: ListState,
    action_state: ListState,
    focus: ProjectStatusPane,
    detail_scroll: u16,
    pending_search: Option<SearchPromptState>,
    last_search: Option<SearchState>,
    search_status: String,
}

#[cfg(feature = "tui")]
struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

#[cfg(feature = "tui")]
impl TerminalSession {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

#[cfg(feature = "tui")]
impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

impl ProjectStatusTuiState {
    pub(crate) fn new(document: ProjectStatus) -> Self {
        let mut domain_state = ListState::default();
        domain_state.select((!document.domains.is_empty()).then_some(0));
        let mut action_state = ListState::default();
        action_state.select((!document.next_actions.is_empty()).then_some(0));
        let mut state = Self {
            document,
            domain_state,
            action_state,
            focus: ProjectStatusPane::Home,
            detail_scroll: 0,
            pending_search: None,
            last_search: None,
            search_status: "Search idle. Use / or ? in Domains or Actions.".to_string(),
        };
        state.sync_action_selection();
        state
    }

    pub(crate) fn document(&self) -> &ProjectStatus {
        &self.document
    }

    pub(crate) fn focus(&self) -> ProjectStatusPane {
        self.focus
    }

    pub(crate) fn detail_scroll(&self) -> u16 {
        self.detail_scroll
    }

    pub(crate) fn pending_search(&self) -> Option<&SearchPromptState> {
        self.pending_search.as_ref()
    }

    pub(crate) fn search_status(&self) -> &str {
        &self.search_status
    }

    pub(crate) fn domain_state_mut(&mut self) -> &mut ListState {
        &mut self.domain_state
    }

    pub(crate) fn action_state_mut(&mut self) -> &mut ListState {
        &mut self.action_state
    }

    pub(crate) fn current_domain_index(&self) -> Option<usize> {
        self.domain_state.selected()
    }

    pub(crate) fn current_action_index(&self) -> Option<usize> {
        self.action_state.selected()
    }

    pub(crate) fn current_domain(&self) -> Option<&ProjectDomainStatus> {
        self.current_domain_index()
            .and_then(|index| self.document.domains.get(index))
    }

    pub(crate) fn current_action(&self) -> Option<&ProjectStatusAction> {
        self.current_action_index()
            .and_then(|index| self.document.next_actions.get(index))
    }

    fn search_target(&self) -> SearchTarget {
        match self.focus {
            ProjectStatusPane::Actions => SearchTarget::Actions,
            ProjectStatusPane::Home | ProjectStatusPane::Domains | ProjectStatusPane::Details => {
                SearchTarget::Domains
            }
        }
    }

    pub(crate) fn project_home_target_domain_index(&self) -> Option<usize> {
        self.document
            .domains
            .iter()
            .enumerate()
            .find_map(|(index, domain)| {
                if domain.status == PROJECT_STATUS_BLOCKED || domain.blocker_count > 0 {
                    Some(index)
                } else {
                    None
                }
            })
            .or_else(|| {
                self.document
                    .domains
                    .iter()
                    .enumerate()
                    .find_map(|(index, domain)| {
                        if !domain.next_actions.is_empty() {
                            Some(index)
                        } else {
                            None
                        }
                    })
            })
            .or_else(|| (!self.document.domains.is_empty()).then_some(0))
    }

    pub(crate) fn project_home_target_domain_label(&self) -> Option<String> {
        self.project_home_target_domain_index().and_then(|index| {
            self.document
                .domains
                .get(index)
                .map(|domain| domain.id.clone())
        })
    }

    pub(crate) fn project_home_target_action(&self) -> Option<&ProjectStatusAction> {
        let target_domain = self.project_home_target_domain_index()?;
        let domain_id = &self.document.domains.get(target_domain)?.id;
        self.document
            .next_actions
            .iter()
            .find(|action| &action.domain == domain_id)
            .or_else(|| self.document.next_actions.first())
    }

    pub(crate) fn project_home_target_action_label(&self) -> Option<String> {
        self.project_home_target_action()
            .map(|action| format!("{} -> {}", action.domain, action.action))
    }

    pub(crate) fn project_home_top_blocker_label(&self) -> Option<String> {
        let target_domain = self.project_home_target_domain_index()?;
        let domain = self.document.domains.get(target_domain)?;
        let blocker = domain.blockers.first()?;
        Some(format!(
            "{}: {}={} from {} (blockers={})",
            domain.id, blocker.kind, blocker.count, blocker.source, domain.blocker_count
        ))
    }

    pub(crate) fn home_lines(&self) -> Vec<String> {
        let overall = &self.document.overall;
        let mut lines = vec![
            "Project Home: review the top blocker, then hand off to the recommended domain and matching action.".to_string(),
            format!(
                "Overall: status={} scope={} domains={} present={} blocked={} blockers={} warnings={}",
                overall.status,
                self.document.scope,
                overall.domain_count,
                overall.present_count,
                overall.blocked_count,
                overall.blocker_count,
                overall.warning_count
            ),
            match self.project_home_target_domain_label() {
                Some(label) => format!(
                    "Recommended handoff domain: {label} | Project Home -> Domains -> Actions"
                ),
                None => "Recommended handoff domain: none | Project Home -> Domains -> Actions"
                    .to_string(),
            },
        ];

        lines.push(match self.project_home_top_blocker_label() {
            Some(label) => format!("Current top blocker: {label}"),
            None => "Current top blocker: none".to_string(),
        });

        lines.push(match self.project_home_target_action() {
            Some(action) => format!(
                "Recommended handoff action: {} -> {} reason={} | Project Home -> Domains -> Actions",
                action.domain, action.action, action.reason_code
            ),
            None => "Recommended handoff action: none | Project Home -> Domains -> Actions"
                .to_string(),
        });

        lines.push(
            "Navigation: Enter hands off from Home to the recommended domain and preselects the matching action. Path: Home -> Domains -> Actions.".to_string(),
        );
        lines.push(format!(
            "Domains: {}",
            self.document
                .domains
                .iter()
                .map(|domain| format!("{}={}", domain.id, domain.status))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
        lines
    }

    pub(crate) fn current_domain_lines(&self) -> Vec<String> {
        self.current_domain()
            .map(|domain| {
                let mut lines = vec![
                    format!("Domain: {}", domain.id),
                    format!("Scope: {}   Mode: {}", domain.scope, domain.mode),
                    format!(
                        "Status: {} reason={} primary={} blockers={} warnings={}",
                        domain.status,
                        domain.reason_code,
                        domain.primary_count,
                        domain.blocker_count,
                        domain.warning_count
                    ),
                    format!("Sources: {}", join_or_none(&domain.source_kinds)),
                    format!("Signals: {}", join_or_none(&domain.signal_keys)),
                    format!(
                        "Freshness: status={} sources={} newest={} oldest={}",
                        domain.freshness.status,
                        domain.freshness.source_count,
                        optional_age(domain.freshness.newest_age_seconds),
                        optional_age(domain.freshness.oldest_age_seconds)
                    ),
                ];
                lines.push(render_finding_block("Blockers", &domain.blockers));
                lines.push(render_finding_block("Warnings", &domain.warnings));
                if domain.next_actions.is_empty() {
                    lines.push("Next actions: none".to_string());
                } else {
                    lines.push("Next actions:".to_string());
                    lines.extend(
                        domain
                            .next_actions
                            .iter()
                            .map(|action| format!("- {action}")),
                    );
                }
                lines
            })
            .unwrap_or_else(|| vec!["No domain selected.".to_string()])
    }

    pub(crate) fn status_line(&self) -> String {
        let focus = match self.focus {
            ProjectStatusPane::Home => "Home",
            ProjectStatusPane::Domains => "Domains",
            ProjectStatusPane::Details => "Details",
            ProjectStatusPane::Actions => "Actions",
        };
        let domain = self
            .current_domain()
            .map(|domain| domain.id.as_str())
            .unwrap_or("No domain");
        let action = self
            .current_action()
            .map(|action| action.action.as_str())
            .unwrap_or("No action");
        let handoff = self
            .project_home_target_action_label()
            .or_else(|| self.project_home_target_domain_label())
            .map(|label| format!("   Home handoff: {label} | Home -> Domains -> Actions"))
            .unwrap_or_default();
        format!(
            "Focus {focus}{handoff}   Domain {}/{}: {}   Action {}/{}: {}   Search: {}",
            self.domain_state
                .selected()
                .map(|index| index + 1)
                .unwrap_or(0),
            self.document.domains.len(),
            domain,
            self.action_state
                .selected()
                .map(|index| index + 1)
                .unwrap_or(0),
            self.document.next_actions.len(),
            action,
            self.search_summary()
        )
    }

    pub(crate) fn search_summary(&self) -> String {
        if let Some(search) = self.pending_search.as_ref() {
            let prefix = match search.direction {
                SearchDirection::Forward => "/",
                SearchDirection::Backward => "?",
            };
            format!("prompt {prefix}{} {}", search.query, search.target.label())
        } else if let Some(search) = self.last_search.as_ref() {
            let prefix = match search.direction {
                SearchDirection::Forward => "/",
                SearchDirection::Backward => "?",
            };
            format!("last {prefix}{} {}", search.query, search.target.label())
        } else {
            "idle".to_string()
        }
    }

    pub(crate) fn start_search(&mut self, direction: SearchDirection) {
        let target = self.search_target();
        self.pending_search = Some(SearchPromptState {
            direction,
            target,
            query: String::new(),
        });
        self.search_status = format!("Search {} in {}.", direction.label(), target.label());
    }

    pub(crate) fn cancel_search(&mut self) {
        self.pending_search = None;
        self.search_status = "Cancelled status search.".to_string();
    }

    pub(crate) fn handle_search_key(&mut self, key: KeyCode) {
        let Some(mut search) = self.pending_search.take() else {
            return;
        };
        match key {
            KeyCode::Esc => self.cancel_search(),
            KeyCode::Enter => {
                let query = search.query.trim().to_string();
                if query.is_empty() {
                    self.search_status = "Search query is empty.".to_string();
                } else if let Some(index) = self.find_match(&query, search.direction, search.target)
                {
                    self.select_search_match(search.target, index);
                    self.last_search = Some(SearchState {
                        direction: search.direction,
                        target: search.target,
                        query: query.clone(),
                    });
                    self.search_status = format!(
                        "Matched '{query}' at {} {} of {}.",
                        search.target.singular_label(),
                        index + 1,
                        self.search_target_len(search.target)
                    );
                } else {
                    self.last_search = Some(SearchState {
                        direction: search.direction,
                        target: search.target,
                        query: query.clone(),
                    });
                    self.search_status = format!("No {} matched '{query}'.", search.target.label());
                }
            }
            KeyCode::Backspace => {
                search.query.pop();
                self.pending_search = Some(search);
            }
            KeyCode::Char(ch) => {
                search.query.push(ch);
                self.pending_search = Some(search);
            }
            _ => {
                self.pending_search = Some(search);
            }
        }
    }

    pub(crate) fn repeat_search(&mut self) {
        let Some(search) = self.last_search.clone() else {
            self.search_status = "No previous status search. Use / or ? first.".to_string();
            return;
        };
        if let Some(index) = self.repeat_last_search() {
            self.select_search_match(search.target, index);
            self.search_status = format!(
                "Next match for '{}' at {} {} of {}.",
                search.query,
                search.target.singular_label(),
                index + 1,
                self.search_target_len(search.target)
            );
        } else {
            self.search_status = format!(
                "No more {} matches for '{}'.",
                search.target.label(),
                search.query
            );
        }
    }

    fn find_match(
        &self,
        query: &str,
        direction: SearchDirection,
        target: SearchTarget,
    ) -> Option<usize> {
        let anchor = match (target, direction) {
            (SearchTarget::Domains, SearchDirection::Forward) => {
                self.domain_state.selected().unwrap_or(0)
            }
            (SearchTarget::Domains, SearchDirection::Backward) => self
                .domain_state
                .selected()
                .unwrap_or_else(|| self.document.domains.len().saturating_sub(1)),
            (SearchTarget::Actions, SearchDirection::Forward) => {
                self.action_state.selected().unwrap_or(0)
            }
            (SearchTarget::Actions, SearchDirection::Backward) => self
                .action_state
                .selected()
                .unwrap_or_else(|| self.document.next_actions.len().saturating_sub(1)),
        };
        self.find_match_from(query, direction, target, anchor, true)
    }

    fn repeat_last_search(&self) -> Option<usize> {
        let search = self.last_search.as_ref()?;
        let anchor = match search.target {
            SearchTarget::Domains => self.domain_state.selected().unwrap_or(0),
            SearchTarget::Actions => self.action_state.selected().unwrap_or(0),
        };
        self.find_match_from(
            &search.query,
            search.direction,
            search.target,
            anchor,
            false,
        )
    }

    fn find_match_from(
        &self,
        query: &str,
        direction: SearchDirection,
        target: SearchTarget,
        anchor: usize,
        include_anchor: bool,
    ) -> Option<usize> {
        let len = self.search_target_len(target);
        if len == 0 {
            return None;
        }
        let normalized = query.to_ascii_lowercase();
        if normalized.trim().is_empty() {
            return None;
        }
        for offset in 0..len {
            if offset == 0 && !include_anchor {
                continue;
            }
            let index = match direction {
                SearchDirection::Forward => (anchor + offset) % len,
                SearchDirection::Backward => (anchor + len - offset) % len,
            };
            if self.search_text(target, index).contains(&normalized) {
                return Some(index);
            }
        }
        None
    }

    fn select_search_match(&mut self, target: SearchTarget, index: usize) {
        match target {
            SearchTarget::Domains => self.select_domain(index),
            SearchTarget::Actions => {
                self.action_state.select(Some(index));
                self.detail_scroll = 0;
                if let Some(domain_index) = self.action_domain_index(index) {
                    self.domain_state.select(Some(domain_index));
                }
            }
        }
    }

    fn action_domain_index(&self, action_index: usize) -> Option<usize> {
        let domain_id = &self.document.next_actions.get(action_index)?.domain;
        self.document
            .domains
            .iter()
            .position(|domain| &domain.id == domain_id)
    }

    fn search_target_len(&self, target: SearchTarget) -> usize {
        match target {
            SearchTarget::Domains => self.document.domains.len(),
            SearchTarget::Actions => self.document.next_actions.len(),
        }
    }

    fn search_text(&self, target: SearchTarget, index: usize) -> String {
        match target {
            SearchTarget::Domains => self
                .document
                .domains
                .get(index)
                .map(domain_search_text)
                .unwrap_or_default(),
            SearchTarget::Actions => self
                .document
                .next_actions
                .get(index)
                .map(action_search_text)
                .unwrap_or_default(),
        }
    }

    pub(crate) fn focus_next(&mut self) {
        self.focus = match self.focus {
            ProjectStatusPane::Home => ProjectStatusPane::Domains,
            ProjectStatusPane::Domains => ProjectStatusPane::Details,
            ProjectStatusPane::Details => ProjectStatusPane::Actions,
            ProjectStatusPane::Actions => ProjectStatusPane::Home,
        };
    }

    pub(crate) fn focus_previous(&mut self) {
        self.focus = match self.focus {
            ProjectStatusPane::Home => ProjectStatusPane::Actions,
            ProjectStatusPane::Domains => ProjectStatusPane::Home,
            ProjectStatusPane::Details => ProjectStatusPane::Domains,
            ProjectStatusPane::Actions => ProjectStatusPane::Details,
        };
    }

    pub(crate) fn focus_home(&mut self) {
        self.focus = ProjectStatusPane::Home;
    }

    pub(crate) fn handoff_from_home(&mut self) {
        let Some(index) = self.project_home_target_domain_index() else {
            return;
        };
        self.select_domain(index);
        self.focus = ProjectStatusPane::Domains;
    }

    pub(crate) fn move_domain_selection(&mut self, delta: isize) {
        let count = self.document.domains.len();
        if count == 0 {
            self.domain_state.select(None);
            self.action_state.select(None);
            self.detail_scroll = 0;
            return;
        }
        let current = self.domain_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, count.saturating_sub(1) as isize) as usize;
        self.select_domain(next);
    }

    pub(crate) fn move_action_selection(&mut self, delta: isize) {
        let count = self.document.next_actions.len();
        if count == 0 {
            self.action_state.select(None);
            self.detail_scroll = 0;
            return;
        }
        let current = self.action_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, count.saturating_sub(1) as isize) as usize;
        self.action_state.select(Some(next));
        self.detail_scroll = 0;
    }

    pub(crate) fn move_detail_scroll(&mut self, delta: isize) {
        let max_scroll = self.current_domain_lines().len().saturating_sub(1) as u16;
        if delta.is_negative() {
            self.detail_scroll = self
                .detail_scroll
                .saturating_sub(delta.unsigned_abs() as u16);
        } else {
            self.detail_scroll = self.detail_scroll.saturating_add(delta as u16);
        }
        self.detail_scroll = self.detail_scroll.min(max_scroll);
    }

    fn select_domain(&mut self, index: usize) {
        self.domain_state.select(Some(index));
        self.sync_action_selection();
        self.detail_scroll = 0;
    }

    fn sync_action_selection(&mut self) {
        if self.document.next_actions.is_empty() {
            self.action_state.select(None);
            return;
        }
        let selected_domain = self.current_domain().map(|domain| domain.id.as_str());
        let action_index = selected_domain.and_then(|domain_id| {
            self.document
                .next_actions
                .iter()
                .position(|action| action.domain == domain_id)
        });
        self.action_state.select(Some(action_index.unwrap_or(0)));
    }
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn optional_age(value: Option<u64>) -> String {
    value
        .map(|seconds| seconds.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn render_finding_block(label: &str, findings: &[ProjectStatusFinding]) -> String {
    if findings.is_empty() {
        format!("{label}: none")
    } else {
        let rendered = findings
            .iter()
            .map(|finding| format!("{}={} from {}", finding.kind, finding.count, finding.source))
            .collect::<Vec<_>>()
            .join(" | ");
        format!("{label}: {rendered}")
    }
}

fn domain_search_text(domain: &ProjectDomainStatus) -> String {
    let blockers = domain
        .blockers
        .iter()
        .map(finding_search_text)
        .collect::<Vec<_>>()
        .join(" ");
    let warnings = domain
        .warnings
        .iter()
        .map(finding_search_text)
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "{} {} {} {} {} {} {} {} {} {} {}",
        domain.id,
        domain.scope,
        domain.mode,
        domain.status,
        domain.reason_code,
        domain.source_kinds.join(" "),
        domain.signal_keys.join(" "),
        domain.next_actions.join(" "),
        blockers,
        warnings,
        domain.freshness.status
    )
    .to_ascii_lowercase()
}

fn action_search_text(action: &ProjectStatusAction) -> String {
    format!("{} {} {}", action.domain, action.reason_code, action.action).to_ascii_lowercase()
}

fn finding_search_text(finding: &ProjectStatusFinding) -> String {
    format!("{} {} {}", finding.kind, finding.count, finding.source)
}

#[cfg(feature = "tui")]
pub(crate) fn run_project_status_interactive(document: ProjectStatus) -> Result<()> {
    let mut session = TerminalSession::enter()?;
    let mut state = ProjectStatusTuiState::new(document);

    loop {
        session
            .terminal
            .draw(|frame| render_project_status_frame(frame, &mut state))?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if state.pending_search().is_some() {
            state.handle_search_key(key.code);
            continue;
        }

        let detail_lines_len = state.current_domain_lines().len();
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Tab => state.focus_next(),
            KeyCode::BackTab => state.focus_previous(),
            KeyCode::Char('h') => state.focus_home(),
            KeyCode::Char('/') => state.start_search(SearchDirection::Forward),
            KeyCode::Char('?') => state.start_search(SearchDirection::Backward),
            KeyCode::Char('n') => state.repeat_search(),
            KeyCode::Enter if state.focus() == ProjectStatusPane::Home => {
                state.handoff_from_home();
            }
            KeyCode::Up => match state.focus() {
                ProjectStatusPane::Home => {}
                ProjectStatusPane::Domains => state.move_domain_selection(-1),
                ProjectStatusPane::Details => state.move_detail_scroll(-1),
                ProjectStatusPane::Actions => state.move_action_selection(-1),
            },
            KeyCode::Down => match state.focus() {
                ProjectStatusPane::Home => {}
                ProjectStatusPane::Domains => state.move_domain_selection(1),
                ProjectStatusPane::Details => state.move_detail_scroll(1),
                ProjectStatusPane::Actions => state.move_action_selection(1),
            },
            KeyCode::PageUp if state.focus() == ProjectStatusPane::Details => {
                state.move_detail_scroll(-10);
            }
            KeyCode::PageDown if state.focus() == ProjectStatusPane::Details => {
                state.move_detail_scroll(10);
            }
            KeyCode::Home => match state.focus() {
                ProjectStatusPane::Home => {}
                ProjectStatusPane::Domains => {
                    let current = state.current_domain_index().unwrap_or(0) as isize;
                    state.move_domain_selection(-current);
                }
                ProjectStatusPane::Details => state.detail_scroll = 0,
                ProjectStatusPane::Actions => {
                    let current = state.current_action_index().unwrap_or(0) as isize;
                    state.move_action_selection(-current);
                }
            },
            KeyCode::End => match state.focus() {
                ProjectStatusPane::Home => {}
                ProjectStatusPane::Domains => {
                    let len = state.document().domains.len();
                    if len > 0 {
                        let current = state.current_domain_index().unwrap_or(0) as isize;
                        state.move_domain_selection(len.saturating_sub(1) as isize - current);
                    }
                }
                ProjectStatusPane::Details => {
                    state.detail_scroll = detail_lines_len.saturating_sub(1) as u16;
                }
                ProjectStatusPane::Actions => {
                    let len = state.document().next_actions.len();
                    if len > 0 {
                        let current = state.current_action_index().unwrap_or(0) as isize;
                        state.move_action_selection(len.saturating_sub(1) as isize - current);
                    }
                }
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::TOOL_VERSION;
    use crate::project_status::{
        ProjectStatusFreshness, ProjectStatusOverall, PROJECT_STATUS_READY,
    };

    #[test]
    fn detail_scroll_clamps_to_current_domain_lines() {
        let mut state = ProjectStatusTuiState::new(sample_project_status());
        let max_scroll = state.current_domain_lines().len().saturating_sub(1) as u16;

        state.move_detail_scroll(999);
        assert_eq!(state.detail_scroll(), max_scroll);

        state.move_detail_scroll(-999);
        assert_eq!(state.detail_scroll(), 0);
    }

    #[test]
    fn domain_search_submit_and_repeat_wrap_selection() {
        let mut state = ProjectStatusTuiState::new(sample_project_status());

        state.start_search(SearchDirection::Forward);
        for ch in ['s', 't', 'a', 'g', 'e', 'd'] {
            state.handle_search_key(KeyCode::Char(ch));
        }
        state.handle_search_key(KeyCode::Enter);

        assert_eq!(state.pending_search(), None);
        assert_eq!(state.current_domain_index(), Some(0));
        assert_eq!(state.search_status(), "Matched 'staged' at domain 1 of 2.");

        state.repeat_search();
        assert_eq!(state.current_domain_index(), Some(1));
        assert_eq!(
            state.search_status(),
            "Next match for 'staged' at domain 2 of 2."
        );

        state.repeat_search();
        assert_eq!(state.current_domain_index(), Some(0));
        assert_eq!(
            state.search_status(),
            "Next match for 'staged' at domain 1 of 2."
        );
    }

    #[test]
    fn action_search_uses_action_focus_and_cancel_keeps_selection() {
        let mut state = ProjectStatusTuiState::new(sample_project_status());
        state.focus = ProjectStatusPane::Actions;

        state.start_search(SearchDirection::Backward);
        assert_eq!(
            state.pending_search(),
            Some(&SearchPromptState {
                direction: SearchDirection::Backward,
                target: SearchTarget::Actions,
                query: String::new(),
            })
        );

        for ch in ['s', 'y', 'n', 'c'] {
            state.handle_search_key(KeyCode::Char(ch));
        }
        state.handle_search_key(KeyCode::Esc);

        assert_eq!(state.pending_search(), None);
        assert_eq!(state.current_action_index(), Some(0));
        assert_eq!(state.search_status(), "Cancelled status search.");
    }

    #[test]
    fn action_search_submit_selects_matching_action_and_domain() {
        let mut state = ProjectStatusTuiState::new(sample_project_status());
        state.focus = ProjectStatusPane::Actions;

        state.start_search(SearchDirection::Forward);
        for ch in ['s', 'y', 'n', 'c'] {
            state.handle_search_key(KeyCode::Char(ch));
        }
        state.handle_search_key(KeyCode::Enter);

        assert_eq!(state.current_action_index(), Some(0));
        assert_eq!(state.current_domain_index(), Some(1));
        assert_eq!(state.search_status(), "Matched 'sync' at action 1 of 1.");
    }

    fn sample_project_status() -> ProjectStatus {
        ProjectStatus {
            schema_version: 1,
            tool_version: TOOL_VERSION.to_string(),
            discovery: None,
            scope: "live".to_string(),
            overall: ProjectStatusOverall {
                status: PROJECT_STATUS_READY.to_string(),
                domain_count: 2,
                present_count: 2,
                blocked_count: 1,
                blocker_count: 3,
                warning_count: 0,
                freshness: ProjectStatusFreshness {
                    status: "current".to_string(),
                    source_count: 1,
                    newest_age_seconds: Some(30),
                    oldest_age_seconds: Some(30),
                },
            },
            domains: vec![
                ProjectDomainStatus {
                    id: "dashboard".to_string(),
                    scope: "staged".to_string(),
                    mode: "inspect-summary".to_string(),
                    status: PROJECT_STATUS_READY.to_string(),
                    reason_code: PROJECT_STATUS_READY.to_string(),
                    primary_count: 4,
                    blocker_count: 0,
                    warning_count: 0,
                    source_kinds: vec!["dashboard-export".to_string()],
                    signal_keys: vec!["summary.dashboardCount".to_string()],
                    blockers: Vec::new(),
                    warnings: Vec::new(),
                    next_actions: vec!["review dashboard governance warnings".to_string()],
                    freshness: ProjectStatusFreshness {
                        status: "current".to_string(),
                        source_count: 1,
                        newest_age_seconds: Some(30),
                        oldest_age_seconds: Some(30),
                    },
                },
                ProjectDomainStatus {
                    id: "sync".to_string(),
                    scope: "staged".to_string(),
                    mode: "staged-documents".to_string(),
                    status: PROJECT_STATUS_BLOCKED.to_string(),
                    reason_code: "blocked-by-blockers".to_string(),
                    primary_count: 6,
                    blocker_count: 3,
                    warning_count: 0,
                    source_kinds: vec!["sync-summary".to_string()],
                    signal_keys: vec!["summary.syncBlockingCount".to_string()],
                    blockers: vec![crate::project_status::status_finding(
                        "sync-blocking",
                        3,
                        "summary.syncBlockingCount",
                    )],
                    warnings: Vec::new(),
                    next_actions: vec!["resolve sync workflow blockers".to_string()],
                    freshness: ProjectStatusFreshness {
                        status: "current".to_string(),
                        source_count: 1,
                        newest_age_seconds: Some(10),
                        oldest_age_seconds: Some(10),
                    },
                },
            ],
            top_blockers: Vec::new(),
            next_actions: vec![ProjectStatusAction {
                domain: "sync".to_string(),
                reason_code: "blocked-by-blockers".to_string(),
                action: "resolve sync workflow blockers".to_string(),
            }],
        }
    }
}
