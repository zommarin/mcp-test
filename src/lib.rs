use anyhow::Result;
use clickhouse::{Client, Row};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

#[derive(Debug, Error)]
pub enum ClickHouseError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },
    #[error("Database '{database}' not found")]
    DatabaseNotFound { database: String },
    #[error("Table '{table}' not found in database '{database}'")]
    TableNotFound { database: String, table: String },
    #[error("Permission denied for operation: {operation}")]
    PermissionDenied { operation: String },
    #[error("Query timeout after {timeout}s")]
    QueryTimeout { timeout: u64 },
    #[error("Invalid identifier '{identifier}': {reason}")]
    InvalidIdentifier { identifier: String, reason: String },
    #[error("Network error: {message}")]
    NetworkError { message: String },
    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },
    #[error("Query failed: {message}")]
    QueryFailed { message: String },
    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

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
    max_retries: u32,
    base_delay: Duration,
}

impl ClickHouseClient {
    pub fn new(url: &str, database: &str, username: &str, password: &str) -> Self {
        let client = Client::default()
            .with_url(url)
            .with_database(database)
            .with_user(username)
            .with_password(password);
        
        Self { 
            client,
            max_retries: 3,
            base_delay: Duration::from_millis(100),
        }
    }
    
    pub fn with_retry_config(mut self, max_retries: u32, base_delay: Duration) -> Self {
        self.max_retries = max_retries;
        self.base_delay = base_delay;
        self
    }
    
    fn validate_identifier(identifier: &str) -> Result<(), ClickHouseError> {
        if identifier.is_empty() {
            return Err(ClickHouseError::InvalidIdentifier {
                identifier: identifier.to_string(),
                reason: "Identifier cannot be empty".to_string(),
            });
        }
        
        if identifier.len() > 64 {
            return Err(ClickHouseError::InvalidIdentifier {
                identifier: identifier.to_string(),
                reason: "Identifier cannot be longer than 64 characters".to_string(),
            });
        }
        
        if !identifier.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ClickHouseError::InvalidIdentifier {
                identifier: identifier.to_string(),
                reason: "Identifier can only contain alphanumeric characters, underscore, and hyphen".to_string(),
            });
        }
        
        if identifier.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(ClickHouseError::InvalidIdentifier {
                identifier: identifier.to_string(),
                reason: "Identifier cannot start with a digit".to_string(),
            });
        }
        
        Ok(())
    }
    
    async fn with_retry<F, T, Fut>(&self, operation: F) -> Result<T, ClickHouseError> 
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, clickhouse::error::Error>>,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = self.base_delay * (2_u32.pow(attempt - 1));
                debug!("Retrying ClickHouse operation after {}ms (attempt {})", delay.as_millis(), attempt);
                sleep(delay).await;
            }
            
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);
                    if attempt == self.max_retries {
                        break;
                    }
                    
                    // Check if error is retryable
                    if !self.is_retryable_error(&last_error.as_ref().unwrap()) {
                        break;
                    }
                    
                    warn!("ClickHouse operation failed (attempt {}): {}", attempt + 1, last_error.as_ref().unwrap());
                }
            }
        }
        
        // Convert clickhouse error to our error type
        if let Some(error) = last_error {
            Err(self.convert_clickhouse_error(error))
        } else {
            Err(ClickHouseError::InternalError {
                message: "Retry loop completed without error".to_string(),
            })
        }
    }
    
    fn is_retryable_error(&self, error: &clickhouse::error::Error) -> bool {
        match error {
            clickhouse::error::Error::Network(_) => true,
            clickhouse::error::Error::BadResponse(_) => false, // Don't retry auth/permission errors
            clickhouse::error::Error::InvalidParams(_) => false, // Don't retry invalid queries
            _ => true, // Retry other errors (like timeouts)
        }
    }
    
    fn convert_clickhouse_error(&self, error: clickhouse::error::Error) -> ClickHouseError {
        match error {
            clickhouse::error::Error::Network(e) => ClickHouseError::NetworkError {
                message: e.to_string(),
            },
            clickhouse::error::Error::InvalidParams(e) => ClickHouseError::QueryFailed {
                message: e.to_string(),
            },
            clickhouse::error::Error::BadResponse(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("Authentication failed") {
                    ClickHouseError::AuthenticationFailed { message: error_msg }
                } else if error_msg.contains("doesn't exist") {
                    if error_msg.contains("Database") {
                        ClickHouseError::DatabaseNotFound {
                            database: "unknown".to_string(),
                        }
                    } else {
                        ClickHouseError::TableNotFound {
                            database: "unknown".to_string(),
                            table: "unknown".to_string(),
                        }
                    }
                } else if error_msg.contains("Access denied") {
                    ClickHouseError::PermissionDenied {
                        operation: "query".to_string(),
                    }
                } else {
                    ClickHouseError::QueryFailed { message: error_msg }
                }
            },
            _ => ClickHouseError::InternalError {
                message: error.to_string(),
            },
        }
    }
    
    pub async fn health_check(&self) -> Result<(), ClickHouseError> {
        info!("Performing ClickHouse health check");
        
        self.with_retry(|| async {
            self.client
                .query("SELECT 1")
                .fetch_one::<u8>()
                .await
        }).await?;
        
        info!("ClickHouse health check passed");
        Ok(())
    }

    pub async fn list_databases(&self) -> Result<Vec<DatabaseInfo>, ClickHouseError> {
        info!("Listing databases");
        
        let databases = self.with_retry(|| async {
            self.client
                .query("SELECT name FROM system.databases ORDER BY name")
                .fetch_all()
                .await
        }).await?;
        
        debug!("Found {} databases", databases.len());
        Ok(databases)
    }

    pub async fn list_tables(&self, database: &str) -> Result<Vec<TableInfo>, ClickHouseError> {
        Self::validate_identifier(database)?;
        info!("Listing tables in database '{}'", database);
        
        // First check if the database exists
        let db_exists: u8 = self.with_retry(|| async {
            self.client
                .query("SELECT count(*) > 0 FROM system.databases WHERE name = ?")
                .bind(database)
                .fetch_one()
                .await
        }).await?;
        
        if db_exists == 0 {
            return Err(ClickHouseError::DatabaseNotFound {
                database: database.to_string(),
            });
        }
        
        let tables = self.with_retry(|| async {
            self.client
                .query("SELECT name, database, engine FROM system.tables WHERE database = ? ORDER BY name")
                .bind(database)
                .fetch_all()
                .await
        }).await.map_err(|e| {
            if let ClickHouseError::QueryFailed { message } = &e {
                if message.contains("doesn't exist") {
                    return ClickHouseError::DatabaseNotFound {
                        database: database.to_string(),
                    };
                }
            }
            e
        })?;
        
        debug!("Found {} tables in database '{}'", tables.len(), database);
        Ok(tables)
    }

    pub async fn get_table_schema(&self, database: &str, table: &str) -> Result<Vec<ColumnInfo>, ClickHouseError> {
        Self::validate_identifier(database)?;
        Self::validate_identifier(table)?;
        info!("Getting schema for table '{}.{}'", database, table);
        
        // First check if the database exists
        let db_exists: u8 = self.with_retry(|| async {
            self.client
                .query("SELECT count(*) > 0 FROM system.databases WHERE name = ?")
                .bind(database)
                .fetch_one()
                .await
        }).await?;
        
        if db_exists == 0 {
            return Err(ClickHouseError::DatabaseNotFound {
                database: database.to_string(),
            });
        }
        
        // Then check if the table exists
        let table_exists: u8 = self.with_retry(|| async {
            self.client
                .query("SELECT count(*) > 0 FROM system.tables WHERE database = ? AND name = ?")
                .bind(database)
                .bind(table)
                .fetch_one()
                .await
        }).await?;
        
        if table_exists == 0 {
            return Err(ClickHouseError::TableNotFound {
                database: database.to_string(),
                table: table.to_string(),
            });
        }
        
        let columns = self.with_retry(|| async {
            self.client
                .query("SELECT name, type, default_kind as default_type, default_expression, comment, is_in_partition_key, is_in_sorting_key, is_in_primary_key, is_in_sampling_key FROM system.columns WHERE database = ? AND table = ? ORDER BY position")
                .bind(database)
                .bind(table)
                .fetch_all()
                .await
        }).await.map_err(|e| {
            if let ClickHouseError::QueryFailed { message } = &e {
                if message.contains("doesn't exist") {
                    return ClickHouseError::TableNotFound {
                        database: database.to_string(),
                        table: table.to_string(),
                    };
                }
            }
            e
        })?;
        
        if columns.is_empty() {
            return Err(ClickHouseError::TableNotFound {
                database: database.to_string(),
                table: table.to_string(),
            });
        }
        
        debug!("Found {} columns in table '{}.{}'", columns.len(), database, table);
        Ok(columns)
    }
}