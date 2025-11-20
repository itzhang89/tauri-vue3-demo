use crate::connection::{ConnectionTester, ProxyConfig};
use crate::db::DataSource;
use anyhow::{Context, Result};
use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

pub struct SQLServerConnector;

impl ConnectionTester for SQLServerConnector {
    fn test_connection(data_source: &DataSource) -> Result<()> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            Self::create_client(data_source)
                .await
                .context("Failed to create SQL Server connection")?;
            Ok::<(), anyhow::Error>(())
        })?;
        Ok(())
    }
}

impl SQLServerConnector {
    pub async fn create_client(data_source: &DataSource) -> Result<Client<TokioAsyncWriteCompatExt<TcpStream>>> {
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
        
        let client = Client::connect(config, tcp.compat_write())
            .await
            .context("Failed to establish SQL Server connection")?;
        
        Ok(client)
    }
}

