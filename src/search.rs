use crate::client::{Context7Client, SearchResult};
use crate::rag::embeddings::EmbeddingModel;
use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct ParsedQuery {
    quoted_phrases: Vec<String>,
    individual_terms: Vec<String>,
    original_query: String,
}

pub struct SearchEngine {
    client: Context7Client,
    matcher: SkimMatcherV2,
    embedding_model: Option<Arc<EmbeddingModel>>,
}

impl SearchEngine {
    /// Create SearchEngine without embeddings (fallback mode)
    pub fn new(client: Context7Client) -> Self {
        Self {
            client,
            matcher: SkimMatcherV2::default(),
            embedding_model: None,
        }
    }

    /// Create SearchEngine with a shared embedding model (for pooling/reuse)
    /// Recommended approach for better performance and memory efficiency
    pub fn with_shared_embeddings(
        client: Context7Client,
        embedding_model: Arc<EmbeddingModel>,
    ) -> Self {
        log::info!("🔄 Reusing shared embedding model for Context7 search");
        Self {
            client,
            matcher: SkimMatcherV2::default(),
            embedding_model: Some(embedding_model),
        }
    }

    /// Check if semantic embeddings are available
    pub fn has_embeddings(&self) -> bool {
        self.embedding_model.is_some()
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
        let (library_id, library_title) = self.client.resolve_library(lib_name, query).await?;

        // Step 2: Parse the query to extract phrases and terms
        let parsed_query = self.parse_search_query(query);

        // Step 3: Multi-pass search with phrase prioritization
        let mut results = self
            .multi_pass_search(&library_id, library, &parsed_query)
            .await?;

        // Step 5: Apply limit if specified
        if let Some(limit) = limit {
            if limit > 0 && results.len() > limit {
                results.truncate(limit);
            }
        }

        // Step 4: Cache individual snippets for later retrieval via snippet command
        if let Ok(cache_manager) = crate::cache::CacheManager::new() {
            for result in &results {
                let snippet_cache_key = format!("{}_{}", library, &result.id);
                // Cache the complete excerpt content
                let _ = cache_manager
                    .set("snippets", &snippet_cache_key, &result.excerpt)
                    .await;
            }
        }

        Ok((results, library_title, library_id))
    }

    async fn multi_pass_search(
        &self,
        library_id: &str,
        library: &str,
        parsed_query: &ParsedQuery,
    ) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();

        // Pass 1: Phrase-priority search if we have quoted phrases
        if !parsed_query.quoted_phrases.is_empty() {
            let phrase_query = self.build_phrase_priority_query(parsed_query);
            let docs = self
                .client
                .get_documentation(library_id, Some(&phrase_query))
                .await?;

            let phrase_results = self
                .parse_documentation_into_results(
                    library,
                    &parsed_query.original_query,
                    &docs,
                    parsed_query,
                    true, // is_phrase_search
                )
                .await?;

            all_results.extend(phrase_results);
        }

        // Pass 2: Individual term search if needed
        let should_do_term_search = parsed_query.quoted_phrases.is_empty() || all_results.len() < 5;

        if should_do_term_search && !parsed_query.individual_terms.is_empty() {
            let term_query = parsed_query.individual_terms.join(" ");
            let docs = self
                .client
                .get_documentation(library_id, Some(&term_query))
                .await?;

            let term_results = self
                .parse_documentation_into_results(
                    library,
                    &parsed_query.original_query,
                    &docs,
                    parsed_query,
                    false, // is_phrase_search
                )
                .await?;

            all_results.extend(term_results);
        }

        // Pass 3: Merge, deduplicate, and rank
        let merged_results = self.merge_and_rank_results(all_results, parsed_query);

        Ok(merged_results)
    }

    fn build_phrase_priority_query(&self, parsed_query: &ParsedQuery) -> String {
        let mut query_parts = Vec::new();

        // Add quoted phrases with quotes preserved for Context7
        for phrase in &parsed_query.quoted_phrases {
            query_parts.push(format!("\"{}\"", phrase));
        }

        // Add individual terms
        query_parts.extend(parsed_query.individual_terms.clone());

        query_parts.join(" ")
    }

    fn parse_search_query(&self, query: &str) -> ParsedQuery {
        let mut quoted_phrases = Vec::new();
        let mut individual_terms = Vec::new();
        let mut current_term = String::new();
        let mut in_quotes = false;

        for ch in query.chars() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                    if !in_quotes && !current_term.is_empty() {
                        // This was a quoted phrase
                        quoted_phrases.push(current_term.clone());
                        current_term.clear();
                    }
                }
                ' ' if !in_quotes => {
                    if !current_term.is_empty() {
                        // This was an individual term
                        individual_terms.push(current_term.clone());
                        current_term.clear();
                    }
                }
                _ => {
                    current_term.push(ch);
                }
            }
        }

        // Handle any remaining term
        if !current_term.is_empty() {
            if in_quotes {
                // Unclosed quote - treat as phrase anyway
                quoted_phrases.push(current_term);
            } else {
                individual_terms.push(current_term);
            }
        }

        // If no terms found, treat the whole query as individual terms
        if quoted_phrases.is_empty() && individual_terms.is_empty() {
            individual_terms.push(query.to_string());
        }

        ParsedQuery {
            quoted_phrases,
            individual_terms,
            original_query: query.to_string(),
        }
    }

    async fn parse_documentation_into_results(
        &self,
        library: &str,
        original_query: &str,
        docs: &str,
        parsed_query: &ParsedQuery,
        is_phrase_search: bool,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Split documentation into individual code snippets/sections
        let sections = self.split_into_sections(docs);

        // OPTIMIZATION: Batch embed all sections at once if embeddings available
        let relevance_scores = if self.embedding_model.is_some() {
            self.calculate_embedding_relevance_batch(
                &sections,
                &parsed_query.original_query,
                parsed_query,
                is_phrase_search,
            )
            .await
            .unwrap_or_else(|e| {
                log::warn!(
                    "Batch embedding failed, falling back to keyword matching: {}",
                    e
                );
                // Fallback to keyword scoring for all sections
                sections
                    .iter()
                    .map(|section| {
                        self.calculate_enhanced_section_relevance(
                            section,
                            parsed_query,
                            is_phrase_search,
                        )
                    })
                    .collect()
            })
        } else {
            // Use keyword-only scoring
            sections
                .iter()
                .map(|section| {
                    self.calculate_enhanced_section_relevance(
                        section,
                        parsed_query,
                        is_phrase_search,
                    )
                })
                .collect()
        };

        for (idx, (section, &relevance)) in sections.iter().zip(relevance_scores.iter()).enumerate()
        {
            // Lower threshold for including results when we have multiple sections
            let relevance_threshold = if sections.len() > 1 { 0.05 } else { 0.1 };

            if relevance > relevance_threshold {
                // Only include sections with reasonable relevance
                let title = self.extract_section_title(section).unwrap_or_else(|| {
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
                    id: format!("{}-doc-{}", library, idx + 1),
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
                let title = self.extract_section_title(section).unwrap_or_else(|| {
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

    fn merge_and_rank_results(
        &self,
        mut all_results: Vec<SearchResult>,
        parsed_query: &ParsedQuery,
    ) -> Vec<SearchResult> {
        // Remove duplicates based on content similarity
        all_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        all_results.dedup_by(|a, b| {
            // Consider results duplicates if titles are very similar
            let similarity = self
                .matcher
                .fuzzy_match(&a.title.to_lowercase(), &b.title.to_lowercase());
            similarity.unwrap_or(0) > 800 // High similarity threshold
        });

        // Apply final phrase boost to top results
        for result in all_results.iter_mut() {
            if self.contains_quoted_phrases(&result.excerpt, &parsed_query.quoted_phrases) {
                result.relevance_score *= 1.5; // Final boost for phrase-containing results
            }
        }

        // Final sort after boost
        all_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        all_results
    }

    fn contains_quoted_phrases(&self, text: &str, phrases: &[String]) -> bool {
        let text_lower = text.to_lowercase();
        phrases
            .iter()
            .any(|phrase| text_lower.contains(&phrase.to_lowercase()))
    }

    fn calculate_enhanced_section_relevance(
        &self,
        section: &str,
        parsed_query: &ParsedQuery,
        is_phrase_search: bool,
    ) -> f32 {
        let section_lower = section.to_lowercase();
        let mut total_score = 0.0;

        // Score quoted phrases with high priority
        for phrase in &parsed_query.quoted_phrases {
            let phrase_lower = phrase.to_lowercase();

            if section_lower.contains(&phrase_lower) {
                // Exact phrase match gets highest score
                let phrase_score = if is_phrase_search { 10.0 } else { 5.0 };
                total_score += phrase_score;

                // Extra bonus for title matches
                if let Some(title_line) = section.lines().find(|line| line.starts_with("TITLE: ")) {
                    if title_line.to_lowercase().contains(&phrase_lower) {
                        total_score += phrase_score * 0.5;
                    }
                }

                // Extra bonus for description matches
                if let Some(desc_line) = section
                    .lines()
                    .find(|line| line.starts_with("DESCRIPTION: "))
                {
                    if desc_line.to_lowercase().contains(&phrase_lower) {
                        total_score += phrase_score * 0.3;
                    }
                }
            } else {
                // Try partial phrase matching (words close together)
                let proximity_score = self.calculate_phrase_proximity(section, phrase);
                total_score += proximity_score;
            }
        }

        // Score individual terms with lower priority
        for term in &parsed_query.individual_terms {
            let term_lower = term.to_lowercase();

            if section_lower.contains(&term_lower) {
                total_score += 1.0;

                // Bonus for title/description matches
                if let Some(title_line) = section.lines().find(|line| line.starts_with("TITLE: ")) {
                    if title_line.to_lowercase().contains(&term_lower) {
                        total_score += 0.5;
                    }
                }
                if let Some(desc_line) = section
                    .lines()
                    .find(|line| line.starts_with("DESCRIPTION: "))
                {
                    if desc_line.to_lowercase().contains(&term_lower) {
                        total_score += 0.3;
                    }
                }
            } else {
                // Fuzzy match for individual terms
                if let Some(score) = self.matcher.fuzzy_match(&section_lower, &term_lower) {
                    total_score += (score as f32) / 1000.0;
                }
            }
        }

        // Normalize by total number of search elements
        let total_elements =
            parsed_query.quoted_phrases.len() + parsed_query.individual_terms.len();
        if total_elements > 0 {
            total_score / total_elements as f32
        } else {
            0.0
        }
    }

    fn calculate_phrase_proximity(&self, section: &str, phrase: &str) -> f32 {
        let words: Vec<&str> = phrase.split_whitespace().collect();
        if words.len() < 2 {
            return 0.0;
        }

        let section_lower = section.to_lowercase();
        let mut max_proximity_score: f32 = 0.0;

        // Look for words appearing close together
        for window in section_lower
            .split_whitespace()
            .collect::<Vec<_>>()
            .windows(words.len())
        {
            let mut proximity_score = 0.0;
            let mut found_words = 0;

            for (i, &target_word) in words.iter().enumerate() {
                if let Some(fuzzy_score) = self.matcher.fuzzy_match(window[i], target_word) {
                    if fuzzy_score > 700 {
                        // Good match threshold
                        proximity_score += 1.0;
                        found_words += 1;
                    }
                }
            }

            if found_words > 0 {
                let proximity_multiplier = found_words as f32 / words.len() as f32;
                proximity_score = proximity_score * proximity_multiplier * 2.0; // Proximity bonus
                max_proximity_score = max_proximity_score.max(proximity_score);
            }
        }

        max_proximity_score
    }

    /// OPTIMIZED: Calculate relevance for all sections using batch embedding generation
    async fn calculate_embedding_relevance_batch(
        &self,
        sections: &[String],
        query: &str,
        parsed_query: &ParsedQuery,
        is_phrase_search: bool,
    ) -> Result<Vec<f32>> {
        let embedding_model = self
            .embedding_model
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Embedding model not available"))?;

        // Generate query embedding once
        let query_embedding = embedding_model.embed_text(query).await?;

        // Prepare all sections for embedding
        let section_texts: Vec<String> = sections
            .iter()
            .map(|section| self.prepare_section_for_embedding(section))
            .collect();

        // Batch generate embeddings for all sections at once (MAJOR OPTIMIZATION)
        let section_text_refs: Vec<&str> = section_texts.iter().map(|s| s.as_str()).collect();
        let section_embeddings = embedding_model.embed_batch(&section_text_refs).await?;

        // Calculate scores for all sections
        let mut scores = Vec::with_capacity(sections.len());

        for (i, section) in sections.iter().enumerate() {
            if let Some(section_embedding) = section_embeddings.get(i) {
                // Calculate semantic similarity (0.0-1.0)
                let embedding_score =
                    EmbeddingModel::cosine_similarity(&query_embedding, section_embedding);

                // Calculate existing keyword-based relevance (0.0-N)
                let keyword_score = self.calculate_enhanced_section_relevance(
                    section,
                    parsed_query,
                    is_phrase_search,
                );

                // Calculate quoted phrase bonus for exact matches
                let phrase_bonus =
                    self.calculate_phrase_bonus(section, parsed_query, is_phrase_search);

                // Hybrid scoring: Embeddings (70%) + Keywords (20%) + Phrase bonus (10%)
                let normalized_keyword_score = (keyword_score / 5.0).min(1.0);
                let normalized_phrase_bonus = (phrase_bonus / 10.0).min(1.0);

                let final_score = (embedding_score * 0.7)
                    + (normalized_keyword_score * 0.2)
                    + (normalized_phrase_bonus * 0.1);

                scores.push(final_score);
            } else {
                // Fallback to keyword-only if embedding failed for this section
                log::warn!("Missing embedding for section {}, using keyword scoring", i);
                scores.push(self.calculate_enhanced_section_relevance(
                    section,
                    parsed_query,
                    is_phrase_search,
                ));
            }
        }

        log::debug!(
            "Batch embedded {} sections with average score: {:.3}",
            sections.len(),
            scores.iter().sum::<f32>() / scores.len() as f32
        );

        Ok(scores)
    }

    /// Calculate section relevance using semantic embeddings + keyword hybrid scoring (single section - legacy)
    #[allow(dead_code)]
    async fn calculate_embedding_section_relevance(
        &self,
        section: &str,
        query: &str,
        parsed_query: &ParsedQuery,
        is_phrase_search: bool,
    ) -> Result<f32> {
        let embedding_model = self
            .embedding_model
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Embedding model not available"))?;

        // Generate embeddings for semantic similarity
        let query_embedding = embedding_model.embed_text(query).await?;

        // Create combined text for section (title + content for better context)
        let section_text = self.prepare_section_for_embedding(section);
        let section_embedding = embedding_model.embed_text(&section_text).await?;

        // Calculate semantic similarity (0.0-1.0)
        let embedding_score =
            EmbeddingModel::cosine_similarity(&query_embedding, &section_embedding);

        // Calculate existing keyword-based relevance (0.0-N)
        let keyword_score =
            self.calculate_enhanced_section_relevance(section, parsed_query, is_phrase_search);

        // Calculate quoted phrase bonus for exact matches
        let phrase_bonus = self.calculate_phrase_bonus(section, parsed_query, is_phrase_search);

        // Hybrid scoring: Embeddings (70%) + Keywords (20%) + Phrase bonus (10%)
        // Normalize keyword score to 0-1 range by dividing by reasonable maximum
        let normalized_keyword_score = (keyword_score / 5.0).min(1.0);
        let normalized_phrase_bonus = (phrase_bonus / 10.0).min(1.0);

        let final_score = (embedding_score * 0.7)
            + (normalized_keyword_score * 0.2)
            + (normalized_phrase_bonus * 0.1);

        log::debug!("Embedding hybrid scoring for section: Embedding={:.3}, Keywords={:.3}, Phrase={:.3}, Final={:.3}",
            embedding_score, normalized_keyword_score, normalized_phrase_bonus, final_score);

        Ok(final_score)
    }

    /// Prepare section text for optimal embedding generation
    fn prepare_section_for_embedding(&self, section: &str) -> String {
        // Extract title and first few lines for context
        let lines: Vec<&str> = section.lines().collect();
        let mut embedding_text = String::new();

        // Add title if available
        if let Some(title_line) = lines.iter().find(|line| line.starts_with("TITLE: ")) {
            embedding_text.push_str(title_line[7..].trim());
            embedding_text.push(' ');
        }

        // Add description if available
        if let Some(desc_line) = lines.iter().find(|line| line.starts_with("DESCRIPTION: ")) {
            embedding_text.push_str(desc_line[13..].trim());
            embedding_text.push(' ');
        }

        // Add first few content lines (up to 200 chars)
        let content_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.starts_with("TITLE: ") && !line.starts_with("DESCRIPTION: "))
            .take(5)
            .copied()
            .collect();

        let content = content_lines.join(" ");
        let content_preview = if content.len() > 200 {
            format!("{}...", &content[..200])
        } else {
            content
        };

        embedding_text.push_str(&content_preview);
        embedding_text.trim().to_string()
    }

    /// Calculate phrase bonus for quoted phrase exact matches
    fn calculate_phrase_bonus(
        &self,
        section: &str,
        parsed_query: &ParsedQuery,
        is_phrase_search: bool,
    ) -> f32 {
        let section_lower = section.to_lowercase();
        let mut phrase_score = 0.0;

        for phrase in &parsed_query.quoted_phrases {
            let phrase_lower = phrase.to_lowercase();
            if section_lower.contains(&phrase_lower) {
                phrase_score += if is_phrase_search { 10.0 } else { 5.0 };

                // Extra bonus for title matches
                if let Some(title_line) = section.lines().find(|line| line.starts_with("TITLE: ")) {
                    if title_line.to_lowercase().contains(&phrase_lower) {
                        phrase_score += 2.0;
                    }
                }
            }
        }

        phrase_score
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
                    if trimmed.len() > 50 {
                        // Only include meaningful paragraphs
                        sections.push(trimmed.to_string());
                    }
                }
            }

            // If still no good sections or too few, split into chunks
            if sections.len() < 3 {
                sections.clear(); // Start fresh
                let chunk_size = 800; // Characters per chunk
                let mut start = 0;
                let mut chunk_count = 0;

                while start < docs.len() && chunk_count < 20 {
                    // Limit to 20 chunks max
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
                    while start < docs.len()
                        && docs.chars().nth(start).is_some_and(|c| c.is_whitespace())
                    {
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

    pub async fn get_documentation(&self, library: &str, query: Option<&str>) -> Result<String> {
        let (lib_name, _version) = parse_library_spec(library);

        // Step 1: Resolve library to Context7 ID
        let (library_id, _library_title) = self
            .client
            .resolve_library(lib_name, query.unwrap_or(""))
            .await?;

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
