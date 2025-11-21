use crate::db::DataSource;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod proxy;
pub mod mysql;
pub mod postgresql;
pub mod sqlserver;
#[cfg(feature = "kafka")]
pub mod kafka;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_type: String, // socks5, http
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub local_port: u16, // Local port for SSH tunnel
}

pub trait ConnectionTester {
    async fn test_connection(data_source: &DataSource) -> Result<()>;
}

pub struct ConnectionManager;

impl ConnectionManager {
    pub async fn test_connection(data_source: &DataSource) -> Result<()> {
        match data_source.data_type.as_str() {
            "mysql" => mysql::MySQLConnector::test_connection(data_source).await,
            "postgresql" => postgresql::PostgreSQLConnector::test_connection(data_source).await,
            "sqlserver" => sqlserver::SQLServerConnector::test_connection(data_source).await,
            #[cfg(feature = "kafka")]
            "kafka" => kafka::KafkaConnector::test_connection(data_source).await,
            #[cfg(not(feature = "kafka"))]
            "kafka" => Err(anyhow::anyhow!("Kafka support is not enabled. Build with --features kafka")),
            _ => Err(anyhow::anyhow!("Unsupported data source type: {}", data_source.data_type)),
        }
    }
}

