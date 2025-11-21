use crate::connection::{ConnectionTester, ProxyConfig};
use crate::db::DataSource;
use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Pool;
use std::time::Duration;

pub struct PostgreSQLConnector;

impl ConnectionTester for PostgreSQLConnector {
    async fn test_connection(data_source: &DataSource) -> Result<()> {
        Self::create_pool(data_source)
            .await
            .context("Failed to create PostgreSQL connection pool")?;
        Ok(())
    }
}

impl PostgreSQLConnector {
    pub async fn create_pool(data_source: &DataSource) -> Result<Pool<sqlx::Postgres>> {
        let url = Self::build_connection_url(data_source)?;
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&url)
            .await
            .context("Failed to connect to PostgreSQL")?;
        
        Ok(pool)
    }

    fn build_connection_url(data_source: &DataSource) -> Result<String> {
        let mut url = format!(
            "postgresql://{}:{}@{}:{}/",
            data_source.username,
            data_source.password,
            data_source.host,
            data_source.port
        );
        
        if let Some(database) = &data_source.database {
            url.push_str(database);
        } else {
            url.push_str("postgres");
        }
        
        // Handle proxy configuration
        if let Some(proxy_type) = &data_source.proxy_type {
            match proxy_type.as_str() {
                "http" => {
                    if let Some(proxy_config) = &data_source.proxy_config {
                        let _proxy: ProxyConfig = serde_json::from_value(proxy_config.clone())
                            .context("Invalid proxy config")?;
                        // Note: sqlx doesn't directly support HTTP proxy for PostgreSQL
                        return Err(anyhow::anyhow!("HTTP proxy for PostgreSQL is not directly supported by sqlx"));
                    }
                }
                "socks5" => {
                    return Err(anyhow::anyhow!("SOCKS5 proxy for PostgreSQL requires custom implementation"));
                }
                _ => {}
            }
        }
        
        Ok(url)
    }
}

