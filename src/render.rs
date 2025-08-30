use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::io;
use termsize;
use crate::client::{SearchResult, Documentation, DocSection, CodeExample};

pub struct Renderer {
    quiet_mode: bool,
    terminal_width: usize,
}

impl Renderer {
    pub fn new(quiet: bool) -> Self {
        let terminal_width = termsize::get()
            .map(|size| size.cols as usize)
            .unwrap_or(80);
        
        Self {
            quiet_mode: quiet,
            terminal_width,
        }
    }
    
    pub fn render_search_results(&self, results: &[SearchResult]) -> io::Result<()> {
        if self.quiet_mode {
            // JSON output for scripting
            println!("{}", serde_json::to_string_pretty(results)?);
            return Ok(());
        }
        
        if results.is_empty() {
            println!("{}", "No results found.".yellow());
            return Ok(());
        }
        
        println!("\n{} {} found:\n", 
            results.len().to_string().cyan().bold(),
            if results.len() == 1 { "result" } else { "results" }
        );
        
        for (idx, result) in results.iter().enumerate() {
            self.render_search_result(idx + 1, result)?;
        }
        
        println!("\n{}", "Tip: Use 'manx snippet <id>' to expand a result.".dimmed());
        Ok(())
    }
    
    fn render_search_result(&self, num: usize, result: &SearchResult) -> io::Result<()> {
        let separator = "─".repeat(self.terminal_width.min(60));
        
        println!("{} {} {}", 
            format!("[{}]", num).cyan().bold(),
            result.title.white().bold(),
            format!("({})", result.library).dimmed()
        );
        
        println!("  {}: {}", "ID".dimmed(), result.id.yellow());
        
        if let Some(url) = &result.url {
            println!("  {}: {}", "URL".dimmed(), url.blue().underline());
        }
        
        println!("\n  {}", 
            self.truncate_text(&result.excerpt, self.terminal_width - 4)
        );
        
        println!("{}\n", separator.dimmed());
        Ok(())
    }
    
    pub fn render_documentation(&self, doc: &Documentation) -> io::Result<()> {
        if self.quiet_mode {
            println!("{}", serde_json::to_string_pretty(doc)?);
            return Ok(());
        }
        
        // Header
        println!("\n{} {} {}",
            "📚".to_string(),
            doc.library.name.cyan().bold(),
            doc.library.version
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
        println!("\n{} {}:",
            "▶".cyan(),
            example.description
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
        let keywords = ["def", "class", "import", "from", "return", "if", "else", 
                        "elif", "for", "while", "in", "as", "with", "try", 
                        "except", "finally", "raise", "yield", "lambda"];
        
        let mut highlighted = code.to_string();
        for keyword in &keywords {
            let _pattern = format!(r"\b{}\b", keyword);
            highlighted = highlighted.replace(keyword, &keyword.magenta().to_string());
        }
        highlighted
    }
    
    fn highlight_javascript(&self, code: &str) -> String {
        let keywords = ["function", "const", "let", "var", "return", "if", "else",
                        "for", "while", "class", "extends", "import", "export",
                        "async", "await", "try", "catch", "throw", "new"];
        
        let mut highlighted = code.to_string();
        for keyword in &keywords {
            let _pattern = format!(r"\b{}\b", keyword);
            highlighted = highlighted.replace(keyword, &keyword.blue().to_string());
        }
        highlighted
    }
    
    fn highlight_rust(&self, code: &str) -> String {
        let keywords = ["fn", "let", "mut", "const", "use", "mod", "pub", "impl",
                        "struct", "enum", "trait", "where", "async", "await",
                        "match", "if", "else", "for", "while", "loop", "return"];
        
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
            format!("{}...", &text[..max_len - 3])
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
                .unwrap()
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
}