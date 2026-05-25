//! Shared read-only TUI browser for list/detail artifact inspection.
#![cfg_attr(not(feature = "tui"), allow(dead_code))]
use crate::common::Result;
#[cfg(feature = "tui")]
use crate::tui_shell;

#[cfg(feature = "tui")]
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
#[cfg(feature = "tui")]
use crossterm::execute;
#[cfg(feature = "tui")]
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
#[cfg(feature = "tui")]
use ratatui::backend::CrosstermBackend;
#[cfg(feature = "tui")]
use ratatui::layout::{Constraint, Direction, Layout};
#[cfg(feature = "tui")]
use ratatui::style::{Color, Modifier, Style};
#[cfg(feature = "tui")]
use ratatui::text::{Line, Span};
#[cfg(feature = "tui")]
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
#[cfg(feature = "tui")]
use ratatui::Terminal;
#[cfg(feature = "tui")]
use std::io::{self, Stdout};
#[cfg(feature = "tui")]
use std::time::Duration;

#[cfg(any(feature = "tui", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BrowserPane {
    Items,
    Detail,
}

#[cfg(any(feature = "tui", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchDirection {
    Forward,
    Backward,
}

#[cfg(any(feature = "tui", test))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchPromptState {
    direction: SearchDirection,
    query: String,
}

#[cfg(any(feature = "tui", test))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchState {
    direction: SearchDirection,
    query: String,
    filter_kind: String,
    match_ordinal: Option<usize>,
    match_count: usize,
}

#[cfg_attr(not(feature = "tui"), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BrowserItem {
    pub(crate) kind: String,
    pub(crate) title: String,
    pub(crate) meta: String,
    pub(crate) details: Vec<String>,
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn append_browser_detail_section(
    details: &mut Vec<String>,
    heading: &str,
    lines: Vec<String>,
) {
    if lines.is_empty() {
        details.push(format!("{heading}: none"));
        return;
    }
    details.push(format!("{heading}:"));
    details.extend(lines);
}

pub(crate) fn browser_detail_fact(label: &str, value: impl std::fmt::Display) -> String {
    format!("{label}: {value}")
}

pub(crate) fn browser_detail_fallback_fact(label: &str, value: &str, fallback: &str) -> String {
    let value = value.trim();
    let value = if value.is_empty() { fallback } else { value };
    format!("{label}: {value}")
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn browser_detail_aligned_fact(label: &str, value: impl std::fmt::Display) -> String {
    format!("{label:<16}: {value}")
}

#[cfg(feature = "tui")]
pub(crate) fn browser_detail_info_lines(lines: &[String]) -> Vec<Line<'static>> {
    browser_detail_info_lines_with(lines, |_| true, |_| None)
}

#[cfg(feature = "tui")]
pub(crate) fn browser_detail_info_lines_with(
    lines: &[String],
    include_line: impl Fn(&str) -> bool,
    special_line: impl Fn(&str) -> Option<Line<'static>>,
) -> Vec<Line<'static>> {
    lines
        .iter()
        .filter(|line| !line.is_empty() && include_line(line))
        .map(|line| {
            if let Some(line) = special_line(line) {
                line
            } else if let Some((label, value)) = line.split_once(':') {
                Line::from(vec![
                    Span::styled(
                        format!("{label:<18}: "),
                        Style::default()
                            .fg(Color::LightBlue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(value.trim().to_string(), Style::default().fg(Color::White)),
                ])
            } else {
                Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .collect()
}

#[cfg(feature = "tui")]
pub(crate) fn browser_review_info_lines(lines: &[String]) -> Vec<Line<'static>> {
    lines
        .iter()
        .map(|line| {
            if let Some((label, value)) = line.split_once(':') {
                let color = if label.contains("blocker") || label.contains("required") {
                    Color::Yellow
                } else {
                    Color::LightCyan
                };
                Line::from(vec![
                    Span::styled(
                        format!("{label:<24}: "),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(value.trim().to_string(), Style::default().fg(Color::White)),
                ])
            } else {
                Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .collect()
}

#[cfg(any(feature = "tui", test))]
impl BrowserItem {
    fn matches_query(&self, query: &str) -> bool {
        let needle = query.trim().to_ascii_lowercase();
        if needle.is_empty() {
            return false;
        }
        self.kind.to_ascii_lowercase().contains(&needle)
            || self.title.to_ascii_lowercase().contains(&needle)
            || self.meta.to_ascii_lowercase().contains(&needle)
            || self
                .details
                .iter()
                .any(|line| line.to_ascii_lowercase().contains(&needle))
    }
}

#[cfg(any(feature = "tui", test))]
#[derive(Default)]
struct BrowserSearchController {
    pending: Option<SearchPromptState>,
    last: Option<SearchState>,
}

#[cfg(any(feature = "tui", test))]
impl BrowserSearchController {
    fn start(&mut self, direction: SearchDirection) {
        self.pending = Some(SearchPromptState {
            direction,
            query: String::new(),
        });
    }

    fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    fn push_char(&mut self, value: char) {
        if let Some(prompt) = self.pending.as_mut() {
            prompt.query.push(value);
        }
    }

    fn pop_char(&mut self) {
        if let Some(prompt) = self.pending.as_mut() {
            prompt.query.pop();
        }
    }

    fn cancel(&mut self) {
        self.pending = None;
    }

    fn apply(
        &mut self,
        items: &[BrowserItem],
        visible_indexes: &[usize],
        selected_visible: Option<usize>,
        filter_kind: &str,
    ) -> Option<usize> {
        let prompt = self.pending.take()?;
        let query = prompt.query.trim().to_string();
        if query.is_empty() {
            return None;
        }

        let matches = matching_visible_indexes(items, visible_indexes, &query);
        let selected = find_match_in_visible(
            items,
            visible_indexes,
            &query,
            prompt.direction,
            selected_visible,
        );
        self.last = Some(build_search_state(
            prompt.direction,
            query,
            filter_kind,
            &matches,
            selected,
        ));
        selected
    }

    fn repeat(
        &mut self,
        items: &[BrowserItem],
        visible_indexes: &[usize],
        selected_visible: Option<usize>,
        filter_kind: &str,
    ) -> Option<usize> {
        let last = self.last.as_ref()?.clone();
        let matches = matching_visible_indexes(items, visible_indexes, &last.query);
        let selected = repeat_match_in_visible(
            items,
            visible_indexes,
            &last.query,
            last.direction,
            selected_visible,
        );
        self.last = Some(build_search_state(
            last.direction,
            last.query,
            filter_kind,
            &matches,
            selected,
        ));
        selected
    }

    fn footer_label(&self) -> String {
        if let Some(prompt) = self.pending.as_ref() {
            format!(
                "Search {}{}",
                search_direction_symbol(prompt.direction),
                prompt.query
            )
        } else if let Some(last) = self.last.as_ref() {
            format!(
                "Last {}{}",
                search_direction_symbol(last.direction),
                last.query
            )
        } else {
            "Search idle".to_string()
        }
    }

    fn summary_line(&self, active_filter: &str) -> String {
        if let Some(prompt) = self.pending.as_ref() {
            return format!(
                "Search prompt {} in filter {}: \"{}\" (Enter search, Esc cancel).",
                search_direction_symbol(prompt.direction),
                active_filter,
                prompt.query
            );
        }
        if let Some(last) = self.last.as_ref() {
            return match last.match_ordinal {
                Some(ordinal) => format!(
                    "Last search {}\"{}\" in filter {} matched {}/{} results. Press n for next match.",
                    search_direction_symbol(last.direction),
                    last.query,
                    last.filter_kind,
                    ordinal,
                    last.match_count
                ),
                None => format!(
                    "Last search {}\"{}\" in filter {} matched 0 results. Press / or ? to try again.",
                    search_direction_symbol(last.direction),
                    last.query,
                    last.filter_kind
                ),
            };
        }
        "Search: / forward, ? backward, n repeat within the active filter.".to_string()
    }
}

#[cfg(any(feature = "tui", test))]
fn repeat_match_in_visible(
    items: &[BrowserItem],
    visible_indexes: &[usize],
    query: &str,
    direction: SearchDirection,
    selected_visible: Option<usize>,
) -> Option<usize> {
    if visible_indexes.is_empty() || query.trim().is_empty() {
        return None;
    }

    match direction {
        SearchDirection::Forward => {
            let start = selected_visible.map(|index| index + 1).unwrap_or(0);
            (start..visible_indexes.len())
                .find(|visible_index| {
                    items
                        .get(visible_indexes[*visible_index])
                        .is_some_and(|item| item.matches_query(query))
                })
                .or_else(|| {
                    let wrap_end = selected_visible.unwrap_or(visible_indexes.len());
                    (0..wrap_end).find(|visible_index| {
                        items
                            .get(visible_indexes[*visible_index])
                            .is_some_and(|item| item.matches_query(query))
                    })
                })
        }
        SearchDirection::Backward => {
            let start = selected_visible
                .and_then(|index| index.checked_sub(1))
                .or_else(|| visible_indexes.len().checked_sub(1))?;
            (0..=start).rev().find(|visible_index| {
                items
                    .get(visible_indexes[*visible_index])
                    .is_some_and(|item| item.matches_query(query))
            })
        }
    }
}

#[cfg(any(feature = "tui", test))]
fn search_direction_symbol(direction: SearchDirection) -> &'static str {
    match direction {
        SearchDirection::Forward => "/",
        SearchDirection::Backward => "?",
    }
}

#[cfg(any(feature = "tui", test))]
fn matching_visible_indexes(
    items: &[BrowserItem],
    visible_indexes: &[usize],
    query: &str,
) -> Vec<usize> {
    visible_indexes
        .iter()
        .enumerate()
        .filter_map(|(visible_index, item_index)| {
            items
                .get(*item_index)
                .filter(|item| item.matches_query(query))
                .map(|_| visible_index)
        })
        .collect()
}

#[cfg(any(feature = "tui", test))]
fn detail_title(
    item: &BrowserItem,
    selected_visible: usize,
    visible_total: usize,
    detail_scroll: u16,
    total_detail_lines: usize,
) -> String {
    format!(
        "Detail {}/{} [{}]  line {}/{}",
        selected_visible + 1,
        visible_total,
        item.kind,
        (detail_scroll as usize + 1).min(total_detail_lines),
        total_detail_lines
    )
}

#[cfg(any(feature = "tui", test))]
fn build_search_state(
    direction: SearchDirection,
    query: String,
    filter_kind: &str,
    matches: &[usize],
    selected: Option<usize>,
) -> SearchState {
    SearchState {
        direction,
        query,
        filter_kind: filter_kind.to_string(),
        match_ordinal: selected.and_then(|visible_index| {
            matches
                .iter()
                .position(|candidate| *candidate == visible_index)
                .map(|index| index + 1)
        }),
        match_count: matches.len(),
    }
}

#[cfg(any(feature = "tui", test))]
fn find_match_in_visible(
    items: &[BrowserItem],
    visible_indexes: &[usize],
    query: &str,
    direction: SearchDirection,
    start: Option<usize>,
) -> Option<usize> {
    if visible_indexes.is_empty() || query.trim().is_empty() {
        return None;
    }

    match direction {
        SearchDirection::Forward => {
            let start = start
                .unwrap_or(0)
                .min(visible_indexes.len().saturating_sub(1));
            (start..visible_indexes.len()).find(|visible_index| {
                items
                    .get(visible_indexes[*visible_index])
                    .is_some_and(|item| item.matches_query(query))
            })
        }
        SearchDirection::Backward => {
            let start = start.unwrap_or(visible_indexes.len().saturating_sub(1));
            (0..=start.min(visible_indexes.len().saturating_sub(1)))
                .rev()
                .find(|visible_index| {
                    items
                        .get(visible_indexes[*visible_index])
                        .is_some_and(|item| item.matches_query(query))
                })
        }
    }
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

#[cfg(feature = "tui")]
fn item_color(kind: &str) -> Color {
    match kind {
        "dashboard" => Color::Yellow,
        "alert" | "alert-rule" => Color::Red,
        "datasource" => Color::Cyan,
        "user" => Color::Green,
        "team" => Color::LightMagenta,
        "warning" => Color::Yellow,
        "violation" => Color::LightRed,
        "drift" => Color::LightRed,
        "policy" => Color::Magenta,
        _ => Color::Gray,
    }
}

#[cfg(feature = "tui")]
fn collect_kind_filters(items: &[BrowserItem]) -> Vec<String> {
    let mut filters = vec!["all".to_string()];
    for item in items {
        if !filters.iter().any(|kind| kind == &item.kind) {
            filters.push(item.kind.clone());
        }
    }
    filters
}

#[cfg(feature = "tui")]
fn visible_item_indexes(items: &[BrowserItem], filter_kind: &str) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if filter_kind == "all" || item.kind == filter_kind {
                Some(index)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(feature = "tui")]
fn selected_detail_line_count(item: Option<&BrowserItem>) -> usize {
    item.map(|candidate| candidate.details.len().max(1))
        .unwrap_or(1)
}

#[cfg(feature = "tui")]
pub(crate) fn run_interactive_browser(
    title: &str,
    summary_lines: &[String],
    items: &[BrowserItem],
) -> Result<()> {
    let mut session = TerminalSession::enter()?;
    let mut state = ListState::default();
    let kind_filters = collect_kind_filters(items);
    let mut active_filter = 0usize;
    let mut visible_indexes = visible_item_indexes(items, &kind_filters[active_filter]);
    state.select((!visible_indexes.is_empty()).then_some(0));
    let mut detail_scroll = 0u16;
    let mut pane_focus = BrowserPane::Items;
    let mut search = BrowserSearchController::default();

    loop {
        session.terminal.draw(|frame| {
            let mut runtime_summary_lines = summary_lines.to_vec();
            runtime_summary_lines.push(search.summary_line(&kind_filters[active_filter]));
            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length((runtime_summary_lines.len().max(1) + 2) as u16),
                    Constraint::Min(1),
                    Constraint::Length(4),
                ])
                .split(frame.area());
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
                .split(outer[1]);
            let selected_visible = state.selected().unwrap_or(0);
            let selected_item = visible_indexes
                .get(selected_visible)
                .and_then(|index| items.get(*index));
            let total_detail_lines = selected_detail_line_count(selected_item);
            let detail_lines = selected_item
                .map(|item| {
                    if item.details.is_empty() {
                        vec!["No detail lines.".to_string()]
                    } else {
                        item.details.clone()
                    }
                })
                .unwrap_or_else(|| vec!["No item selected".to_string()]);
            let detail_selected =
                (detail_scroll as usize).min(detail_lines.len().saturating_sub(1));

            let summary = Paragraph::new(runtime_summary_lines.join("\n"))
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).title(title));
            frame.render_widget(summary, outer[0]);

            let list = List::new(
                visible_indexes
                    .iter()
                    .enumerate()
                    .map(|(visible_index, item_index)| {
                        let item = &items[*item_index];
                        let line = Line::from(vec![
                            Span::styled(
                                format!("{:>2}. ", visible_index + 1),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                format!("[{}]", item.kind.to_uppercase()),
                                Style::default()
                                    .fg(item_color(&item.kind))
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(format!(" {}", item.title)),
                            Span::styled(
                                format!("  {}", item.meta),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]);
                        ListItem::new(line)
                    })
                    .collect::<Vec<_>>(),
            )
            .block(
                pane_block(
                    "Items",
                    pane_focus == BrowserPane::Items,
                    Color::Cyan,
                    Color::Black,
                )
                .title(format!(
                    "Items {}/{}  filter:{}",
                    visible_indexes.len(),
                    items.len(),
                    kind_filters[active_filter]
                )),
            )
            .highlight_symbol("▌ ")
            .repeat_highlight_symbol(true)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            );
            frame.render_stateful_widget(list, panes[0], &mut state);

            let detail_title = selected_item
                .map(|item| {
                    detail_title(
                        item,
                        selected_visible,
                        visible_indexes.len(),
                        detail_scroll,
                        total_detail_lines,
                    )
                })
                .unwrap_or_else(|| "Detail".to_string());
            let detail_items = detail_lines
                .iter()
                .map(|line| {
                    ListItem::new(Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(Color::White),
                    )))
                })
                .collect::<Vec<_>>();
            if pane_focus == BrowserPane::Detail {
                let mut detail_state = ListState::default();
                detail_state.select(Some(detail_selected));
                let detail = List::new(detail_items)
                    .block(
                        pane_block("Detail", true, Color::LightBlue, Color::Black)
                            .title(detail_title),
                    )
                    .highlight_symbol("▌ ")
                    .repeat_highlight_symbol(true)
                    .highlight_style(
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    );
                frame.render_stateful_widget(detail, panes[1], &mut detail_state);
            } else {
                let detail = List::new(detail_items).block(
                    pane_block("Detail", false, Color::LightBlue, Color::Black).title(detail_title),
                );
                frame.render_widget(detail, panes[1]);
            }

            frame.render_widget(
                tui_shell::build_footer_controls(vec![
                    Line::from(vec![
                        tui_shell::label("Selection "),
                        tui_shell::accent(
                            format!(
                                "{}/{}",
                                state.selected().map(|index| index + 1).unwrap_or(0),
                                visible_indexes.len()
                            ),
                            Color::White,
                        ),
                        Span::raw("  "),
                        tui_shell::label("Filter "),
                        tui_shell::accent(kind_filters[active_filter].to_string(), Color::Yellow),
                        Span::raw("  "),
                        tui_shell::focus_label("Focus "),
                        tui_shell::key_chip(
                            match pane_focus {
                                BrowserPane::Items => "Items",
                                BrowserPane::Detail => "Detail",
                            },
                            Color::Blue,
                        ),
                        Span::raw("  "),
                        tui_shell::label("Search "),
                        tui_shell::accent(search.footer_label(), Color::LightGreen),
                    ]),
                    tui_shell::control_line(&[
                        ("Tab", Color::Blue, "next pane"),
                        ("Shift+Tab", Color::Blue, "previous pane"),
                        ("Up/Down", Color::Blue, "move"),
                        ("PgUp/PgDn", Color::Blue, "scroll detail"),
                    ]),
                    if search.has_pending() {
                        tui_shell::control_line(&[
                            ("Backspace", Color::Blue, "edit"),
                            ("Enter", Color::LightGreen, "search"),
                            ("Esc", Color::Yellow, "cancel"),
                            ("q", Color::LightGreen, "search text"),
                        ])
                    } else {
                        tui_shell::control_line(&[
                            ("f/F", Color::Yellow, "change filter"),
                            ("Home/End", Color::Blue, "jump"),
                            ("/ ?", Color::LightGreen, "search"),
                            ("n", Color::LightGreen, "repeat"),
                            ("Esc/q", Color::Gray, "exit"),
                        ])
                    },
                ]),
                outer[2],
            );
        })?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if search.has_pending() {
                match key.code {
                    KeyCode::Esc => search.cancel(),
                    KeyCode::Enter => {
                        let next_selection = search.apply(
                            items,
                            &visible_indexes,
                            state.selected(),
                            &kind_filters[active_filter],
                        );
                        if let Some(next_selection) = next_selection {
                            state.select(Some(next_selection));
                            detail_scroll = 0;
                        }
                    }
                    KeyCode::Backspace => search.pop_char(),
                    KeyCode::Char(_ch) if key.modifiers.contains(KeyModifiers::CONTROL) => {}
                    KeyCode::Char(ch) => search.push_char(ch),
                    _ => {}
                }
                continue;
            }
            let selected_visible = state.selected().unwrap_or(0);
            let selected_item = visible_indexes
                .get(selected_visible)
                .and_then(|index| items.get(*index));
            let total_detail_lines = selected_detail_line_count(selected_item);
            match key.code {
                KeyCode::BackTab => {
                    pane_focus = match pane_focus {
                        BrowserPane::Items => BrowserPane::Detail,
                        BrowserPane::Detail => BrowserPane::Items,
                    };
                }
                KeyCode::Tab => {
                    pane_focus = match pane_focus {
                        BrowserPane::Items => BrowserPane::Detail,
                        BrowserPane::Detail => BrowserPane::Items,
                    };
                }
                KeyCode::Up => match pane_focus {
                    BrowserPane::Items => {
                        let selected = state.selected().unwrap_or(0);
                        state.select(Some(selected.saturating_sub(1)));
                        detail_scroll = 0;
                    }
                    BrowserPane::Detail => {
                        detail_scroll = detail_scroll.saturating_sub(1);
                    }
                },
                KeyCode::Down => match pane_focus {
                    BrowserPane::Items => {
                        let selected = state.selected().unwrap_or(0);
                        state.select(Some(
                            (selected + 1).min(visible_indexes.len().saturating_sub(1)),
                        ));
                        detail_scroll = 0;
                    }
                    BrowserPane::Detail => {
                        detail_scroll = detail_scroll
                            .saturating_add(1)
                            .min(total_detail_lines.saturating_sub(1) as u16);
                    }
                },
                KeyCode::PageUp => {
                    detail_scroll = detail_scroll.saturating_sub(10);
                }
                KeyCode::PageDown => {
                    detail_scroll = detail_scroll
                        .saturating_add(10)
                        .min(total_detail_lines.saturating_sub(1) as u16);
                }
                KeyCode::Home => match pane_focus {
                    BrowserPane::Items => {
                        state.select(Some(0));
                        detail_scroll = 0;
                    }
                    BrowserPane::Detail => detail_scroll = 0,
                },
                KeyCode::End => match pane_focus {
                    BrowserPane::Items => {
                        state.select(Some(visible_indexes.len().saturating_sub(1)));
                        detail_scroll = 0;
                    }
                    BrowserPane::Detail => {
                        detail_scroll = total_detail_lines.saturating_sub(1) as u16;
                    }
                },
                KeyCode::Enter => detail_scroll = 0,
                KeyCode::Char('f') => {
                    active_filter = (active_filter + 1) % kind_filters.len();
                    visible_indexes = visible_item_indexes(items, &kind_filters[active_filter]);
                    state.select((!visible_indexes.is_empty()).then_some(0));
                    detail_scroll = 0;
                }
                KeyCode::Char('F') => {
                    active_filter = if active_filter == 0 {
                        kind_filters.len().saturating_sub(1)
                    } else {
                        active_filter - 1
                    };
                    visible_indexes = visible_item_indexes(items, &kind_filters[active_filter]);
                    state.select((!visible_indexes.is_empty()).then_some(0));
                    detail_scroll = 0;
                }
                KeyCode::Char('/') => search.start(SearchDirection::Forward),
                KeyCode::Char('?') => search.start(SearchDirection::Backward),
                KeyCode::Char('n') => {
                    let next_selection = search.repeat(
                        items,
                        &visible_indexes,
                        state.selected(),
                        &kind_filters[active_filter],
                    );
                    if next_selection.is_some() {
                        state.select(next_selection);
                        detail_scroll = 0;
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}

#[cfg(feature = "tui")]
fn pane_block(title: &str, focused: bool, accent: Color, bg: Color) -> Block<'static> {
    let title_bg = if focused { accent } else { bg };
    let title_fg = if focused { Color::Black } else { Color::White };
    Block::default()
        .borders(Borders::ALL)
        .title(if focused {
            format!("{title} [Focused]")
        } else {
            title.to_string()
        })
        .style(Style::default().bg(bg))
        .border_style(Style::default().fg(if focused { accent } else { Color::Gray }))
        .title_style(
            Style::default()
                .fg(title_fg)
                .bg(title_bg)
                .add_modifier(Modifier::BOLD),
        )
}

#[cfg(not(feature = "tui"))]
pub(crate) fn run_interactive_browser(
    _title: &str,
    _summary_lines: &[String],
    _items: &[BrowserItem],
) -> Result<()> {
    Err(crate::common::tui_feature_required(
        "Shared interactive browser",
    ))
}

#[cfg(all(test, not(feature = "tui")))]
#[test]
fn run_interactive_browser_returns_tui_error_when_feature_disabled() {
    let error = run_interactive_browser(
        "Test",
        &[],
        &[BrowserItem {
            kind: "dashboard".to_string(),
            title: "Example".to_string(),
            meta: "meta".to_string(),
            details: vec!["detail".to_string()],
        }],
    )
    .expect_err("feature-disabled browser should return an error");

    assert_eq!(
        error.to_string(),
        "Shared interactive browser requires the `tui` feature."
    );
}

#[cfg(test)]
mod tests {
    use super::{
        append_browser_detail_section, browser_detail_aligned_fact, browser_detail_fact,
        browser_detail_fallback_fact, build_search_state, detail_title, find_match_in_visible,
        matching_visible_indexes, BrowserItem, BrowserSearchController, SearchDirection,
    };

    fn sample_items() -> Vec<BrowserItem> {
        vec![
            BrowserItem {
                kind: "dashboard".to_string(),
                title: "CPU Overview".to_string(),
                meta: "folder=ops".to_string(),
                details: vec!["Prometheus datasource".to_string()],
            },
            BrowserItem {
                kind: "alert".to_string(),
                title: "Disk alert".to_string(),
                meta: "sev=high".to_string(),
                details: vec!["filesystem saturation".to_string()],
            },
            BrowserItem {
                kind: "dashboard".to_string(),
                title: "Memory Board".to_string(),
                meta: "folder=infra".to_string(),
                details: vec!["CPU and memory detail".to_string()],
            },
        ]
    }

    #[test]
    fn browser_item_search_matches_kind_title_meta_and_details() {
        let item = BrowserItem {
            kind: "dashboard".to_string(),
            title: "CPU Overview".to_string(),
            meta: "folder=ops".to_string(),
            details: vec!["prometheus datasource".to_string()],
        };

        assert!(item.matches_query("dash"));
        assert!(item.matches_query("cpu"));
        assert!(item.matches_query("ops"));
        assert!(item.matches_query("datasource"));
        assert!(!item.matches_query("loki"));
    }

    #[test]
    fn append_browser_detail_section_formats_empty_and_populated_sections() {
        let mut details = vec!["Node ID: dashboard:db".to_string()];

        append_browser_detail_section(&mut details, "Inbound edge summary", Vec::new());
        append_browser_detail_section(
            &mut details,
            "Outbound edge summary",
            vec!["  uses -> Env [variable]".to_string()],
        );

        assert_eq!(
            details,
            vec![
                "Node ID: dashboard:db".to_string(),
                "Inbound edge summary: none".to_string(),
                "Outbound edge summary:".to_string(),
                "  uses -> Env [variable]".to_string(),
            ]
        );
    }

    #[test]
    fn browser_detail_fact_formats_label_value_rows() {
        assert_eq!(
            browser_detail_fact("Dashboard UID", "cpu-main"),
            "Dashboard UID: cpu-main"
        );
        assert_eq!(browser_detail_fact("Query Count", 3), "Query Count: 3");
    }

    #[test]
    fn browser_detail_fallback_fact_trims_or_uses_fallback() {
        assert_eq!(
            browser_detail_fallback_fact("Org", " Main Org. ", "-"),
            "Org: Main Org."
        );
        assert_eq!(browser_detail_fallback_fact("UID", "  ", "-"), "UID: -");
    }

    #[test]
    fn browser_detail_aligned_fact_formats_full_detail_rows() {
        assert_eq!(
            browser_detail_aligned_fact("Kind", "dashboard-summary"),
            "Kind            : dashboard-summary"
        );
        assert_eq!(
            browser_detail_aligned_fact("Summary", "uid=cpu-main"),
            "Summary         : uid=cpu-main"
        );
    }

    #[test]
    fn search_uses_only_active_filter_visible_indexes() {
        let items = sample_items();
        let visible_indexes = vec![0, 2];

        let matches = matching_visible_indexes(&items, &visible_indexes, "disk");
        let selected = find_match_in_visible(
            &items,
            &visible_indexes,
            "cpu",
            SearchDirection::Forward,
            Some(0),
        );

        assert!(matches.is_empty());
        assert_eq!(selected, Some(0));
    }

    #[test]
    fn detail_title_uses_filtered_visible_position_and_total() {
        let items = sample_items();
        let visible_indexes = [0, 2];
        let selected_visible = 1;
        let item = &items[visible_indexes[selected_visible]];

        assert_eq!(
            detail_title(
                item,
                selected_visible,
                visible_indexes.len(),
                0,
                item.details.len()
            ),
            "Detail 2/2 [dashboard]  line 1/1"
        );
    }

    #[test]
    fn repeat_search_advances_from_current_selection() {
        let items = sample_items();
        let visible_indexes = vec![0, 2];
        let mut search = BrowserSearchController::default();

        search.start(SearchDirection::Forward);
        search.push_char('c');
        search.push_char('p');
        search.push_char('u');

        let first = search.apply(&items, &visible_indexes, Some(0), "dashboard");
        let repeated = search.repeat(&items, &visible_indexes, first, "dashboard");

        assert_eq!(first, Some(0));
        assert_eq!(repeated, Some(1));
    }

    #[test]
    fn repeat_search_wraps_forward_to_first_visible_match() {
        let items = sample_items();
        let visible_indexes = vec![0, 2];
        let mut search = BrowserSearchController::default();

        search.start(SearchDirection::Forward);
        search.push_char('c');
        search.push_char('p');
        search.push_char('u');
        let first = search.apply(&items, &visible_indexes, Some(1), "dashboard");
        let repeated = search.repeat(&items, &visible_indexes, first, "dashboard");

        assert_eq!(first, Some(1));
        assert_eq!(repeated, Some(0));
        assert_eq!(
            search.summary_line("dashboard"),
            "Last search /\"cpu\" in filter dashboard matched 1/2 results. Press n for next match."
        );
    }

    #[test]
    fn repeat_search_wraps_backward_to_last_visible_match() {
        let items = sample_items();
        let visible_indexes = vec![0, 2];
        let mut search = BrowserSearchController::default();

        search.start(SearchDirection::Backward);
        search.push_char('c');
        search.push_char('p');
        search.push_char('u');
        let first = search.apply(&items, &visible_indexes, Some(0), "dashboard");
        let repeated = search.repeat(&items, &visible_indexes, first, "dashboard");

        assert_eq!(first, Some(0));
        assert_eq!(repeated, Some(1));
        assert_eq!(
            search.summary_line("dashboard"),
            "Last search ?\"cpu\" in filter dashboard matched 2/2 results. Press n for next match."
        );
    }

    #[test]
    fn cancel_search_prompt_preserves_last_search_state() {
        let items = sample_items();
        let visible_indexes = vec![0, 2];
        let mut search = BrowserSearchController::default();

        search.start(SearchDirection::Forward);
        search.push_char('c');
        search.push_char('p');
        search.push_char('u');
        let applied = search.apply(&items, &visible_indexes, Some(0), "dashboard");
        assert_eq!(applied, Some(0));

        search.start(SearchDirection::Backward);
        search.push_char('d');
        search.cancel();

        assert_eq!(
            search.summary_line("dashboard"),
            "Last search /\"cpu\" in filter dashboard matched 1/2 results. Press n for next match."
        );
    }

    #[test]
    fn search_summary_reports_no_matches() {
        let state = build_search_state(
            SearchDirection::Backward,
            "missing".to_string(),
            "alert",
            &[],
            None,
        );
        let search = BrowserSearchController {
            pending: None,
            last: Some(state),
        };

        assert_eq!(
            search.summary_line("dashboard"),
            "Last search ?\"missing\" in filter alert matched 0 results. Press / or ? to try again."
        );
    }

    #[test]
    fn pending_search_summary_uses_compact_prompt_hints_without_repeat_key() {
        let mut search = BrowserSearchController::default();

        search.start(SearchDirection::Backward);
        search.push_char('c');
        search.push_char('p');
        search.push_char('u');

        let summary = search.summary_line("dashboard");

        assert!(summary.contains("Search prompt ? in filter dashboard"));
        assert!(summary.contains("Enter search"));
        assert!(summary.contains("Esc cancel"));
        assert!(!summary.contains("Enter apply"));
        assert!(!summary.contains("n repeat"));
    }
}
