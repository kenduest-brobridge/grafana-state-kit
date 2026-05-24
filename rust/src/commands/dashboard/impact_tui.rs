#![cfg(feature = "tui")]
// Specialized interactive browser for dashboard impact review.
#![cfg_attr(test, allow(dead_code))]
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

use crate::common::Result;
use crate::interactive_browser::BrowserItem;
use crate::tui_shell;

use super::topology::{build_impact_browser_items, ImpactDocument};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImpactPane {
    Groups,
    Items,
    Detail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ImpactGroup {
    pub(crate) label: String,
    pub(crate) kind: String,
    pub(crate) count: usize,
    pub(crate) subtitle: String,
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

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

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn pane_title(label: &str, active: bool) -> String {
    if active {
        format!("{label} [active]")
    } else {
        label.to_string()
    }
}

fn pane_block(label: &str, active: bool) -> Block<'static> {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(pane_title(label, active))
        .title_style(
            Style::default()
                .fg(if active { Color::Black } else { Color::White })
                .bg(if active { Color::Cyan } else { Color::Reset })
                .add_modifier(Modifier::BOLD),
        );
    if active {
        block = block.border_style(Style::default().fg(Color::Cyan));
    }
    block
}

fn item_color(kind: &str) -> Color {
    match kind {
        "dashboard" => Color::Yellow,
        "alert-rule" => Color::LightRed,
        "mute-timing" => Color::LightMagenta,
        "contact-point" => Color::Green,
        "policy" => Color::Magenta,
        "template" => Color::Cyan,
        _ => Color::Gray,
    }
}

pub(crate) fn build_impact_tui_groups(document: &ImpactDocument) -> Vec<ImpactGroup> {
    vec![
        ImpactGroup {
            label: "All".to_string(),
            kind: "all".to_string(),
            count: document.summary.dashboard_count + document.summary.alert_resource_count,
            subtitle: "Full blast radius".to_string(),
        },
        ImpactGroup {
            label: "Dashboards".to_string(),
            kind: "dashboard".to_string(),
            count: document.summary.dashboard_count,
            subtitle: "Panels and queries at risk".to_string(),
        },
        ImpactGroup {
            label: "Alert Rules".to_string(),
            kind: "alert-rule".to_string(),
            count: document.summary.alert_rule_count,
            subtitle: "Rules directly tied to the datasource".to_string(),
        },
        ImpactGroup {
            label: "Mute Timings".to_string(),
            kind: "mute-timing".to_string(),
            count: document.summary.mute_timing_count,
            subtitle: "Timing resources inside the blast radius".to_string(),
        },
        ImpactGroup {
            label: "Contact Points".to_string(),
            kind: "contact-point".to_string(),
            count: document.summary.contact_point_count,
            subtitle: "Downstream notification endpoints".to_string(),
        },
        ImpactGroup {
            label: "Policies".to_string(),
            kind: "policy".to_string(),
            count: document.summary.notification_policy_count,
            subtitle: "Routing policies touched by affected alerts".to_string(),
        },
        ImpactGroup {
            label: "Templates".to_string(),
            kind: "template".to_string(),
            count: document.summary.template_count,
            subtitle: "Notification templates in the path".to_string(),
        },
    ]
}

pub(crate) fn filter_impact_tui_items(
    document: &ImpactDocument,
    group_kind: &str,
) -> Vec<BrowserItem> {
    build_impact_browser_items(document)
        .into_iter()
        .filter(|item| group_kind == "all" || item.kind == group_kind)
        .collect()
}

pub(crate) fn filter_impact_tui_items_by_query(
    document: &ImpactDocument,
    group_kind: &str,
    query: &str,
) -> Vec<BrowserItem> {
    let items = filter_impact_tui_items(document, group_kind);
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return items;
    }
    items
        .into_iter()
        .filter(|item| impact_item_matches_query(item, &needle))
        .collect()
}

fn impact_item_matches_query(item: &BrowserItem, needle: &str) -> bool {
    item.kind.to_ascii_lowercase().contains(needle)
        || item.title.to_ascii_lowercase().contains(needle)
        || item.meta.to_ascii_lowercase().contains(needle)
        || item
            .details
            .iter()
            .any(|line| line.to_ascii_lowercase().contains(needle))
}

pub(crate) fn build_impact_footer_control_lines(
    focus: &str,
    selection: &str,
    active_search: Option<&str>,
    pending_search: Option<&str>,
) -> Vec<Line<'static>> {
    let search_summary = if let Some(query) = pending_search {
        format!("Search prompt {query}")
    } else if let Some(query) = active_search {
        format!("Search filter {query}")
    } else {
        "Search idle".to_string()
    };
    let mut lines = vec![Line::from(vec![
        tui_shell::focus_label("Focus "),
        tui_shell::key_chip(focus, Color::Blue),
        Span::raw("  "),
        tui_shell::label("Selection "),
        tui_shell::accent(selection.to_string(), Color::White),
        Span::raw("  "),
        tui_shell::label("Search "),
        tui_shell::accent(search_summary, Color::LightMagenta),
    ])];
    if pending_search.is_some() {
        lines.push(tui_shell::control_line(&[
            ("Backspace", Color::Blue, "edit"),
            ("Enter", Color::LightGreen, "search items"),
            ("Esc", Color::Yellow, "cancel"),
        ]));
    } else {
        lines.push(tui_shell::control_line(&[
            ("Tab", Color::Blue, "next pane"),
            ("Up/Down", Color::Blue, "move"),
            ("PgUp/PgDn", Color::Blue, "scroll detail"),
            ("Home/End", Color::Blue, "jump"),
        ]));
        lines.push(tui_shell::control_line(&[
            ("/?", Color::LightGreen, "search items"),
            ("Enter", Color::Blue, "reset detail"),
            ("Esc/q", Color::Gray, "exit"),
        ]));
    }
    lines
}

fn filtered_impact_items(
    document: &ImpactDocument,
    group_kind: &str,
    search_query: Option<&str>,
) -> Vec<BrowserItem> {
    search_query
        .map(|query| filter_impact_tui_items_by_query(document, group_kind, query))
        .unwrap_or_else(|| filter_impact_tui_items(document, group_kind))
}

pub(crate) fn run_impact_interactive(document: &ImpactDocument) -> Result<()> {
    let groups = build_impact_tui_groups(document);
    let mut group_state = ListState::default();
    group_state.select(Some(0));
    let mut item_state = ListState::default();
    let mut search_query: Option<String> = None;
    let mut pending_search: Option<String> = None;
    let mut items = filtered_impact_items(document, &groups[0].kind, search_query.as_deref());
    item_state.select((!items.is_empty()).then_some(0));
    let mut detail_scroll = 0u16;
    let mut active_pane = ImpactPane::Groups;
    let mut session = TerminalSession::enter()?;

    loop {
        session.terminal.draw(|frame| {
            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(1), Constraint::Length(4)])
                .split(frame.area());
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(22),
                    Constraint::Percentage(33),
                    Constraint::Percentage(45),
                ])
                .split(outer[1]);

            let summary_lines = vec![
                Line::from(vec![
                    Span::styled(
                        format!("Datasource {}", document.summary.datasource_uid),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("   "),
                    Span::raw(format!(
                        "dashboards={} alert-resources={}",
                        document.summary.dashboard_count, document.summary.alert_resource_count
                    )),
                ]),
                Line::from(format!(
                    "alert-rules={} contact-points={} policies={} templates={} mute-timings={}",
                    document.summary.alert_rule_count,
                    document.summary.contact_point_count,
                    document.summary.notification_policy_count,
                    document.summary.template_count,
                    document.summary.mute_timing_count
                )),
                Line::from("Use groups to narrow the blast radius, then inspect affected items and their downstream context."),
            ];
            let summary = Paragraph::new(summary_lines)
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).title("Impact Summary"));
            frame.render_widget(summary, outer[0]);

            let group_list = List::new(
                groups
                    .iter()
                    .map(|group| {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("{:>2} ", group.count),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                group.label.clone(),
                                Style::default()
                                    .fg(item_color(&group.kind))
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]))
                    })
                    .collect::<Vec<_>>(),
            )
            .block(pane_block("Groups", active_pane == ImpactPane::Groups))
            .highlight_symbol("▌ ")
            .repeat_highlight_symbol(true)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_stateful_widget(group_list, panes[0], &mut group_state);

            let selected_group = group_state.selected().unwrap_or(0);
            let group_title = if let Some(group) = groups.get(selected_group) {
                format!("Affected Items {}/{}  {}", items.len(), document.summary.alert_resource_count + document.summary.dashboard_count, group.label)
            } else {
                "Affected Items".to_string()
            };
            let item_list = List::new(
                items.iter()
                    .enumerate()
                    .map(|(index, item)| {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("{:>2}. ", index + 1),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                format!("[{}]", item.kind.to_uppercase()),
                                Style::default()
                                    .fg(item_color(&item.kind))
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(format!(" {}", item.title)),
                        ]))
                    })
                    .collect::<Vec<_>>(),
            )
            .block(pane_block(&group_title, active_pane == ImpactPane::Items))
            .highlight_symbol("▌ ")
            .repeat_highlight_symbol(true)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_stateful_widget(item_list, panes[1], &mut item_state);

            let selected_item = item_state.selected().and_then(|index| items.get(index));
            let detail_text = selected_item
                .map(|item| item.details.join("\n"))
                .unwrap_or_else(|| "No affected item in this group.".to_string());
            let detail_total_lines = selected_item
                .map(|item| item.details.len().max(1))
                .unwrap_or(1);
            let detail_title = selected_item
                .map(|item| {
                    format!(
                        "Detail [{}/{}] {}  line {}/{}",
                        item_state.selected().map(|index| index + 1).unwrap_or(0),
                        items.len(),
                        item.title,
                        (detail_scroll as usize + 1).min(detail_total_lines),
                        detail_total_lines
                    )
                })
                .unwrap_or_else(|| "Detail".to_string());
            let detail = Paragraph::new(detail_text)
                .scroll((detail_scroll, 0))
                .wrap(Wrap { trim: false })
                .block(pane_block(&detail_title, active_pane == ImpactPane::Detail));
            frame.render_widget(detail, panes[2]);

            let focus_label = match active_pane {
                ImpactPane::Groups => "Groups",
                ImpactPane::Items => "Items",
                ImpactPane::Detail => "Detail",
            };
            let selection_label = format!(
                "group {}/{}  item {}/{}",
                group_state.selected().map(|index| index + 1).unwrap_or(0),
                groups.len(),
                item_state.selected().map(|index| index + 1).unwrap_or(0),
                items.len()
            );
            frame.render_widget(
                tui_shell::build_footer_controls(build_impact_footer_control_lines(
                    focus_label,
                    &selection_label,
                    search_query.as_deref(),
                    pending_search.as_deref(),
                )),
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
            if let Some(query) = pending_search.as_mut() {
                match key.code {
                    KeyCode::Esc => pending_search = None,
                    KeyCode::Enter => {
                        let query = query.trim().to_string();
                        pending_search = None;
                        search_query = (!query.is_empty()).then_some(query);
                        let selected_group = group_state.selected().unwrap_or(0);
                        items = filtered_impact_items(
                            document,
                            &groups[selected_group].kind,
                            search_query.as_deref(),
                        );
                        item_state.select((!items.is_empty()).then_some(0));
                        detail_scroll = 0;
                    }
                    KeyCode::Backspace => {
                        query.pop();
                    }
                    KeyCode::Char(ch) => {
                        query.push(ch);
                    }
                    _ => {}
                }
                continue;
            }
            match key.code {
                KeyCode::Char('/') | KeyCode::Char('?') => pending_search = Some(String::new()),
                KeyCode::Tab => {
                    active_pane = match active_pane {
                        ImpactPane::Groups => ImpactPane::Items,
                        ImpactPane::Items => ImpactPane::Detail,
                        ImpactPane::Detail => ImpactPane::Groups,
                    };
                }
                KeyCode::Up => match active_pane {
                    ImpactPane::Groups => {
                        let selected = group_state.selected().unwrap_or(0).saturating_sub(1);
                        group_state.select(Some(selected));
                        let selected_group = &groups[selected];
                        items = filtered_impact_items(
                            document,
                            &selected_group.kind,
                            search_query.as_deref(),
                        );
                        item_state.select((!items.is_empty()).then_some(0));
                        detail_scroll = 0;
                    }
                    ImpactPane::Items => {
                        let selected = item_state.selected().unwrap_or(0).saturating_sub(1);
                        item_state.select((!items.is_empty()).then_some(selected));
                        detail_scroll = 0;
                    }
                    ImpactPane::Detail => {
                        detail_scroll = detail_scroll.saturating_sub(1);
                    }
                },
                KeyCode::Down => match active_pane {
                    ImpactPane::Groups => {
                        let selected = (group_state.selected().unwrap_or(0) + 1)
                            .min(groups.len().saturating_sub(1));
                        group_state.select(Some(selected));
                        let selected_group = &groups[selected];
                        items = filtered_impact_items(
                            document,
                            &selected_group.kind,
                            search_query.as_deref(),
                        );
                        item_state.select((!items.is_empty()).then_some(0));
                        detail_scroll = 0;
                    }
                    ImpactPane::Items => {
                        let selected = (item_state.selected().unwrap_or(0) + 1)
                            .min(items.len().saturating_sub(1));
                        item_state.select((!items.is_empty()).then_some(selected));
                        detail_scroll = 0;
                    }
                    ImpactPane::Detail => {
                        detail_scroll = detail_scroll.saturating_add(1);
                    }
                },
                KeyCode::PageUp if active_pane == ImpactPane::Detail => {
                    detail_scroll = detail_scroll.saturating_sub(10);
                }
                KeyCode::PageDown if active_pane == ImpactPane::Detail => {
                    detail_scroll = detail_scroll.saturating_add(10);
                }
                KeyCode::Home => match active_pane {
                    ImpactPane::Groups => {
                        group_state.select(Some(0));
                        items = filtered_impact_items(
                            document,
                            &groups[0].kind,
                            search_query.as_deref(),
                        );
                        item_state.select((!items.is_empty()).then_some(0));
                        detail_scroll = 0;
                    }
                    ImpactPane::Items => {
                        item_state.select((!items.is_empty()).then_some(0));
                        detail_scroll = 0;
                    }
                    ImpactPane::Detail => detail_scroll = 0,
                },
                KeyCode::End => match active_pane {
                    ImpactPane::Groups => {
                        let selected = groups.len().saturating_sub(1);
                        group_state.select(Some(selected));
                        items = filtered_impact_items(
                            document,
                            &groups[selected].kind,
                            search_query.as_deref(),
                        );
                        item_state.select((!items.is_empty()).then_some(0));
                        detail_scroll = 0;
                    }
                    ImpactPane::Items => {
                        item_state
                            .select((!items.is_empty()).then_some(items.len().saturating_sub(1)));
                        detail_scroll = 0;
                    }
                    ImpactPane::Detail => {
                        let max_scroll = selected_item_max_scroll(&items, item_state.selected());
                        detail_scroll = max_scroll as u16;
                    }
                },
                KeyCode::Enter => detail_scroll = 0,
                KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}

fn selected_item_max_scroll(items: &[BrowserItem], selected: Option<usize>) -> usize {
    selected
        .and_then(|index| items.get(index))
        .map(|item| item.details.len().saturating_sub(1))
        .unwrap_or(0)
}
