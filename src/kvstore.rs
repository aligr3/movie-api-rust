use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoreData {
    data: HashMap<String, String>,
}

/// Thread-safe, fault-tolerant key-value store
/// Supports both JSON and YAML formats
#[derive(Clone)]
pub struct KVStore {
    data: Arc<RwLock<HashMap<String, String>>>,
    file_path: String,
    format: StorageFormat,
}

#[derive(Clone, Copy)]
pub enum StorageFormat {
    Json,
    Yaml,
}

impl KVStore {
    /// Create a new KVStore with the specified file path and format
    pub fn new(file_path: &str, format: StorageFormat) -> Result<Self> {
        let mut store = Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            file_path: file_path.to_string(),
            format,
        };
        
        // Try to load existing data, create new file if it doesn't exist
        if Path::new(file_path).exists() {
            store.load()?;
        } else {
            store.save()?; // Create empty file
        }
        
        Ok(store)
    }

    /// Set a key-value pair
    pub fn set(&self, key: String, value: String) -> Result<()> {
        {
            let mut data = self.data.write()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            data.insert(key, value);
        }
        self.save()
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let data = self.data.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(data.get(key).cloned())
    }

    /// Delete a key-value pair
    pub fn delete(&self, key: &str) -> Result<bool> {
        let removed = {
            let mut data = self.data.write()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            data.remove(key).is_some()
        };
        
        if removed {
            self.save()?;
        }
        
        Ok(removed)
    }

    /// Check if a key exists
    pub fn exists(&self, key: &str) -> Result<bool> {
        let data = self.data.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(data.contains_key(key))
    }

    /// Get all keys
    pub fn keys(&self) -> Result<Vec<String>> {
        let data = self.data.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(data.keys().cloned().collect())
    }

    /// Get the number of key-value pairs
    pub fn len(&self) -> Result<usize> {
        let data = self.data.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(data.len())
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Clear all key-value pairs
    pub fn clear(&self) -> Result<()> {
        {
            let mut data = self.data.write()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            data.clear();
        }
        self.save()
    }

    /// Save data to file (fault-tolerant with atomic write)
    fn save(&self) -> Result<()> {
        let data = self.data.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        let store_data = StoreData {
            data: data.clone(),
        };

        // Write to temporary file first (atomic operation)
        let temp_path = format!("{}.tmp", self.file_path);
        let temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .context("Failed to create temp file")?;

        let mut writer = BufWriter::new(temp_file);

        match self.format {
            StorageFormat::Json => {
                serde_json::to_writer_pretty(&mut writer, &store_data)
                    .context("Failed to serialize to JSON")?;
            }
            StorageFormat::Yaml => {
                serde_yaml::to_writer(&mut writer, &store_data)
                    .context("Failed to serialize to YAML")?;
            }
        }

        writer.flush().context("Failed to flush writer")?;
        drop(writer);

        // Atomically replace old file with new one
        std::fs::rename(&temp_path, &self.file_path)
            .context("Failed to replace file")?;

        Ok(())
    }

    /// Load data from file
    fn load(&mut self) -> Result<()> {
        let file = File::open(&self.file_path)
            .context("Failed to open file")?;
        
        let reader = BufReader::new(file);

        let store_data: StoreData = match self.format {
            StorageFormat::Json => {
                serde_json::from_reader(reader)
                    .context("Failed to deserialize JSON")?
            }
            StorageFormat::Yaml => {
                serde_yaml::from_reader(reader)
                    .context("Failed to deserialize YAML")?
            }
        };

        let mut data = self.data.write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        *data = store_data.data;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::env;

    #[test]
    fn test_basic_operations() {
        println!("\n=== Test Basic Operations ===");
        println!("Arbeitsverzeichnis: {:?}", env::current_dir().unwrap());
        
        let store = KVStore::new("test_store.json", StorageFormat::Json).unwrap();
        
        // Test set and get
        store.set("key1".to_string(), "value1".to_string()).unwrap();
        store.set("name".to_string(), "Alice".to_string()).unwrap();
        store.set("age".to_string(), "30".to_string()).unwrap();
        
        println!("Alle Keys: {:?}", store.keys().unwrap());
        println!("JSON-Datei erstellt: test_store.json");
        
        assert_eq!(store.get("key1").unwrap(), Some("value1".to_string()));
        
        // Test exists
        assert!(store.exists("key1").unwrap());
        assert!(!store.exists("key2").unwrap());
        
        // Test delete
        assert!(store.delete("key1").unwrap());
        assert_eq!(store.get("key1").unwrap(), None);
        
        println!("Keys nach Delete: {:?}", store.keys().unwrap());
        
        std::fs::remove_file("test_store.json").ok();
    }

    #[test]
    fn test_create_sample_json() {
        println!("\n=== Erstelle Beispiel JSON ===");
        println!("Arbeitsverzeichnis: {:?}", env::current_dir().unwrap());
        
        let store = KVStore::new("my_data.json", StorageFormat::Json).unwrap();
        
        // Füge verschiedene Daten hinzu
        store.set("username".to_string(), "Max".to_string()).unwrap();
        store.set("email".to_string(), "max@example.com".to_string()).unwrap();
        store.set("city".to_string(), "Berlin".to_string()).unwrap();
        store.set("favorite_color".to_string(), "blue".to_string()).unwrap();
        
        println!("\n✅ JSON-Datei erstellt: my_data.json");
        println!("Anzahl Keys: {}", store.len().unwrap());
        println!("Alle Keys: {:?}", store.keys().unwrap());
        
        // Zeige Inhalt
        for key in store.keys().unwrap() {
            let value = store.get(&key).unwrap().unwrap();
            println!("  {} = {}", key, value);
        }
        
        // Cleanup auskommentiert, damit die Datei erhalten bleibt
        // std::fs::remove_file("my_data.json").ok();
    }

    #[test]
    fn test_thread_safety() {
        let store = KVStore::new("test_concurrent.json", StorageFormat::Json).unwrap();
        let mut handles = vec![];

        for i in 0..10 {
            let store_clone = store.clone();
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    let value = format!("value_{}_{}", i, j);
                    store_clone.set(key.clone(), value.clone()).unwrap();
                    assert_eq!(store_clone.get(&key).unwrap(), Some(value));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(store.len().unwrap(), 1000);
        std::fs::remove_file("test_concurrent.json").ok();
    }

    #[test]
    fn test_persistence() {
        {
            let store = KVStore::new("test_persist.yaml", StorageFormat::Yaml).unwrap();
            store.set("persist_key".to_string(), "persist_value".to_string()).unwrap();
        }

        // Create new instance and check if data persists
        let store = KVStore::new("test_persist.yaml", StorageFormat::Yaml).unwrap();
        assert_eq!(store.get("persist_key").unwrap(), Some("persist_value".to_string()));
        
        std::fs::remove_file("test_persist.yaml").ok();
    }
}