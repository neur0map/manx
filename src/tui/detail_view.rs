use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};

use super::Theme;
use super::results_view::ResultItem;

pub struct DetailView {
    pub scroll_offset: u16,
    pub content_height: u16,
}

impl DetailView {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            content_height: 0,
        }
    }

    pub fn scroll_down(&mut self, amount: u16) {
        if self.content_height > 0 {
            self.scroll_offset = self.scroll_offset.saturating_add(amount)
                .min(self.content_height.saturating_sub(1));
        }
    }

    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, item: &ResultItem, theme: &Theme) {
        let text = build_detail_text(item, theme);
        self.content_height = text.lines.len() as u16;

        let block = Block::default()
            .title(format!(" {} ", item.title))
            .title_style(theme.header_title)
            .borders(Borders::ALL)
            .border_style(theme.border_focused);

        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0));

        frame.render_widget(paragraph, area);

        // Scrollbar
        if self.content_height > area.height {
            let mut scrollbar_state = ScrollbarState::new(self.content_height as usize)
                .position(self.scroll_offset as usize);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("^"))
                .end_symbol(Some("v"));
            frame.render_stateful_widget(
                scrollbar,
                area.inner(Margin { vertical: 1, horizontal: 0 }),
                &mut scrollbar_state,
            );
        }
    }

    /// Render as a preview pane (right side, showing selected item summary)
    pub fn render_preview(&self, frame: &mut Frame, area: Rect, item: &ResultItem, theme: &Theme) {
        let text = build_preview_text(item, theme);

        let block = Block::default()
            .title(" Preview ")
            .title_style(theme.dimmed)
            .borders(Borders::ALL)
            .border_style(theme.border);

        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

fn build_preview_text<'a>(item: &ResultItem, theme: &Theme) -> Text<'a> {
    let mut lines = vec![];

    // Title
    lines.push(Line::from(Span::styled(
        item.title.clone(),
        theme.result_title,
    )));
    lines.push(Line::from(""));

    // Library + ID
    lines.push(Line::from(vec![
        Span::styled("Library: ", theme.dimmed),
        Span::styled(item.library.clone(), theme.header_title),
    ]));
    lines.push(Line::from(vec![
        Span::styled("ID: ", theme.dimmed),
        Span::styled(item.id.clone(), theme.result_id),
    ]));

    if let Some(url) = &item.url {
        lines.push(Line::from(vec![
            Span::styled("URL: ", theme.dimmed),
            Span::styled(url.clone(), theme.result_url),
        ]));
    }

    lines.push(Line::from(""));

    // Excerpt
    for line in item.excerpt.lines() {
        lines.push(Line::from(Span::styled(
            line.to_string(),
            theme.result_excerpt,
        )));
    }

    Text::from(lines)
}

fn build_detail_text<'a>(item: &ResultItem, theme: &Theme) -> Text<'a> {
    let mut lines = vec![];

    // Metadata header
    lines.push(Line::from(vec![
        Span::styled("Library: ", theme.dimmed),
        Span::styled(item.library.clone(), theme.header_title),
        Span::styled("  ID: ", theme.dimmed),
        Span::styled(item.id.clone(), theme.result_id),
    ]));

    if let Some(url) = &item.url {
        lines.push(Line::from(vec![
            Span::styled("Source: ", theme.dimmed),
            Span::styled(url.clone(), theme.result_url),
        ]));
    }

    lines.push(Line::from(""));

    // Full content with basic markdown-like parsing
    let content = &item.full_content;
    let mut in_code_block = false;

    for line in content.lines() {
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            lines.push(Line::from(Span::styled(
                line.to_string(),
                theme.dimmed,
            )));
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                theme.code_block,
            )));
        } else if line.starts_with("# ") || line.starts_with("TITLE: ") {
            let heading = line.trim_start_matches("# ").trim_start_matches("TITLE: ");
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                heading.to_string(),
                theme.section_heading,
            )));
        } else if line.starts_with("## ") {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                line.trim_start_matches("## ").to_string(),
                theme.section_heading,
            )));
        } else if line.starts_with("DESCRIPTION: ") {
            lines.push(Line::from(Span::styled(
                line.trim_start_matches("DESCRIPTION: ").to_string(),
                theme.dimmed,
            )));
        } else if line.starts_with("SOURCE: ") {
            lines.push(Line::from(vec![
                Span::styled("Source: ", theme.dimmed),
                Span::styled(
                    line.trim_start_matches("SOURCE: ").to_string(),
                    theme.result_url,
                ),
            ]));
        } else if line.starts_with("LANGUAGE: ") {
            lines.push(Line::from(Span::styled(
                format!("Language: {}", line.trim_start_matches("LANGUAGE: ")),
                theme.warning,
            )));
        } else if line.starts_with("---") {
            lines.push(Line::from(Span::styled(
                "-".repeat(60),
                theme.dimmed,
            )));
        } else {
            lines.push(Line::from(line.to_string()));
        }
    }

    Text::from(lines)
}
