#![cfg(feature = "tui")]
#![cfg_attr(test, allow(dead_code))]

use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

#[cfg(feature = "tui")]
use crate::tui_shell;

use super::{ProjectStatusPane, ProjectStatusTuiState, SearchDirection, SearchPromptState};

pub(crate) fn render_project_status_frame(
    frame: &mut ratatui::Frame,
    state: &mut ProjectStatusTuiState,
) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Min(1),
            Constraint::Length(status_footer_height()),
        ])
        .split(frame.area());
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(42),
            Constraint::Percentage(28),
        ])
        .split(outer[2]);

    let header_lines = vec![
        summary_line(&[
            summary_cell("Scope", state.document().scope.clone(), Color::White),
            summary_cell(
                "Domains",
                state.document().overall.domain_count.to_string(),
                Color::White,
            ),
            summary_cell(
                "Present",
                state.document().overall.present_count.to_string(),
                Color::White,
            ),
            summary_cell(
                "Blocked",
                state.document().overall.blocked_count.to_string(),
                Color::LightRed,
            ),
            summary_cell(
                "Warnings",
                state.document().overall.warning_count.to_string(),
                Color::Yellow,
            ),
        ]),
        summary_line(&[
            summary_cell(
                "Overall",
                state.document().overall.status.clone(),
                status_color(state.document().overall.status.as_str()),
            ),
            summary_cell(
                "Freshness",
                state.document().overall.freshness.status.clone(),
                Color::White,
            ),
            summary_cell(
                "Domain",
                state
                    .current_domain()
                    .map(|domain| domain.id.as_str())
                    .unwrap_or("No domain"),
                Color::White,
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Focus ",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
            key_chip(focus_label(state.focus()), Color::Blue),
            Span::raw("  "),
            Span::styled(
                "Path ",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
            plain("Home -> Domains -> Details -> Actions"),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(header_lines)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Project Status Workbench")
                    .border_style(Style::default().fg(Color::LightBlue)),
            ),
        outer[0],
    );

    let home_lines = state
        .home_lines()
        .into_iter()
        .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::White))))
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(home_lines)
            .wrap(Wrap { trim: false })
            .block(pane_block(
                "Project Home",
                state.focus() == ProjectStatusPane::Home,
                status_color(state.document().overall.status.as_str()),
            )),
        outer[1],
    );

    let domain_items = state
        .document()
        .domains
        .iter()
        .enumerate()
        .map(|(index, domain)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>2}. ", index + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    domain.id.clone(),
                    Style::default()
                        .fg(status_color(domain.status.as_str()))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "  {}  blockers={} warnings={}",
                        domain.status, domain.blocker_count, domain.warning_count
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect::<Vec<_>>();
    let domain_title = state
        .current_domain()
        .map(|domain| {
            format!(
                "Domains {}/{}  current={}",
                state
                    .current_domain_index()
                    .map(|index| index + 1)
                    .unwrap_or(0),
                state.document().domains.len(),
                domain.id
            )
        })
        .unwrap_or_else(|| "Domains".to_string());
    frame.render_stateful_widget(
        List::new(domain_items)
            .block(pane_block(
                &domain_title,
                state.focus() == ProjectStatusPane::Domains,
                status_color(
                    state
                        .current_domain()
                        .map(|domain| domain.status.as_str())
                        .unwrap_or("unknown"),
                ),
            ))
            .highlight_symbol("▌ ")
            .repeat_highlight_symbol(true)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
        body[0],
        state.domain_state_mut(),
    );

    let detail_lines = state
        .current_domain_lines()
        .into_iter()
        .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::White))))
        .collect::<Vec<_>>();
    let detail_title = state
        .current_domain()
        .map(|domain| {
            format!(
                "Domain Detail {}/{}  {}",
                state
                    .current_domain_index()
                    .map(|index| index + 1)
                    .unwrap_or(0),
                state.document().domains.len(),
                domain.id
            )
        })
        .unwrap_or_else(|| "Domain Detail".to_string());
    frame.render_widget(
        Paragraph::new(detail_lines)
            .wrap(Wrap { trim: false })
            .scroll((state.detail_scroll(), 0))
            .block(pane_block(
                &detail_title,
                state.focus() == ProjectStatusPane::Details,
                Color::Cyan,
            )),
        body[1],
    );

    let action_items = state
        .document()
        .next_actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>2}. ", index + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    action.domain.clone(),
                    Style::default()
                        .fg(action_color(action.reason_code.as_str()))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {}  {}", action.reason_code, action.action),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect::<Vec<_>>();
    let action_title = state
        .current_action()
        .map(|action| {
            format!(
                "Actions {}/{}  recommended={}",
                state
                    .current_action_index()
                    .map(|index| index + 1)
                    .unwrap_or(0),
                state.document().next_actions.len(),
                action.domain
            )
        })
        .unwrap_or_else(|| "Actions".to_string());
    frame.render_stateful_widget(
        List::new(action_items)
            .block(pane_block(
                &action_title,
                state.focus() == ProjectStatusPane::Actions,
                Color::Yellow,
            ))
            .highlight_symbol("▌ ")
            .repeat_highlight_symbol(true)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        body[2],
        state.action_state_mut(),
    );

    frame.render_widget(build_status_footer(state), outer[3]);

    if let Some(search) = state.pending_search() {
        render_search_prompt(frame, search);
    }
}

fn pane_block(title: &str, focused: bool, accent: Color) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title(if focused {
            format!("{title} [Focused]")
        } else {
            title.to_string()
        })
        .border_style(Style::default().fg(if focused { accent } else { Color::Gray }))
        .title_style(
            Style::default()
                .fg(if focused { Color::Black } else { Color::White })
                .bg(if focused { accent } else { Color::Reset })
                .add_modifier(Modifier::BOLD),
        )
}

fn status_color(status: &str) -> Color {
    match status {
        "blocked" => Color::LightRed,
        "partial" => Color::Yellow,
        "ready" => Color::Green,
        _ => Color::Gray,
    }
}

fn action_color(reason_code: &str) -> Color {
    match reason_code {
        "blocked-by-blockers" => Color::LightRed,
        "blocked-by-warnings" => Color::Yellow,
        "ready" => Color::Green,
        _ => Color::Cyan,
    }
}

fn key_chip(label: &str, color: Color) -> Span<'static> {
    Span::styled(
        format!(" {label} "),
        Style::default()
            .fg(Color::White)
            .bg(color)
            .add_modifier(Modifier::BOLD),
    )
}

fn plain(value: impl Into<String>) -> Span<'static> {
    Span::styled(value.into(), Style::default().fg(Color::White))
}

struct SummaryCell {
    label: String,
    value: String,
    color: Color,
}

fn summary_cell(label: impl Into<String>, value: impl Into<String>, color: Color) -> SummaryCell {
    SummaryCell {
        label: label.into(),
        value: value.into(),
        color,
    }
}

fn summary_line(items: &[SummaryCell]) -> Line<'static> {
    let cell_width = items
        .iter()
        .map(|item| item.label.chars().count() + item.value.chars().count() + 2)
        .max()
        .unwrap_or(0);
    let mut spans = Vec::new();
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        let used_width = item.label.chars().count() + item.value.chars().count() + 2;
        let trailing_padding = cell_width.saturating_sub(used_width);
        spans.push(Span::styled(
            format!("{} ", item.label),
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            item.value.clone(),
            Style::default().fg(item.color).add_modifier(Modifier::BOLD),
        ));
        if trailing_padding > 0 {
            spans.push(Span::raw(" ".repeat(trailing_padding)));
        }
    }
    Line::from(spans)
}

fn status_footer_height() -> u16 {
    const FOOTER_LINE_COUNT: usize = 4;
    #[cfg(feature = "tui")]
    {
        tui_shell::footer_height(FOOTER_LINE_COUNT)
    }
    #[cfg(not(feature = "tui"))]
    {
        FOOTER_LINE_COUNT.saturating_add(3) as u16
    }
}

fn build_status_footer(state: &ProjectStatusTuiState) -> Paragraph<'static> {
    let lines = vec![
        Line::from(Span::styled(
            state.status_line(),
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            state.search_status().to_string(),
            Style::default().fg(Color::Gray),
        )),
        footer_control_line(&[
            ("Tab", Color::Blue, "next pane"),
            ("Shift+Tab", Color::Blue, "previous pane"),
            ("h", Color::Magenta, "home"),
            ("Enter", Color::Magenta, "open handoff"),
        ]),
        footer_control_line(&[
            ("Up/Down", Color::Blue, "move"),
            ("Home/End", Color::Blue, "jump"),
            ("PgUp/PgDn", Color::Blue, "scroll detail"),
            ("/ ?", Color::Yellow, "search"),
            ("n", Color::Yellow, "repeat"),
            ("Esc/q", Color::Gray, "exit"),
        ]),
    ];

    #[cfg(feature = "tui")]
    {
        tui_shell::build_footer_controls(lines)
    }
    #[cfg(not(feature = "tui"))]
    {
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Status & Controls")
                    .style(Style::default().bg(Color::Rgb(16, 22, 30)))
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .style(Style::default().bg(Color::Rgb(16, 22, 30)).fg(Color::White))
    }
}

fn render_search_prompt(frame: &mut ratatui::Frame, search: &SearchPromptState) {
    let area = Rect {
        x: frame.area().x + 6,
        y: frame.area().y + frame.area().height.saturating_sub(6),
        width: frame.area().width.saturating_sub(12).min(70),
        height: 4,
    };
    frame.render_widget(Clear, area);
    let prefix = match search.direction {
        SearchDirection::Forward => "/",
        SearchDirection::Backward => "?",
    };
    let prompt = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!(" {prefix} "),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(164, 116, 19))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(search.query.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(Span::styled(
            "Enter search   Esc cancel   n repeat",
            Style::default().fg(Color::Gray),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Search {prefix}"))
            .style(Style::default().bg(Color::Rgb(18, 20, 26)))
            .border_style(Style::default().fg(Color::Yellow)),
    )
    .style(Style::default().bg(Color::Rgb(18, 20, 26)));
    frame.render_widget(prompt, area);
    let max_offset = area.width.saturating_sub(6) as usize;
    let offset = search.query.chars().count().min(max_offset) as u16;
    frame.set_cursor_position(Position::new(area.x + 5 + offset, area.y + 1));
}

fn footer_control_line(items: &[(&str, Color, &str)]) -> Line<'static> {
    #[cfg(feature = "tui")]
    {
        tui_shell::control_line(items)
    }
    #[cfg(not(feature = "tui"))]
    {
        aligned_control_line(items)
    }
}

fn focus_label(focus: ProjectStatusPane) -> &'static str {
    match focus {
        ProjectStatusPane::Home => "Home",
        ProjectStatusPane::Domains => "Domains",
        ProjectStatusPane::Details => "Details",
        ProjectStatusPane::Actions => "Actions",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::TOOL_VERSION;
    use crate::project_status::{
        ProjectDomainStatus, ProjectStatus, ProjectStatusAction, ProjectStatusFreshness,
        ProjectStatusOverall, ProjectStatusRankedFinding, PROJECT_STATUS_BLOCKED,
        PROJECT_STATUS_READY,
    };
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn interactive_render_uses_compact_exit_label_in_footer() {
        let mut state = ProjectStatusTuiState::new(sample_project_status());
        let mut terminal = Terminal::new(TestBackend::new(180, 40)).unwrap();

        terminal
            .draw(|frame| render_project_status_frame(frame, &mut state))
            .unwrap();

        let screen = format!("{}", terminal.backend());
        assert!(screen.contains("Status & Controls"));
        assert!(screen.contains("/ ?"));
        assert!(screen.contains("n"));
        assert!(screen.contains("Search idle"));
        assert!(screen.contains("Home/End"));
        assert!(screen.contains("Esc/q"));
        assert!(!screen.contains(" q "));
        assert!(!screen.contains(" Esc "));
    }

    #[test]
    fn interactive_render_surfaces_search_prompt() {
        let mut state = ProjectStatusTuiState::new(sample_project_status());
        state.start_search(SearchDirection::Forward);
        state.handle_search_key(crossterm::event::KeyCode::Char('s'));
        state.handle_search_key(crossterm::event::KeyCode::Char('y'));
        let mut terminal = Terminal::new(TestBackend::new(180, 40)).unwrap();

        terminal
            .draw(|frame| render_project_status_frame(frame, &mut state))
            .unwrap();

        let screen = format!("{}", terminal.backend());
        assert!(screen.contains("Search /"));
        assert!(screen.contains("sy"));
        assert!(screen.contains("Enter search"));
        assert!(screen.contains("Esc cancel"));
        assert!(screen.contains("n repeat"));
    }

    fn sample_project_status() -> ProjectStatus {
        ProjectStatus {
            schema_version: 1,
            tool_version: TOOL_VERSION.to_string(),
            discovery: None,
            scope: "live".to_string(),
            overall: ProjectStatusOverall {
                status: PROJECT_STATUS_BLOCKED.to_string(),
                domain_count: 2,
                present_count: 2,
                blocked_count: 1,
                blocker_count: 3,
                warning_count: 1,
                freshness: ProjectStatusFreshness {
                    status: "current".to_string(),
                    source_count: 2,
                    newest_age_seconds: Some(30),
                    oldest_age_seconds: Some(120),
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
                    warning_count: 1,
                    source_kinds: vec!["dashboard-export".to_string()],
                    signal_keys: vec![
                        "summary.dashboardCount".to_string(),
                        "summary.queryCount".to_string(),
                    ],
                    blockers: Vec::new(),
                    warnings: vec![crate::project_status::status_finding(
                        "risk-records",
                        1,
                        "summary.riskRecordCount",
                    )],
                    next_actions: vec![
                        "review dashboard governance warnings before promotion or apply"
                            .to_string(),
                    ],
                    freshness: ProjectStatusFreshness {
                        status: "stale".to_string(),
                        source_count: 1,
                        newest_age_seconds: Some(86_400),
                        oldest_age_seconds: Some(86_400),
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
                    source_kinds: vec!["sync-summary".to_string(), "bundle-preflight".to_string()],
                    signal_keys: vec![
                        "summary.resourceCount".to_string(),
                        "summary.syncBlockingCount".to_string(),
                    ],
                    blockers: vec![crate::project_status::status_finding(
                        "sync-blocking",
                        3,
                        "summary.syncBlockingCount",
                    )],
                    warnings: Vec::new(),
                    next_actions: vec![
                        "resolve sync workflow blockers in the fixed order: sync, provider"
                            .to_string(),
                    ],
                    freshness: ProjectStatusFreshness {
                        status: "current".to_string(),
                        source_count: 2,
                        newest_age_seconds: Some(15),
                        oldest_age_seconds: Some(45),
                    },
                },
            ],
            top_blockers: vec![ProjectStatusRankedFinding {
                domain: "sync".to_string(),
                kind: "sync-blocking".to_string(),
                count: 3,
                source: "summary.syncBlockingCount".to_string(),
            }],
            next_actions: vec![ProjectStatusAction {
                domain: "sync".to_string(),
                reason_code: "blocked-by-blockers".to_string(),
                action: "resolve sync workflow blockers in the fixed order: sync, provider"
                    .to_string(),
            }],
        }
    }
}
