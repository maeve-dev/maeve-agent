use serde::{Deserialize, Serialize};

/// Configuration for the MAEVE agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub provider: String,
    pub chat_model: String,
    pub default_model: String,
    pub idle_interval_secs: u64,
    pub context_budget: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            chat_model: "gpt-4".to_string(),
            default_model: "gpt-4".to_string(),
            idle_interval_secs: 30,
            context_budget: 128_000,
        }
    }
}
