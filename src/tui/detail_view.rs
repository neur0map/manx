use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

use super::results_view::ResultItem;
use super::Theme;

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
            self.scroll_offset = self
                .scroll_offset
                .saturating_add(amount)
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
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    }

    /// Render as a preview pane — title inside content, not in border
    pub fn render_preview(&self, frame: &mut Frame, area: Rect, item: &ResultItem, theme: &Theme) {
        let text = build_preview_text(item, theme);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border);

        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn build_preview_text<'a>(item: &ResultItem, theme: &Theme) -> Text<'a> {
    let mut lines = vec![];

    // Title as first line inside the pane (wraps naturally)
    lines.push(Line::from(Span::styled(
        item.title.clone(),
        theme.section_heading,
    )));
    lines.push(Line::from(""));

    // Source URL if available
    if let Some(url) = &item.url {
        lines.push(Line::from(Span::styled(url.clone(), theme.result_url)));
        lines.push(Line::from(""));
    }

    // Show full content, parsed with Context7 awareness
    render_content_lines(&item.full_content, theme, &mut lines);

    Text::from(lines)
}

fn build_detail_text<'a>(item: &ResultItem, theme: &Theme) -> Text<'a> {
    let mut lines = vec![];

    // Title
    lines.push(Line::from(Span::styled(
        item.title.clone(),
        theme.section_heading,
    )));

    if let Some(url) = &item.url {
        lines.push(Line::from(Span::styled(url.clone(), theme.result_url)));
    }
    lines.push(Line::from(""));

    // Full content
    render_content_lines(&item.full_content, theme, &mut lines);

    Text::from(lines)
}

/// Parse Context7 / markdown-ish content into styled lines
fn render_content_lines<'a>(content: &str, theme: &Theme, lines: &mut Vec<Line<'a>>) {
    let mut in_code_block = false;

    for line in content.lines() {
        // Code block toggles
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                let lang = line.trim_start_matches('`').trim();
                if !lang.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        format!("--- {} ---", lang),
                        theme.warning,
                    )));
                }
            } else {
                lines.push(Line::from(Span::styled(
                    "----------",
                    theme.dimmed,
                )));
                lines.push(Line::from(""));
            }
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                theme.code_block,
            )));
            continue;
        }

        // Context7 structured fields
        if let Some(title) = line.strip_prefix("TITLE: ") {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                title.to_string(),
                theme.section_heading,
            )));
        } else if let Some(desc) = line.strip_prefix("DESCRIPTION: ") {
            lines.push(Line::from(Span::styled(desc.to_string(), theme.dimmed)));
            lines.push(Line::from(""));
        } else if let Some(source) = line.strip_prefix("SOURCE: ") {
            lines.push(Line::from(Span::styled(
                source.to_string(),
                theme.result_url,
            )));
        } else if let Some(lang) = line.strip_prefix("LANGUAGE: ") {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("--- {} ---", lang),
                theme.warning,
            )));
        } else if line.starts_with("CODE:") {
            // Skip the "CODE:" label, the ``` follows
        } else if line.starts_with("====") {
            // Skip separator lines
        } else if line.starts_with("---") {
            lines.push(Line::from(""));
        } else if line.starts_with("# ") {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                line.trim_start_matches("# ").to_string(),
                theme.section_heading,
            )));
        } else if line.starts_with("## ") {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                line.trim_start_matches("## ").to_string(),
                theme.section_heading,
            )));
        } else if line.starts_with("- ") || line.starts_with("* ") {
            lines.push(Line::from(format!("  {}", line)));
        } else if line.trim().is_empty() {
            lines.push(Line::from(""));
        } else {
            lines.push(Line::from(line.to_string()));
        }
    }
}
