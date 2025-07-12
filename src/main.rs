use anyhow::Result;
use log::{debug, error, info, warn};
use mcp_test::ClickHouseClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<Value>,
    id: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    capabilities: Value,
    #[serde(rename = "clientInfo")]
    client_info: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolCallParams {
    name: String,
    arguments: Option<Value>,
}


struct McpServer {
    initialized: bool,
    clickhouse_client: Option<ClickHouseClient>,
}

impl McpServer {
    fn new() -> Self {
        debug!("Creating new MCP server instance");
        Self {
            initialized: false,
            clickhouse_client: None,
        }
    }

    fn connect_clickhouse(&mut self) -> Result<()> {
        let url = std::env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string());
        let database = std::env::var("CLICKHOUSE_DATABASE").unwrap_or_else(|_| "default".to_string());
        let username = std::env::var("CLICKHOUSE_USERNAME").unwrap_or_else(|_| "default".to_string());
        let password = std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_else(|_| "".to_string());
        
        info!("Connecting to ClickHouse at {} with database {}", url, database);
        
        let client = ClickHouseClient::new(&url, &database, &username, &password);
        
        self.clickhouse_client = Some(client);
        Ok(())
    }

    async fn handle_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Handling request: method={}, id={:?}", request.method, request.id);
        
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "initialized" => self.handle_initialized(request).await,
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            _ => {
                warn!("Unknown method requested: {}", request.method);
                Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(serde_json::json!({
                        "code": -32601,
                        "message": "Method not found"
                    })),
                    id: request.id,
                })
            }
        }
    }

    async fn handle_initialize(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        info!("Initializing MCP server");
        
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    },
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "mcp-test",
                    "version": "0.1.0"
                }
            })),
            error: None,
            id: request.id,
        };
        
        debug!("Sent initialize response");
        Ok(response)
    }

    async fn handle_initialized(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        self.initialized = true;
        info!("MCP server initialization completed");
        
        if let Err(e) = self.connect_clickhouse() {
            warn!("Failed to connect to ClickHouse: {}", e);
        }
        
        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({})),
            error: None,
            id: request.id,
        })
    }

    async fn handle_tools_list(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Listing available tools");
        
        let tools = vec![
            serde_json::json!({
                "name": "list_databases",
                "description": "List all databases in the ClickHouse instance",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }),
            serde_json::json!({
                "name": "list_tables",
                "description": "List all tables in a specific database",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "database": {
                            "type": "string",
                            "description": "The database name to list tables from"
                        }
                    },
                    "required": ["database"]
                }
            }),
            serde_json::json!({
                "name": "get_table_schema",
                "description": "Get the schema (columns) of a specific table",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "database": {
                            "type": "string",
                            "description": "The database name"
                        },
                        "table": {
                            "type": "string",
                            "description": "The table name"
                        }
                    },
                    "required": ["database", "table"]
                }
            })
        ];
        
        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"tools": tools})),
            error: None,
            id: request.id,
        })
    }

    async fn handle_tools_call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let params: ToolCallParams = serde_json::from_value(request.params.unwrap_or_default())?;
        debug!("Calling tool: {}", params.name);
        
        if self.clickhouse_client.is_none() {
            return Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(serde_json::json!({
                    "code": -32603,
                    "message": "ClickHouse client not connected"
                })),
                id: request.id,
            });
        }
        
        let result = match params.name.as_str() {
            "list_databases" => self.list_databases().await,
            "list_tables" => {
                let args = params.arguments.unwrap_or_default();
                let database = args.get("database")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing database argument"))?;
                self.list_tables(database).await
            },
            "get_table_schema" => {
                let args = params.arguments.unwrap_or_default();
                let database = args.get("database")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing database argument"))?;
                let table = args.get("table")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing table argument"))?;
                self.get_table_schema(database, table).await
            },
            _ => Err(anyhow::anyhow!("Unknown tool: {}", params.name)),
        };
        
        match result {
            Ok(content) => Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": content
                    }]
                })),
                error: None,
                id: request.id,
            }),
            Err(e) => {
                error!("Tool call failed: {}", e);
                Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(serde_json::json!({
                        "code": -32603,
                        "message": format!("Tool execution failed: {}", e)
                    })),
                    id: request.id,
                })
            }
        }
    }

    async fn list_databases(&self) -> Result<String> {
        let client = self.clickhouse_client.as_ref().unwrap();
        
        let databases = client.list_databases().await?;
        
        let mut result = String::from("Available databases:\n");
        for db in databases {
            result.push_str(&format!("- {}\n", db.name));
        }
        
        Ok(result)
    }

    async fn list_tables(&self, database: &str) -> Result<String> {
        let client = self.clickhouse_client.as_ref().unwrap();
        
        let tables = client.list_tables(database).await?;
        
        let mut result = format!("Tables in database '{}':\n", database);
        for table in tables {
            result.push_str(&format!("- {} (Engine: {})\n", table.name, table.engine));
        }
        
        Ok(result)
    }

    async fn get_table_schema(&self, database: &str, table: &str) -> Result<String> {
        let client = self.clickhouse_client.as_ref().unwrap();
        
        let columns = client.get_table_schema(database, table).await?;
        
        let mut result = format!("Schema for table '{}.{}':\n", database, table);
        result.push_str("\nColumns:\n");
        
        for col in columns {
            result.push_str(&format!("- {}: {}", col.name, col.r#type));
            
            if !col.comment.is_empty() {
                result.push_str(&format!(" -- {}", col.comment));
            }
            
            let mut key_info = Vec::new();
            if col.is_in_primary_key == 1 {
                key_info.push("PRIMARY KEY");
            }
            if col.is_in_sorting_key == 1 {
                key_info.push("SORTING KEY");
            }
            if col.is_in_partition_key == 1 {
                key_info.push("PARTITION KEY");
            }
            if col.is_in_sampling_key == 1 {
                key_info.push("SAMPLING KEY");
            }
            
            if !key_info.is_empty() {
                result.push_str(&format!(" [{}]", key_info.join(", ")));
            }
            
            result.push('\n');
        }
        
        Ok(result)
    }

    async fn run(&mut self) -> Result<()> {
        info!("Starting MCP server main loop");
        
        let stdin = tokio::io::stdin();
        let mut reader = AsyncBufReader::new(stdin);
        let mut stdout = tokio::io::stdout();
        
        let mut line = String::new();
        
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;
            
            if bytes_read == 0 {
                info!("End of input reached, shutting down server");
                break;
            }
            
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            debug!("Received line: {}", line);
            
            match serde_json::from_str::<JsonRpcRequest>(line) {
                Ok(request) => {
                    match self.handle_request(request).await {
                        Ok(response) => {
                            let response_json = serde_json::to_string(&response)?;
                            debug!("Sending response: {}", response_json);
                            stdout.write_all(response_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                        Err(e) => {
                            error!("Request handling failed: {}", e);
                            let error_response = JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: None,
                                error: Some(serde_json::json!({
                                    "code": -32603,
                                    "message": format!("Internal error: {}", e)
                                })),
                                id: None,
                            };
                            let response_json = serde_json::to_string(&error_response)?;
                            stdout.write_all(response_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse JSON-RPC request: {} - Input: {}", e, line);
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(serde_json::json!({
                            "code": -32700,
                            "message": "Parse error"
                        })),
                        id: None,
                    };
                    let response_json = serde_json::to_string(&error_response)?;
                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    info!("Starting MCP server v{}", env!("CARGO_PKG_VERSION"));
    
    let mut server = McpServer::new();
    server.run().await?;
    Ok(())
}
