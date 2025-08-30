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
        _limit: Option<usize>,
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
        let results =
            self.parse_documentation_into_results(library, query, &docs, &search_terms)?;

        // Step 5: Cache individual snippets for later retrieval via snippet command
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

            if relevance > 0.1 {
                // Only include sections with reasonable relevance
                let title = self
                    .extract_section_title(section)
                    .unwrap_or_else(|| format!("{} - {}", library, original_query));

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

        // If no specific sections matched well, return the full documentation as one result
        if results.is_empty() {
            results.push(SearchResult {
                id: "doc-1".to_string(),
                library: library.to_string(),
                title: format!("{} - {}", library, original_query),
                excerpt: if docs.len() > 300 {
                    format!("{}...", &docs[..300])
                } else {
                    docs.to_string()
                },
                url: None,
                relevance_score: 1.0,
            });
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

        // If no sections found, return the full document
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
