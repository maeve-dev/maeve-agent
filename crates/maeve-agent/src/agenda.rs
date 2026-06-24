use serde::{Deserialize, Serialize};

/// An item on the agent's agenda.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaItem {
    pub id: String,
    pub description: String,
    pub status: AgendaStatus,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgendaStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl std::fmt::Display for AgendaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgendaStatus::Pending => write!(f, "pending"),
            AgendaStatus::InProgress => write!(f, "in_progress"),
            AgendaStatus::Completed => write!(f, "completed"),
            AgendaStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}
