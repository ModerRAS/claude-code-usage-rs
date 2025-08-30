//! MCP (Model Context Protocol) server module for ccusage-rs
//! 
//! This module provides a simple MCP server for integrating with
//! Claude Code and other MCP-compatible applications.

use crate::config::ConfigManager;
use crate::data::{DataLoader, DataSourceType};
use crate::analysis::{CostCalculator, StatisticsCalculator};
use crate::error::{Result, CcusageError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub enable_auth: bool,
    pub api_key: Option<String>,
    pub allowed_origins: Vec<String>,
}

/// MCP server
pub struct McpServer {
    config: ServerConfig,
    config_manager: Arc<RwLock<ConfigManager>>,
}

/// MCP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// MCP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

/// MCP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

/// MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(config: ServerConfig, config_manager: &ConfigManager) -> Result<Self> {
        Ok(Self {
            config,
            config_manager: Arc::new(RwLock::new(config_manager.clone())),
        })
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<()> {
        println!("Starting MCP server on {}:{}", self.config.host, self.config.port);
        
        // This is a simplified implementation
        // In a real implementation, you would use a web framework like axum or warp
        println!("MCP server started successfully");
        println!("Available tools:");
        
        for tool in self.get_available_tools() {
            println!("  - {}: {}", tool.name, tool.description);
        }
        
        // Keep the server running
        tokio::signal::ctrl_c().await.unwrap();
        println!("MCP server shutting down");
        
        Ok(())
    }

    /// Get available tools
    pub fn get_available_tools(&self) -> Vec<McpTool> {
        vec![
            McpTool {
                name: "get_usage_stats".to_string(),
                description: "Get usage statistics for a time period".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "start_date": {"type": "string", "description": "Start date (YYYY-MM-DD)"},
                        "end_date": {"type": "string", "description": "End date (YYYY-MM-DD)"}
                    },
                    "required": ["start_date", "end_date"]
                }),
            },
            McpTool {
                name: "get_cost_analysis".to_string(),
                description: "Get cost analysis for a time period".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "start_date": {"type": "string", "description": "Start date (YYYY-MM-DD)"},
                        "end_date": {"type": "string", "description": "End date (YYYY-MM-DD)"}
                    },
                    "required": ["start_date", "end_date"]
                }),
            },
            McpTool {
                name: "get_daily_report".to_string(),
                description: "Get daily usage report".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "date": {"type": "string", "description": "Date (YYYY-MM-DD)"}
                    },
                    "required": ["date"]
                }),
            },
            McpTool {
                name: "get_monthly_report".to_string(),
                description: "Get monthly usage report".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "year": {"type": "integer", "description": "Year"},
                        "month": {"type": "integer", "description": "Month (1-12)"}
                    },
                    "required": ["year", "month"]
                }),
            },
            McpTool {
                name: "get_budget_status".to_string(),
                description: "Get current budget status".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ]
    }

    /// Handle MCP request
    pub async fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            _ => McpResponse {
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: "Method not found".to_string(),
                }),
            },
        }
    }

    /// Handle tools list request
    async fn handle_tools_list(&self, request: McpRequest) -> McpResponse {
        let tools = self.get_available_tools();
        
        McpResponse {
            id: request.id,
            result: Some(serde_json::json!({
                "tools": tools
            })),
            error: None,
        }
    }

    /// Handle tools call request
    async fn handle_tools_call(&self, request: McpRequest) -> McpResponse {
        let params = request.params;
        
        let tool_name = params.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let arguments = params.get("arguments").unwrap_or(&serde_json::Value::Null);
        
        match tool_name {
            "get_usage_stats" => self.get_usage_stats(arguments).await,
            "get_cost_analysis" => self.get_cost_analysis(arguments).await,
            "get_daily_report" => self.get_daily_report(arguments).await,
            "get_monthly_report" => self.get_monthly_report(arguments).await,
            "get_budget_status" => self.get_budget_status().await,
            _ => McpResponse {
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Unknown tool: {}", tool_name),
                }),
            },
        }
    }

    /// Get usage statistics
    async fn get_usage_stats(&self, arguments: &serde_json::Value) -> McpResponse {
        let start_date = arguments.get("start_date")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let end_date = arguments.get("end_date")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Load data and calculate statistics
        let config_manager = self.config_manager.read().await;
        let source_path = config_manager.get_data_source_path("json")
            .unwrap_or("./data/usage.json");
        
        let data_loader = DataLoader::with_source(DataSourceType::Json, source_path.to_string());
        
        match data_loader.load_usage_data().await {
            Ok(records) => {
                let stats = StatisticsCalculator::calculate_usage_stats(&records);
                
                McpResponse {
                    id: "usage_stats".to_string(),
                    result: Some(serde_json::json!({
                        "start_date": start_date,
                        "end_date": end_date,
                        "total_requests": stats.total_requests,
                        "total_tokens": stats.total_tokens,
                        "total_cost": stats.total_cost,
                        "average_tokens_per_request": stats.average_tokens_per_request,
                        "average_cost_per_request": stats.average_cost_per_request,
                        "model_usage": stats.model_usage
                    })),
                    error: None,
                }
            }
            Err(e) => McpResponse {
                id: "usage_stats".to_string(),
                result: None,
                error: Some(McpError {
                    code: -32000,
                    message: format!("Failed to load usage data: {}", e),
                }),
            },
        }
    }

    /// Get cost analysis
    async fn get_cost_analysis(&self, arguments: &serde_json::Value) -> McpResponse {
        let start_date = arguments.get("start_date")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let end_date = arguments.get("end_date")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Load data and calculate cost analysis
        let config_manager = self.config_manager.read().await;
        let source_path = config_manager.get_data_source_path("json")
            .unwrap_or("./data/usage.json");
        
        let data_loader = DataLoader::with_source(DataSourceType::Json, source_path.to_string());
        
        match data_loader.load_usage_data().await {
            Ok(records) => {
                let calculator = CostCalculator::default();
                match calculator.calculate_detailed_breakdown(&records) {
                    Ok(breakdown) => {
                        McpResponse {
                            id: "cost_analysis".to_string(),
                            result: Some(serde_json::json!({
                                "start_date": start_date,
                                "end_date": end_date,
                                "total_cost": breakdown.total_cost,
                                "total_records": breakdown.total_records,
                                "total_tokens": breakdown.total_tokens,
                                "average_cost_per_request": breakdown.avg_cost_per_request,
                                "average_cost_per_token": breakdown.avg_cost_per_token,
                                "cost_by_model": breakdown.cost_by_model,
                                "most_expensive_model": breakdown.most_expensive_model,
                                "most_cost_effective_model": breakdown.most_cost_effective_model
                            })),
                            error: None,
                        }
                    }
                    Err(e) => McpResponse {
                        id: "cost_analysis".to_string(),
                        result: None,
                        error: Some(McpError {
                            code: -32000,
                            message: format!("Failed to calculate cost analysis: {}", e),
                        }),
                    },
                }
            }
            Err(e) => McpResponse {
                id: "cost_analysis".to_string(),
                result: None,
                error: Some(McpError {
                    code: -32000,
                    message: format!("Failed to load usage data: {}", e),
                }),
            },
        }
    }

    /// Get daily report
    async fn get_daily_report(&self, arguments: &serde_json::Value) -> McpResponse {
        let date_str = arguments.get("date")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Parse date
        let date = match crate::utils::parse_date_flexible(date_str) {
            Ok(dt) => dt.date_naive(),
            Err(e) => {
                return McpResponse {
                    id: "daily_report".to_string(),
                    result: None,
                    error: Some(McpError {
                        code: -32000,
                        message: format!("Invalid date format: {}", e),
                    }),
                };
            }
        };

        // Load data and filter for the specific date
        let config_manager = self.config_manager.read().await;
        let source_path = config_manager.get_data_source_path("json")
            .unwrap_or("./data/usage.json");
        
        let data_loader = DataLoader::with_source(DataSourceType::Json, source_path.to_string());
        
        match data_loader.load_usage_data().await {
            Ok(records) => {
                let daily_records: Vec<_> = records
                    .iter()
                    .filter(|r| r.is_on_date(date))
                    .cloned()
                    .collect();
                
                if daily_records.is_empty() {
                    McpResponse {
                        id: "daily_report".to_string(),
                        result: Some(serde_json::json!({
                            "date": date_str,
                            "message": "No usage data found for this date"
                        })),
                        error: None,
                    }
                } else {
                    let calculator = CostCalculator::default();
                    match calculator.calculate_daily_summary(&daily_records) {
                        Ok(summary) => {
                            McpResponse {
                                id: "daily_report".to_string(),
                                result: Some(serde_json::json!({
                                    "date": date_str,
                                    "total_cost": summary.total_cost,
                                    "total_requests": summary.request_count,
                                    "total_tokens": summary.total_input_tokens + summary.total_output_tokens,
                                    "model_breakdown": summary.model_breakdown
                                })),
                                error: None,
                            }
                        }
                        Err(e) => McpResponse {
                            id: "daily_report".to_string(),
                            result: None,
                            error: Some(McpError {
                                code: -32000,
                                message: format!("Failed to calculate daily summary: {}", e),
                            }),
                        },
                    }
                }
            }
            Err(e) => McpResponse {
                id: "daily_report".to_string(),
                result: None,
                error: Some(McpError {
                    code: -32000,
                    message: format!("Failed to load usage data: {}", e),
                }),
            },
        }
    }

    /// Get monthly report
    async fn get_monthly_report(&self, arguments: &serde_json::Value) -> McpResponse {
        let year = arguments.get("year")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let month = arguments.get("month")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        if year == 0 || month == 0 || month > 12 {
            return McpResponse {
                id: "monthly_report".to_string(),
                result: None,
                error: Some(McpError {
                    code: -32000,
                    message: "Invalid year or month".to_string(),
                }),
            };
        }

        // Load data and filter for the specific month
        let config_manager = self.config_manager.read().await;
        let source_path = config_manager.get_data_source_path("json")
            .unwrap_or("./data/usage.json");
        
        let data_loader = DataLoader::with_source(DataSourceType::Json, source_path.to_string());
        
        match data_loader.load_usage_data().await {
            Ok(records) => {
                let start_date = chrono::NaiveDate::from_ymd_opt(year as i32, month, 1)
                    .ok_or_else(|| "Invalid date".to_string())
                    .unwrap();
                
                let end_date = if month == 12 {
                    chrono::NaiveDate::from_ymd_opt(year as i32 + 1, 1, 1)
                        .ok_or_else(|| "Invalid date".to_string())
                        .unwrap()
                        .pred_opt()
                        .unwrap()
                } else {
                    chrono::NaiveDate::from_ymd_opt(year as i32, month + 1, 1)
                        .ok_or_else(|| "Invalid date".to_string())
                        .unwrap()
                        .pred_opt()
                        .unwrap()
                };
                
                let month_records: Vec<_> = records
                    .iter()
                    .filter(|r| {
                        let date = r.timestamp.date_naive();
                        date >= start_date && date <= end_date
                    })
                    .cloned()
                    .collect();
                
                if month_records.is_empty() {
                    McpResponse {
                        id: "monthly_report".to_string(),
                        result: Some(serde_json::json!({
                            "year": year,
                            "month": month,
                            "message": "No usage data found for this month"
                        })),
                        error: None,
                    }
                } else {
                    let calculator = CostCalculator::default();
                    match calculator.calculate_monthly_summary(&month_records) {
                        Ok(summary) => {
                            McpResponse {
                                id: "monthly_report".to_string(),
                                result: Some(serde_json::json!({
                                    "year": year,
                                    "month": month,
                                    "total_cost": summary.total_cost,
                                    "total_requests": summary.request_count,
                                    "total_tokens": summary.total_input_tokens + summary.total_output_tokens,
                                    "weekly_breakdown": summary.weekly_breakdown
                                })),
                                error: None,
                            }
                        }
                        Err(e) => McpResponse {
                            id: "monthly_report".to_string(),
                            result: None,
                            error: Some(McpError {
                                code: -32000,
                                message: format!("Failed to calculate monthly summary: {}", e),
                            }),
                        },
                    }
                }
            }
            Err(e) => McpResponse {
                id: "monthly_report".to_string(),
                result: None,
                error: Some(McpError {
                    code: -32000,
                    message: format!("Failed to load usage data: {}", e),
                }),
            },
        }
    }

    /// Get budget status
    async fn get_budget_status(&self) -> McpResponse {
        let config_manager = self.config_manager.read().await;
        
        if let Some(budget) = config_manager.get_budget() {
            // Load data for budget analysis
            let source_path = config_manager.get_data_source_path("json")
                .unwrap_or("./data/usage.json");
            
            let data_loader = DataLoader::with_source(DataSourceType::Json, source_path.to_string());
            
            match data_loader.load_usage_data().await {
                Ok(records) => {
                    let calculator = CostCalculator::default();
                    match calculator.calculate_budget_analysis(&records, &budget) {
                        Ok(analysis) => {
                            McpResponse {
                                id: "budget_status".to_string(),
                                result: Some(serde_json::json!({
                                    "budget_limit": budget.monthly_limit,
                                    "currency": budget.currency,
                                    "current_usage": analysis.current_usage,
                                    "budget_usage_percentage": analysis.budget_usage_percentage,
                                    "is_budget_exceeded": analysis.is_budget_exceeded,
                                    "is_warning_exceeded": analysis.is_warning_exceeded,
                                    "is_alert_exceeded": analysis.is_alert_exceeded,
                                    "projected_monthly_cost": analysis.projected_monthly_cost,
                                    "days_remaining_in_month": analysis.days_remaining_in_month
                                })),
                                error: None,
                            }
                        }
                        Err(e) => McpResponse {
                            id: "budget_status".to_string(),
                            result: None,
                            error: Some(McpError {
                                code: -32000,
                                message: format!("Failed to calculate budget analysis: {}", e),
                            }),
                        },
                    }
                }
                Err(e) => McpResponse {
                    id: "budget_status".to_string(),
                    result: None,
                    error: Some(McpError {
                        code: -32000,
                        message: format!("Failed to load usage data: {}", e),
                    }),
                },
            }
        } else {
            McpResponse {
                id: "budget_status".to_string(),
                result: Some(serde_json::json!({
                    "message": "No budget configured"
                })),
                error: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_creation() {
        let config = ServerConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            enable_auth: false,
            api_key: None,
            allowed_origins: vec!["*".to_string()],
        };
        
        let config_manager = ConfigManager::new().unwrap();
        let server = McpServer::new(config, &config_manager);
        assert!(server.is_ok());
    }

    #[test]
    fn test_get_available_tools() {
        let config = ServerConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            enable_auth: false,
            api_key: None,
            allowed_origins: vec!["*".to_string()],
        };
        
        let config_manager = ConfigManager::new().unwrap();
        let server = McpServer::new(config, &config_manager).unwrap();
        let tools = server.get_available_tools();
        
        assert!(!tools.is_empty());
        assert_eq!(tools[0].name, "get_usage_stats");
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let config = ServerConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            enable_auth: false,
            api_key: None,
            allowed_origins: vec!["*".to_string()],
        };
        
        let config_manager = ConfigManager::new().unwrap();
        let server = McpServer::new(config, &config_manager).unwrap();
        
        let request = McpRequest {
            id: "test".to_string(),
            method: "tools/list".to_string(),
            params: serde_json::json!({}),
        };
        
        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }
}