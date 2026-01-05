// lib.rs - Expose modules for reuse

pub mod movie;
pub mod kvstore;

// Example of how to integrate KV Store with your movie API
use crate::kvstore::{KVStore, StorageFormat};
use crate::movie::Movie;
use anyhow::Result;

/// Example: Movie cache using KV Store
pub struct MovieCache {
    store: KVStore,
}

impl MovieCache {
    pub fn new(path: &str) -> Result<Self> {
        Ok(Self {
            store: KVStore::new(path, StorageFormat::Json)?,
        })
    }

    /// Cache a movie by ID
    pub fn cache_movie(&self, movie: &Movie) -> Result<()> {
        let key = format!("movie:{}", movie.id);
        let value = serde_json::to_string(movie)?;
        self.store.set(key, value)
    }

    /// Get a cached movie by ID
    pub fn get_movie(&self, id: i32) -> Result<Option<Movie>> {
        let key = format!("movie:{}", id);
        if let Some(value) = self.store.get(&key)? {
            let movie: Movie = serde_json::from_str(&value)?;
            Ok(Some(movie))
        } else {
            Ok(None)
        }
    }

    /// Cache movie list
    pub fn cache_movies(&self, movies: &[Movie]) -> Result<()> {
        let value = serde_json::to_string(movies)?;
        self.store.set("movies:all".to_string(), value)
    }

    /// Get cached movie list
    pub fn get_movies(&self) -> Result<Option<Vec<Movie>>> {
        if let Some(value) = self.store.get("movies:all")? {
            let movies: Vec<Movie> = serde_json::from_str(&value)?;
            Ok(Some(movies))
        } else {
            Ok(None)
        }
    }

    /// Clear all cached data
    pub fn clear_cache(&self) -> Result<()> {
        self.store.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movie_cache() {
        let cache = MovieCache::new("test_cache.json").unwrap();
        
        let movie = Movie {
            id: 1,
            title: "Test Movie".to_string(),
            tagline: Some("A test".to_string()),
            vote_average: Some(8.5),
            vote_count: Some(1000),
            popularity: Some(100.0),
            release_date: Some("2024-01-01".to_string()),
        };

        // Test caching
        cache.cache_movie(&movie).unwrap();
        
        // Test retrieval
        let cached = cache.get_movie(1).unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().title, "Test Movie");

        // Cleanup
        std::fs::remove_file("test_cache.json").ok();
    }
}