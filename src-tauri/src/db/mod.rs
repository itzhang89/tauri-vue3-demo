use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: i64,
    pub context_id: i64,
    pub name: String,
    pub data_type: String, // mysql, postgresql, sqlserver, kafka
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
    pub username: String,
    pub password: String, // TODO: 加密存储
    pub proxy_type: Option<String>, // socks5, http, ssh
    pub proxy_config: Option<serde_json::Value>,
    pub ssh_config: Option<serde_json::Value>,
    pub schema_registry_url: Option<String>, // For Kafka
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataCache {
    pub id: i64,
    pub data_source_id: i64,
    pub cache_type: String, // tables, schemas, topics, etc.
    pub cache_key: String,
    pub cache_data: serde_json::Value,
    pub cached_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let conn = Connection::open(&db_path)
            .context("Failed to open database connection")?;
        
        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn get_db_path() -> Result<PathBuf> {
        // Use Tauri's app data directory
        // For now, use a local path. In production, use tauri::api::path::app_data_dir()
        let mut path = std::env::current_dir()?;
        path.push("data");
        path.push("app.db");
        
        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        Ok(path)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS contexts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS data_sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                context_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                data_type TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                database TEXT,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                proxy_type TEXT,
                proxy_config TEXT,
                ssh_config TEXT,
                schema_registry_url TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (context_id) REFERENCES contexts(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                data_source_id INTEGER NOT NULL,
                cache_type TEXT NOT NULL,
                cache_key TEXT NOT NULL,
                cache_data TEXT NOT NULL,
                cached_at TEXT NOT NULL,
                expires_at TEXT,
                FOREIGN KEY (data_source_id) REFERENCES data_sources(id) ON DELETE CASCADE,
                UNIQUE(data_source_id, cache_type, cache_key)
            )",
            [],
        )?;

        // Create indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_data_sources_context_id ON data_sources(context_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_metadata_cache_data_source_id ON metadata_cache(data_source_id)",
            [],
        )?;

        Ok(())
    }

    // Context CRUD operations
    pub fn create_context(&self, name: &str, description: Option<&str>) -> Result<i64> {
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO contexts (name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![name, description, now.to_rfc3339(), now.to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_contexts(&self) -> Result<Vec<Context>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM contexts ORDER BY created_at DESC"
        )?;
        
        let contexts = stmt.query_map([], |row| {
            Ok(Context {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
        
        Ok(contexts)
    }

    pub fn update_context(&self, id: i64, name: &str, description: Option<&str>) -> Result<()> {
        let now = Utc::now();
        self.conn.execute(
            "UPDATE contexts SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, description, now.to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn delete_context(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM contexts WHERE id = ?1", params![id])?;
        Ok(())
    }

    // DataSource CRUD operations
    pub fn create_data_source(&self, ds: &DataSource) -> Result<i64> {
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO data_sources (
                context_id, name, data_type, host, port, database, username, password,
                proxy_type, proxy_config, ssh_config, schema_registry_url, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                ds.context_id,
                ds.name,
                ds.data_type,
                ds.host,
                ds.port,
                ds.database,
                ds.username,
                ds.password,
                ds.proxy_type,
                ds.proxy_config.as_ref().map(|v| serde_json::to_string(v).unwrap()),
                ds.ssh_config.as_ref().map(|v| serde_json::to_string(v).unwrap()),
                ds.schema_registry_url,
                now.to_rfc3339(),
                now.to_rfc3339()
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_data_sources(&self, context_id: Option<i64>) -> Result<Vec<DataSource>> {
        let query = if context_id.is_some() {
            "SELECT id, context_id, name, data_type, host, port, database, username, password,
                    proxy_type, proxy_config, ssh_config, schema_registry_url, created_at, updated_at
             FROM data_sources WHERE context_id = ?1 ORDER BY created_at DESC"
        } else {
            "SELECT id, context_id, name, data_type, host, port, database, username, password,
                    proxy_type, proxy_config, ssh_config, schema_registry_url, created_at, updated_at
             FROM data_sources ORDER BY created_at DESC"
        };

        let mut stmt = self.conn.prepare(query)?;
        
        let data_sources = if let Some(cid) = context_id {
            stmt.query_map(params![cid], |row| self.row_to_data_source(row))?
        } else {
            stmt.query_map([], |row| self.row_to_data_source(row))?
        }
        .collect::<SqliteResult<Vec<_>>>()?;
        
        Ok(data_sources)
    }

    fn row_to_data_source(&self, row: &rusqlite::Row) -> SqliteResult<DataSource> {
        Ok(DataSource {
            id: row.get(0)?,
            context_id: row.get(1)?,
            name: row.get(2)?,
            data_type: row.get(3)?,
            host: row.get(4)?,
            port: row.get(5)?,
            database: row.get(6)?,
            username: row.get(7)?,
            password: row.get(8)?,
            proxy_type: row.get(9)?,
            proxy_config: row.get::<_, Option<String>>(10)?
                .map(|s| serde_json::from_str(&s).unwrap_or(serde_json::Value::Null)),
            ssh_config: row.get::<_, Option<String>>(11)?
                .map(|s| serde_json::from_str(&s).unwrap_or(serde_json::Value::Null)),
            schema_registry_url: row.get(12)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?)
                .unwrap()
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(14)?)
                .unwrap()
                .with_timezone(&Utc),
        })
    }

    pub fn get_data_source(&self, id: i64) -> Result<DataSource> {
        let mut stmt = self.conn.prepare(
            "SELECT id, context_id, name, data_type, host, port, database, username, password,
                    proxy_type, proxy_config, ssh_config, schema_registry_url, created_at, updated_at
             FROM data_sources WHERE id = ?1"
        )?;
        
        let ds = stmt.query_row(params![id], |row| self.row_to_data_source(row))?;
        Ok(ds)
    }

    pub fn update_data_source(&self, ds: &DataSource) -> Result<()> {
        let now = Utc::now();
        self.conn.execute(
            "UPDATE data_sources SET
                context_id = ?1, name = ?2, data_type = ?3, host = ?4, port = ?5,
                database = ?6, username = ?7, password = ?8, proxy_type = ?9,
                proxy_config = ?10, ssh_config = ?11, schema_registry_url = ?12, updated_at = ?13
             WHERE id = ?14",
            params![
                ds.context_id,
                ds.name,
                ds.data_type,
                ds.host,
                ds.port,
                ds.database,
                ds.username,
                ds.password,
                ds.proxy_type,
                ds.proxy_config.as_ref().map(|v| serde_json::to_string(v).unwrap()),
                ds.ssh_config.as_ref().map(|v| serde_json::to_string(v).unwrap()),
                ds.schema_registry_url,
                now.to_rfc3339(),
                ds.id
            ],
        )?;
        Ok(())
    }

    pub fn delete_data_source(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM data_sources WHERE id = ?1", params![id])?;
        Ok(())
    }

    // MetadataCache operations
    pub fn save_metadata_cache(&self, cache: &MetadataCache) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO metadata_cache
                (data_source_id, cache_type, cache_key, cache_data, cached_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                cache.data_source_id,
                cache.cache_type,
                cache.cache_key,
                serde_json::to_string(&cache.cache_data)?,
                cache.cached_at.to_rfc3339(),
                cache.expires_at.map(|dt| dt.to_rfc3339())
            ],
        )?;
        Ok(())
    }

    pub fn get_metadata_cache(
        &self,
        data_source_id: i64,
        cache_type: &str,
        cache_key: &str,
    ) -> Result<Option<MetadataCache>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, data_source_id, cache_type, cache_key, cache_data, cached_at, expires_at
             FROM metadata_cache
             WHERE data_source_id = ?1 AND cache_type = ?2 AND cache_key = ?3"
        )?;
        
        let result = stmt.query_row(
            params![data_source_id, cache_type, cache_key],
            |row| {
                let expires_at_str: Option<String> = row.get(6)?;
                let expires_at = expires_at_str
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc));
                
                // Check if expired
                if let Some(exp) = expires_at {
                    if exp < Utc::now() {
                        return Err(rusqlite::Error::InvalidColumnType(6, "expired".to_string(), rusqlite::types::Type::Text));
                    }
                }
                
                Ok(MetadataCache {
                    id: row.get(0)?,
                    data_source_id: row.get(1)?,
                    cache_type: row.get(2)?,
                    cache_key: row.get(3)?,
                    cache_data: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or(serde_json::Value::Null),
                    cached_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    expires_at,
                })
            },
        );
        
        match result {
            Ok(cache) => Ok(Some(cache)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }

    pub fn delete_metadata_cache(&self, data_source_id: i64, cache_type: Option<&str>) -> Result<()> {
        if let Some(ct) = cache_type {
            self.conn.execute(
                "DELETE FROM metadata_cache WHERE data_source_id = ?1 AND cache_type = ?2",
                params![data_source_id, ct],
            )?;
        } else {
            self.conn.execute(
                "DELETE FROM metadata_cache WHERE data_source_id = ?1",
                params![data_source_id],
            )?;
        }
        Ok(())
    }
}

// Global database instance
use std::sync::Mutex;
use once_cell::sync::Lazy;

static DB: Lazy<Mutex<Option<Database>>> = Lazy::new(|| Mutex::new(None));

pub fn init_db() -> Result<()> {
    let db = Database::new()?;
    *DB.lock().unwrap() = Some(db);
    Ok(())
}

pub fn get_db() -> Result<std::sync::MutexGuard<'static, Option<Database>>> {
    let guard = DB.lock().unwrap();
    if guard.is_none() {
        drop(guard);
        init_db()?;
        Ok(DB.lock().unwrap())
    } else {
        Ok(guard)
    }
}

