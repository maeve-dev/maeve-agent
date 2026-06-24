use serde::{Deserialize, Serialize};

/// A goal tracked by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub status: GoalStatus,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalStatus {
    Active,
    Completed,
    Cancelled,
    Blocked,
}

impl std::fmt::Display for GoalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoalStatus::Active => write!(f, "active"),
            GoalStatus::Completed => write!(f, "completed"),
            GoalStatus::Cancelled => write!(f, "cancelled"),
            GoalStatus::Blocked => write!(f, "blocked"),
        }
    }
}
