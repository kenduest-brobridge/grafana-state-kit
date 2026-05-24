//! Shared internal side-by-side review diff model.
//!
//! This stays document-shape based so sync/workspace and compatible domain
//! review rows can share visualization without changing public contracts.

use crate::common::{message, Result};
#[cfg(feature = "tui")]
use crate::tui_shell;
#[cfg(feature = "tui")]
use ratatui::style::{Color, Modifier, Style};
#[cfg(feature = "tui")]
use ratatui::text::{Line, Span};
#[cfg(feature = "tui")]
use ratatui::widgets::ListItem;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewDiffLine {
    pub changed: bool,
    pub marker: char,
    pub content: String,
    pub highlight_range: Option<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewDiffModel {
    pub title: String,
    pub action: String,
    pub live_lines: Vec<ReviewDiffLine>,
    pub desired_lines: Vec<ReviewDiffLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReviewDiffPaneFocus {
    Live,
    Desired,
}

pub(crate) struct ReviewDiffControlsState {
    pub selected: usize,
    pub total: usize,
    pub diff_focus: ReviewDiffPaneFocus,
    pub live_wrap_lines: bool,
    pub desired_wrap_lines: bool,
    pub live_diff_cursor: usize,
    pub live_horizontal_offset: usize,
    pub desired_diff_cursor: usize,
    pub desired_horizontal_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewDiffInput<'a> {
    pub title: String,
    pub action: String,
    pub live: Option<&'a Map<String, Value>>,
    pub desired: Option<&'a Map<String, Value>>,
    pub changed_fields: Vec<String>,
}

type HighlightRange = Option<(usize, usize)>;

impl<'a> ReviewDiffInput<'a> {
    pub(crate) fn from_operation(operation: &'a Value) -> Result<Self> {
        let object = operation
            .as_object()
            .ok_or_else(|| message("Review diff operation must be a JSON object."))?;
        let action = object
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let title = format!(
            "{} {}",
            object
                .get("resourceKind")
                .or_else(|| object.get("kind"))
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            object
                .get("identity")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        );
        let changed_fields = object
            .get("changedFields")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect::<Vec<_>>();

        Ok(Self {
            title,
            action,
            live: object.get("live").and_then(Value::as_object),
            desired: object.get("desired").and_then(Value::as_object),
            changed_fields,
        })
    }
}

fn pretty_inline_json(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "null".to_string(),
        Some(Value::String(text)) => format!("{text:?}"),
        Some(other) => serde_json::to_string(other).unwrap_or_else(|_| "null".to_string()),
    }
}

fn numbered_line(index: usize, content: String) -> String {
    format!("{:>3} | {content}", index + 1)
}

fn diff_highlight_ranges(left: &str, right: &str) -> (HighlightRange, HighlightRange) {
    if left == right {
        return (None, None);
    }
    let left_bytes = left.as_bytes();
    let right_bytes = right.as_bytes();
    let mut prefix = 0usize;
    let min_len = left_bytes.len().min(right_bytes.len());
    while prefix < min_len && left_bytes[prefix] == right_bytes[prefix] {
        prefix += 1;
    }

    let mut left_suffix = left_bytes.len();
    let mut right_suffix = right_bytes.len();
    while left_suffix > prefix
        && right_suffix > prefix
        && left_bytes[left_suffix - 1] == right_bytes[right_suffix - 1]
    {
        left_suffix -= 1;
        right_suffix -= 1;
    }

    let left_range = if prefix == left_suffix {
        None
    } else {
        Some((prefix, left_suffix))
    };
    let right_range = if prefix == right_suffix {
        None
    } else {
        Some((prefix, right_suffix))
    };
    (left_range, right_range)
}

pub(crate) fn build_review_diff_model(input: ReviewDiffInput<'_>) -> Result<ReviewDiffModel> {
    let fields = if input.changed_fields.is_empty() {
        let mut combined = BTreeSet::new();
        if let Some(object) = input.live {
            combined.extend(object.keys().cloned());
        }
        if let Some(object) = input.desired {
            combined.extend(object.keys().cloned());
        }
        combined.into_iter().collect::<Vec<_>>()
    } else {
        input.changed_fields
    };
    if fields.is_empty() {
        return Ok(ReviewDiffModel {
            title: input.title,
            action: input.action,
            live_lines: vec![ReviewDiffLine {
                changed: false,
                marker: '=',
                content: numbered_line(0, "<no managed field changes>".to_string()),
                highlight_range: None,
            }],
            desired_lines: vec![ReviewDiffLine {
                changed: false,
                marker: '=',
                content: numbered_line(0, "<no managed field changes>".to_string()),
                highlight_range: None,
            }],
        });
    }
    let mut ordered_fields = fields
        .into_iter()
        .map(|field| {
            let live_value = input.live.and_then(|object| object.get(&field));
            let desired_value = input.desired.and_then(|object| object.get(&field));
            let changed = live_value != desired_value;
            (field, changed, live_value, desired_value)
        })
        .collect::<Vec<_>>();
    ordered_fields.sort_by_key(|(_, changed, _, _)| if *changed { 0 } else { 1 });

    let mut live_lines = Vec::new();
    let mut desired_lines = Vec::new();
    for (index, (field, changed, live_value, desired_value)) in
        ordered_fields.into_iter().enumerate()
    {
        let live_value_text = pretty_inline_json(live_value);
        let desired_value_text = pretty_inline_json(desired_value);
        let (live_range, desired_range) =
            diff_highlight_ranges(&live_value_text, &desired_value_text);
        let base_prefix = format!("{field}: ");
        let live_content = numbered_line(index, format!("{base_prefix}{live_value_text}"));
        let desired_content = numbered_line(index, format!("{base_prefix}{desired_value_text}"));
        let value_offset = numbered_line(index, base_prefix).len();

        live_lines.push(ReviewDiffLine {
            changed,
            marker: if changed { '-' } else { '=' },
            content: live_content,
            highlight_range: live_range
                .map(|(start, end)| (value_offset + start, value_offset + end)),
        });
        desired_lines.push(ReviewDiffLine {
            changed,
            marker: if changed { '+' } else { '=' },
            content: desired_content,
            highlight_range: desired_range
                .map(|(start, end)| (value_offset + start, value_offset + end)),
        });
    }

    Ok(ReviewDiffModel {
        title: input.title,
        action: input.action,
        live_lines,
        desired_lines,
    })
}

pub(crate) fn wrap_text_chunks(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    if text.is_empty() {
        return vec![String::new()];
    }
    let chars = text.chars().collect::<Vec<_>>();
    chars
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

pub(crate) fn clip_text_window(text: &str, offset: usize, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    text.chars().skip(offset).take(width).collect::<String>()
}

#[cfg(feature = "tui")]
pub(crate) fn render_review_diff_items(
    lines: &[ReviewDiffLine],
    color: Color,
    content_width: usize,
    wrap_lines: bool,
    horizontal_offset: usize,
) -> Vec<ListItem<'static>> {
    lines
        .iter()
        .map(|line| {
            let marker_style = if line.changed {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let content_style = if line.changed {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let wrapped = if wrap_lines {
                wrap_text_chunks(&line.content, content_width.max(1))
            } else {
                vec![clip_text_window(
                    &line.content,
                    horizontal_offset,
                    content_width.max(1),
                )]
            };
            let body = wrapped
                .into_iter()
                .enumerate()
                .map(|(index, chunk)| {
                    let marker = if index == 0 {
                        format!("{} ", line.marker)
                    } else {
                        "  ".to_string()
                    };
                    let marker_span = Span::styled(marker, marker_style);
                    let visible_highlight = if wrap_lines || index > 0 {
                        None
                    } else {
                        line.highlight_range.and_then(|(start, end)| {
                            let visible_start = start.max(horizontal_offset);
                            let visible_end = end.min(horizontal_offset + content_width.max(1));
                            if visible_start < visible_end {
                                Some((
                                    visible_start.saturating_sub(horizontal_offset),
                                    visible_end.saturating_sub(horizontal_offset),
                                ))
                            } else {
                                None
                            }
                        })
                    };
                    let content_span = match visible_highlight {
                        Some((start, end)) if start < end && end <= chunk.len() => {
                            let prefix = chunk[..start].to_string();
                            let middle = chunk[start..end].to_string();
                            let suffix = chunk[end..].to_string();
                            let mut spans = vec![marker_span, Span::raw(prefix)];
                            spans.push(Span::styled(
                                middle,
                                Style::default()
                                    .fg(color)
                                    .bg(Color::Black)
                                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                            ));
                            spans.push(Span::raw(suffix));
                            return Line::from(spans);
                        }
                        Some(_) if line.changed && index == 0 => Span::styled(
                            chunk,
                            Style::default()
                                .fg(color)
                                .bg(Color::Black)
                                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        ),
                        _ if line.changed => Span::styled(chunk, content_style),
                        _ => Span::raw(chunk),
                    };
                    Line::from(vec![marker_span, content_span])
                })
                .collect::<Vec<_>>();
            ListItem::new(body)
        })
        .collect()
}

pub(crate) fn review_diff_pane_title(
    pane: &str,
    action: &str,
    title: &str,
    position: usize,
    total: usize,
) -> String {
    format!("{pane} {}/{} [{}] {title}", position + 1, total, action)
}

pub(crate) fn review_diff_scroll_max(model: &ReviewDiffModel, focus: ReviewDiffPaneFocus) -> usize {
    match focus {
        ReviewDiffPaneFocus::Live => model.live_lines.len().saturating_sub(1),
        ReviewDiffPaneFocus::Desired => model.desired_lines.len().saturating_sub(1),
    }
}

#[cfg(feature = "tui")]
pub(crate) fn build_review_diff_controls_lines(
    state: &ReviewDiffControlsState,
) -> Vec<Line<'static>> {
    let focus = match state.diff_focus {
        ReviewDiffPaneFocus::Live => "LIVE",
        ReviewDiffPaneFocus::Desired => "DESIRED",
    };
    vec![
        Line::from(vec![
            tui_shell::label("Item "),
            tui_shell::accent(
                format!("{}/{}", state.selected + 1, state.total),
                Color::White,
            ),
            Span::raw("  "),
            tui_shell::focus_label("Focus "),
            tui_shell::key_chip(focus, Color::Blue),
            Span::raw("  "),
            Span::styled(
                format!(
                    "Live wrap {}",
                    if state.live_wrap_lines { "ON" } else { "OFF" }
                ),
                Style::default().fg(Color::Red),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "Desired wrap {}",
                    if state.desired_wrap_lines {
                        "ON"
                    } else {
                        "OFF"
                    }
                ),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled(
                "w active  W both".to_string(),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("  "),
            Span::styled("Left/Right pan", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "Live {} @{}",
                    state.live_diff_cursor + 1,
                    state.live_horizontal_offset
                ),
                Style::default().fg(Color::Red),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "Desired {} @{}",
                    state.desired_diff_cursor + 1,
                    state.desired_horizontal_offset
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        tui_shell::control_line(&[
            ("Tab", Color::Blue, "switch pane"),
            ("Up/Down", Color::Blue, "scroll"),
            ("[/]", Color::Blue, "item"),
            ("PgUp/PgDn", Color::Blue, "jump"),
        ]),
        tui_shell::control_line(&[
            ("Home/End", Color::Blue, "bounds"),
            ("Space", Color::Yellow, "keep/drop"),
            ("c", Color::Green, "confirm staged"),
            ("Esc/q", Color::Gray, "return"),
        ]),
    ]
}
