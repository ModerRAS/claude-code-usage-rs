//! Session command implementation

use crate::data::models::*;
use crate::analysis::calculator::CostCalculator;
use crate::error::Result;

/// Session command handler
pub struct SessionCommand {
    session_id: Option<String>,
    list: bool,
}

impl SessionCommand {
    /// Create a new session command
    pub fn new(session_id: Option<String>, list: bool) -> Self {
        Self { session_id, list }
    }

    /// Execute the session command
    pub async fn execute(&self, records: &[UsageRecord]) -> Result<SessionResult> {
        if self.list {
            // List all sessions
            let sessions = self.group_records_by_session(records);
            Ok(SessionResult::SessionList(sessions))
        } else if let Some(session_id) = &self.session_id {
            // Analyze specific session
            let session_records: Vec<_> = records
                .iter()
                .filter(|r| r.session_id.as_ref() == Some(session_id))
                .cloned()
                .collect();

            if session_records.is_empty() {
                return Err(crate::error::CcusageError::Validation(
                    format!("Session '{}' not found", session_id)
                ));
            }

            let calculator = CostCalculator::default();
            let session_analysis = calculator.calculate_session_analysis(&session_records)?;
            Ok(SessionResult::SessionAnalysis(session_analysis))
        } else {
            Err(crate::error::CcusageError::Validation(
                "Either --session-id or --list must be specified".to_string()
            ))
        }
    }

    fn group_records_by_session(&self, records: &[UsageRecord]) -> Vec<Session> {
        let mut sessions: std::collections::HashMap<String, Session> = std::collections::HashMap::new();

        for record in records {
            if let Some(session_id) = &record.session_id {
                let session = sessions.entry(session_id.clone())
                    .or_insert_with(|| Session::new(
                        session_id.clone(),
                        record.timestamp,
                        record.user_id.clone(),
                    ));

                session.add_record(record);
            }
        }

        sessions.into_values().collect()
    }
}

/// Session command result
pub enum SessionResult {
    SessionList(Vec<Session>),
    SessionAnalysis(SessionAnalysis),
}

/// Session analysis result
pub struct SessionAnalysis {
    pub session_id: String,
    pub total_cost: f64,
    pub request_count: u32,
    pub total_tokens: u64,
    pub duration: u64,
}