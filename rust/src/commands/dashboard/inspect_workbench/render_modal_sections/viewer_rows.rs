use crate::interactive_browser::browser_wrapped_labeled_detail_lines;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

pub(super) struct ViewerRenderRow {
    pub(super) logical_index: usize,
    pub(super) line: Line<'static>,
}

pub(super) fn viewer_rows(lines: Vec<String>, width: usize, wrapped: bool) -> Vec<ViewerRenderRow> {
    lines
        .into_iter()
        .enumerate()
        .flat_map(|(logical_index, line)| {
            if line.trim().is_empty() {
                return vec![ViewerRenderRow {
                    logical_index,
                    line: Line::from(""),
                }];
            }
            if let Some((label, value)) = line.split_once(':') {
                return browser_wrapped_labeled_detail_lines(
                    label,
                    value.trim(),
                    16,
                    width,
                    wrapped,
                )
                .into_iter()
                .map(|line| ViewerRenderRow {
                    logical_index,
                    line,
                })
                .collect::<Vec<_>>();
            }
            wrap_plain_viewer_line(&line, width, wrapped)
                .into_iter()
                .map(|line| ViewerRenderRow {
                    logical_index,
                    line,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn wrap_plain_viewer_line(line: &str, width: usize, wrapped: bool) -> Vec<Line<'static>> {
    if !wrapped || width == 0 {
        return vec![Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::White),
        ))];
    }
    crate::interactive_browser::wrap_text_chunks(line, width.max(1))
        .into_iter()
        .map(|chunk| Line::from(Span::styled(chunk, Style::default().fg(Color::White))))
        .collect()
}
