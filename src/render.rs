use crate::client::{CodeExample, DocSection, Documentation, SearchResult};
use crate::config::Config;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::io;

pub struct Renderer {
    quiet_mode: bool,
    terminal_width: usize,
    config: Option<Config>,
}

impl Renderer {
    pub fn new(quiet: bool) -> Self {
        let terminal_width = termsize::get().map(|size| size.cols as usize).unwrap_or(80);
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
            // JSON output for scripting
            println!("{}", serde_json::to_string_pretty(results)?);
            return Ok(());
        }

        if results.is_empty() {
            println!("{}", "No results found.".yellow());
            return Ok(());
        }

        println!(
            "{} {} found:",
            results.len().to_string().cyan().bold(),
            if results.len() == 1 {
                "result"
            } else {
                "results"
            }
        );

        if let Some((library_title, library_id)) = library_info {
            println!(
                "📚 Using library: {} ({})\n",
                library_title.bright_blue(),
                library_id.dimmed()
            );
        } else {
            println!();
        }

        // Use provided limit or default to 10 (0 means unlimited)
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
                "\n{}",
                format!(
                    "... and {} more results. Use --limit 0 to show all, or --save-all to export.",
                    total_results - display_limit
                )
                .yellow()
            );
        }

        println!(
            "\n{}",
            "Tip: Use 'manx snippet <id>' to expand a result.".dimmed()
        );
        Ok(())
    }

    fn render_search_result(&self, num: usize, result: &SearchResult) -> io::Result<()> {
        let separator = "─".repeat(self.terminal_width.min(60));

        println!(
            "{} {} {}",
            format!("[{}]", num).cyan().bold(),
            result.title.white().bold(),
            format!("({})", result.library).dimmed()
        );

        println!("  {}: {}", "ID".dimmed(), result.id.yellow());

        if let Some(url) = &result.url {
            println!("  {}: {}", "URL".dimmed(), url.blue().underline());
        }

        println!();

        // Parse and display Context7 content in a more readable format
        if result.excerpt.contains("CODE SNIPPETS") {
            self.render_context7_excerpt(&result.excerpt)?;
        } else {
            // Show more of the excerpt for better distinction
            let max_width = self.terminal_width.max(100) - 4;
            println!("  {}", self.truncate_text(&result.excerpt, max_width));
        }

        println!("{}\n", separator.dimmed());
        Ok(())
    }

    fn render_context7_excerpt(&self, content: &str) -> io::Result<()> {
        // Find the first meaningful content after CODE SNIPPETS header
        let lines: Vec<&str> = content.lines().collect();
        let mut found_title = false;

        for line in lines.iter().take(10) {
            // Only show first few lines for excerpt
            if line.starts_with("TITLE: ") && !found_title {
                let title = &line[7..];
                println!("  {}", title.white().bold());
                found_title = true;
            } else if line.starts_with("DESCRIPTION: ") && found_title {
                let desc = &line[13..];
                let truncated = self.truncate_text(desc, self.terminal_width - 4);
                println!("  {}", truncated.dimmed());
                break;
            }
        }

        if !found_title {
            println!("  {}", "Documentation snippets available...".dimmed());
        }

        Ok(())
    }

    pub fn render_documentation(&self, doc: &Documentation) -> io::Result<()> {
        if self.quiet_mode {
            println!("{}", serde_json::to_string_pretty(doc)?);
            return Ok(());
        }

        // Header
        println!(
            "\n📚 {} {}",
            doc.library.name.cyan().bold(),
            doc.library
                .version
                .as_ref()
                .map(|v| format!("v{}", v))
                .unwrap_or_default()
                .dimmed()
        );

        if let Some(desc) = &doc.library.description {
            println!("{}\n", desc.dimmed());
        }

        // Sections
        for section in &doc.sections {
            self.render_doc_section(section)?;
        }

        Ok(())
    }

    fn render_doc_section(&self, section: &DocSection) -> io::Result<()> {
        println!("\n{}", section.title.green().bold());

        if let Some(url) = &section.url {
            println!("{}: {}", "Source".dimmed(), url.blue().underline());
        }

        println!("\n{}", section.content);

        // Code examples
        for example in &section.code_examples {
            self.render_code_example(example)?;
        }

        Ok(())
    }

    fn render_code_example(&self, example: &CodeExample) -> io::Result<()> {
        println!(
            "\n{} {}:",
            "▶".cyan(),
            example
                .description
                .as_ref()
                .unwrap_or(&"Example".to_string())
                .yellow()
        );

        println!("{}", format!("```{}", example.language).dimmed());

        // Simple syntax highlighting for common languages
        let highlighted = self.highlight_code(&example.code, &example.language);
        println!("{}", highlighted);

        println!("{}", "```".dimmed());
        Ok(())
    }

    fn highlight_code(&self, code: &str, language: &str) -> String {
        if self.quiet_mode {
            return code.to_string();
        }

        // Basic syntax highlighting
        match language {
            "python" | "py" => self.highlight_python(code),
            "javascript" | "js" | "typescript" | "ts" => self.highlight_javascript(code),
            "rust" | "rs" => self.highlight_rust(code),
            _ => code.to_string(),
        }
    }

    fn highlight_python(&self, code: &str) -> String {
        let keywords = [
            "def", "class", "import", "from", "return", "if", "else", "elif", "for", "while", "in",
            "as", "with", "try", "except", "finally", "raise", "yield", "lambda",
        ];

        let mut highlighted = code.to_string();
        for keyword in &keywords {
            let _pattern = format!(r"\b{}\b", keyword);
            highlighted = highlighted.replace(keyword, &keyword.magenta().to_string());
        }
        highlighted
    }

    fn highlight_javascript(&self, code: &str) -> String {
        let keywords = [
            "function", "const", "let", "var", "return", "if", "else", "for", "while", "class",
            "extends", "import", "export", "async", "await", "try", "catch", "throw", "new",
        ];

        let mut highlighted = code.to_string();
        for keyword in &keywords {
            let _pattern = format!(r"\b{}\b", keyword);
            highlighted = highlighted.replace(keyword, &keyword.blue().to_string());
        }
        highlighted
    }

    fn highlight_rust(&self, code: &str) -> String {
        let keywords = [
            "fn", "let", "mut", "const", "use", "mod", "pub", "impl", "struct", "enum", "trait",
            "where", "async", "await", "match", "if", "else", "for", "while", "loop", "return",
        ];

        let mut highlighted = code.to_string();
        for keyword in &keywords {
            let _pattern = format!(r"\b{}\b", keyword);
            highlighted = highlighted.replace(keyword, &keyword.red().to_string());
        }
        highlighted
    }

    fn truncate_text(&self, text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            // Try to break at a word boundary
            let truncate_at = max_len - 3;
            if let Some(last_space) = text[..truncate_at].rfind(' ') {
                format!("{}...", &text[..last_space])
            } else {
                format!("{}...", &text[..truncate_at])
            }
        }
    }

    pub fn show_progress(&self, message: &str) -> ProgressBar {
        if self.quiet_mode {
            return ProgressBar::hidden();
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }

    pub fn print_error(&self, error: &str) {
        if self.quiet_mode {
            eprintln!("{{\"error\": \"{}\"}}", error);
        } else {
            eprintln!("{} {}", "✗".red().bold(), error.red());
        }
    }

    pub fn print_success(&self, message: &str) {
        if !self.quiet_mode {
            println!("{} {}", "✓".green().bold(), message.green());
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

        // Header
        println!(
            "\n{} {} {}",
            "📚".cyan().bold(),
            library.white().bold(),
            "Documentation".dimmed()
        );

        // Parse and render the Context7 format with limit
        self.parse_and_render_context7_content_with_limit(content, limit)?;

        // Cache individual sections for the open command
        let sections = self.extract_doc_sections(content);
        if self.cache_doc_sections(library, &sections).is_err() {
            // Silently continue if caching fails
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
        let section_limit = limit.unwrap_or(10); // Default to 10 sections

        while i < lines.len() {
            // Check if we've reached the limit (but only if limit is not 0, which means unlimited)
            if limit.is_some() && limit.unwrap() > 0 && sections_shown >= section_limit {
                let remaining = self.count_remaining_sections(&lines[i..]);
                if remaining > 0 {
                    println!(
                        "\n{}",
                        format!(
                            "... and {} more sections. Use --limit 0 to show all.",
                            remaining
                        )
                        .yellow()
                    );
                }
                break;
            }
            let line = lines[i];

            // Skip headers and separators
            if line.starts_with("========================") {
                if i + 1 < lines.len() && lines[i + 1].starts_with("CODE SNIPPETS") {
                    println!("\n{}", "📝 Code Examples & Snippets".green().bold());
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }

            // Parse title blocks
            if let Some(title) = line.strip_prefix("TITLE: ") {
                sections_shown += 1;
                println!(
                    "\n{} {}",
                    format!("[{}]", sections_shown).cyan().bold(),
                    title.white().bold()
                );
                i += 1;

                // Look for description
                if i < lines.len() && lines[i].starts_with("DESCRIPTION: ") {
                    let desc = &lines[i][13..];
                    println!("{}", desc.dimmed());
                    i += 1;
                }

                // Skip empty lines
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Look for source
                while i < lines.len() && lines[i].starts_with("SOURCE: ") {
                    let source = &lines[i][8..];
                    println!("{}: {}", "Source".dimmed(), source.blue());
                    i += 1;
                }

                // Skip empty lines
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Look for language and code block
                if i < lines.len() && lines[i].starts_with("LANGUAGE: ") {
                    let language = &lines[i][10..];
                    i += 1;

                    // Skip "CODE:" line
                    if i < lines.len() && lines[i].starts_with("CODE:") {
                        i += 1;
                    }

                    // Parse code block
                    if i < lines.len() && lines[i].starts_with("```") {
                        println!("\n{} {}:", "▶".cyan(), language.yellow());
                        println!("{}", lines[i].dimmed());
                        i += 1;

                        // Print code content
                        while i < lines.len() && !lines[i].starts_with("```") {
                            let highlighted =
                                self.highlight_code(lines[i], &language.to_lowercase());
                            println!("{}", highlighted);
                            i += 1;
                        }

                        // Print closing ```
                        if i < lines.len() && lines[i].starts_with("```") {
                            println!("{}", lines[i].dimmed());
                            i += 1;
                        }
                    }
                }

                // Skip separators
                while i < lines.len() && (lines[i].trim().is_empty() || lines[i].starts_with("---"))
                {
                    if lines[i].starts_with("---") {
                        let separator = "─".repeat(self.terminal_width.min(60));
                        println!("\n{}", separator.dimmed());
                    }
                    i += 1;
                }

                continue;
            }

            i += 1;
        }

        // Add tip message about opening sections
        if sections_shown > 0 {
            println!(
                "\n{}",
                "Tip: Use 'manx open <section-id>' to expand a specific section.".dimmed()
            );
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

            // Look for title blocks (start of a section)
            if let Some(_title) = line.strip_prefix("TITLE: ") {
                let section_start = i;
                let mut section_end = lines.len();

                // Find the end of this section (next TITLE or end of content)
                for (j, line) in lines.iter().enumerate().skip(i + 1) {
                    if line.starts_with("TITLE: ") {
                        section_end = j;
                        break;
                    }
                }

                // Extract the complete section
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

        println!(
            "\n{} {} {}",
            "📖".cyan().bold(),
            id.yellow().bold(),
            "Documentation Section".dimmed()
        );

        // Parse and render just this section
        self.render_single_section(content)?;

        Ok(())
    }

    fn render_single_section(&self, content: &str) -> io::Result<()> {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Skip headers and separators
            if line.starts_with("========================") {
                if i + 1 < lines.len() && lines[i + 1].starts_with("CODE SNIPPETS") {
                    println!("\n{}", "📝 Code Examples & Snippets".green().bold());
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }

            // Parse title blocks (but don't add numbering)
            if let Some(title) = line.strip_prefix("TITLE: ") {
                println!("\n{}", title.white().bold());
                i += 1;

                // Look for description
                if i < lines.len() && lines[i].starts_with("DESCRIPTION: ") {
                    let desc = &lines[i][13..];
                    println!("{}", desc.dimmed());
                    i += 1;
                }

                // Skip empty lines
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Look for source
                while i < lines.len() && lines[i].starts_with("SOURCE: ") {
                    let source = &lines[i][8..];
                    println!("{}: {}", "Source".dimmed(), source.blue());
                    i += 1;
                }

                // Skip empty lines
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Look for language and code block
                if i < lines.len() && lines[i].starts_with("LANGUAGE: ") {
                    let language = &lines[i][10..];
                    i += 1;

                    // Skip "CODE:" line
                    if i < lines.len() && lines[i].starts_with("CODE:") {
                        i += 1;
                    }

                    // Parse code block
                    if i < lines.len() && lines[i].starts_with("```") {
                        println!("\n{} {}:", "▶".cyan(), language.yellow());
                        println!("{}", lines[i].dimmed());
                        i += 1;

                        // Print code content
                        while i < lines.len() && !lines[i].starts_with("```") {
                            let highlighted =
                                self.highlight_code(lines[i], &language.to_lowercase());
                            println!("{}", highlighted);
                            i += 1;
                        }

                        // Print closing ```
                        if i < lines.len() && lines[i].starts_with("```") {
                            println!("{}", lines[i].dimmed());
                            i += 1;
                        }
                    }
                }

                // Skip separators
                while i < lines.len() && (lines[i].trim().is_empty() || lines[i].starts_with("---"))
                {
                    if lines[i].starts_with("---") {
                        let separator = "─".repeat(self.terminal_width.min(60));
                        println!("\n{}", separator.dimmed());
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
}
