use crate::connection::ConnectionTester;
use crate::db::DataSource;
use anyhow::{Context, Result};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::FutureProducer;
use std::time::Duration;

pub struct KafkaConnector;

impl ConnectionTester for KafkaConnector {
    async fn test_connection(data_source: &DataSource) -> Result<()> {
        Self::create_admin_client(data_source)
            .await
            .context("Failed to create Kafka admin client")?;
        Ok(())
    }
}

impl KafkaConnector {
    pub async fn create_admin_client(data_source: &DataSource) -> Result<rdkafka::admin::AdminClient<rdkafka::client::DefaultClientContext>> {
        let mut config = ClientConfig::new();
        config.set("bootstrap.servers", format!("{}:{}", data_source.host, data_source.port));
        config.set("client.id", "data-explorer");
        config.set("request.timeout.ms", "5000");
        
        // Handle authentication if needed
        if !data_source.username.is_empty() {
            config.set("security.protocol", "SASL_PLAINTEXT");
            config.set("sasl.mechanism", "PLAIN");
            config.set("sasl.username", &data_source.username);
            config.set("sasl.password", &data_source.password);
        }
        
        let admin_client: rdkafka::admin::AdminClient<rdkafka::client::DefaultClientContext> = config
            .create()
            .context("Failed to create Kafka admin client")?;
        
        // Test connection by listing topics
        let metadata = admin_client
            .inner()
            .fetch_metadata(None, Duration::from_secs(5))
            .context("Failed to fetch Kafka metadata")?;
        
        Ok(admin_client)
    }

    pub async fn create_consumer(data_source: &DataSource) -> Result<StreamConsumer> {
        let mut config = ClientConfig::new();
        config.set("bootstrap.servers", format!("{}:{}", data_source.host, data_source.port));
        config.set("group.id", "data-explorer-consumer");
        config.set("enable.partition.eof", "false");
        config.set("session.timeout.ms", "6000");
        config.set("enable.auto.commit", "true");
        
        if !data_source.username.is_empty() {
            config.set("security.protocol", "SASL_PLAINTEXT");
            config.set("sasl.mechanism", "PLAIN");
            config.set("sasl.username", &data_source.username);
            config.set("sasl.password", &data_source.password);
        }
        
        let consumer: StreamConsumer = config
            .create()
            .context("Failed to create Kafka consumer")?;
        
        Ok(consumer)
    }

    pub async fn create_producer(data_source: &DataSource) -> Result<FutureProducer> {
        let mut config = ClientConfig::new();
        config.set("bootstrap.servers", format!("{}:{}", data_source.host, data_source.port));
        config.set("client.id", "data-explorer-producer");
        
        if !data_source.username.is_empty() {
            config.set("security.protocol", "SASL_PLAINTEXT");
            config.set("sasl.mechanism", "PLAIN");
            config.set("sasl.username", &data_source.username);
            config.set("sasl.password", &data_source.password);
        }
        
        let producer: FutureProducer = config
            .create()
            .context("Failed to create Kafka producer")?;
        
        Ok(producer)
    }
}

