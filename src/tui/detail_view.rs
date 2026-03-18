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

fn build_preview_text<'a>(item: &ResultItem, theme: &Theme) -> Text<'a> {
    let mut lines = vec![];

    lines.push(Line::from(Span::styled(
        item.title.clone(),
        theme.section_heading,
    )));
    lines.push(Line::from(""));

    if let Some(url) = &item.url {
        lines.push(Line::from(Span::styled(url.clone(), theme.result_url)));
        lines.push(Line::from(""));
    }

    render_content_lines(&item.full_content, theme, &mut lines);

    Text::from(lines)
}

fn build_detail_text<'a>(item: &ResultItem, theme: &Theme) -> Text<'a> {
    let mut lines = vec![];

    lines.push(Line::from(Span::styled(
        item.title.clone(),
        theme.section_heading,
    )));

    if let Some(url) = &item.url {
        lines.push(Line::from(Span::styled(url.clone(), theme.result_url)));
    }
    lines.push(Line::from(""));

    render_content_lines(&item.full_content, theme, &mut lines);

    Text::from(lines)
}

/// Parse content into styled lines with inline markdown support
fn render_content_lines<'a>(content: &str, theme: &Theme, lines: &mut Vec<Line<'a>>) {
    let mut in_code_block = false;
    let mut collected_sources: Vec<String> = Vec::new();

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
                lines.push(Line::from(Span::styled("----------", theme.dimmed)));
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
            collected_sources.push(source.to_string());
        } else if line.starts_with("Sources:") {
            // LLM sources section header — collect what follows
            continue;
        } else if let Some(lang) = line.strip_prefix("LANGUAGE: ") {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("--- {} ---", lang),
                theme.warning,
            )));
        } else if line.starts_with("CODE:") {
            // Skip
        } else if line.starts_with("====") {
            // Skip
        } else if line.starts_with("---") && !line.starts_with("--- ") {
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
        } else if line.trim().is_empty() {
            lines.push(Line::from(""));
        } else {
            // Parse inline markdown: **bold**, [Source N], `code`
            let cleaned = strip_source_refs(line);
            let spans = parse_inline_markdown(&cleaned, theme);
            lines.push(Line::from(spans));
        }
    }

    // Render collected sources at the bottom
    if !collected_sources.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Sources", theme.section_heading)));
        for source in &collected_sources {
            lines.push(Line::from(Span::styled(
                format!("  {}", source),
                theme.result_url,
            )));
        }
    }
}

/// Strip [Source N] references from text, return cleaned string
fn strip_source_refs(text: &str) -> String {
    let mut result = text.to_string();
    // Remove patterns like [Source 1], [Source 2], [Source 3], etc.
    loop {
        if let Some(start) = result.find("[Source ") {
            if let Some(end) = result[start..].find(']') {
                result = format!("{}{}", &result[..start], &result[start + end + 1..]);
                continue;
            }
        }
        break;
    }
    // Clean up double spaces left behind
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    result
}

/// Parse **bold**, `code`, and bullet points into styled spans
fn parse_inline_markdown<'a>(text: &str, theme: &Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut remaining = text.to_string();

    // Handle list items with bold headers like "- **Key Points**"
    let is_list = remaining.starts_with("- ") || remaining.starts_with("* ");
    if is_list {
        spans.push(Span::styled(
            "  ".to_string(),
            Style::default(),
        ));
        remaining = remaining[2..].to_string();
    }

    // Handle numbered items like "1. **Key Points**"
    let numbered = parse_numbered_prefix(&remaining);
    if let Some((prefix, rest)) = numbered {
        spans.push(Span::styled(
            format!("  {}. ", prefix),
            theme.result_number,
        ));
        remaining = rest;
    }

    // Parse **bold** and `code` segments
    while !remaining.is_empty() {
        if let Some(bold_start) = remaining.find("**") {
            // Text before bold
            if bold_start > 0 {
                spans.push(Span::raw(remaining[..bold_start].to_string()));
            }
            let after_start = &remaining[bold_start + 2..];
            if let Some(bold_end) = after_start.find("**") {
                // Bold text
                let bold_text = &after_start[..bold_end];
                spans.push(Span::styled(
                    bold_text.to_string(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
                remaining = after_start[bold_end + 2..].to_string();
            } else {
                // Unmatched **, just output rest
                spans.push(Span::raw(remaining));
                break;
            }
        } else if let Some(code_start) = remaining.find('`') {
            if code_start > 0 {
                spans.push(Span::raw(remaining[..code_start].to_string()));
            }
            let after_start = &remaining[code_start + 1..];
            if let Some(code_end) = after_start.find('`') {
                let code_text = &after_start[..code_end];
                spans.push(Span::styled(
                    code_text.to_string(),
                    theme.code_block,
                ));
                remaining = after_start[code_end + 1..].to_string();
            } else {
                spans.push(Span::raw(remaining));
                break;
            }
        } else {
            spans.push(Span::raw(remaining));
            break;
        }
    }

    if spans.is_empty() {
        spans.push(Span::raw(text.to_string()));
    }

    spans
}

/// Try to parse "N. " prefix from a line, return (number, rest)
fn parse_numbered_prefix(text: &str) -> Option<(String, String)> {
    let trimmed = text.trim_start();
    let dot_pos = trimmed.find(". ")?;
    if dot_pos > 3 {
        return None; // Too many digits
    }
    let num_part = &trimmed[..dot_pos];
    if num_part.chars().all(|c| c.is_ascii_digit()) {
        Some((num_part.to_string(), trimmed[dot_pos + 2..].to_string()))
    } else {
        None
    }
}
