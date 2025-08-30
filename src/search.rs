use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::client::{Context7Client, SearchResult, Documentation};

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
    ) -> Result<Vec<SearchResult>> {
        // Parse library@version format
        let (lib_name, version) = parse_library_spec(library);
        
        // Build versioned library string if needed
        let library_id = if let Some(ver) = version {
            format!("{}@{}", lib_name, ver)
        } else {
            lib_name.to_string()
        };
        
        // Perform search
        let mut results = self.client.search(&library_id, query, limit).await?;
        
        // Sort by relevance and fuzzy match score
        results.sort_by(|a, b| {
            let score_a = self.calculate_relevance(&a, query);
            let score_b = self.calculate_relevance(&b, query);
            score_b.partial_cmp(&score_a).unwrap()
        });
        
        Ok(results)
    }
    
    pub async fn get_documentation(
        &self,
        library: &str,
        query: Option<&str>,
    ) -> Result<Documentation> {
        let (lib_name, version) = parse_library_spec(library);
        
        let library_id = if let Some(ver) = version {
            format!("{}@{}", lib_name, ver)
        } else {
            lib_name.to_string()
        };
        
        self.client.get_documentation(&library_id, query).await
    }
    
    fn calculate_relevance(&self, result: &SearchResult, query: &str) -> f32 {
        let title_score = self.matcher.fuzzy_match(&result.title, query)
            .unwrap_or(0) as f32;
        let excerpt_score = self.matcher.fuzzy_match(&result.excerpt, query)
            .unwrap_or(0) as f32;
        
        // Combine API relevance score with fuzzy matching
        result.relevance_score * 0.5 + (title_score + excerpt_score) * 0.25
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
            matcher.fuzzy_match(lib, query)
                .map(|score| (lib.clone(), score))
        })
        .collect();
    
    matches.sort_by_key(|(_, score)| -score);
    matches.truncate(5);
    matches
}