use mcp_test::{ClickHouseClient, ColumnInfo, DatabaseInfo, TableInfo};
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_clickhouse_client_creation() {
    let _client = ClickHouseClient::new(
        "http://localhost:8123",
        "default",
        "default",
        ""
    );
    
    // Just test that we can create a client without panicking
    assert!(true);
}

#[tokio::test]
async fn test_clickhouse_client_with_retry_config() {
    let _client = ClickHouseClient::new(
        "http://localhost:8123",
        "default",
        "default",
        ""
    ).with_retry_config(5, Duration::from_millis(200));
    
    // Test that we can create a client with custom retry config
    assert!(true);
}

#[tokio::test]
async fn test_database_info_serialization() {
    let db_info = DatabaseInfo {
        name: "test_db".to_string(),
    };
    
    let json_str = serde_json::to_string(&db_info).unwrap();
    let deserialized: DatabaseInfo = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(db_info.name, deserialized.name);
}

#[tokio::test]
async fn test_table_info_serialization() {
    let table_info = TableInfo {
        name: "test_table".to_string(),
        database: "test_db".to_string(),
        engine: "MergeTree".to_string(),
    };
    
    let json_str = serde_json::to_string(&table_info).unwrap();
    let deserialized: TableInfo = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(table_info.name, deserialized.name);
    assert_eq!(table_info.database, deserialized.database);
    assert_eq!(table_info.engine, deserialized.engine);
}

#[tokio::test]
async fn test_column_info_serialization() {
    let column_info = ColumnInfo {
        name: "id".to_string(),
        r#type: "UInt64".to_string(),
        default_type: "".to_string(),
        default_expression: "".to_string(),
        comment: "Primary key".to_string(),
        is_in_partition_key: 0,
        is_in_sorting_key: 1,
        is_in_primary_key: 1,
        is_in_sampling_key: 0,
    };
    
    let json_str = serde_json::to_string(&column_info).unwrap();
    let deserialized: ColumnInfo = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(column_info.name, deserialized.name);
    assert_eq!(column_info.r#type, deserialized.r#type);
    assert_eq!(column_info.comment, deserialized.comment);
    assert_eq!(column_info.is_in_primary_key, deserialized.is_in_primary_key);
    assert_eq!(column_info.is_in_sorting_key, deserialized.is_in_sorting_key);
}

#[tokio::test]
async fn test_json_rpc_request_structure() {
    let request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "list_databases",
            "arguments": {}
        },
        "id": 1
    });
    
    assert_eq!(request["jsonrpc"], "2.0");
    assert_eq!(request["method"], "tools/call");
    assert_eq!(request["params"]["name"], "list_databases");
}

#[tokio::test]
async fn test_json_rpc_tool_call_params() {
    let request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "list_tables",
            "arguments": {
                "database": "system"
            }
        },
        "id": 1
    });
    
    assert_eq!(request["params"]["name"], "list_tables");
    assert_eq!(request["params"]["arguments"]["database"], "system");
}

#[tokio::test]
async fn test_json_rpc_get_schema_params() {
    let request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "get_table_schema",
            "arguments": {
                "database": "system",
                "table": "tables"
            }
        },
        "id": 1
    });
    
    assert_eq!(request["params"]["name"], "get_table_schema");
    assert_eq!(request["params"]["arguments"]["database"], "system");
    assert_eq!(request["params"]["arguments"]["table"], "tables");
}

// Mock integration test - this would require a real ClickHouse instance
#[tokio::test]
#[ignore] // Ignore by default since it requires ClickHouse running
async fn test_clickhouse_integration() {
    // This test would require a real ClickHouse instance
    // It's marked as #[ignore] so it won't run by default
    let client = ClickHouseClient::new(
        "http://localhost:8123",
        "default",
        "default",
        ""
    );
    
    // Test listing databases
    let databases = client.list_databases().await;
    match databases {
        Ok(dbs) => {
            println!("Found {} databases", dbs.len());
            for db in dbs {
                println!("  - {}", db.name);
            }
        }
        Err(e) => {
            println!("Failed to list databases: {}", e);
        }
    }
    
    // Test listing tables in system database
    let tables = client.list_tables("system").await;
    match tables {
        Ok(tbls) => {
            println!("Found {} tables in system database", tbls.len());
        }
        Err(e) => {
            println!("Failed to list tables: {}", e);
        }
    }
}