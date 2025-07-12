use anyhow::Result;
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct DatabaseInfo {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct TableInfo {
    pub name: String,
    pub database: String,
    pub engine: String,
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct ColumnInfo {
    pub name: String,
    pub r#type: String,
    pub default_type: String,
    pub default_expression: String,
    pub comment: String,
    pub is_in_partition_key: u8,
    pub is_in_sorting_key: u8,
    pub is_in_primary_key: u8,
    pub is_in_sampling_key: u8,
}

pub struct ClickHouseClient {
    client: Client,
}

impl ClickHouseClient {
    pub fn new(url: &str, database: &str, username: &str, password: &str) -> Self {
        let client = Client::default()
            .with_url(url)
            .with_database(database)
            .with_user(username)
            .with_password(password);
        
        Self { client }
    }

    pub async fn list_databases(&self) -> Result<Vec<DatabaseInfo>> {
        let databases = self.client
            .query("SELECT name FROM system.databases ORDER BY name")
            .fetch_all()
            .await?;
        
        Ok(databases)
    }

    pub async fn list_tables(&self, database: &str) -> Result<Vec<TableInfo>> {
        let tables = self.client
            .query("SELECT name, database, engine FROM system.tables WHERE database = ? ORDER BY name")
            .bind(database)
            .fetch_all()
            .await?;
        
        Ok(tables)
    }

    pub async fn get_table_schema(&self, database: &str, table: &str) -> Result<Vec<ColumnInfo>> {
        let columns = self.client
            .query("SELECT name, type, default_type, default_expression, comment, is_in_partition_key, is_in_sorting_key, is_in_primary_key, is_in_sampling_key FROM system.columns WHERE database = ? AND table = ? ORDER BY position")
            .bind(database)
            .bind(table)
            .fetch_all()
            .await?;
        
        Ok(columns)
    }
}