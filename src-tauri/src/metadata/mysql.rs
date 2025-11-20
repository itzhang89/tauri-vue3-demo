use crate::connection::mysql::MySQLConnector;
use crate::db::DataSource;
use crate::metadata::{ColumnInfo, TableInfo};
use anyhow::{Context, Result};
use sqlx::MySqlPool;

pub struct MySQLMetadata;

impl MySQLMetadata {
    pub async fn get_tables(data_source: &DataSource) -> Result<Vec<TableInfo>> {
        let pool = MySQLConnector::create_pool(data_source).await?;
        let database = data_source.database.as_deref().unwrap_or("information_schema");
        
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = ? ORDER BY table_name"
        )
        .bind(database)
        .fetch_all(&pool)
        .await?;
        
        let mut result = Vec::new();
        for (table_name,) in tables {
            let structure = Self::get_table_structure(data_source, Some(database), &table_name).await?;
            result.push(structure);
        }
        
        Ok(result)
    }

    pub async fn get_table_structure(
        data_source: &DataSource,
        schema: Option<&str>,
        table_name: &str,
    ) -> Result<TableInfo> {
        let pool = MySQLConnector::create_pool(data_source).await?;
        let schema = schema.unwrap_or_else(|| data_source.database.as_deref().unwrap_or("information_schema"));
        
        let columns: Vec<(String, String, String, Option<String>, String)> = sqlx::query_as(
            "SELECT 
                column_name, 
                data_type, 
                is_nullable,
                column_default,
                column_key
            FROM information_schema.columns 
            WHERE table_schema = ? AND table_name = ?
            ORDER BY ordinal_position"
        )
        .bind(schema)
        .bind(table_name)
        .fetch_all(&pool)
        .await?;
        
        let column_infos: Vec<ColumnInfo> = columns
            .into_iter()
            .map(|(name, data_type, is_nullable, default_value, column_key)| {
                let mut constraints = Vec::new();
                if column_key == "PRI" {
                    constraints.push("PRIMARY KEY".to_string());
                }
                
                ColumnInfo {
                    name,
                    data_type,
                    is_nullable: is_nullable == "YES",
                    default_value,
                    constraints,
                }
            })
            .collect();
        
        Ok(TableInfo {
            name: table_name.to_string(),
            schema: Some(schema.to_string()),
            row_count: None,
            columns: column_infos,
        })
    }

    pub async fn get_table_row_count(
        data_source: &DataSource,
        schema: Option<&str>,
        table_name: &str,
    ) -> Result<i64> {
        let pool = MySQLConnector::create_pool(data_source).await?;
        let schema = schema.unwrap_or_else(|| data_source.database.as_deref().unwrap_or("information_schema"));
        
        let query = format!("SELECT COUNT(*) as count FROM `{}`.`{}`", schema, table_name);
        let row: (i64,) = sqlx::query_as(&query).fetch_one(&pool).await?;
        
        Ok(row.0)
    }
}

