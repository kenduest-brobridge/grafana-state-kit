#![cfg(feature = "tui")]
#![cfg_attr(test, allow(dead_code))]

use ratatui::layout::Position;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};

use crate::tui_shell;

use super::super::overview_kind::parse_overview_artifact_kind;
use super::{OverviewWorkbenchState, SearchDirection, SearchPromptState};

fn section_color(kind: &str) -> Color {
    parse_overview_artifact_kind(kind)
        .map(|artifact_kind| artifact_kind.section_color())
        .unwrap_or(Color::Gray)
}

fn item_color(kind: &str) -> Color {
    match kind {
        "dashboard" => Color::Yellow,
        "datasource" => Color::Cyan,
        "alert" | "alert-rule" => Color::Red,
        "user" | "team" | "org" | "service-account" => Color::Green,
        "warning" | "violation" => Color::LightRed,
        "drift" => Color::LightBlue,
        "policy" => Color::Magenta,
        _ => Color::Gray,
    }
}

fn search_mode_label(state: &OverviewWorkbenchState) -> (&'static str, Color) {
    if state.pending_search.is_some() {
        ("search", Color::Yellow)
    } else {
        ("browse", Color::Green)
    }
}

fn footer_search_status(state: &OverviewWorkbenchState) -> String {
    let search = state.search_summary();
    if state.pending_search.is_some() {
        format!(
            "Mode=search   Search={search}   {} Enter/Esc returns to browse.",
            state.search_status
        )
    } else if search == "idle" {
        "Mode=browse   Search=idle   Use / or ? within the current view items. Esc/q exits."
            .to_string()
    } else {
        format!(
            "Mode=browse   Search={search}   {} Esc/q exits.",
            state.search_status
        )
    }
}

fn build_header_lines(state: &OverviewWorkbenchState) -> Vec<Line<'static>> {
    let (search_mode, search_mode_color) = search_mode_label(state);
    let mut lines = vec![
        Line::from(vec![
            tui_shell::label("Artifacts "),
            tui_shell::accent(
                state.document.summary.artifact_count.to_string(),
                Color::White,
            ),
            Span::raw("  "),
            tui_shell::label("Sections "),
            tui_shell::accent(state.document.sections.len().to_string(), Color::White),
            Span::raw("  "),
            tui_shell::focus_label("Focus "),
            tui_shell::key_chip(state.status_focus_label(), Color::Blue),
        ]),
        Line::from(vec![
            tui_shell::label("Section "),
            tui_shell::accent(
                state
                    .current_section()
                    .map(|section| section.label.clone())
                    .unwrap_or_else(|| "none".to_string()),
                state
                    .current_section()
                    .map(|section| section_color(&section.kind))
                    .unwrap_or(Color::Gray),
            ),
            Span::raw("  "),
            tui_shell::label("View "),
            tui_shell::accent(state.current_view_label(), Color::White),
            Span::raw("  "),
            tui_shell::label("Item "),
            tui_shell::accent(
                state
                    .selected_item()
                    .map(|item| item.title.clone())
                    .unwrap_or_else(|| "none".to_string()),
                Color::White,
            ),
        ]),
        Line::from(vec![
            tui_shell::label("Mode "),
            tui_shell::accent(search_mode, search_mode_color),
            Span::raw("  "),
            tui_shell::label("Search "),
            tui_shell::accent(state.search_summary(), Color::Yellow),
            Span::raw("  "),
            tui_shell::label("Status "),
            tui_shell::accent(state.search_status.clone(), Color::White),
        ]),
    ];
    lines.extend(
        state
            .project_home_lines()
            .into_iter()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::White)))),
    );
    lines
}

fn pane_block(title: &str, focused: bool, accent: Color) -> Block<'static> {
    tui_shell::pane_block(title, focused, accent, Color::Black)
}

pub(super) fn render_overview_frame(
    frame: &mut ratatui::Frame,
    state: &mut OverviewWorkbenchState,
) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(1),
            Constraint::Length(5),
        ])
        .split(frame.area());
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(18),
            Constraint::Percentage(22),
            Constraint::Percentage(27),
            Constraint::Percentage(33),
        ])
        .split(outer[1]);

    let header_lines = build_header_lines(state);
    frame.render_widget(tui_shell::build_header("Overview", header_lines), outer[0]);

    let section_items = state
        .document
        .sections
        .iter()
        .enumerate()
        .map(|(index, section)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>2}. ", index + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    section.label.clone(),
                    Style::default()
                        .fg(section_color(&section.kind))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {} views  {}", section.views.len(), section.subtitle),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect::<Vec<_>>();
    frame.render_stateful_widget(
        List::new(section_items)
            .block(pane_block(
                "Sections",
                state.focus == super::OverviewPane::Sections,
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
        panes[0],
        &mut state.section_state,
    );

    let view_items = state
        .current_section()
        .map(|section| {
            section
                .views
                .iter()
                .enumerate()
                .map(|(index, view)| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:>2}. ", index + 1),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            view.label.clone(),
                            Style::default()
                                .fg(Color::LightCyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("  {} items", view.items.len()),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let view_title = state
        .current_section()
        .map(|section| {
            format!(
                "Views {}/{}",
                state
                    .view_state
                    .selected()
                    .map(|index| index + 1)
                    .unwrap_or(0),
                section.views.len()
            )
        })
        .unwrap_or_else(|| "Views".to_string());
    frame.render_stateful_widget(
        List::new(view_items)
            .block(pane_block(
                &view_title,
                state.focus == super::OverviewPane::Views,
                Color::LightCyan,
            ))
            .highlight_symbol("▌ ")
            .repeat_highlight_symbol(true)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
        panes[1],
        &mut state.view_state,
    );

    let item_items = state
        .current_items()
        .iter()
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
                Span::styled(
                    format!("  {}", item.meta),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect::<Vec<_>>();
    let item_title = state
        .current_section()
        .map(|section| {
            format!(
                "Items {}/{}  {} / {}",
                state
                    .item_state
                    .selected()
                    .map(|index| index + 1)
                    .unwrap_or(0),
                state.current_items().len(),
                section.label,
                state.current_view_label()
            )
        })
        .unwrap_or_else(|| "Items".to_string());
    frame.render_stateful_widget(
        List::new(item_items)
            .block(pane_block(
                &item_title,
                state.focus == super::OverviewPane::Items,
                Color::Cyan,
            ))
            .highlight_symbol("▌ ")
            .repeat_highlight_symbol(true)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        panes[2],
        &mut state.item_state,
    );

    let detail_lines = state.current_detail_lines();
    let detail_title = state
        .selected_item()
        .map(|item| {
            format!(
                "Details {}/{} [{}]  line {}/{}",
                state
                    .item_state
                    .selected()
                    .map(|index| index + 1)
                    .unwrap_or(0),
                state.current_items().len(),
                item.kind,
                (state.detail_scroll as usize + 1).min(detail_lines.len().max(1)),
                detail_lines.len().max(1)
            )
        })
        .unwrap_or_else(|| "Details".to_string());
    let detail_items = detail_lines
        .iter()
        .map(|line| {
            ListItem::new(Line::from(Span::styled(
                line.clone(),
                Style::default().fg(Color::White),
            )))
        })
        .collect::<Vec<_>>();
    let mut detail_state = ListState::default();
    detail_state.select(Some(
        (state.detail_scroll as usize).min(detail_lines.len().saturating_sub(1)),
    ));
    frame.render_stateful_widget(
        List::new(detail_items)
            .block(pane_block(
                &detail_title,
                state.focus == super::OverviewPane::Details,
                Color::LightBlue,
            ))
            .highlight_symbol("▌ ")
            .repeat_highlight_symbol(true)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
        panes[3],
        &mut detail_state,
    );

    frame.render_widget(
        tui_shell::build_footer(
            vec![
                tui_shell::control_line(&[
                    ("Tab", Color::Blue, "next pane"),
                    ("Shift+Tab", Color::Blue, "previous pane"),
                    ("h", Color::Blue, "home"),
                    ("Enter", Color::Blue, "open handoff"),
                ]),
                tui_shell::control_line(&[
                    ("Up/Down", Color::Blue, "move"),
                    ("Home/End", Color::Blue, "jump"),
                    ("PgUp/PgDn", Color::Blue, "scroll detail"),
                    ("Esc/q", Color::Gray, "exit"),
                ]),
                tui_shell::control_line(&[
                    ("/", Color::Yellow, "search forward"),
                    ("?", Color::Yellow, "search backward"),
                    ("n", Color::Yellow, "repeat"),
                ]),
            ],
            footer_search_status(state),
        ),
        outer[2],
    );

    if let Some(search) = &state.pending_search {
        render_search_prompt(frame, search);
    }
}

fn render_search_prompt(frame: &mut ratatui::Frame, search: &SearchPromptState) {
    let title = match search.direction {
        SearchDirection::Forward => "Search /",
        SearchDirection::Backward => "Search ?",
    };
    let area = tui_shell::render_dialog_shell(frame, title, 60, 5, Color::Yellow);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::raw(search.query.clone())),
            Line::from(Span::styled(
                "Enter search   Esc cancel   n repeat",
                Style::default().fg(Color::Gray),
            )),
        ])
        .style(Style::default().fg(Color::White).bg(Color::Rgb(16, 22, 30))),
        area,
    );
    let offset = search
        .query
        .chars()
        .count()
        .min(area.width.saturating_sub(1) as usize) as u16;
    frame.set_cursor_position(Position::new(area.x + offset, area.y));
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render_state() -> OverviewWorkbenchState {
        OverviewWorkbenchState::new(super::super::tests::test_document())
    }

    #[test]
    fn render_copy_surfaces_search_controls_and_compact_exit_label() {
        let mut state = render_state();
        let mut terminal = Terminal::new(TestBackend::new(180, 40)).unwrap();

        terminal
            .draw(|frame| render_overview_frame(frame, &mut state))
            .unwrap();

        let screen = format!("{}", terminal.backend());
        assert!(screen.contains("Mode"));
        assert!(screen.contains("browse"));
        assert!(screen.contains("Search idle"));
        assert!(screen.contains("Esc/q"));
        assert!(screen.contains("search forward"));
        assert!(screen.contains("search backward"));
        assert!(screen.contains("repeat"));
    }

    #[test]
    fn render_search_prompt_uses_directional_title() {
        let mut state = render_state();
        state.pending_search = Some(SearchPromptState {
            direction: SearchDirection::Backward,
            query: "block".to_string(),
        });
        let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();

        terminal
            .draw(|frame| render_overview_frame(frame, &mut state))
            .unwrap();

        let screen = format!("{}", terminal.backend());
        assert!(screen.contains("Search ?"));
        assert!(screen.contains("Enter search"));
        assert!(screen.contains("Esc cancel"));
        assert!(screen.contains("n repeat"));
        assert!(!screen.contains("repeat last search"));
    }

    #[test]
    fn render_copy_surfaces_explicit_pending_search_state() {
        let mut state = render_state();
        state.pending_search = Some(SearchPromptState {
            direction: SearchDirection::Backward,
            query: "block".to_string(),
        });
        state.search_status = "Search backward within the current view items.".to_string();
        let mut terminal = Terminal::new(TestBackend::new(180, 40)).unwrap();

        terminal
            .draw(|frame| render_overview_frame(frame, &mut state))
            .unwrap();

        let screen = format!("{}", terminal.backend());
        assert!(screen.contains("Mode"));
        assert!(screen.contains("search"));
        assert!(screen.contains("Search prompt ?block"));
        assert!(screen.contains("Search ?"));
    }
}
