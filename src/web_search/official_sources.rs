//! Official documentation source management
//!
//! This module manages the whitelist of official documentation sites
//! and provides utilities for source classification and query building.

use std::collections::HashMap;

/// Tier levels for documentation sources
#[derive(Debug, Clone, PartialEq)]
pub enum SourceTier {
    OfficialDocs = 1,     // docs.python.org, reactjs.org
    OfficialRepos = 2,    // github.com/official-orgs
    TrustedCommunity = 3, // stackoverflow.com, developer community sites
    General = 4,          // Other sources (fallback only)
}

/// Official documentation source manager
#[derive(Debug)]
pub struct OfficialSourceManager {
    official_domains: HashMap<String, SourceTier>,
    official_github_orgs: Vec<String>,
}

impl OfficialSourceManager {
    /// Create new official source manager with predefined official sources
    pub fn new() -> Self {
        let mut official_domains = HashMap::new();

        // Language Documentation (Tier 1)
        official_domains.insert("docs.python.org".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("docs.oracle.com".to_string(), SourceTier::OfficialDocs);
        official_domains.insert(
            "developer.mozilla.org".to_string(),
            SourceTier::OfficialDocs,
        );
        official_domains.insert("doc.rust-lang.org".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("docs.golang.org".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("docs.microsoft.com".to_string(), SourceTier::OfficialDocs);

        // Framework Documentation (Tier 1)
        official_domains.insert("reactjs.org".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("vuejs.org".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("angular.io".to_string(), SourceTier::OfficialDocs);
        official_domains.insert(
            "docs.djangoproject.com".to_string(),
            SourceTier::OfficialDocs,
        );
        official_domains.insert(
            "flask.palletsprojects.com".to_string(),
            SourceTier::OfficialDocs,
        );
        official_domains.insert("expressjs.com".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("nextjs.org".to_string(), SourceTier::OfficialDocs);

        // Tools & Libraries Documentation (Tier 1)
        official_domains.insert("hydra.cc".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("pytorch.org".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("tensorflow.org".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("docs.docker.com".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("kubernetes.io".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("docs.npmjs.com".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("yarnpkg.com".to_string(), SourceTier::OfficialDocs);

        // Cloud Platform Documentation (Tier 1)
        official_domains.insert("docs.aws.amazon.com".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("cloud.google.com".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("docs.microsoft.com".to_string(), SourceTier::OfficialDocs);

        // Standards Organizations (Tier 1)
        official_domains.insert("w3.org".to_string(), SourceTier::OfficialDocs);
        official_domains.insert("ietf.org".to_string(), SourceTier::OfficialDocs);
        official_domains.insert(
            "ecma-international.org".to_string(),
            SourceTier::OfficialDocs,
        );

        // Trusted Community Sources (Tier 3)
        official_domains.insert(
            "stackoverflow.com".to_string(),
            SourceTier::TrustedCommunity,
        );
        official_domains.insert(
            "developer.mozilla.org".to_string(),
            SourceTier::TrustedCommunity,
        );

        // Official GitHub organizations for repository documentation (Tier 2)
        let official_github_orgs = vec![
            "python".to_string(),
            "facebook".to_string(), // React
            "vuejs".to_string(),
            "angular".to_string(),
            "django".to_string(),
            "pallets".to_string(), // Flask
            "expressjs".to_string(),
            "vercel".to_string(),           // Next.js
            "facebookresearch".to_string(), // Hydra, PyTorch
            "pytorch".to_string(),
            "tensorflow".to_string(),
            "docker".to_string(),
            "kubernetes".to_string(),
            "npm".to_string(),
            "yarnpkg".to_string(),
            "rust-lang".to_string(),
            "golang".to_string(),
            "microsoft".to_string(),
            "aws".to_string(),
            "google".to_string(),
        ];

        Self {
            official_domains,
            official_github_orgs,
        }
    }

    /// Check if a domain is official documentation
    pub fn is_official_domain(&self, domain: &str) -> bool {
        // Direct domain match
        if self.official_domains.contains_key(domain) {
            return true;
        }

        // Check for GitHub official organization repositories
        if domain == "github.com" {
            // This would need URL parsing in practice, simplified here
            return false; // Will be handled in URL-level checking
        }

        // Check subdomain matches (e.g., api.reactjs.org)
        for official_domain in self.official_domains.keys() {
            if domain.ends_with(&format!(".{}", official_domain)) || domain == official_domain {
                return true;
            }
        }

        false
    }

    /// Get source tier for a domain
    pub fn get_source_tier(&self, domain: &str, url: &str) -> SourceTier {
        // Check direct domain matches
        if let Some(tier) = self.official_domains.get(domain) {
            return tier.clone();
        }

        // Check for official GitHub repositories
        if domain == "github.com" {
            if let Some(org) = self.extract_github_org(url) {
                if self.official_github_orgs.contains(&org) {
                    return SourceTier::OfficialRepos;
                }
            }
        }

        // Check subdomain matches
        for (official_domain, tier) in &self.official_domains {
            if domain.ends_with(&format!(".{}", official_domain)) || domain == official_domain {
                return tier.clone();
            }
        }

        SourceTier::General
    }

    /// Extract GitHub organization from URL
    fn extract_github_org(&self, url: &str) -> Option<String> {
        // Simple extraction: github.com/org/repo -> org
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 4 && parts[2] == "github.com" {
            Some(parts[3].to_string())
        } else {
            None
        }
    }

    /// Build DuckDuckGo query targeting official sources
    pub fn build_official_query(&self, query: &str) -> String {
        // Build site: restrictions for top-tier official documentation
        let official_sites: Vec<String> = self
            .official_domains
            .iter()
            .filter(|(_, tier)| **tier == SourceTier::OfficialDocs)
            .take(10) // Limit to avoid overly long queries
            .map(|(domain, _)| format!("site:{}", domain))
            .collect();

        // Add some key GitHub organizations
        let github_orgs = ["facebook", "pytorch", "facebookresearch", "python", "vuejs"]
            .iter()
            .map(|org| format!("site:github.com/{}", org))
            .collect::<Vec<String>>();

        let all_sites = [official_sites, github_orgs].concat();

        if all_sites.is_empty() {
            query.to_string()
        } else {
            format!("{} ({})", query, all_sites.join(" OR "))
        }
    }

    /// Calculate score boost based on source tier
    pub fn get_score_boost(&self, tier: &SourceTier) -> f32 {
        match tier {
            SourceTier::OfficialDocs => 10.0,    // 10x boost for official docs
            SourceTier::OfficialRepos => 5.0,    // 5x boost for official repos
            SourceTier::TrustedCommunity => 2.0, // 2x boost for trusted community
            SourceTier::General => 1.0,          // No boost for general sources
        }
    }

    /// Get human-readable tier description
    pub fn get_tier_description(&self, tier: &SourceTier) -> &'static str {
        match tier {
            SourceTier::OfficialDocs => "Official Documentation",
            SourceTier::OfficialRepos => "Official Repository",
            SourceTier::TrustedCommunity => "Trusted Community",
            SourceTier::General => "General Source",
        }
    }

    /// Add custom official domain (for user configuration)
    pub fn add_official_domain(&mut self, domain: String, tier: SourceTier) {
        self.official_domains.insert(domain, tier);
    }

    /// Get all official domains for debugging/config display
    pub fn get_official_domains(&self) -> &HashMap<String, SourceTier> {
        &self.official_domains
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_official_domain_detection() {
        let manager = OfficialSourceManager::new();

        assert!(manager.is_official_domain("docs.python.org"));
        assert!(manager.is_official_domain("reactjs.org"));
        assert!(!manager.is_official_domain("random-blog.com"));
        assert!(!manager.is_official_domain("example.com"));
    }

    #[test]
    fn test_source_tier_assignment() {
        let manager = OfficialSourceManager::new();

        assert_eq!(
            manager.get_source_tier("docs.python.org", "https://docs.python.org/3/"),
            SourceTier::OfficialDocs
        );

        assert_eq!(
            manager.get_source_tier("github.com", "https://github.com/facebook/react"),
            SourceTier::OfficialRepos
        );

        assert_eq!(
            manager.get_source_tier("random-blog.com", "https://random-blog.com/post"),
            SourceTier::General
        );
    }

    #[test]
    fn test_github_org_extraction() {
        let manager = OfficialSourceManager::new();

        assert_eq!(
            manager.extract_github_org("https://github.com/facebook/react"),
            Some("facebook".to_string())
        );

        assert_eq!(
            manager.extract_github_org(
                "https://github.com/facebookresearch/hydra/blob/main/README.md"
            ),
            Some("facebookresearch".to_string())
        );

        assert_eq!(
            manager.extract_github_org("https://docs.python.org/3/"),
            None
        );
    }

    #[test]
    fn test_score_boost_calculation() {
        let manager = OfficialSourceManager::new();

        assert_eq!(manager.get_score_boost(&SourceTier::OfficialDocs), 10.0);
        assert_eq!(manager.get_score_boost(&SourceTier::OfficialRepos), 5.0);
        assert_eq!(manager.get_score_boost(&SourceTier::TrustedCommunity), 2.0);
        assert_eq!(manager.get_score_boost(&SourceTier::General), 1.0);
    }

    #[test]
    fn test_official_query_building() {
        let manager = OfficialSourceManager::new();
        let query = manager.build_official_query("python logging");

        // Test basic query structure
        assert!(query.contains("python logging"));
        assert!(query.contains("site:"));

        // Test that it contains some official domains (HashMap ordering is not guaranteed)
        let contains_official = query.contains("docs.microsoft.com")
            || query.contains("nextjs.org")
            || query.contains("docs.djangoproject.com")
            || query.contains("docs.python.org");
        assert!(
            contains_official,
            "Query should contain at least one official domain"
        );
    }
}
