//! Query Intent Analysis for Enhanced Search
//!
//! This module provides intelligent query preprocessing that enhances both
//! embedding-based semantic search and LLM result synthesis by:
//! - Detecting framework/library context 
//! - Expanding queries with domain-specific terms
//! - Suggesting better search strategies
//! - Working collaboratively with embeddings rather than replacing them

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query analysis result that enhances search without replacing embedding work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnalysis {
    pub original_query: String,
    pub detected_frameworks: Vec<DetectedFramework>,
    pub enhanced_query: String,
    pub search_strategy: SearchStrategy,
    pub domain_context: DomainContext,
    pub confidence: f32,
    pub suggested_sites: Vec<String>,
    pub query_type: QueryType,
}

/// Detected framework/library with confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFramework {
    pub name: String,
    pub category: FrameworkCategory,
    pub confidence: f32,
    pub official_sites: Vec<String>,
    pub common_terms: Vec<String>,
}

/// Framework categories for better search targeting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FrameworkCategory {
    WebFramework,     // React, Vue, Angular
    BackendFramework, // Django, FastAPI, Express
    DesktopFramework, // Tauri, Electron, Flutter
    DatabaseTool,     // PostgreSQL, MongoDB
    DevTool,         // Docker, Kubernetes
    Language,        // Rust, Python, JavaScript
    Library,         // Pandas, NumPy
    Other,
}

/// Search strategy based on query analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchStrategy {
    FrameworkSpecific { framework: String, sites: Vec<String> },
    OfficialDocsFirst { frameworks: Vec<String> },
    CommunityAndOfficial,
    GeneralSearch,
}

/// Domain context for better embedding understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainContext {
    pub primary_domain: String,  // "web-development", "data-science", etc.
    pub sub_domains: Vec<String>, // "ui-components", "state-management"
    pub technical_level: TechnicalLevel,
    pub context_keywords: Vec<String>, // Additional terms to help embeddings
}

/// Technical complexity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TechnicalLevel {
    Beginner,
    Intermediate,
    Advanced,
    Reference,
}

/// Type of query to optimize search approach
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryType {
    HowTo,        // "how to create tables in Tauri"
    Reference,    // "Tauri table API"
    Troubleshoot, // "Tauri table not rendering"
    Comparison,   // "Tauri vs Electron tables"
    Example,      // "Tauri table examples"
    General,      // "Tauri tables"
}

/// Query analyzer that enhances search without replacing embeddings
pub struct QueryAnalyzer {
    framework_database: FrameworkDatabase,
}

impl QueryAnalyzer {
    /// Create new query analyzer
    pub fn new() -> Self {
        Self {
            framework_database: FrameworkDatabase::default(),
        }
    }

    /// Analyze query intent to enhance both embedding and LLM processing
    pub async fn analyze_query(
        &self,
        query: &str,
        llm_client: Option<&crate::rag::llm::LlmClient>,
    ) -> Result<QueryAnalysis> {
        log::info!("🧠 Analyzing query intent: {}", query);

        // Step 1: Detect frameworks using pattern matching (fast, deterministic)
        let detected_frameworks = self.detect_frameworks(query);
        
        // Step 2: Determine query type (helps both embeddings and LLM)
        let query_type = self.classify_query_type(query);
        
        // Step 3: Build domain context (enhances embedding understanding)
        let domain_context = self.build_domain_context(query, &detected_frameworks);
        
        // Step 4: Create enhanced query (adds context for better search)
        let enhanced_query = self.enhance_query(query, &detected_frameworks, &domain_context);
        
        // Step 5: Determine search strategy
        let search_strategy = self.determine_search_strategy(&detected_frameworks, &query_type);
        
        // Step 6: Get suggested sites for targeted search
        let suggested_sites = self.get_suggested_sites(&detected_frameworks);
        
        // Step 7: Optional LLM enhancement (if available)
        let (final_enhanced_query, confidence) = if let Some(llm_client) = llm_client {
            self.llm_enhance_analysis(query, &enhanced_query, &detected_frameworks, llm_client).await?
        } else {
            (enhanced_query.clone(), self.calculate_confidence(&detected_frameworks))
        };

        Ok(QueryAnalysis {
            original_query: query.to_string(),
            detected_frameworks,
            enhanced_query: final_enhanced_query,
            search_strategy,
            domain_context,
            confidence,
            suggested_sites,
            query_type,
        })
    }

    /// Detect frameworks using pattern matching and keyword analysis
    fn detect_frameworks(&self, query: &str) -> Vec<DetectedFramework> {
        let query_lower = query.to_lowercase();
        let mut detected = Vec::new();
        
        for (framework_name, framework_info) in &self.framework_database.frameworks {
            let mut confidence: f32 = 0.0;
            
            // Direct name match (high confidence)
            if query_lower.contains(&framework_name.to_lowercase()) {
                confidence += 0.8;
            }
            
            // Keyword matches (medium confidence)
            for keyword in &framework_info.keywords {
                if query_lower.contains(&keyword.to_lowercase()) {
                    confidence += 0.3;
                }
            }
            
            // Alias matches
            for alias in &framework_info.aliases {
                if query_lower.contains(&alias.to_lowercase()) {
                    confidence += 0.6;
                }
            }
            
            // Only include if we have reasonable confidence
            if confidence >= 0.5 {
                detected.push(DetectedFramework {
                    name: framework_name.clone(),
                    category: framework_info.category.clone(),
                    confidence: confidence.min(1.0),
                    official_sites: framework_info.official_sites.clone(),
                    common_terms: framework_info.common_terms.clone(),
                });
            }
        }
        
        // Sort by confidence
        detected.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        detected
    }

    /// Classify the type of query to help both search and LLM processing
    fn classify_query_type(&self, query: &str) -> QueryType {
        let query_lower = query.to_lowercase();
        
        if query_lower.starts_with("how to") || query_lower.contains("how do i") {
            QueryType::HowTo
        } else if query_lower.contains("api") || query_lower.contains("reference") || query_lower.contains("documentation") {
            QueryType::Reference
        } else if query_lower.contains("error") || query_lower.contains("not working") || query_lower.contains("issue") {
            QueryType::Troubleshoot
        } else if query_lower.contains("vs") || query_lower.contains("versus") || query_lower.contains("compared to") {
            QueryType::Comparison
        } else if query_lower.contains("example") || query_lower.contains("sample") || query_lower.contains("demo") {
            QueryType::Example
        } else {
            QueryType::General
        }
    }

    /// Build domain context to enhance embedding understanding
    fn build_domain_context(&self, query: &str, frameworks: &[DetectedFramework]) -> DomainContext {
        let mut context_keywords = Vec::new();
        let mut primary_domain = "software-development".to_string();
        let mut sub_domains = Vec::new();
        
        // Add framework-specific context
        for framework in frameworks {
            context_keywords.extend(framework.common_terms.clone());
            
            // Determine primary domain from framework category
            primary_domain = match framework.category {
                FrameworkCategory::WebFramework => "web-development".to_string(),
                FrameworkCategory::BackendFramework => "backend-development".to_string(),
                FrameworkCategory::DesktopFramework => "desktop-development".to_string(),
                FrameworkCategory::DatabaseTool => "database-management".to_string(),
                FrameworkCategory::DevTool => "devops".to_string(),
                FrameworkCategory::Language => "programming".to_string(),
                FrameworkCategory::Library => "software-library".to_string(),
                FrameworkCategory::Other => "software-development".to_string(),
            };
        }
        
        // Detect sub-domains from query content
        let query_lower = query.to_lowercase();
        if query_lower.contains("table") || query_lower.contains("grid") {
            sub_domains.push("ui-components".to_string());
            context_keywords.push("data-display".to_string());
        }
        if query_lower.contains("component") {
            sub_domains.push("component-development".to_string());
        }
        if query_lower.contains("state") {
            sub_domains.push("state-management".to_string());
        }
        
        // Determine technical level
        let technical_level = if query_lower.contains("beginner") || query_lower.contains("tutorial") {
            TechnicalLevel::Beginner
        } else if query_lower.contains("advanced") || query_lower.contains("internals") {
            TechnicalLevel::Advanced
        } else if query_lower.contains("api") || query_lower.contains("reference") {
            TechnicalLevel::Reference
        } else {
            TechnicalLevel::Intermediate
        };
        
        DomainContext {
            primary_domain,
            sub_domains,
            technical_level,
            context_keywords,
        }
    }

    /// Enhance query with additional context without losing original meaning
    fn enhance_query(&self, original_query: &str, frameworks: &[DetectedFramework], context: &DomainContext) -> String {
        let mut enhanced = original_query.to_string();
        
        // Add framework context if detected
        if let Some(main_framework) = frameworks.first() {
            // Add official name if query used alias
            if !original_query.to_lowercase().contains(&main_framework.name.to_lowercase()) {
                enhanced = format!("{} {}", main_framework.name, enhanced);
            }
            
            // Add category context
            let category_term = match main_framework.category {
                FrameworkCategory::DesktopFramework => "desktop application",
                FrameworkCategory::WebFramework => "web development",
                FrameworkCategory::BackendFramework => "backend development",
                _ => "",
            };
            
            if !category_term.is_empty() && !enhanced.to_lowercase().contains(category_term) {
                enhanced = format!("{} {}", enhanced, category_term);
            }
        }
        
        // Add domain context keywords
        for keyword in &context.context_keywords {
            if !enhanced.to_lowercase().contains(&keyword.to_lowercase()) {
                enhanced = format!("{} {}", enhanced, keyword);
            }
        }
        
        enhanced
    }

    /// Determine search strategy based on detected frameworks
    fn determine_search_strategy(&self, frameworks: &[DetectedFramework], query_type: &QueryType) -> SearchStrategy {
        if let Some(main_framework) = frameworks.first() {
            if main_framework.confidence > 0.8 {
                return SearchStrategy::FrameworkSpecific {
                    framework: main_framework.name.clone(),
                    sites: main_framework.official_sites.clone(),
                };
            } else if main_framework.confidence > 0.6 {
                return SearchStrategy::OfficialDocsFirst {
                    frameworks: frameworks.iter().map(|f| f.name.clone()).collect(),
                };
            }
        }
        
        match query_type {
            QueryType::Reference => SearchStrategy::OfficialDocsFirst {
                frameworks: frameworks.iter().map(|f| f.name.clone()).collect(),
            },
            QueryType::Troubleshoot => SearchStrategy::CommunityAndOfficial,
            _ => SearchStrategy::GeneralSearch,
        }
    }

    /// Get suggested sites for targeted search
    fn get_suggested_sites(&self, frameworks: &[DetectedFramework]) -> Vec<String> {
        let mut sites = Vec::new();
        
        for framework in frameworks {
            sites.extend(framework.official_sites.clone());
        }
        
        // Remove duplicates
        sites.sort();
        sites.dedup();
        sites
    }

    /// Calculate confidence score
    fn calculate_confidence(&self, frameworks: &[DetectedFramework]) -> f32 {
        if frameworks.is_empty() {
            0.3 // Low confidence for general queries
        } else {
            frameworks.first().unwrap().confidence
        }
    }

    /// Use LLM to enhance the analysis (optional step)
    async fn llm_enhance_analysis(
        &self,
        original_query: &str,
        enhanced_query: &str,
        frameworks: &[DetectedFramework],
        llm_client: &crate::rag::llm::LlmClient,
    ) -> Result<(String, f32)> {
        let framework_context = if frameworks.is_empty() {
            "No specific frameworks detected.".to_string()
        } else {
            format!(
                "Detected frameworks: {}",
                frameworks.iter()
                    .map(|f| format!("{} ({}%)", f.name, (f.confidence * 100.0) as u8))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let prompt = format!(
            r#"You are helping enhance a search query for better documentation results.

Original Query: "{}"
Current Enhanced Query: "{}"
Context: {}

Your job is to suggest a SLIGHTLY improved search query that:
1. Preserves the user's original intent
2. Adds helpful context for search engines
3. Uses official terminology when possible
4. Stays concise and focused

Respond with ONLY the improved query text, no explanation.
If the current enhanced query is already good, return it as-is."#,
            original_query, enhanced_query, framework_context
        );

        // Create mock RAG results for the LLM synthesis method
        let mock_results = vec![crate::rag::RagSearchResult {
            id: "query_enhancement".to_string(),
            content: prompt.clone(),
            source_path: std::path::PathBuf::from("query_analysis"),
            source_type: crate::rag::SourceType::Web,
            title: Some("Query Enhancement".to_string()),
            section: None,
            score: 1.0,
            metadata: crate::rag::DocumentMetadata {
                file_type: "analysis".to_string(),
                size: prompt.len() as u64,
                modified: chrono::Utc::now(),
                tags: vec!["query".to_string()],
                language: Some("en".to_string()),
            },
        }];

        match llm_client.synthesize_answer("enhance query", &mock_results).await {
            Ok(response) => {
                let enhanced = response.answer.trim().to_string();
                Ok((enhanced, 0.9)) // High confidence with LLM enhancement
            }
            Err(e) => {
                log::warn!("LLM query enhancement failed: {}", e);
                Ok((enhanced_query.to_string(), 0.7)) // Medium confidence without LLM
            }
        }
    }
}

/// Framework database with known frameworks and their metadata
struct FrameworkDatabase {
    frameworks: HashMap<String, FrameworkInfo>,
}

struct FrameworkInfo {
    category: FrameworkCategory,
    keywords: Vec<String>,
    aliases: Vec<String>,
    official_sites: Vec<String>,
    common_terms: Vec<String>,
}

impl Default for FrameworkDatabase {
    fn default() -> Self {
        let mut frameworks = HashMap::new();
        
        // Tauri framework
        frameworks.insert("Tauri".to_string(), FrameworkInfo {
            category: FrameworkCategory::DesktopFramework,
            keywords: vec!["rust".to_string(), "desktop".to_string(), "app".to_string()],
            aliases: vec!["tauri-app".to_string()],
            official_sites: vec!["tauri.app".to_string(), "docs.rs/tauri".to_string()],
            common_terms: vec!["webview".to_string(), "native".to_string(), "cross-platform".to_string()],
        });
        
        // React
        frameworks.insert("React".to_string(), FrameworkInfo {
            category: FrameworkCategory::WebFramework,
            keywords: vec!["jsx".to_string(), "component".to_string(), "hook".to_string()],
            aliases: vec!["react.js".to_string(), "reactjs".to_string()],
            official_sites: vec!["reactjs.org".to_string(), "react.dev".to_string()],
            common_terms: vec!["virtual-dom".to_string(), "state".to_string(), "props".to_string()],
        });
        
        // FastAPI
        frameworks.insert("FastAPI".to_string(), FrameworkInfo {
            category: FrameworkCategory::BackendFramework,
            keywords: vec!["python".to_string(), "api".to_string(), "async".to_string()],
            aliases: vec!["fast-api".to_string()],
            official_sites: vec!["fastapi.tiangolo.com".to_string()],
            common_terms: vec!["pydantic".to_string(), "swagger".to_string(), "openapi".to_string()],
        });
        
        // Add more frameworks as needed...
        
        Self { frameworks }
    }
}