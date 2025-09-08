//! Manx CLI library crate
//!
//! This library provides the core functionality for the Manx documentation finder,
//! including RAG (Retrieval-Augmented Generation) capabilities, web search,
//! caching, and rendering utilities.

pub mod cache;
pub mod cli;
pub mod client;
pub mod config;
pub mod export;
pub mod rag;
pub mod render;
pub mod search;
pub mod web_search;

// Re-export commonly used types
pub use client::{Documentation, SearchResult};
pub use rag::{EmbeddingConfig, EmbeddingProvider};
