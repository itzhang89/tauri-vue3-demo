use crate::db::{Database, DataSource, MetadataCache};
use crate::metadata::{MetadataFetcher, TableInfo, KafkaTopicInfo, SchemaInfo};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;

pub struct CacheManager;

impl CacheManager {
    const CACHE_EXPIRY_HOURS: i64 = 24;

    pub async fn get_tables_cached(
        data_source: &DataSource,
        force_refresh: bool,
    ) -> Result<Vec<TableInfo>> {
        let cache_key = format!("tables:{}", data_source.database.as_deref().unwrap_or("default"));
        
        if !force_refresh {
            if let Some(cached) = Self::get_cache(data_source.id, "tables", &cache_key)? {
                if let Ok(tables) = serde_json::from_value::<Vec<TableInfo>>(cached.cache_data.clone()) {
                    return Ok(tables);
                }
            }
        }
        
        // Fetch fresh data
        let tables = MetadataFetcher::get_tables(data_source).await?;
        
        // Cache the result
        let cache_data = serde_json::to_value(&tables)?;
        let expires_at = Utc::now() + Duration::hours(Self::CACHE_EXPIRY_HOURS);
        
        let cache = MetadataCache {
            id: 0,
            data_source_id: data_source.id,
            cache_type: "tables".to_string(),
            cache_key: cache_key.clone(),
            cache_data,
            cached_at: Utc::now(),
            expires_at: Some(expires_at),
        };
        
        Self::save_cache(&cache)?;
        Ok(tables)
    }

    pub async fn get_table_structure_cached(
        data_source: &DataSource,
        schema: Option<&str>,
        table_name: &str,
        force_refresh: bool,
    ) -> Result<TableInfo> {
        let cache_key = format!("table_structure:{}:{}", 
            schema.unwrap_or("default"), 
            table_name
        );
        
        if !force_refresh {
            if let Some(cached) = Self::get_cache(data_source.id, "table_structure", &cache_key)? {
                if let Ok(table) = serde_json::from_value::<TableInfo>(cached.cache_data.clone()) {
                    return Ok(table);
                }
            }
        }
        
        // Fetch fresh data
        let table = MetadataFetcher::get_table_structure(data_source, schema, table_name).await?;
        
        // Cache the result
        let cache_data = serde_json::to_value(&table)?;
        let expires_at = Utc::now() + Duration::hours(Self::CACHE_EXPIRY_HOURS);
        
        let cache = MetadataCache {
            id: 0,
            data_source_id: data_source.id,
            cache_type: "table_structure".to_string(),
            cache_key: cache_key.clone(),
            cache_data,
            cached_at: Utc::now(),
            expires_at: Some(expires_at),
        };
        
        Self::save_cache(&cache)?;
        Ok(table)
    }

    pub async fn get_kafka_topics_cached(
        data_source: &DataSource,
        force_refresh: bool,
    ) -> Result<Vec<KafkaTopicInfo>> {
        let cache_key = "topics".to_string();
        
        if !force_refresh {
            if let Some(cached) = Self::get_cache(data_source.id, "topics", &cache_key)? {
                if let Ok(topics) = serde_json::from_value::<Vec<KafkaTopicInfo>>(cached.cache_data.clone()) {
                    return Ok(topics);
                }
            }
        }
        
        // Fetch fresh data
        let topics = MetadataFetcher::get_kafka_topics(data_source).await?;
        
        // Cache the result
        let cache_data = serde_json::to_value(&topics)?;
        let expires_at = Utc::now() + Duration::hours(Self::CACHE_EXPIRY_HOURS);
        
        let cache = MetadataCache {
            id: 0,
            data_source_id: data_source.id,
            cache_type: "topics".to_string(),
            cache_key: cache_key.clone(),
            cache_data,
            cached_at: Utc::now(),
            expires_at: Some(expires_at),
        };
        
        Self::save_cache(&cache)?;
        Ok(topics)
    }

    pub async fn get_schema_registry_schemas_cached(
        data_source: &DataSource,
        force_refresh: bool,
    ) -> Result<Vec<SchemaInfo>> {
        let cache_key = "schemas".to_string();
        
        if !force_refresh {
            if let Some(cached) = Self::get_cache(data_source.id, "schemas", &cache_key)? {
                if let Ok(schemas) = serde_json::from_value::<Vec<SchemaInfo>>(cached.cache_data.clone()) {
                    return Ok(schemas);
                }
            }
        }
        
        // Fetch fresh data
        let schemas = MetadataFetcher::get_schema_registry_schemas(data_source).await?;
        
        // Cache the result
        let cache_data = serde_json::to_value(&schemas)?;
        let expires_at = Utc::now() + Duration::hours(Self::CACHE_EXPIRY_HOURS);
        
        let cache = MetadataCache {
            id: 0,
            data_source_id: data_source.id,
            cache_type: "schemas".to_string(),
            cache_key: cache_key.clone(),
            cache_data,
            cached_at: Utc::now(),
            expires_at: Some(expires_at),
        };
        
        Self::save_cache(&cache)?;
        Ok(schemas)
    }

    pub fn clear_cache(data_source_id: i64, cache_type: Option<&str>) -> Result<()> {
        use crate::db::get_db;
        let db_guard = get_db()?;
        if let Some(ref db) = *db_guard {
            db.delete_metadata_cache(data_source_id, cache_type)?;
        }
        Ok(())
    }

    fn get_cache(
        data_source_id: i64,
        cache_type: &str,
        cache_key: &str,
    ) -> Result<Option<MetadataCache>> {
        use crate::db::get_db;
        // Get cache data before any async operations
        let cached_data = {
            let db_guard = get_db()?;
            if let Some(ref db) = *db_guard {
                db.get_metadata_cache(data_source_id, cache_type, cache_key)?
            } else {
                None
            }
        };
        Ok(cached_data)
    }
    
    fn save_cache(cache: &MetadataCache) -> Result<()> {
        use crate::db::get_db;
        let db_guard = get_db()?;
        if let Some(ref db) = *db_guard {
            db.save_metadata_cache(cache)?;
        }
        Ok(())
    }
}

