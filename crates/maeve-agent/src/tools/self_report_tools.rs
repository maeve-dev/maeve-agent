use anyhow::Result;
use serde_json::{json, Value};

use crate::context::AgentContext;

// ---------------------------------------------------------------------------
// Tool: self_report
// ---------------------------------------------------------------------------

/// Returns a comprehensive JSON snapshot of the agent's current state,
/// including version, turn count, goals, agenda, rumination digest, and
/// configuration.
pub struct SelfReport;

impl super::Tool for SelfReport {
    fn name(&self) -> &'static str {
        "self_report"
    }

    fn description(&self) -> &'static str {
        "Returns a comprehensive report of the agent's current state: version info, turn count, active goals, agenda items, rumination digest, and config values."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }

    fn call(&self, _args: Value, ctx: &AgentContext) -> Result<Value> {
        Ok(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "build_date": option_env!("BUILD_DATE").unwrap_or("unknown"),
            "git_commit": option_env!("GIT_COMMIT").unwrap_or("unknown"),
            "turn_count": ctx.turn_count,
            "goals": ctx.goals,
            "agenda": ctx.agenda,
            "rumination": ctx.rumination_digest,
            "config": {
                "agent.provider": ctx.config.provider,
                "agent.chat_model": ctx.config.chat_model,
                "agent.default_model": ctx.config.default_model,
                "agent.idle_interval_secs": ctx.config.idle_interval_secs,
                "agent.context_budget": ctx.config.context_budget,
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::config::Config;
    use crate::context::AgentContext;
    use crate::tools::Tool;

    use super::SelfReport;

    #[test]
    fn test_self_report_basic() {
        let ctx = AgentContext::new(Config::default());
        let result = SelfReport.call(json!({}), &ctx).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj["version"], json!(env!("CARGO_PKG_VERSION")));
        assert_eq!(obj["turn_count"], json!(0));
        assert_eq!(obj["goals"], json!([]));
        assert_eq!(obj["agenda"], json!([]));
        assert_eq!(obj["rumination"], json!(null));
    }

    #[test]
    fn test_self_report_with_state() {
        let mut ctx = AgentContext::new(Config::default());
        ctx.turn_count = 42;
        ctx.goals = vec![crate::goals::Goal {
            id: "g1".into(),
            description: "Test goal".into(),
            status: crate::goals::GoalStatus::Active,
            priority: 1,
        }];
        ctx.agenda = vec![crate::agenda::AgendaItem {
            id: "a1".into(),
            description: "Test agenda item".into(),
            status: crate::agenda::AgendaStatus::Pending,
            priority: 1,
        }];
        ctx.rumination_digest = Some("Thinking about things...".into());

        let result = SelfReport.call(json!({}), &ctx).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj["turn_count"], json!(42));
        assert_eq!(
            obj["goals"],
            json!([{"id": "g1", "description": "Test goal", "status": "Active", "priority": 1}])
        );
        assert_eq!(
            obj["agenda"],
            json!([{"id": "a1", "description": "Test agenda item", "status": "Pending", "priority": 1}])
        );
        assert_eq!(obj["rumination"], json!("Thinking about things..."));
    }

    #[test]
    fn test_self_report_config() {
        let config = Config {
            provider: "anthropic".into(),
            chat_model: "claude-3-opus".into(),
            default_model: "claude-3-sonnet".into(),
            idle_interval_secs: 60,
            context_budget: 200_000,
        };
        let ctx = AgentContext::new(config);
        let result = SelfReport.call(json!({}), &ctx).unwrap();
        let config_obj = &result.as_object().unwrap()["config"];

        assert_eq!(config_obj["agent.provider"], json!("anthropic"));
        assert_eq!(config_obj["agent.chat_model"], json!("claude-3-opus"));
        assert_eq!(config_obj["agent.default_model"], json!("claude-3-sonnet"));
        assert_eq!(config_obj["agent.idle_interval_secs"], json!(60));
        assert_eq!(config_obj["agent.context_budget"], json!(200_000));
    }
}
