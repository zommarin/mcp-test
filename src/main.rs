use anyhow::Result;
use log::{debug, error, info, warn};
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

struct McpServer {
    initialized: bool,
}

impl McpServer {
    fn new() -> Self {
        debug!("Creating new MCP server instance");
        Self {
            initialized: false,
        }
    }

    async fn handle_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Handling request: method={}, id={:?}", request.method, request.id);
        
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "initialized" => self.handle_initialized(request).await,
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
                    "tools": {},
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
        
        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({})),
            error: None,
            id: request.id,
        })
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
                    let response = self.handle_request(request).await?;
                    let response_json = serde_json::to_string(&response)?;
                    debug!("Sending response: {}", response_json);
                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
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
