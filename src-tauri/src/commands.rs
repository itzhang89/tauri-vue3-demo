use crate::cache::CacheManager;
use crate::connection::ConnectionManager;
use crate::db::{Database, DataSource};
use crate::metadata::{MetadataFetcher, TableComparison};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// Context commands
#[tauri::command]
pub async fn create_context(name: String, description: Option<String>) -> Result<i64, String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        db.create_context(&name, description.as_deref())
            .map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn list_contexts() -> Result<Vec<crate::db::Context>, String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        db.list_contexts().map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn update_context(
    id: i64,
    name: String,
    description: Option<String>,
) -> Result<(), String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        db.update_context(id, &name, description.as_deref())
            .map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn delete_context(id: i64) -> Result<(), String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        db.delete_context(id).map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

// DataSource commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDataSourceRequest {
    pub context_id: i64,
    pub name: String,
    pub data_type: String,
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
    pub username: String,
    pub password: String,
    pub proxy_type: Option<String>,
    pub proxy_config: Option<serde_json::Value>,
    pub ssh_config: Option<serde_json::Value>,
    pub schema_registry_url: Option<String>,
}

#[tauri::command]
pub async fn create_data_source(req: CreateDataSourceRequest) -> Result<i64, String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        let data_source = DataSource {
            id: 0,
            context_id: req.context_id,
            name: req.name,
            data_type: req.data_type,
            host: req.host,
            port: req.port,
            database: req.database,
            username: req.username,
            password: req.password,
            proxy_type: req.proxy_type,
            proxy_config: req.proxy_config,
            ssh_config: req.ssh_config,
            schema_registry_url: req.schema_registry_url,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        db.create_data_source(&data_source).map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn list_data_sources(context_id: Option<i64>) -> Result<Vec<DataSource>, String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        db.list_data_sources(context_id).map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_data_source(id: i64) -> Result<DataSource, String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        db.get_data_source(id).map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn update_data_source(data_source: DataSource) -> Result<(), String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        db.update_data_source(&data_source).map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn delete_data_source(id: i64) -> Result<(), String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        db.delete_data_source(id).map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn test_connection(data_source: DataSource) -> Result<(), String> {
    ConnectionManager::test_connection(&data_source).map_err(|e| e.to_string())
}

// Metadata commands
#[tauri::command]
pub async fn get_tables(
    data_source_id: i64,
    force_refresh: bool,
) -> Result<Vec<crate::metadata::TableInfo>, String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        let data_source = db.get_data_source(data_source_id).map_err(|e| e.to_string())?;
        CacheManager::get_tables_cached(&data_source, force_refresh)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_table_structure(
    data_source_id: i64,
    schema: Option<String>,
    table_name: String,
    force_refresh: bool,
) -> Result<crate::metadata::TableInfo, String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        let data_source = db.get_data_source(data_source_id).map_err(|e| e.to_string())?;
        CacheManager::get_table_structure_cached(
            &data_source,
            schema.as_deref(),
            &table_name,
            force_refresh,
        )
        .await
        .map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_kafka_topics(
    data_source_id: i64,
    force_refresh: bool,
) -> Result<Vec<crate::metadata::KafkaTopicInfo>, String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        let data_source = db.get_data_source(data_source_id).map_err(|e| e.to_string())?;
        CacheManager::get_kafka_topics_cached(&data_source, force_refresh)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_schema_registry_schemas(
    data_source_id: i64,
    force_refresh: bool,
) -> Result<Vec<crate::metadata::SchemaInfo>, String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        let data_source = db.get_data_source(data_source_id).map_err(|e| e.to_string())?;
        CacheManager::get_schema_registry_schemas_cached(&data_source, force_refresh)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn refresh_metadata(
    data_source_id: i64,
    cache_type: Option<String>,
) -> Result<(), String> {
    CacheManager::clear_cache(data_source_id, cache_type.as_deref())
        .map_err(|e| e.to_string())
}

// Comparison command
#[tauri::command]
pub async fn compare_tables(
    source1_id: i64,
    source2_id: i64,
    schema1: Option<String>,
    schema2: Option<String>,
    table_name: String,
) -> Result<TableComparison, String> {
    let db = Database::get_db().map_err(|e| e.to_string())?;
    if let Some(ref db) = *db {
        let source1 = db.get_data_source(source1_id).map_err(|e| e.to_string())?;
        let source2 = db.get_data_source(source2_id).map_err(|e| e.to_string())?;
        
        MetadataFetcher::compare_tables(
            &source1,
            &source2,
            schema1.as_deref(),
            schema2.as_deref(),
            &table_name,
        )
        .await
        .map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

