use crate::connection::{ConnectionTester, ProxyConfig};
use crate::db::DataSource;
use anyhow::{Context, Result};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::Pool;
use std::time::Duration;

pub struct MySQLConnector;

impl ConnectionTester for MySQLConnector {
    async fn test_connection(data_source: &DataSource) -> Result<()> {
        Self::create_pool(data_source)
            .await
            .context("Failed to create MySQL connection pool")?;
        Ok(())
    }
}

impl MySQLConnector {
    pub async fn create_pool(data_source: &DataSource) -> Result<Pool<sqlx::MySql>> {
        let url = Self::build_connection_url(data_source)?;
        
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&url)
            .await
            .context("Failed to connect to MySQL")?;
        
        Ok(pool)
    }

    fn build_connection_url(data_source: &DataSource) -> Result<String> {
        let mut url = format!(
            "mysql://{}:{}@{}:{}/",
            data_source.username,
            data_source.password,
            data_source.host,
            data_source.port
        );
        
        if let Some(database) = &data_source.database {
            url.push_str(database);
        }
        
        // Handle proxy configuration
        if let Some(proxy_type) = &data_source.proxy_type {
            match proxy_type.as_str() {
                "http" => {
                    if let Some(proxy_config) = &data_source.proxy_config {
                        let _proxy: ProxyConfig = serde_json::from_value(proxy_config.clone())
                            .context("Invalid proxy config")?;
                        // Note: sqlx doesn't directly support HTTP proxy for MySQL
                        // You might need to use a different approach or library
                        return Err(anyhow::anyhow!("HTTP proxy for MySQL is not directly supported by sqlx"));
                    }
                }
                "socks5" => {
                    // SOCKS5 proxy would need custom connection handling
                    return Err(anyhow::anyhow!("SOCKS5 proxy for MySQL requires custom implementation"));
                }
                _ => {}
            }
        }
        
        Ok(url)
    }
}

