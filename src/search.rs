use crate::client::{Context7Client, SearchResult};
use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct SearchEngine {
    client: Context7Client,
    matcher: SkimMatcherV2,
}

impl SearchEngine {
    pub fn new(client: Context7Client) -> Self {
        Self {
            client,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub async fn search(
        &self,
        library: &str,
        query: &str,
        limit: Option<usize>,
    ) -> Result<(Vec<SearchResult>, String, String)> {
        // Parse library@version format
        let (lib_name, _version) = parse_library_spec(library);

        // Step 1: Resolve library to Context7 ID
        let (library_id, library_title) = self.client.resolve_library(lib_name).await?;

        // Step 2: Parse the query to extract key terms and phrases
        let search_terms = self.parse_search_query(query);
        let optimized_query = search_terms.join(" ");

        // Step 3: Get documentation with the optimized query as topic
        let docs = self
            .client
            .get_documentation(&library_id, Some(&optimized_query))
            .await?;

        // Step 4: Parse documentation into multiple results and rank them
        let mut results =
            self.parse_documentation_into_results(library, query, &docs, &search_terms)?;

        // Step 5: Apply limit if specified
        if let Some(limit) = limit {
            if limit > 0 && results.len() > limit {
                results.truncate(limit);
            }
        }

        // Step 6: Cache individual snippets for later retrieval via snippet command
        // We'll store each section separately so users can access them via "manx snippet doc-N"
        if let Ok(cache_manager) = crate::cache::CacheManager::new() {
            let sections = self.split_into_sections(&docs);
            for (idx, result) in results.iter().enumerate() {
                if idx < sections.len() {
                    let snippet_cache_key = format!("{}_{}", library, &result.id);
                    // Cache the complete section content, not just the excerpt
                    let _ = cache_manager
                        .set("snippets", &snippet_cache_key, &sections[idx])
                        .await;
                }
            }
        }

        Ok((results, library_title, library_id))
    }

    fn parse_search_query(&self, query: &str) -> Vec<String> {
        let mut terms = Vec::new();
        let mut current_term = String::new();
        let mut in_quotes = false;
        let chars = query.chars().peekable();

        for ch in chars {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                    if !in_quotes && !current_term.is_empty() {
                        terms.push(current_term.clone());
                        current_term.clear();
                    }
                }
                ' ' if !in_quotes => {
                    if !current_term.is_empty() {
                        terms.push(current_term.clone());
                        current_term.clear();
                    }
                }
                _ => {
                    current_term.push(ch);
                }
            }
        }

        if !current_term.is_empty() {
            terms.push(current_term);
        }

        // If no terms found, use the original query
        if terms.is_empty() {
            vec![query.to_string()]
        } else {
            terms
        }
    }

    fn parse_documentation_into_results(
        &self,
        library: &str,
        original_query: &str,
        docs: &str,
        search_terms: &[String],
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Split documentation into individual code snippets/sections
        let sections = self.split_into_sections(docs);

        for (idx, section) in sections.iter().enumerate() {
            // Calculate relevance score based on search terms
            let relevance = self.calculate_section_relevance(section, search_terms);

            // Lower threshold for including results when we have multiple sections
            let relevance_threshold = if sections.len() > 1 { 0.05 } else { 0.1 };

            if relevance > relevance_threshold {
                // Only include sections with reasonable relevance
                let title = self
                    .extract_section_title(section)
                    .unwrap_or_else(|| {
                        // Try to create a meaningful title from the section content
                        let first_line = section.lines().next().unwrap_or("");
                        let title_candidate = if first_line.len() > 60 {
                            format!("{}...", &first_line[..57])
                        } else if first_line.is_empty() {
                            format!("{} - Result {}", original_query, idx + 1)
                        } else {
                            first_line.to_string()
                        };
                        format!("{} ({})", title_candidate, library)
                    });

                let excerpt = self.extract_section_excerpt(section);

                results.push(SearchResult {
                    id: format!("doc-{}", idx + 1),
                    library: library.to_string(),
                    title,
                    excerpt,
                    url: None,
                    relevance_score: relevance,
                });
            }
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // If no specific sections matched well, create results from all sections anyway
        if results.is_empty() && !sections.is_empty() {
            for (idx, section) in sections.iter().enumerate().take(10) {
                // Limit to first 10 sections
                let title = self
                    .extract_section_title(section)
                    .unwrap_or_else(|| {
                        // Try to extract a meaningful title from the section
                        let lines: Vec<&str> = section.lines().take(3).collect();
                        let mut title_candidate = String::new();
                        
                        // Look for the first non-empty, meaningful line
                        for line in &lines {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() && trimmed.len() > 10 {
                                title_candidate = if trimmed.len() > 60 {
                                    format!("{}...", &trimmed[..57])
                                } else {
                                    trimmed.to_string()
                                };
                                break;
                            }
                        }
                        
                        if title_candidate.is_empty() {
                            format!("{} - Section {}", original_query, idx + 1)
                        } else {
                            title_candidate
                        }
                    });

                // Create a unique excerpt from this specific section
                let excerpt = self.create_unique_excerpt(section, idx);

                results.push(SearchResult {
                    id: format!("doc-{}", idx + 1),
                    library: library.to_string(),
                    title,
                    excerpt,
                    url: None,
                    relevance_score: 0.5, // Default relevance for unmatched sections
                });
            }
        }

        Ok(results)
    }

    fn split_into_sections(&self, docs: &str) -> Vec<String> {
        // Split by title markers but preserve complete documentation structure
        let mut sections = Vec::new();
        let lines: Vec<&str> = docs.lines().collect();
        let mut current_section = Vec::new();
        let mut in_section = false;

        for line in lines {
            if line.starts_with("TITLE: ") {
                // Save previous section if it exists
                if in_section && !current_section.is_empty() {
                    let section_text = current_section.join("\n");
                    if section_text.len() > 20 {
                        sections.push(section_text);
                    }
                }
                // Start new section
                current_section.clear();
                current_section.push(line);
                in_section = true;
            } else if in_section {
                current_section.push(line);
            }
        }

        // Add the last section
        if in_section && !current_section.is_empty() {
            let section_text = current_section.join("\n");
            if section_text.len() > 20 {
                sections.push(section_text);
            }
        }

        // If no sections found, try alternative splitting methods
        if sections.is_empty() {
            // Try splitting by double newlines (paragraphs)
            let paragraphs: Vec<&str> = docs.split("\n\n").collect();
            if paragraphs.len() > 1 {
                for paragraph in paragraphs {
                    let trimmed = paragraph.trim();
                    if trimmed.len() > 50 {  // Only include meaningful paragraphs
                        sections.push(trimmed.to_string());
                    }
                }
            }
            
            // If still no good sections or too few, split into chunks
            if sections.len() < 3 {
                sections.clear();  // Start fresh
                let chunk_size = 800;  // Characters per chunk
                let mut start = 0;
                let mut chunk_count = 0;
                
                while start < docs.len() && chunk_count < 20 {  // Limit to 20 chunks max
                    let end = (start + chunk_size).min(docs.len());
                    // Try to break at a sentence or paragraph boundary
                    let mut actual_end = end;
                    if end < docs.len() {
                        // Look for a good break point
                        if let Some(pos) = docs[start..end].rfind("\n\n") {
                            actual_end = start + pos;
                        } else if let Some(pos) = docs[start..end].rfind(".\n") {
                            actual_end = start + pos + 1;
                        } else if let Some(pos) = docs[start..end].rfind(". ") {
                            actual_end = start + pos + 1;
                        } else if let Some(pos) = docs[start..end].rfind('\n') {
                            actual_end = start + pos;
                        }
                    }
                    
                    // Make sure we're making progress
                    if actual_end <= start {
                        actual_end = end;
                    }
                    
                    let chunk = docs[start..actual_end].trim();
                    if !chunk.is_empty() && chunk.len() > 50 {
                        sections.push(chunk.to_string());
                        chunk_count += 1;
                    }
                    
                    start = actual_end;
                    // Skip whitespace for next chunk
                    while start < docs.len() && docs.chars().nth(start).map_or(false, |c| c.is_whitespace()) {
                        start += 1;
                    }
                }
            }
        }

        // Return at least something
        if sections.is_empty() {
            vec![docs.to_string()]
        } else {
            sections
        }
    }

    fn extract_section_title(&self, section: &str) -> Option<String> {
        section
            .lines()
            .find(|line| line.starts_with("TITLE: "))
            .map(|line| line[7..].to_string())
    }

    fn extract_section_excerpt(&self, section: &str) -> String {
        // Try to find description, otherwise use first few lines
        if let Some(desc_line) = section
            .lines()
            .find(|line| line.starts_with("DESCRIPTION: "))
        {
            let desc = &desc_line[13..];
            if desc.len() > 300 {
                format!("{}...", &desc[..300])
            } else {
                desc.to_string()
            }
        } else {
            // Take first 300 chars of the section
            if section.len() > 300 {
                format!("{}...", &section[..300])
            } else {
                section.to_string()
            }
        }
    }

    fn create_unique_excerpt(&self, section: &str, offset: usize) -> String {
        let lines: Vec<&str> = section.lines().collect();
        let mut excerpt_lines = Vec::new();
        let mut char_count = 0;
        
        // Skip some lines based on offset to get different content for each chunk
        let skip_lines = offset.saturating_mul(2);
        
        for line in lines.iter().skip(skip_lines) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                excerpt_lines.push(trimmed);
                char_count += trimmed.len();
                
                // Stop when we have enough content
                if char_count > 200 || excerpt_lines.len() >= 3 {
                    break;
                }
            }
        }
        
        // If we didn't get enough content, try from the beginning
        if excerpt_lines.is_empty() {
            for line in lines.iter().take(5) {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    excerpt_lines.push(trimmed);
                    char_count += trimmed.len();
                    if char_count > 200 {
                        break;
                    }
                }
            }
        }
        
        let result = excerpt_lines.join(" ");
        if result.len() > 300 {
            format!("{}...", &result[..297])
        } else if result.is_empty() {
            // Last resort - just take raw content
            if section.len() > 300 {
                format!("{}...", &section[..297])
            } else {
                section.to_string()
            }
        } else {
            result
        }
    }

    fn calculate_section_relevance(&self, section: &str, search_terms: &[String]) -> f32 {
        let section_lower = section.to_lowercase();
        let mut total_score = 0.0;

        for term in search_terms {
            let term_lower = term.to_lowercase();

            // Exact phrase match (highest score)
            if section_lower.contains(&term_lower) {
                total_score += 1.0;

                // Bonus for title matches
                if let Some(title_line) = section.lines().find(|line| line.starts_with("TITLE: ")) {
                    if title_line.to_lowercase().contains(&term_lower) {
                        total_score += 0.5;
                    }
                }

                // Bonus for description matches
                if let Some(desc_line) = section
                    .lines()
                    .find(|line| line.starts_with("DESCRIPTION: "))
                {
                    if desc_line.to_lowercase().contains(&term_lower) {
                        total_score += 0.3;
                    }
                }
            } else {
                // Fuzzy match using the existing matcher
                if let Some(score) = self.matcher.fuzzy_match(&section_lower, &term_lower) {
                    total_score += (score as f32) / 1000.0; // Normalize fuzzy score
                }
            }
        }

        // Normalize by number of search terms
        if !search_terms.is_empty() {
            total_score / search_terms.len() as f32
        } else {
            0.0
        }
    }

    pub async fn get_documentation(&self, library: &str, query: Option<&str>) -> Result<String> {
        let (lib_name, _version) = parse_library_spec(library);

        // Step 1: Resolve library to Context7 ID
        let (library_id, _library_title) = self.client.resolve_library(lib_name).await?;

        // Step 2: Get documentation
        self.client.get_documentation(&library_id, query).await
    }
}

fn parse_library_spec(spec: &str) -> (&str, Option<&str>) {
    if let Some(at_pos) = spec.find('@') {
        let (lib, ver) = spec.split_at(at_pos);
        (lib, Some(&ver[1..]))
    } else {
        (spec, None)
    }
}

pub fn fuzzy_find_libraries(query: &str, libraries: &[String]) -> Vec<(String, i64)> {
    let matcher = SkimMatcherV2::default();
    let mut matches: Vec<(String, i64)> = libraries
        .iter()
        .filter_map(|lib| {
            matcher
                .fuzzy_match(lib, query)
                .map(|score| (lib.clone(), score))
        })
        .collect();

    matches.sort_by_key(|(_, score)| -score);
    matches.truncate(5);
    matches
}
