//! DuckDuckGo search engine integration
//!
//! This module provides anonymous, privacy-focused web search using DuckDuckGo's API.
//! No user data is logged or transmitted beyond the search query itself.

use crate::web_search::RawSearchResult;
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::Deserialize;
use std::time::Duration;

/// DuckDuckGo search response structure
#[derive(Debug, Deserialize)]
struct DuckDuckGoResponse {
    #[serde(rename = "Results")]
    results: Vec<DuckDuckGoResult>,
    #[serde(rename = "RelatedTopics")]
    related_topics: Vec<RelatedTopic>,
}

#[derive(Debug, Deserialize)]
struct DuckDuckGoResult {
    #[serde(rename = "Text")]
    text: String,
    #[serde(rename = "FirstURL")]
    first_url: String,
}

#[derive(Debug, Deserialize)]
struct RelatedTopic {
    #[serde(rename = "Text")]
    text: Option<String>,
    #[serde(rename = "FirstURL")]
    first_url: Option<String>,
}

/// Search DuckDuckGo for documentation results
pub async fn search_duckduckgo(
    query: &str,
    max_results: usize,
    user_agent: &str,
    timeout_seconds: u64,
) -> Result<Vec<RawSearchResult>> {
    if query.trim().is_empty() {
        return Err(anyhow!("Search query cannot be empty"));
    }

    log::info!("Searching DuckDuckGo with query: {}", query);

    // Use DuckDuckGo's Instant Answer API for initial results
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .user_agent(user_agent)
        .build()?;

    // DuckDuckGo Instant Answer API
    let instant_api_url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding::encode(query)
    );

    let instant_response = client.get(&instant_api_url).send().await;
    let mut results = Vec::new();

    // Try instant answers first
    if let Ok(response) = instant_response {
        if response.status().is_success() {
            if let Ok(ddg_response) = response.json::<DuckDuckGoResponse>().await {
                // Process direct results
                for result in ddg_response.results.into_iter().take(max_results / 2) {
                    if !result.text.is_empty() && !result.first_url.is_empty() {
                        results.push(create_raw_result(
                            &result.text,
                            &result.first_url,
                            &result.text,
                        )?);
                    }
                }

                // Process related topics
                for topic in ddg_response
                    .related_topics
                    .into_iter()
                    .take(max_results / 2)
                {
                    if let (Some(text), Some(url)) = (topic.text, topic.first_url) {
                        if !text.is_empty() && !url.is_empty() {
                            results.push(create_raw_result(&text, &url, &text)?);
                        }
                    }
                }
            }
        }
    }

    // If we don't have enough results from instant API, try HTML search
    if results.len() < max_results {
        log::info!("Expanding search with HTML scraping");
        let html_results =
            search_duckduckgo_html(query, max_results - results.len(), &client).await?;
        results.extend(html_results);
    }

    // Remove duplicates based on URL
    results.sort_by(|a, b| a.url.cmp(&b.url));
    results.dedup_by(|a, b| a.url == b.url);

    // Limit to max_results
    results.truncate(max_results);

    log::info!("Found {} results from DuckDuckGo", results.len());
    Ok(results)
}

/// Search DuckDuckGo HTML interface (fallback method)
async fn search_duckduckgo_html(
    query: &str,
    max_results: usize,
    client: &reqwest::Client,
) -> Result<Vec<RawSearchResult>> {
    // DuckDuckGo HTML search URL
    let search_url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        urlencoding::encode(query)
    );

    let response = client.get(&search_url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "DuckDuckGo search failed with status: {}",
            response.status()
        ));
    }

    let html = response.text().await?;
    parse_duckduckgo_html(&html, max_results)
}

/// Parse DuckDuckGo HTML results
fn parse_duckduckgo_html(html: &str, max_results: usize) -> Result<Vec<RawSearchResult>> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    // Selectors for DuckDuckGo result elements
    let result_selector =
        Selector::parse(".result").map_err(|e| anyhow!("Invalid selector: {:?}", e))?;
    let title_selector =
        Selector::parse(".result__title a").map_err(|e| anyhow!("Invalid selector: {:?}", e))?;
    let snippet_selector =
        Selector::parse(".result__snippet").map_err(|e| anyhow!("Invalid selector: {:?}", e))?;
    let url_selector =
        Selector::parse(".result__url").map_err(|e| anyhow!("Invalid selector: {:?}", e))?;

    let mut results = Vec::new();

    for result_element in document.select(&result_selector).take(max_results) {
        // Extract title
        let title = result_element
            .select(&title_selector)
            .next()
            .map(|e| e.inner_html())
            .unwrap_or_default()
            .trim()
            .to_string();

        // Extract URL - try dedicated URL selector first, then fallback to title link
        let url = result_element
            .select(&url_selector)
            .next()
            .map(|e| e.inner_html())
            .or_else(|| {
                result_element
                    .select(&title_selector)
                    .next()
                    .and_then(|e| e.value().attr("href"))
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();

        // Extract snippet
        let snippet = result_element
            .select(&snippet_selector)
            .next()
            .map(|e| e.inner_html())
            .unwrap_or_default()
            .trim()
            .to_string();

        // Only include results with all required fields
        if !title.is_empty() && !url.is_empty() && !snippet.is_empty() {
            results.push(create_raw_result(&title, &url, &snippet)?);
        }
    }

    Ok(results)
}

/// Create a RawSearchResult with proper URL validation and domain extraction
fn create_raw_result(title: &str, url: &str, snippet: &str) -> Result<RawSearchResult> {
    // Clean and validate URL
    let cleaned_url = clean_url(url)?;
    let domain = extract_domain(&cleaned_url)?;

    // Clean HTML entities and tags from text
    let clean_title = clean_html_text(title);
    let clean_snippet = clean_html_text(snippet);

    Ok(RawSearchResult {
        title: clean_title,
        url: cleaned_url,
        snippet: clean_snippet,
        source_domain: domain,
        timestamp: Some(Utc::now()),
    })
}

/// Clean and validate URL
fn clean_url(url: &str) -> Result<String> {
    let url = url.trim();

    // Remove DuckDuckGo redirect wrapper
    if url.starts_with("/l/?uddg=") {
        // Extract the actual URL from DuckDuckGo's redirect
        if let Some(start) = url.find("https://") {
            let actual_url = &url[start..];
            if let Some(end) = actual_url.find('&') {
                return Ok(actual_url[..end].to_string());
            } else {
                return Ok(actual_url.to_string());
            }
        }
    }

    // Ensure URL has protocol
    if !url.starts_with("http://") && !url.starts_with("https://") {
        Ok(format!("https://{}", url))
    } else {
        Ok(url.to_string())
    }
}

/// Extract domain from URL
fn extract_domain(url: &str) -> Result<String> {
    use url::Url;

    let parsed_url = Url::parse(url).map_err(|e| anyhow!("Invalid URL '{}': {}", url, e))?;

    let domain = parsed_url
        .host_str()
        .ok_or_else(|| anyhow!("No domain found in URL: {}", url))?;

    Ok(domain.to_string())
}

/// Clean HTML entities and tags from text
fn clean_html_text(text: &str) -> String {
    // Remove HTML tags
    let no_tags = regex::Regex::new(r"<[^>]*>").unwrap().replace_all(text, "");

    // Decode common HTML entities
    let decoded = no_tags
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");

    // Clean up whitespace
    regex::Regex::new(r"\s+")
        .unwrap()
        .replace_all(&decoded, " ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_url() {
        assert_eq!(
            clean_url("docs.python.org/3/").unwrap(),
            "https://docs.python.org/3/"
        );

        assert_eq!(
            clean_url("https://reactjs.org/docs").unwrap(),
            "https://reactjs.org/docs"
        );
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://docs.python.org/3/library/").unwrap(),
            "docs.python.org"
        );

        assert_eq!(
            extract_domain("https://github.com/facebook/react").unwrap(),
            "github.com"
        );
    }

    #[test]
    fn test_clean_html_text() {
        assert_eq!(
            clean_html_text("<b>Python</b> &amp; <i>Django</i>"),
            "Python & Django"
        );

        assert_eq!(
            clean_html_text("Multiple   spaces\n\tand   tabs"),
            "Multiple spaces and tabs"
        );
    }
}
