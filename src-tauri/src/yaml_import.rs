use crate::commands::CreateDataSourceRequest;
use crate::db::{get_db, Database};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlDataSource {
    pub name: String,
    pub data_type: String, // mysql, postgresql, sqlserver, kafka
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
    pub username: String,
    pub password: String,
    pub proxy_type: Option<String>, // socks5, http, ssh
    pub proxy_config: Option<serde_yaml::Value>,
    pub ssh_config: Option<serde_yaml::Value>,
    pub schema_registry_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlImportData {
    pub context_id: Option<i64>,
    pub context_name: Option<String>,
    pub data_sources: Vec<YamlDataSource>,
}

#[tauri::command]
pub async fn import_data_sources_from_yaml(
    file_path: String,
    context_id: Option<i64>,
) -> Result<Vec<i64>, String> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let import_data: YamlImportData = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;
    
    let db_guard = get_db().map_err(|e| e.to_string())?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    
    // Determine context_id
    let final_context_id = if let Some(cid) = context_id {
        cid
    } else if let Some(context_name) = &import_data.context_name {
        // Create or get context
        let contexts = db.list_contexts().map_err(|e| e.to_string())?;
        if let Some(existing) = contexts.iter().find(|c| c.name == *context_name) {
            existing.id
        } else {
            db.create_context(context_name, None).map_err(|e| e.to_string())?
        }
    } else if let Some(cid) = import_data.context_id {
        cid
    } else {
        return Err("Context ID or context name must be provided".to_string());
    };
    
    let mut created_ids = Vec::new();
    
    for yaml_ds in import_data.data_sources {
        // Convert YAML values to JSON values
        let proxy_config = yaml_ds.proxy_config.map(|v| {
            serde_json::to_value(&v).unwrap_or(serde_json::Value::Null)
        });
        
        let ssh_config = yaml_ds.ssh_config.map(|v| {
            serde_json::to_value(&v).unwrap_or(serde_json::Value::Null)
        });
        
        let req = CreateDataSourceRequest {
            context_id: final_context_id,
            name: yaml_ds.name,
            data_type: yaml_ds.data_type,
            host: yaml_ds.host,
            port: yaml_ds.port,
            database: yaml_ds.database,
            username: yaml_ds.username,
            password: yaml_ds.password,
            proxy_type: yaml_ds.proxy_type,
            proxy_config,
            ssh_config,
            schema_registry_url: yaml_ds.schema_registry_url,
        };
        
        let data_source = crate::db::DataSource {
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
        
        let id = db.create_data_source(&data_source).map_err(|e| e.to_string())?;
        created_ids.push(id);
    }
    
    Ok(created_ids)
}

