use crate::client::{CodeExample, DocSection, Documentation, SearchResult};
use crate::config::Config;
use anyhow::Result;
use std::io;

/// Renderer handles non-interactive output (JSON/quiet mode, piped output).
/// Interactive display is handled by the TUI module.
pub struct Renderer {
    quiet_mode: bool,
    terminal_width: usize,
    config: Option<Config>,
}

impl Renderer {
    pub fn new(quiet: bool) -> Self {
        let terminal_width = crossterm::terminal::size()
            .map(|(cols, _)| cols as usize)
            .unwrap_or(80);
        let config = Config::load().ok();

        Self {
            quiet_mode: quiet,
            terminal_width,
            config,
        }
    }

    pub fn render_search_results(&self, results: &[SearchResult]) -> io::Result<()> {
        self.render_search_results_with_library(results, None, None)
    }

    pub fn render_search_results_with_library(
        &self,
        results: &[SearchResult],
        library_info: Option<(&str, &str)>,
        limit: Option<usize>,
    ) -> io::Result<()> {
        if self.quiet_mode {
            println!("{}", serde_json::to_string_pretty(results)?);
            return Ok(());
        }

        if results.is_empty() {
            println!("No results found.");
            return Ok(());
        }

        let count_label = if results.len() == 1 { "Result" } else { "Results" };
        println!("{}: {}", count_label, results.len());

        if let Some((library_title, library_id)) = library_info {
            println!("Using library: {} ({})\n", library_title, library_id);
        } else {
            println!();
        }

        let display_limit = limit.unwrap_or(10);
        let total_results = results.len();
        let results_to_show = if display_limit == 0 {
            results.iter().take(total_results)
        } else {
            results.iter().take(display_limit)
        };

        for (idx, result) in results_to_show.enumerate() {
            self.render_search_result(idx + 1, result)?;
        }

        if display_limit > 0 && total_results > display_limit {
            println!(
                "\n... and {} more results. Use --limit 0 to show all, or --save-all to export.",
                total_results - display_limit
            );
        }

        Ok(())
    }

    fn render_search_result(&self, num: usize, result: &SearchResult) -> io::Result<()> {
        let separator = "-".repeat(self.terminal_width.min(70));
        let title = Self::strip_emojis(&result.title);

        println!("[{}] {} ({})", num, title, result.library);
        println!("  ID: {}", result.id);

        if let Some(url) = &result.url {
            println!("  URL: {}", url);
        }

        println!();

        let excerpt = Self::strip_emojis(&result.excerpt);
        if excerpt.contains("CODE SNIPPETS") {
            self.render_context7_excerpt(&excerpt)?;
        } else {
            let max_width = self.terminal_width.saturating_sub(4).max(60);
            let text = self.truncate_text(&excerpt, max_width);
            println!("  {}", text);
        }

        println!("{}\n", separator);
        Ok(())
    }

    fn render_context7_excerpt(&self, content: &str) -> io::Result<()> {
        let lines: Vec<&str> = content.lines().collect();
        let mut found_title = false;

        for line in lines.iter().take(10) {
            if line.starts_with("TITLE: ") && !found_title {
                let title = &line[7..];
                println!("  {}", title);
                found_title = true;
            } else if line.starts_with("DESCRIPTION: ") && found_title {
                let desc = &line[13..];
                let truncated = self.truncate_text(desc, self.terminal_width - 4);
                println!("  {}", truncated);
                break;
            }
        }

        if !found_title {
            println!("  Documentation snippets available...");
        }

        Ok(())
    }

    pub fn render_documentation(&self, doc: &Documentation) -> io::Result<()> {
        if self.quiet_mode {
            println!("{}", serde_json::to_string_pretty(doc)?);
            return Ok(());
        }

        println!(
            "\n{} {}",
            doc.library.name,
            doc.library
                .version
                .as_ref()
                .map(|v| format!("v{}", v))
                .unwrap_or_default()
        );

        if let Some(desc) = &doc.library.description {
            println!("{}\n", desc);
        }

        for section in &doc.sections {
            self.render_doc_section(section)?;
        }

        Ok(())
    }

    fn render_doc_section(&self, section: &DocSection) -> io::Result<()> {
        let title = Self::strip_emojis(&section.title);
        println!("\n{}", title);

        if let Some(url) = &section.url {
            println!("Source: {}", url);
        }

        let content = Self::strip_emojis(&section.content);
        println!("\n{}", content);

        for example in &section.code_examples {
            self.render_code_example(example)?;
        }

        Ok(())
    }

    fn render_code_example(&self, example: &CodeExample) -> io::Result<()> {
        println!(
            "\n> {}:",
            example
                .description
                .as_ref()
                .unwrap_or(&"Example".to_string())
        );
        println!("```{}", example.language);
        println!("{}", example.code);
        println!("```");
        Ok(())
    }

    pub fn show_progress(&self, message: &str) -> ProgressHandle {
        if self.quiet_mode {
            return ProgressHandle::hidden();
        }
        eprint!("{}", message);
        ProgressHandle { active: true }
    }

    pub fn print_error(&self, error: &str) {
        if self.quiet_mode {
            eprintln!("{{\"error\": \"{}\"}}", error);
        } else {
            eprintln!("ERROR: {}", error);
        }
    }

    pub fn print_success(&self, message: &str) {
        if !self.quiet_mode {
            println!("OK {}", message);
        }
    }

    pub fn render_context7_documentation(&self, library: &str, content: &str) -> io::Result<()> {
        self.render_context7_documentation_with_limit(library, content, None)
    }

    pub fn render_context7_documentation_with_limit(
        &self,
        library: &str,
        content: &str,
        limit: Option<usize>,
    ) -> io::Result<()> {
        if self.quiet_mode {
            println!("{}", content);
            return Ok(());
        }

        println!("\n{} Documentation", library);

        let clean_content = Self::strip_emojis(content);
        self.parse_and_render_context7_content_with_limit(&clean_content, limit)?;

        let sections = self.extract_doc_sections(content);
        if self.cache_doc_sections(library, &sections).is_err() {
            // Silently continue
        }

        Ok(())
    }

    fn parse_and_render_context7_content_with_limit(
        &self,
        content: &str,
        limit: Option<usize>,
    ) -> io::Result<()> {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut sections_shown = 0;
        let section_limit = limit.unwrap_or(10);

        while i < lines.len() {
            if limit.is_some() && limit.unwrap() > 0 && sections_shown >= section_limit {
                let remaining = self.count_remaining_sections(&lines[i..]);
                if remaining > 0 {
                    println!(
                        "\n... and {} more sections. Use --limit 0 to show all.",
                        remaining
                    );
                }
                break;
            }
            let line = lines[i];

            if line.starts_with("========================") {
                if i + 1 < lines.len() && lines[i + 1].starts_with("CODE SNIPPETS") {
                    println!("\nCode Examples & Snippets");
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }

            if let Some(title) = line.strip_prefix("TITLE: ") {
                sections_shown += 1;
                println!("\n[{}] {}", sections_shown, title);
                i += 1;

                if i < lines.len() && lines[i].starts_with("DESCRIPTION: ") {
                    println!("{}", &lines[i][13..]);
                    i += 1;
                }

                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                while i < lines.len() && lines[i].starts_with("SOURCE: ") {
                    println!("Source: {}", &lines[i][8..]);
                    i += 1;
                }

                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                if i < lines.len() && lines[i].starts_with("LANGUAGE: ") {
                    let language = &lines[i][10..];
                    i += 1;

                    if i < lines.len() && lines[i].starts_with("CODE:") {
                        i += 1;
                    }

                    if i < lines.len() && lines[i].starts_with("```") {
                        println!("\n> {}:", language);
                        println!("{}", lines[i]);
                        i += 1;

                        while i < lines.len() && !lines[i].starts_with("```") {
                            println!("{}", lines[i]);
                            i += 1;
                        }

                        if i < lines.len() && lines[i].starts_with("```") {
                            println!("{}", lines[i]);
                            i += 1;
                        }
                    }
                }

                while i < lines.len()
                    && (lines[i].trim().is_empty() || lines[i].starts_with("---"))
                {
                    if lines[i].starts_with("---") {
                        println!("\n{}", "-".repeat(self.terminal_width.min(60)));
                    }
                    i += 1;
                }

                continue;
            }

            i += 1;
        }

        Ok(())
    }

    fn count_remaining_sections(&self, lines: &[&str]) -> usize {
        lines
            .iter()
            .filter(|line| line.starts_with("TITLE: "))
            .count()
    }

    fn extract_doc_sections(&self, content: &str) -> Vec<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut sections = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            if let Some(_title) = line.strip_prefix("TITLE: ") {
                let section_start = i;
                let mut section_end = lines.len();

                for (j, line) in lines.iter().enumerate().skip(i + 1) {
                    if line.starts_with("TITLE: ") {
                        section_end = j;
                        break;
                    }
                }

                let section_lines = &lines[section_start..section_end];
                let section_content = section_lines.join("\n").trim().to_string();

                if !section_content.is_empty() {
                    sections.push(section_content);
                }

                i = section_end;
            } else {
                i += 1;
            }
        }

        sections
    }

    pub fn render_open_section(&self, id: &str, content: &str) -> io::Result<()> {
        if self.quiet_mode {
            println!("{}", content);
            return Ok(());
        }

        println!("\n{} - Documentation Section", id);
        self.render_single_section(content)?;

        Ok(())
    }

    fn render_single_section(&self, content: &str) -> io::Result<()> {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            if line.starts_with("========================") {
                if i + 1 < lines.len() && lines[i + 1].starts_with("CODE SNIPPETS") {
                    println!("\nCode Examples & Snippets");
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }

            if let Some(title) = line.strip_prefix("TITLE: ") {
                println!("\n{}", title);
                i += 1;

                if i < lines.len() && lines[i].starts_with("DESCRIPTION: ") {
                    println!("{}", &lines[i][13..]);
                    i += 1;
                }

                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                while i < lines.len() && lines[i].starts_with("SOURCE: ") {
                    println!("Source: {}", &lines[i][8..]);
                    i += 1;
                }

                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                if i < lines.len() && lines[i].starts_with("LANGUAGE: ") {
                    let language = &lines[i][10..];
                    i += 1;

                    if i < lines.len() && lines[i].starts_with("CODE:") {
                        i += 1;
                    }

                    if i < lines.len() && lines[i].starts_with("```") {
                        println!("\n> {}:", language);
                        println!("{}", lines[i]);
                        i += 1;

                        while i < lines.len() && !lines[i].starts_with("```") {
                            println!("{}", lines[i]);
                            i += 1;
                        }

                        if i < lines.len() && lines[i].starts_with("```") {
                            println!("{}", lines[i]);
                            i += 1;
                        }
                    }
                }

                while i < lines.len()
                    && (lines[i].trim().is_empty() || lines[i].starts_with("---"))
                {
                    if lines[i].starts_with("---") {
                        println!("\n{}", "-".repeat(self.terminal_width.min(60)));
                    }
                    i += 1;
                }

                continue;
            }

            i += 1;
        }

        Ok(())
    }

    fn cache_doc_sections(&self, library: &str, sections: &[String]) -> Result<()> {
        if let Some(config) = &self.config {
            if config.auto_cache_enabled {
                if let Ok(cache_manager) = crate::cache::CacheManager::new() {
                    let library_clean = library.to_string();
                    let sections_clone = sections.to_vec();

                    tokio::spawn(async move {
                        for (idx, section) in sections_clone.iter().enumerate() {
                            let cache_key = format!("{}_doc-{}", library_clean, idx + 1);
                            let _ = cache_manager.set("doc_sections", &cache_key, section).await;
                        }
                    });
                }
            }
        }
        Ok(())
    }

    pub fn strip_emojis(text: &str) -> String {
        text.chars()
            .filter(|c| {
                let cp = *c as u32;
                cp < 0x2600
                    || (cp >= 0x2700 && cp < 0x2800)
                    || (cp >= 0x2000 && cp < 0x2100)
                    || (cp >= 0x2100 && cp < 0x2200)
                    || matches!(cp, 0x2500..=0x257F)
            })
            .filter(|c| {
                let cp = *c as u32;
                !matches!(cp,
                    0x1F300..=0x1F9FF
                    | 0x2600..=0x26FF
                    | 0xFE00..=0xFE0F
                    | 0x200D
                    | 0x20E3
                    | 0x2702..=0x27B0
                    | 0x2934..=0x2935
                    | 0x25AA..=0x25AB
                    | 0x25B6 | 0x25C0
                    | 0x25FB..=0x25FE
                    | 0x2B05..=0x2B07
                    | 0x2B1B..=0x2B1C
                    | 0x2B50 | 0x2B55
                    | 0x3030 | 0x303D
                    | 0x3297 | 0x3299
                    | 0x2139
                    | 0x2328
                    | 0x23CF
                    | 0x23E9..=0x23F3
                    | 0x23F8..=0x23FA
                )
            })
            .collect::<String>()
            .replace("  ", " ")
            .trim()
            .to_string()
    }

    fn truncate_text(&self, text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            let truncate_at = max_len - 3;
            if let Some(last_space) = text[..truncate_at].rfind(' ') {
                format!("{}...", &text[..last_space])
            } else {
                format!("{}...", &text[..truncate_at])
            }
        }
    }
}

/// Simple progress handle replacing indicatif::ProgressBar
pub struct ProgressHandle {
    active: bool,
}

impl ProgressHandle {
    pub fn hidden() -> Self {
        Self { active: false }
    }

    pub fn finish_and_clear(&self) {
        if self.active {
            eprint!("\r\x1b[K"); // clear line
        }
    }

    pub fn set_message(&self, _msg: impl Into<String>) {}

    pub fn finish_with_message(&self, _msg: impl Into<String>) {
        if self.active {
            eprint!("\r\x1b[K");
        }
    }
}
