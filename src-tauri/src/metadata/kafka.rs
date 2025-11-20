use crate::connection::kafka::KafkaConnector;
use crate::db::DataSource;
use crate::metadata::{KafkaTopicInfo, PartitionInfo, SchemaInfo};
use anyhow::{Context, Result};
use std::time::Duration;
use reqwest::Client as HttpClient;

pub struct KafkaMetadata;

impl KafkaMetadata {
    pub async fn get_topics(data_source: &DataSource) -> Result<Vec<KafkaTopicInfo>> {
        let admin_client = KafkaConnector::create_admin_client(data_source).await?;
        
        let metadata = admin_client
            .inner()
            .fetch_metadata(None, Duration::from_secs(10))
            .context("Failed to fetch Kafka metadata")?;
        
        let mut topics = Vec::new();
        
        for topic in metadata.topics() {
            let mut partitions = Vec::new();
            
            for partition in topic.partitions() {
                partitions.push(PartitionInfo {
                    id: partition.id(),
                    leader: partition.leader(),
                    replicas: partition.replicas().to_vec(),
                    isr: partition.isr().to_vec(),
                });
            }
            
            // Get consumer groups for this topic
            let consumer_groups = Self::get_consumer_groups_for_topic(data_source, topic.name()).await?;
            
            topics.push(KafkaTopicInfo {
                name: topic.name().to_string(),
                partitions,
                consumer_groups,
            });
        }
        
        Ok(topics)
    }

    pub async fn get_consumer_groups(data_source: &DataSource) -> Result<Vec<String>> {
        // This is a simplified implementation
        // In production, you'd use Kafka's AdminClient to list consumer groups
        let consumer = KafkaConnector::create_consumer(data_source).await?;
        
        // Note: rdkafka doesn't have a direct API to list all consumer groups
        // You might need to use Kafka's AdminClient or connect to Kafka's internal topics
        // For now, return empty list
        Ok(Vec::new())
    }

    async fn get_consumer_groups_for_topic(
        data_source: &DataSource,
        topic_name: &str,
    ) -> Result<Vec<String>> {
        // Simplified implementation
        // In production, you'd query Kafka's __consumer_offsets topic or use AdminClient
        Ok(Vec::new())
    }

    pub async fn get_schema_registry_schemas(
        data_source: &DataSource,
    ) -> Result<Vec<SchemaInfo>> {
        let registry_url = data_source.schema_registry_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Schema Registry URL not configured"))?;
        
        let client = HttpClient::new();
        let url = format!("{}/subjects", registry_url.trim_end_matches('/'));
        
        let response = client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch subjects from Schema Registry")?;
        
        let subjects: Vec<String> = response
            .json()
            .await
            .context("Failed to parse Schema Registry response")?;
        
        let mut schemas = Vec::new();
        
        for subject in subjects {
            // Get latest version
            let version_url = format!("{}/subjects/{}/versions/latest", registry_url.trim_end_matches('/'), subject);
            let version_response = client
                .get(&version_url)
                .send()
                .await
                .context(format!("Failed to fetch schema for subject: {}", subject))?;
            
            if version_response.status().is_success() {
                let schema_data: Value = version_response
                    .json()
                    .await
                    .context("Failed to parse schema data")?;
                
                let version = schema_data.get("version")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                
                let schema_type = schema_data.get("schemaType")
                    .or_else(|| schema_data.get("schema_type"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("AVRO")
                    .to_string();
                
                let schema = schema_data.get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                
                schemas.push(SchemaInfo {
                    subject,
                    version,
                    schema_type,
                    schema,
                });
            }
        }
        
        Ok(schemas)
    }
}

