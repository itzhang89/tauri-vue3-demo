use crate::connection::postgresql::PostgreSQLConnector;
use crate::db::DataSource;
use crate::metadata::{ColumnInfo, TableInfo};
use anyhow::{Context, Result};
use sqlx::PgPool;

pub struct PostgreSQLMetadata;

impl PostgreSQLMetadata {
    pub async fn get_tables(data_source: &DataSource) -> Result<Vec<TableInfo>> {
        let pool = PostgreSQLConnector::create_pool(data_source).await?;
        let schema = data_source.database.as_deref().unwrap_or("public");
        
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables 
             WHERE table_schema = $1 AND table_type = 'BASE TABLE'
             ORDER BY table_name"
        )
        .bind(schema)
        .fetch_all(&pool)
        .await?;
        
        let mut result = Vec::new();
        for (table_name,) in tables {
            let structure = Self::get_table_structure(data_source, Some(schema), &table_name).await?;
            result.push(structure);
        }
        
        Ok(result)
    }

    pub async fn get_table_structure(
        data_source: &DataSource,
        schema: Option<&str>,
        table_name: &str,
    ) -> Result<TableInfo> {
        let pool = PostgreSQLConnector::create_pool(data_source).await?;
        let schema = schema.unwrap_or_else(|| data_source.database.as_deref().unwrap_or("public"));
        
        let columns: Vec<(String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT 
                column_name, 
                data_type, 
                is_nullable,
                column_default,
                (SELECT string_agg(constraint_type, ', ')
                 FROM information_schema.table_constraints tc
                 JOIN information_schema.key_column_usage kcu 
                   ON tc.constraint_name = kcu.constraint_name
                 WHERE tc.table_schema = $1 
                   AND tc.table_name = $2
                   AND kcu.column_name = c.column_name) as constraints
            FROM information_schema.columns c
            WHERE table_schema = $1 AND table_name = $2
            ORDER BY ordinal_position"
        )
        .bind(schema)
        .bind(table_name)
        .fetch_all(&pool)
        .await?;
        
        let column_infos: Vec<ColumnInfo> = columns
            .into_iter()
            .map(|(name, data_type, is_nullable, default_value, constraints_str)| {
                let constraints = constraints_str
                    .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                    .unwrap_or_default();
                
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
        let pool = PostgreSQLConnector::create_pool(data_source).await?;
        let schema = schema.unwrap_or_else(|| data_source.database.as_deref().unwrap_or("public"));
        
        let query = format!("SELECT COUNT(*) as count FROM \"{}\".\"{}\"", schema, table_name);
        let row: (i64,) = sqlx::query_as(&query).fetch_one(&pool).await?;
        
        Ok(row.0)
    }
}

