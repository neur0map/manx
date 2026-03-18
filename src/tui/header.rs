use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use tui_big_text::{BigText, PixelSize};

use super::Theme;

pub fn render_header(frame: &mut Frame, area: Rect, query: &str, library: &str, theme: &Theme) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // big text banner
        Constraint::Length(1), // query info
    ])
    .split(area);

    // MANX banner using big text (half-height for compactness)
    let banner = BigText::builder()
        .pixel_size(PixelSize::HalfHeight)
        .style(theme.header_title)
        .lines(vec!["MANX".into()])
        .build();
    frame.render_widget(banner, chunks[0]);

    // Query info line
    let info = if query.is_empty() {
        Line::from(vec![
            Span::styled(library, theme.result_title),
        ])
    } else {
        Line::from(vec![
            Span::styled(library, theme.result_title),
            Span::styled(" > ", theme.dimmed),
            Span::styled(query, theme.header_title),
        ])
    };

    let info_bar = Paragraph::new(info);
    frame.render_widget(info_bar, chunks[1]);
}

pub fn render_compact_header(frame: &mut Frame, area: Rect, query: &str, library: &str, result_count: usize, theme: &Theme) {
    let info = Line::from(vec![
        Span::styled(" MANX ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" ", theme.dimmed),
        Span::styled(library, theme.result_title),
        Span::styled(" > ", theme.dimmed),
        Span::styled(query, theme.header_title),
        Span::styled(
            format!("  {} results", result_count),
            theme.dimmed,
        ),
    ]);

    let bar = Paragraph::new(info)
        .block(Block::default().borders(Borders::BOTTOM).border_style(theme.border));
    frame.render_widget(bar, area);
}
