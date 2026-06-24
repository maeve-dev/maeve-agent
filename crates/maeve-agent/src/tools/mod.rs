pub mod self_report_tools;

use anyhow::Result;
use serde_json::{json, Value};

use crate::context::AgentContext;

// ---------------------------------------------------------------------------
// Tool trait – every tool implements this.
// ---------------------------------------------------------------------------

pub trait Tool: Send + Sync {
    /// The machine-readable name used to invoke the tool.
    fn name(&self) -> &'static str;

    /// A human-readable description for the LLM.
    fn description(&self) -> &'static str;

    /// JSON Schema for the tool's parameters (empty object if none).
    fn parameters(&self) -> Value;

    /// Execute the tool with the given arguments (JSON value) and the current
    /// agent context.  Returns a JSON value to send back to the LLM.
    fn call(&self, _args: Value, _ctx: &AgentContext) -> Result<Value>;
}

// ---------------------------------------------------------------------------
// Tool: list_agenda
// ---------------------------------------------------------------------------

pub struct ListAgenda;

impl Tool for ListAgenda {
    fn name(&self) -> &'static str {
        "list_agenda"
    }

    fn description(&self) -> &'static str {
        "Lists all items currently on the agent's agenda."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }

    fn call(&self, _args: Value, ctx: &AgentContext) -> Result<Value> {
        Ok(serde_json::to_value(&ctx.agenda)?)
    }
}

// ---------------------------------------------------------------------------
// Tool: list_goals
// ---------------------------------------------------------------------------

pub struct ListGoals;

impl Tool for ListGoals {
    fn name(&self) -> &'static str {
        "list_goals"
    }

    fn description(&self) -> &'static str {
        "Lists all goals on the agent's goal stack."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }

    fn call(&self, _args: Value, ctx: &AgentContext) -> Result<Value> {
        Ok(serde_json::to_value(&ctx.goals)?)
    }
}

// ---------------------------------------------------------------------------
// Tool: list_rules
// ---------------------------------------------------------------------------

pub struct ListRules;

impl Tool for ListRules {
    fn name(&self) -> &'static str {
        "list_rules"
    }

    fn description(&self) -> &'static str {
        "Lists the behavioural rules the agent follows."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }

    fn call(&self, _args: Value, _ctx: &AgentContext) -> Result<Value> {
        // Stub – real implementation would read from a rule store.
        Ok(json!([]))
    }
}

// ---------------------------------------------------------------------------
// Tool: list_workflows
// ---------------------------------------------------------------------------

pub struct ListWorkflows;

impl Tool for ListWorkflows {
    fn name(&self) -> &'static str {
        "list_workflows"
    }

    fn description(&self) -> &'static str {
        "Lists the workflows available to the agent."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }

    fn call(&self, _args: Value, _ctx: &AgentContext) -> Result<Value> {
        // Stub – real implementation would read from a workflow registry.
        Ok(json!([]))
    }
}

// ---------------------------------------------------------------------------
// Tool registry
// ---------------------------------------------------------------------------

/// Returns all registered tools.
pub fn all_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(self_report_tools::SelfReport),
        Box::new(super::memory_tools::Recall),
        Box::new(ListAgenda),
        Box::new(ListGoals),
        Box::new(ListRules),
        Box::new(ListWorkflows),
    ]
}

// ---------------------------------------------------------------------------
// Dispatch helper
// ---------------------------------------------------------------------------

/// Look up a tool by name and call it.
pub fn dispatch_tool(name: &str, args: Value, ctx: &AgentContext) -> Result<Value> {
    for tool in all_tools() {
        if tool.name() == name {
            return tool.call(args, ctx);
        }
    }
    anyhow::bail!("unknown tool: {}", name)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::agenda::AgendaItem;
    use crate::config::Config;
    use crate::context::AgentContext;
    use crate::goals::{Goal, GoalStatus};
    use crate::tools::{dispatch_tool, self_report_tools::SelfReport, Tool};

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
        ctx.goals = vec![Goal {
            id: "g1".into(),
            description: "Test goal".into(),
            status: GoalStatus::Active,
            priority: 1,
        }];
        ctx.agenda = vec![AgendaItem {
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

    #[test]
    fn test_dispatch_self_report() {
        let ctx = AgentContext::new(Config::default());
        let result = dispatch_tool("self_report", json!({}), &ctx).unwrap();
        assert!(result.as_object().unwrap().contains_key("version"));
    }

    #[test]
    fn test_other_tools_still_work() {
        let ctx = AgentContext::new(Config::default());
        let result = dispatch_tool("list_agenda", json!({}), &ctx).unwrap();
        assert_eq!(result, json!([]));

        let result = dispatch_tool("list_goals", json!({}), &ctx).unwrap();
        assert_eq!(result, json!([]));
    }

    #[test]
    fn test_unknown_tool() {
        let ctx = AgentContext::new(Config::default());
        let result = dispatch_tool("nonexistent", json!({}), &ctx);
        assert!(result.is_err());
    }
}
