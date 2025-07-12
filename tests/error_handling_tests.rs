use mcp_test::{ClickHouseClient, ClickHouseError};
use std::time::Duration;

#[tokio::test]
async fn test_invalid_identifier_validation() {
    let client = ClickHouseClient::new(
        "http://localhost:8123",
        "default",
        "default",
        ""
    );

    // Test empty identifier
    let result = client.list_tables("").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ClickHouseError::InvalidIdentifier { identifier, reason } => {
            assert_eq!(identifier, "");
            assert!(reason.contains("cannot be empty"));
        }
        _ => panic!("Expected InvalidIdentifier error"),
    }
}

#[tokio::test]
async fn test_long_identifier_validation() {
    let client = ClickHouseClient::new(
        "http://localhost:8123",
        "default",
        "default",
        ""
    );

    // Test identifier that's too long
    let long_name = "a".repeat(65);
    let result = client.list_tables(&long_name).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ClickHouseError::InvalidIdentifier { identifier, reason } => {
            assert_eq!(identifier, long_name);
            assert!(reason.contains("longer than 64 characters"));
        }
        _ => panic!("Expected InvalidIdentifier error"),
    }
}

#[tokio::test]
async fn test_invalid_characters_validation() {
    let client = ClickHouseClient::new(
        "http://localhost:8123",
        "default",
        "default",
        ""
    );

    // Test identifier with invalid characters
    let invalid_name = "table@name!";
    let result = client.list_tables(invalid_name).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ClickHouseError::InvalidIdentifier { identifier, reason } => {
            assert_eq!(identifier, invalid_name);
            assert!(reason.contains("can only contain"));
        }
        _ => panic!("Expected InvalidIdentifier error"),
    }
}

#[tokio::test]
async fn test_identifier_starting_with_digit() {
    let client = ClickHouseClient::new(
        "http://localhost:8123",
        "default",
        "default",
        ""
    );

    // Test identifier starting with digit
    let invalid_name = "1table";
    let result = client.list_tables(invalid_name).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ClickHouseError::InvalidIdentifier { identifier, reason } => {
            assert_eq!(identifier, invalid_name);
            assert!(reason.contains("cannot start with a digit"));
        }
        _ => panic!("Expected InvalidIdentifier error"),
    }
}

#[tokio::test]
async fn test_valid_identifiers() {
    let client = ClickHouseClient::new(
        "http://localhost:8123",
        "default",
        "default",
        ""
    );

    // These should pass validation (though they may fail at query time)
    let valid_names = vec!["table1", "my_table", "valid-name", "_underscore", "a"];
    
    for name in valid_names {
        // We only test that validation passes - the actual query may fail due to no ClickHouse server
        // but that would be a different error type
        let result = client.list_tables(name).await;
        if let Err(ClickHouseError::InvalidIdentifier { .. }) = result {
            panic!("Identifier '{}' should be valid", name);
        }
    }
}

#[tokio::test]
async fn test_error_display_formatting() {
    let errors = vec![
        ClickHouseError::ConnectionFailed { message: "timeout".to_string() },
        ClickHouseError::DatabaseNotFound { database: "test_db".to_string() },
        ClickHouseError::TableNotFound { database: "test_db".to_string(), table: "test_table".to_string() },
        ClickHouseError::PermissionDenied { operation: "SELECT".to_string() },
        ClickHouseError::QueryTimeout { timeout: 30 },
        ClickHouseError::InvalidIdentifier { identifier: "123invalid".to_string(), reason: "starts with digit".to_string() },
    ];

    for error in errors {
        let error_string = error.to_string();
        assert!(!error_string.is_empty());
        // Each error should contain meaningful information
        match error {
            ClickHouseError::DatabaseNotFound { database } => {
                assert!(error_string.contains(&database));
            }
            ClickHouseError::TableNotFound { database, table } => {
                assert!(error_string.contains(&database));
                assert!(error_string.contains(&table));
            }
            _ => {} // Other error types are tested by their presence in the string
        }
    }
}

#[tokio::test]
async fn test_schema_validation_for_both_database_and_table() {
    let client = ClickHouseClient::new(
        "http://localhost:8123",
        "default",
        "default",
        ""
    );

    // Test invalid database name
    let result = client.get_table_schema("", "valid_table").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ClickHouseError::InvalidIdentifier { .. }));

    // Test invalid table name
    let result = client.get_table_schema("valid_db", "").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ClickHouseError::InvalidIdentifier { .. }));

    // Test both invalid
    let result = client.get_table_schema("", "").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ClickHouseError::InvalidIdentifier { .. }));
}

#[tokio::test]
#[ignore] // Requires ClickHouse server - only run manually
async fn test_connection_error_handling() {
    // This test requires no ClickHouse server running on port 9999
    let client = ClickHouseClient::new(
        "http://localhost:9999",
        "default",
        "default",
        ""
    ).with_retry_config(1, Duration::from_millis(10)); // Fast failure for test

    let result = client.health_check().await;
    assert!(result.is_err());
    
    // Should be a network error since the port doesn't exist
    match result.unwrap_err() {
        ClickHouseError::NetworkError { .. } => {
            // Expected
        }
        other => panic!("Expected NetworkError, got: {:?}", other),
    }
}

#[tokio::test]
#[ignore] // Requires ClickHouse server - only run manually  
async fn test_nonexistent_database_error() {
    // This test requires a ClickHouse server to be running
    let client = ClickHouseClient::new(
        "http://localhost:8123",
        "default",
        "default",
        ""
    );

    let result = client.list_tables("nonexistent_database_12345").await;
    assert!(result.is_err());
    
    // Should be a DatabaseNotFound error
    match result.unwrap_err() {
        ClickHouseError::DatabaseNotFound { database } => {
            assert_eq!(database, "nonexistent_database_12345");
        }
        other => panic!("Expected DatabaseNotFound, got: {:?}", other),
    }
}