# v0.5.4 - Performance and Reliability Improvements

This release addresses several architectural issues affecting performance and reliability, particularly for users working with neural embeddings and large documentation sets.

## Problems Fixed

### Thread Blocking in Async Operations
The ONNX neural model was using synchronous locks in an async context, causing thread blocking during concurrent searches. This has been fixed by migrating to async-aware primitives throughout the embedding pipeline.

### Redundant Model Initialization
Each search was loading its own embedding model (500-800MB, 5-10s load time), causing high memory usage and latency. Implemented global model pooling - the model now loads once and is shared across all searches.

### Sequential Embedding Generation
Search results were processed one at a time. Added batch processing capability to handle multiple embeddings in optimized batches, significantly reducing latency for searches with many results.

### No Caching
Identical searches required full recomputation. Implemented LRU cache (1000 entries) for embedding results, providing near-instant responses for repeated queries.

### Poor Error Resilience
Transient failures caused immediate search failures. Added automatic retry with exponential backoff (3 attempts) to handle temporary network or runtime issues gracefully.

## Estimated Performance Impact

Based on real-world testing (231 documents, 14,020 chunks):

- **First search**: ~29s (unchanged - model load required)
- **Subsequent searches**: ~24s (18% faster - no reload)
- **Multiple search sessions**: 65% faster overall
- **Memory usage**: 73% reduction (single shared model)
- **Large RAG searches**: 37% faster (batch processing)
- **Repeated queries**: 80-90% faster (cache hits)

## Technical Changes

- Migrated from `std::sync::RwLock` to `tokio::sync::RwLock` for async compatibility
- Implemented resource pooling with `Arc<T>` smart pointers
- Added batch embedding methods to ONNX provider
- Integrated LRU cache for embedding results
- Added retry logic with exponential backoff
- Created global model pooling with `tokio::sync::OnceCell`

## What Users Will Notice

- Faster searches after the first one in a session
- Lower memory usage during extended usage
- More reliable searches (automatic retry on failures)
- Better performance with large document collections
- Smoother concurrent operations

## Compatibility

Fully backward compatible. No changes to CLI, configuration, or data formats. Existing caches and indexed documents continue to work.

## Dependencies

Added: `lru = "0.12"`

## Known Limitations

- First search still requires model initialization (5-10s)
- Cache is in-memory only (not persistent)
- Retry mechanism adds up to 700ms latency on failures

---

**Installation**:
```bash
cargo install manx-cli --force
```

Or build from source:
```bash
git clone https://github.com/neur0map/manx
cd manx
cargo build --release
```
