use crate::db::{DataSource, MetadataCache};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod mysql;
pub mod postgresql;
pub mod sqlserver;
pub mod kafka;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: Option<String>,
    pub row_count: Option<i64>,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableComparison {
    pub table_name: String,
    pub source1: TableInfo,
    pub source2: TableInfo,
    pub structure_diff: Vec<StructureDiff>,
    pub row_count_diff: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureDiff {
    pub column_name: String,
    pub diff_type: String, // added, removed, modified
    pub source1_value: Option<String>,
    pub source2_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaTopicInfo {
    pub name: String,
    pub partitions: Vec<PartitionInfo>,
    pub consumer_groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub id: i32,
    pub leader: i32,
    pub replicas: Vec<i32>,
    pub isr: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub subject: String,
    pub version: i32,
    pub schema_type: String,
    pub schema: String,
}

pub struct MetadataFetcher;

impl MetadataFetcher {
    pub async fn get_tables(data_source: &DataSource) -> Result<Vec<TableInfo>> {
        match data_source.data_type.as_str() {
            "mysql" => mysql::MySQLMetadata::get_tables(data_source).await,
            "postgresql" => postgresql::PostgreSQLMetadata::get_tables(data_source).await,
            "sqlserver" => sqlserver::SQLServerMetadata::get_tables(data_source).await,
            _ => Err(anyhow::anyhow!("Unsupported data source type for table metadata: {}", data_source.data_type)),
        }
    }

    pub async fn get_table_structure(
        data_source: &DataSource,
        schema: Option<&str>,
        table_name: &str,
    ) -> Result<TableInfo> {
        match data_source.data_type.as_str() {
            "mysql" => mysql::MySQLMetadata::get_table_structure(data_source, schema, table_name).await,
            "postgresql" => postgresql::PostgreSQLMetadata::get_table_structure(data_source, schema, table_name).await,
            "sqlserver" => sqlserver::SQLServerMetadata::get_table_structure(data_source, schema, table_name).await,
            _ => Err(anyhow::anyhow!("Unsupported data source type for table structure: {}", data_source.data_type)),
        }
    }

    pub async fn get_table_row_count(
        data_source: &DataSource,
        schema: Option<&str>,
        table_name: &str,
    ) -> Result<i64> {
        match data_source.data_type.as_str() {
            "mysql" => mysql::MySQLMetadata::get_table_row_count(data_source, schema, table_name).await,
            "postgresql" => postgresql::PostgreSQLMetadata::get_table_row_count(data_source, schema, table_name).await,
            "sqlserver" => sqlserver::SQLServerMetadata::get_table_row_count(data_source, schema, table_name).await,
            _ => Err(anyhow::anyhow!("Unsupported data source type for row count: {}", data_source.data_type)),
        }
    }

    pub async fn get_kafka_topics(data_source: &DataSource) -> Result<Vec<KafkaTopicInfo>> {
        kafka::KafkaMetadata::get_topics(data_source).await
    }

    pub async fn get_kafka_consumer_groups(data_source: &DataSource) -> Result<Vec<String>> {
        kafka::KafkaMetadata::get_consumer_groups(data_source).await
    }

    pub async fn get_schema_registry_schemas(
        data_source: &DataSource,
    ) -> Result<Vec<SchemaInfo>> {
        kafka::KafkaMetadata::get_schema_registry_schemas(data_source).await
    }

    pub async fn compare_tables(
        source1: &DataSource,
        source2: &DataSource,
        schema1: Option<&str>,
        schema2: Option<&str>,
        table_name: &str,
    ) -> Result<TableComparison> {
        let table1 = Self::get_table_structure(source1, schema1, table_name).await?;
        let table2 = Self::get_table_structure(source2, schema2, table_name).await?;
        
        let row_count1 = Self::get_table_row_count(source1, schema1, table_name).await?;
        let row_count2 = Self::get_table_row_count(source2, schema2, table_name).await?;
        
        let mut table1_final = table1.clone();
        table1_final.row_count = Some(row_count1);
        
        let mut table2_final = table2.clone();
        table2_final.row_count = Some(row_count2);
        
        let structure_diff = Self::compare_structure(&table1, &table2);
        let row_count_diff = Some(row_count1 - row_count2);
        
        Ok(TableComparison {
            table_name: table_name.to_string(),
            source1: table1_final,
            source2: table2_final,
            structure_diff,
            row_count_diff,
        })
    }

    fn compare_structure(table1: &TableInfo, table2: &TableInfo) -> Vec<StructureDiff> {
        let mut diffs = Vec::new();
        
        let mut cols1: HashMap<String, &ColumnInfo> = table1.columns.iter()
            .map(|c| (c.name.clone(), c))
            .collect();
        let mut cols2: HashMap<String, &ColumnInfo> = table2.columns.iter()
            .map(|c| (c.name.clone(), c))
            .collect();
        
        // Find added columns (in table2 but not in table1)
        for (name, col2) in &cols2 {
            if !cols1.contains_key(name) {
                diffs.push(StructureDiff {
                    column_name: name.clone(),
                    diff_type: "added".to_string(),
                    source1_value: None,
                    source2_value: Some(format!("{:?}", col2)),
                });
            }
        }
        
        // Find removed columns (in table1 but not in table2)
        for (name, col1) in &cols1 {
            if !cols2.contains_key(name) {
                diffs.push(StructureDiff {
                    column_name: name.clone(),
                    diff_type: "removed".to_string(),
                    source1_value: Some(format!("{:?}", col1)),
                    source2_value: None,
                });
            }
        }
        
        // Find modified columns
        for (name, col1) in &cols1 {
            if let Some(col2) = cols2.get(name) {
                if col1.data_type != col2.data_type
                    || col1.is_nullable != col2.is_nullable
                    || col1.default_value != col2.default_value
                {
                    diffs.push(StructureDiff {
                        column_name: name.clone(),
                        diff_type: "modified".to_string(),
                        source1_value: Some(format!("{:?}", col1)),
                        source2_value: Some(format!("{:?}", col2)),
                    });
                }
            }
        }
        
        diffs
    }
}

