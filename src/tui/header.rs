use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::Theme;

pub fn render_compact_header(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    library: &str,
    result_count: usize,
    theme: &Theme,
) {
    let info = Line::from(vec![
        Span::styled(
            " MANX ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", theme.dimmed),
        Span::styled(library, theme.result_title),
        if !query.is_empty() {
            Span::styled(format!(" > {}", query), theme.header_title)
        } else {
            Span::raw("")
        },
        Span::styled(format!("  {} results", result_count), theme.dimmed),
    ]);

    let bar = Paragraph::new(info)
        .block(Block::default().borders(Borders::BOTTOM).border_style(theme.border));
    frame.render_widget(bar, area);
}
