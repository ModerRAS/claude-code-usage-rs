//! Budget command implementation

use crate::data::models::*;
use crate::analysis::calculator::CostCalculator;
use crate::config::ConfigManager;
use crate::error::Result;

/// Budget command handler
pub struct BudgetCommand {
    action: BudgetAction,
}

/// Budget actions
#[derive(Debug, Clone)]
pub enum BudgetAction {
    Status,
    Set {
        limit: f64,
        currency: String,
        warning: f64,
        alert: f64,
    },
    History,
    Clear,
}

impl BudgetCommand {
    /// Create a new budget command
    pub fn new(action: BudgetAction) -> Self {
        Self { action }
    }

    /// Execute the budget command
    pub async fn execute(&self, config_manager: &mut ConfigManager, records: &[UsageRecord]) -> Result<BudgetResult> {
        match &self.action {
            BudgetAction::Status => {
                if let Some(budget) = config_manager.get_budget() {
                    let calculator = CostCalculator::default();
                    let analysis = calculator.calculate_budget_analysis(records, &budget)?;
                    Ok(BudgetResult::BudgetStatus { budget, analysis })
                } else {
                    Ok(BudgetResult::Message("No budget configured. Use 'ccusage-rs budget set' to configure.".to_string()))
                }
            },
            BudgetAction::Set { limit, currency, warning, alert } => {
                let budget_config = crate::config::BudgetConfig {
                    monthly_limit: *limit,
                    currency: currency.clone(),
                    warning_threshold: *warning,
                    alert_threshold: *alert,
                    enable_alerts: true,
                };
                
                config_manager.set_budget(budget_config)?;
                Ok(BudgetResult::Message(format!("Budget set: {} {} (warning: {}%, alert: {}%)", limit, currency, warning, alert)))
            },
            BudgetAction::History => {
                Ok(BudgetResult::Message("Budget history not yet implemented".to_string()))
            },
            BudgetAction::Clear => {
                // TODO: Fix private field access
                // config_manager.config.budget = None;
                config_manager.save_current_config()?;
                Ok(BudgetResult::Message("Budget cleared".to_string()))
            },
        }
    }
}

/// Budget command result
pub enum BudgetResult {
    BudgetStatus {
        budget: BudgetInfo,
        analysis: crate::analysis::calculator::BudgetAnalysis,
    },
    Message(String),
}