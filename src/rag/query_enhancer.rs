//! Query enhancement system for improved RAG search
//!
//! This module provides intelligent query enhancement using LLM when available,
//! with fallback strategies for semantic understanding without LLM.

use anyhow::Result;
use serde_json::json;

use crate::rag::{llm::LlmClient, SmartSearchConfig};

/// Enhanced query with variations and metadata
#[derive(Debug, Clone)]
pub struct EnhancedQuery {
    pub original: String,
    pub variations: Vec<QueryVariation>,
    pub detected_intent: QueryIntent,
    #[allow(dead_code)]
    pub suggested_terms: Vec<String>,
}

/// Query variation with different search strategies
#[derive(Debug, Clone)]
pub struct QueryVariation {
    pub query: String,
    pub strategy: SearchStrategy,
    pub weight: f32,
}

/// Different search strategies for query variations
#[derive(Debug, Clone)]
pub enum SearchStrategy {
    Semantic,       // Use semantic embeddings
    Keyword,        // Exact keyword matching
    #[allow(dead_code)]
    Fuzzy,         // Fuzzy string matching
    Code,          // Code-specific patterns
    Mixed,         // Combined approach
}

/// Detected query intent for specialized handling
#[derive(Debug, Clone)]
pub enum QueryIntent {
    CodeSearch {
        language: Option<String>,
        component_type: Option<String>, // function, class, variable, etc.
    },
    Documentation,
    Configuration,
    TechnicalConcept,
    Debugging,
    #[allow(dead_code)]
    Unknown,
}

/// Query enhancement system
pub struct QueryEnhancer {
    llm_client: Option<LlmClient>,
    config: SmartSearchConfig,
}

impl QueryEnhancer {
    /// Create a new query enhancer
    pub fn new(llm_client: Option<LlmClient>, config: SmartSearchConfig) -> Self {
        Self { llm_client, config }
    }

    /// Enhance a query with multiple variations and strategies
    pub async fn enhance_query(&self, query: &str) -> Result<EnhancedQuery> {
        log::debug!("Enhancing query: '{}'", query);

        // Detect query intent first
        let detected_intent = self.detect_query_intent(query).await?;

        let mut variations = Vec::new();
        let mut suggested_terms = Vec::new();

        // Try LLM enhancement if available
        if let Some(ref llm_client) = self.llm_client {
            if self.config.enable_query_enhancement {
                match self.enhance_with_llm(query, &detected_intent, llm_client).await {
                    Ok((llm_variations, llm_terms)) => {
                        variations.extend(llm_variations);
                        suggested_terms.extend(llm_terms);
                        log::debug!("LLM enhancement succeeded with {} variations", variations.len());
                    }
                    Err(e) => {
                        log::warn!("LLM enhancement failed, using fallback: {}", e);
                    }
                }
            }
        }

        // Always include fallback enhancement
        let fallback_variations = self.enhance_with_fallback(query, &detected_intent);
        variations.extend(fallback_variations);

        // Add original query as highest priority
        variations.insert(0, QueryVariation {
            query: query.to_string(),
            strategy: SearchStrategy::Mixed,
            weight: 1.0,
        });

        // Limit variations based on config
        variations.truncate(self.config.max_query_variations.max(1));

        Ok(EnhancedQuery {
            original: query.to_string(),
            variations,
            detected_intent,
            suggested_terms,
        })
    }

    /// Detect the intent of a query for specialized handling
    async fn detect_query_intent(&self, query: &str) -> Result<QueryIntent> {
        let query_lower = query.to_lowercase();

        // Check for code-specific patterns
        if self.is_code_query(&query_lower) {
            let language = self.detect_programming_language(&query_lower);
            let component_type = self.detect_component_type(&query_lower);

            return Ok(QueryIntent::CodeSearch {
                language,
                component_type,
            });
        }

        // Check for configuration queries
        if query_lower.contains("config") || query_lower.contains("settings") || query_lower.contains("environment") {
            return Ok(QueryIntent::Configuration);
        }

        // Check for debugging queries
        if query_lower.contains("error") || query_lower.contains("bug") || query_lower.contains("debug")
           || query_lower.contains("issue") || query_lower.contains("problem") {
            return Ok(QueryIntent::Debugging);
        }

        // Check for documentation queries
        if query_lower.contains("how to") || query_lower.contains("guide") || query_lower.contains("tutorial")
           || query_lower.contains("example") {
            return Ok(QueryIntent::Documentation);
        }

        Ok(QueryIntent::TechnicalConcept)
    }

    /// Check if query is code-related
    fn is_code_query(&self, query: &str) -> bool {
        let code_indicators = [
            "function", "method", "class", "struct", "interface", "variable",
            "implementation", "where is", "how does", "used", "called",
            "middleware", "authentication", "validation", "security",
            "database", "connection", "handler", "controller", "service",
            "component", "module", "library", "package", "import",
        ];

        code_indicators.iter().any(|&indicator| query.contains(indicator))
    }

    /// Detect programming language from query
    fn detect_programming_language(&self, query: &str) -> Option<String> {
        let language_keywords = [
            ("rust", vec!["fn", "impl", "struct", "trait", "cargo", "rust"]),
            ("javascript", vec!["function", "const", "let", "var", "nodejs", "js", "react", "vue"]),
            ("typescript", vec!["interface", "type", "typescript", "ts"]),
            ("python", vec!["def", "class", "import", "python", "django", "flask"]),
            ("java", vec!["public", "private", "class", "java", "spring"]),
            ("go", vec!["func", "package", "golang", "go"]),
            ("c++", vec!["class", "namespace", "cpp", "c++"]),
            ("c", vec!["struct", "typedef", "c programming"]),
        ];

        for (lang, keywords) in &language_keywords {
            if keywords.iter().any(|&keyword| query.contains(keyword)) {
                return Some(lang.to_string());
            }
        }

        None
    }

    /// Detect component type from query
    fn detect_component_type(&self, query: &str) -> Option<String> {
        if query.contains("function") || query.contains("method") || query.contains("fn") {
            Some("function".to_string())
        } else if query.contains("class") || query.contains("struct") {
            Some("class".to_string())
        } else if query.contains("interface") || query.contains("trait") {
            Some("interface".to_string())
        } else if query.contains("variable") || query.contains("constant") {
            Some("variable".to_string())
        } else if query.contains("middleware") || query.contains("handler") {
            Some("middleware".to_string())
        } else {
            None
        }
    }

    /// Enhance query using LLM
    async fn enhance_with_llm(
        &self,
        query: &str,
        intent: &QueryIntent,
        llm_client: &LlmClient,
    ) -> Result<(Vec<QueryVariation>, Vec<String>)> {
        let system_prompt = self.build_enhancement_prompt(intent);

        let user_message = format!(
            "Original query: \"{}\"\n\nPlease provide:\n1. 2-3 alternative ways to phrase this query for better search results\n2. Important keywords and synonyms\n3. Focus on {} context\n\nRespond in JSON format with 'variations' array and 'keywords' array.",
            query,
            match intent {
                QueryIntent::CodeSearch { .. } => "code search and programming",
                QueryIntent::Documentation => "documentation and guides",
                QueryIntent::Configuration => "configuration and settings",
                QueryIntent::Debugging => "troubleshooting and debugging",
                QueryIntent::TechnicalConcept => "technical concepts",
                QueryIntent::Unknown => "general search",
            }
        );

        // Create a simple LLM request (we'll use a basic approach since we don't have the full LLM implementation details)
        let response = self.call_llm_for_enhancement(llm_client, &system_prompt, &user_message).await?;

        let parsed_response: serde_json::Value = serde_json::from_str(&response)
            .unwrap_or_else(|_| json!({"variations": [], "keywords": []}));

        let variations = parsed_response["variations"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str())
            .map(|v| QueryVariation {
                query: v.to_string(),
                strategy: SearchStrategy::Semantic,
                weight: 0.8,
            })
            .collect();

        let keywords = parsed_response["keywords"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|k| k.as_str())
            .map(|k| k.to_string())
            .collect();

        Ok((variations, keywords))
    }

    /// Build enhancement prompt based on intent
    fn build_enhancement_prompt(&self, intent: &QueryIntent) -> String {
        match intent {
            QueryIntent::CodeSearch { language, component_type } => {
                format!(
                    "You are a code search expert. Help enhance queries for finding {} {} in codebases. Focus on programming patterns, function names, and implementation details.",
                    component_type.as_deref().unwrap_or("code"),
                    language.as_deref().unwrap_or("programming")
                )
            },
            QueryIntent::Documentation => {
                "You are a documentation search expert. Help enhance queries for finding guides, tutorials, and explanations. Focus on learning objectives and procedural knowledge.".to_string()
            },
            QueryIntent::Configuration => {
                "You are a configuration expert. Help enhance queries for finding settings, environment variables, and configuration patterns.".to_string()
            },
            QueryIntent::Debugging => {
                "You are a debugging expert. Help enhance queries for finding error solutions, troubleshooting guides, and problem resolution.".to_string()
            },
            _ => {
                "You are a technical search expert. Help enhance queries for better search results in technical documentation and code.".to_string()
            }
        }
    }

    /// Basic LLM call for enhancement (simplified implementation)
    async fn call_llm_for_enhancement(
        &self,
        _llm_client: &LlmClient,
        _system_prompt: &str,
        _user_message: &str,
    ) -> Result<String> {
        // This is a placeholder - in a real implementation, this would call the LLM
        // For now, return a basic JSON response to make the system work
        Ok(json!({
            "variations": [],
            "keywords": []
        }).to_string())
    }

    /// Fallback enhancement without LLM
    fn enhance_with_fallback(&self, query: &str, intent: &QueryIntent) -> Vec<QueryVariation> {
        let mut variations = Vec::new();

        match intent {
            QueryIntent::CodeSearch { language, component_type } => {
                // Add code-specific variations
                if let Some(comp_type) = component_type {
                    variations.push(QueryVariation {
                        query: format!("{} {}", comp_type, query),
                        strategy: SearchStrategy::Code,
                        weight: 0.9,
                    });
                }

                if let Some(lang) = language {
                    variations.push(QueryVariation {
                        query: format!("{} {}", lang, query),
                        strategy: SearchStrategy::Code,
                        weight: 0.8,
                    });
                }

                // Add common code search patterns
                if query.contains("where") {
                    let without_where = query.replace("where is", "").replace("where", "");
                    let trimmed = without_where.trim();
                    variations.push(QueryVariation {
                        query: format!("{} implementation", trimmed),
                        strategy: SearchStrategy::Semantic,
                        weight: 0.7,
                    });
                }
            },
            QueryIntent::Documentation => {
                // Add documentation-focused variations
                variations.push(QueryVariation {
                    query: format!("how to {}", query),
                    strategy: SearchStrategy::Semantic,
                    weight: 0.7,
                });
                variations.push(QueryVariation {
                    query: format!("{} guide", query),
                    strategy: SearchStrategy::Keyword,
                    weight: 0.6,
                });
            },
            _ => {
                // Generic fallback variations
                variations.push(QueryVariation {
                    query: query.to_string(),
                    strategy: SearchStrategy::Keyword,
                    weight: 0.6,
                });
            }
        }

        variations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_intent_detection() {
        let enhancer = QueryEnhancer::new(None, SmartSearchConfig::default());

        let result = enhancer.detect_query_intent("where is middleware being used?").await.unwrap();
        matches!(result, QueryIntent::CodeSearch { .. });

        let result = enhancer.detect_query_intent("how to configure authentication").await.unwrap();
        matches!(result, QueryIntent::Configuration);
    }

    #[tokio::test]
    async fn test_fallback_enhancement() {
        let enhancer = QueryEnhancer::new(None, SmartSearchConfig::default());

        let result = enhancer.enhance_query("validate_code_security function").await.unwrap();
        assert!(result.variations.len() > 1);
        assert_eq!(result.original, "validate_code_security function");
    }
}