use crate::db::DataSource;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod proxy;
pub mod mysql;
pub mod postgresql;
pub mod sqlserver;
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
    fn test_connection(&self, data_source: &DataSource) -> Result<()>;
}

pub struct ConnectionManager;

impl ConnectionManager {
    pub fn test_connection(data_source: &DataSource) -> Result<()> {
        match data_source.data_type.as_str() {
            "mysql" => mysql::MySQLConnector::test_connection(data_source),
            "postgresql" => postgresql::PostgreSQLConnector::test_connection(data_source),
            "sqlserver" => sqlserver::SQLServerConnector::test_connection(data_source),
            "kafka" => kafka::KafkaConnector::test_connection(data_source),
            _ => Err(anyhow::anyhow!("Unsupported data source type: {}", data_source.data_type)),
        }
    }
}

