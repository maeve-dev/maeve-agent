use serde::{Deserialize, Serialize};

use crate::agenda::AgendaItem;
use crate::config::Config;
use crate::goals::Goal;

/// The agent's runtime context, threaded through the cognition loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// How many turns the agent has taken so far.
    pub turn_count: u64,

    /// The current goal stack.
    pub goals: Vec<Goal>,

    /// The current agenda items.
    pub agenda: Vec<AgendaItem>,

    /// Digest of the agent's rumination (latest introspection).
    pub rumination_digest: Option<String>,

    /// Agent configuration.
    pub config: Config,
}

impl AgentContext {
    pub fn new(config: Config) -> Self {
        Self {
            turn_count: 0,
            goals: Vec::new(),
            agenda: Vec::new(),
            rumination_digest: None,
            config,
        }
    }
}
