use crate::connection::sqlserver::SQLServerConnector;
use crate::db::DataSource;
use crate::metadata::{ColumnInfo, TableInfo};
use anyhow::{Context, Result};
use tiberius::{Client, Query, Row};
use tokio_util::compat::TokioAsyncWriteCompatExt;
use tokio::net::TcpStream;

pub struct SQLServerMetadata;

impl SQLServerMetadata {
    pub async fn get_tables(data_source: &DataSource) -> Result<Vec<TableInfo>> {
        let mut client = SQLServerConnector::create_client(data_source).await?;
        let schema = data_source.database.as_deref().unwrap_or("dbo");
        
        let query = format!(
            "SELECT table_name FROM information_schema.tables 
             WHERE table_schema = '{}' AND table_type = 'BASE TABLE'
             ORDER BY table_name",
            schema
        );
        
        let mut stream = Query::new(query).query(&mut client).await?;
        let mut tables = Vec::new();
        
        while let Some(row) = stream.into_row().await? {
            if let Some(table_name) = row.get::<&str, _>(0) {
                let structure = Self::get_table_structure(data_source, Some(schema), table_name).await?;
                tables.push(structure);
            }
        }
        
        Ok(tables)
    }

    pub async fn get_table_structure(
        data_source: &DataSource,
        schema: Option<&str>,
        table_name: &str,
    ) -> Result<TableInfo> {
        let mut client = SQLServerConnector::create_client(data_source).await?;
        let schema = schema.unwrap_or_else(|| data_source.database.as_deref().unwrap_or("dbo"));
        
        let query = format!(
            "SELECT 
                c.column_name, 
                c.data_type, 
                c.is_nullable,
                c.column_default,
                (SELECT STRING_AGG(tc.constraint_type, ', ')
                 FROM information_schema.table_constraints tc
                 JOIN information_schema.key_column_usage kcu 
                   ON tc.constraint_name = kcu.constraint_name
                 WHERE tc.table_schema = '{}' 
                   AND tc.table_name = '{}'
                   AND kcu.column_name = c.column_name) as constraints
            FROM information_schema.columns c
            WHERE c.table_schema = '{}' AND c.table_name = '{}'
            ORDER BY c.ordinal_position",
            schema, table_name, schema, table_name
        );
        
        let mut stream = Query::new(query).query(&mut client).await?;
        let mut columns = Vec::new();
        
        while let Some(row) = stream.into_row().await? {
            let name: String = row.get(0).unwrap_or_default();
            let data_type: String = row.get(1).unwrap_or_default();
            let is_nullable: String = row.get(2).unwrap_or_default();
            let default_value: Option<String> = row.get(3);
            let constraints_str: Option<String> = row.get(4);
            
            let constraints = constraints_str
                .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                .unwrap_or_default();
            
            columns.push(ColumnInfo {
                name,
                data_type,
                is_nullable: is_nullable == "YES",
                default_value,
                constraints,
            });
        }
        
        Ok(TableInfo {
            name: table_name.to_string(),
            schema: Some(schema.to_string()),
            row_count: None,
            columns,
        })
    }

    pub async fn get_table_row_count(
        data_source: &DataSource,
        schema: Option<&str>,
        table_name: &str,
    ) -> Result<i64> {
        let mut client = SQLServerConnector::create_client(data_source).await?;
        let schema = schema.unwrap_or_else(|| data_source.database.as_deref().unwrap_or("dbo"));
        
        let query = format!(
            "SELECT COUNT(*) as count FROM [{}].[{}]",
            schema, table_name
        );
        
        let mut stream = Query::new(query).query(&mut client).await?;
        if let Some(row) = stream.into_row().await? {
            if let Some(count) = row.get::<i32, _>(0) {
                return Ok(count as i64);
            }
        }
        
        Ok(0)
    }
}

