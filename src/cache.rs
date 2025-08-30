use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CACHE_VERSION: u32 = 1;
const DEFAULT_TTL_HOURS: u64 = 24;
const MAX_CACHE_SIZE_MB: u64 = 100;

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry<T> {
    version: u32,
    data: T,
    timestamp: u64,
    ttl_hours: u64,
}

pub struct CacheManager {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl CacheManager {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            ttl: Duration::from_secs(DEFAULT_TTL_HOURS * 3600),
        })
    }

    pub fn with_custom_dir(dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self {
            cache_dir: dir,
            ttl: Duration::from_secs(DEFAULT_TTL_HOURS * 3600),
        })
    }

    fn get_cache_dir() -> Result<PathBuf> {
        Ok(ProjectDirs::from("", "", "manx")
            .context("Failed to determine cache directory")?
            .cache_dir()
            .to_path_buf())
    }

    pub fn cache_key(&self, category: &str, key: &str) -> PathBuf {
        let safe_key = key.replace('/', "_").replace('@', "_v_").replace(' ', "_");

        self.cache_dir
            .join(category)
            .join(format!("{}.json", safe_key))
    }

    pub async fn get<T>(&self, category: &str, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let path = self.cache_key(category, key);

        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(&path).context("Failed to read cache file")?;

        let entry: CacheEntry<T> =
            serde_json::from_str(&data).context("Failed to parse cache entry")?;

        // Check version
        if entry.version != CACHE_VERSION {
            fs::remove_file(&path).ok();
            return Ok(None);
        }

        // Check TTL
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let age = now.saturating_sub(entry.timestamp);
        let ttl_secs = self.ttl.as_secs();

        if age > ttl_secs {
            fs::remove_file(&path).ok();
            return Ok(None);
        }

        Ok(Some(entry.data))
    }

    pub async fn set<T>(&self, category: &str, key: &str, data: T) -> Result<()>
    where
        T: Serialize,
    {
        let path = self.cache_key(category, key);

        // Create category directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let entry = CacheEntry {
            version: CACHE_VERSION,
            data,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            ttl_hours: DEFAULT_TTL_HOURS,
        };

        let json = serde_json::to_string_pretty(&entry)?;
        fs::write(&path, json)?;

        // Check cache size and clean if needed
        self.clean_if_needed().await?;

        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    pub async fn stats(&self) -> Result<CacheStats> {
        let mut total_size = 0u64;
        let mut file_count = 0u32;
        let mut categories = Vec::new();

        if !self.cache_dir.exists() {
            return Ok(CacheStats {
                total_size_mb: 0.0,
                file_count: 0,
                categories,
            });
        }

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                categories.push(entry.file_name().to_string_lossy().to_string());

                for file in fs::read_dir(&path)? {
                    let file = file?;
                    if file.path().is_file() {
                        let metadata = file.metadata()?;
                        total_size += metadata.len();
                        file_count += 1;
                    }
                }
            }
        }

        Ok(CacheStats {
            total_size_mb: total_size as f64 / 1_048_576.0,
            file_count,
            categories,
        })
    }

    pub async fn list_cached(&self) -> Result<Vec<CachedItem>> {
        let mut items = Vec::new();

        if !self.cache_dir.exists() {
            return Ok(items);
        }

        for category_entry in fs::read_dir(&self.cache_dir)? {
            let category_entry = category_entry?;
            let category_path = category_entry.path();

            if category_path.is_dir() {
                let category = category_entry.file_name().to_string_lossy().to_string();

                for file_entry in fs::read_dir(&category_path)? {
                    let file_entry = file_entry?;
                    let file_path = file_entry.path();

                    if file_path.is_file()
                        && file_path.extension() == Some(std::ffi::OsStr::new("json"))
                    {
                        let name = file_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let metadata = file_entry.metadata()?;
                        let size_kb = metadata.len() as f64 / 1024.0;

                        items.push(CachedItem {
                            category: category.clone(),
                            name,
                            size_kb,
                        });
                    }
                }
            }
        }

        items.sort_by(|a, b| a.category.cmp(&b.category).then(a.name.cmp(&b.name)));
        Ok(items)
    }

    async fn clean_if_needed(&self) -> Result<()> {
        let stats = self.stats().await?;

        if stats.total_size_mb > MAX_CACHE_SIZE_MB as f64 {
            // Remove oldest files until under limit
            let mut files: Vec<(PathBuf, SystemTime)> = Vec::new();

            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    for file in fs::read_dir(&path)? {
                        let file = file?;
                        let file_path = file.path();
                        if file_path.is_file() {
                            let modified = file.metadata()?.modified()?;
                            files.push((file_path, modified));
                        }
                    }
                }
            }

            // Sort by modification time (oldest first)
            files.sort_by_key(|(_, time)| *time);

            // Remove oldest files
            let mut current_size = stats.total_size_mb;
            for (file_path, _) in files {
                if current_size <= MAX_CACHE_SIZE_MB as f64 * 0.8 {
                    break;
                }

                if let Ok(metadata) = fs::metadata(&file_path) {
                    let file_size_mb = metadata.len() as f64 / 1_048_576.0;
                    fs::remove_file(&file_path).ok();
                    current_size -= file_size_mb;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub total_size_mb: f64,
    pub file_count: u32,
    pub categories: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CachedItem {
    pub category: String,
    pub name: String,
    pub size_kb: f64,
}
