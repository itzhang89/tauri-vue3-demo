use crate::connection::{ConnectionTester, ProxyConfig};
use crate::db::DataSource;
use anyhow::{Context, Result};
use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

pub struct SQLServerConnector;

impl ConnectionTester for SQLServerConnector {
    async fn test_connection(data_source: &DataSource) -> Result<()> {
        Self::create_client(data_source)
            .await
            .context("Failed to create SQL Server connection")?;
        Ok(())
    }
}

impl SQLServerConnector {
    pub async fn create_client(data_source: &DataSource) -> Result<Client<tokio_util::compat::Compat<TcpStream>>> {
        let mut config = Config::new();
        config.host(data_source.host.clone());
        config.port(data_source.port);
        config.authentication(AuthMethod::sql_server(&data_source.username, &data_source.password));
        
        if let Some(database) = &data_source.database {
            config.database(database);
        }
        
        config.trust_cert();
        
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .context("Failed to connect to SQL Server")?;
        
        tcp.set_nodelay(true)?;
        
        let compat_stream = tcp.compat_write();
        let client = Client::connect(config, compat_stream)
            .await
            .context("Failed to establish SQL Server connection")?;
        
        Ok(client)
    }
}

