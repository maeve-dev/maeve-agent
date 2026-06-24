use anyhow::Result;
use serde_json::{json, Value};
use time::OffsetDateTime;

use maeve_memory::store::{LocalNoteStore, NoteMetadata, NoteStore};

use crate::context::AgentContext;

// ---------------------------------------------------------------------------
// Tool: recall
// ---------------------------------------------------------------------------

/// A tool that lets the agent search its memory notes.
pub struct Recall;

impl crate::tools::Tool for Recall {
    fn name(&self) -> &'static str {
        "recall"
    }

    fn description(&self) -> &'static str {
        "Search the agent's memory notes. Returns notes matching the query, optionally filtered by creation time."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to match against note titles and tags."
                },
                "since": {
                    "type": "string",
                    "description": "Optional RFC3339 timestamp. When set, only returns notes created at or after this time. Example: '2026-06-01T00:00:00Z'"
                }
            },
            "required": ["query"],
            "additionalProperties": false
        })
    }

    fn call(&self, args: Value, _ctx: &AgentContext) -> Result<Value> {
        let rt = tokio::runtime::Runtime::new()?;
        let store = default_note_store();
        rt.block_on(dispatch_recall(args, &store))
    }
}

/// Dispatch logic for the `recall` tool.
///
/// Accepts a reference to any `NoteStore` implementation, making it testable
/// without touching the filesystem.
pub(crate) async fn dispatch_recall(args: Value, store: &impl NoteStore) -> Result<Value> {
    let query = require_str(&args, "query")?;
    let since = args.get("since").and_then(|v| v.as_str());

    let hits: Vec<NoteMetadata> = if let Some(since_str) = since {
        let since_time =
            OffsetDateTime::parse(since_str, &time::format_description::well_known::Rfc3339)
                .map_err(|e| anyhow::anyhow!("invalid since timestamp: {e}"))?;
        store
            .search_since(query, since_time)
            .await
            .map_err(|e| anyhow::anyhow!("store error: {e}"))?
    } else {
        store
            .search(query)
            .await
            .map_err(|e| anyhow::anyhow!("store error: {e}"))?
    };

    Ok(json!({
        "query": query,
        "count": hits.len(),
        "results": hits
    }))
}

/// Helper: extract a required string field from a JSON object.
fn require_str<'a>(input: &'a Value, key: &str) -> Result<&'a str> {
    input
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing required parameter: {key}"))
}

/// Create a default local note store.
fn default_note_store() -> LocalNoteStore {
    let base = dirs_next::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("maeve")
        .join("notes");
    LocalNoteStore::new(base)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use time::macros::datetime;

    use crate::tools::Tool;
    use maeve_memory::store::{MemoryError, NoteMetadata, NoteStore};

    use super::Recall;

    /// A test store that returns predictable results without touching disk.
    struct TestNoteStore {
        call_count: AtomicUsize,
    }

    impl TestNoteStore {
        fn new() -> Self {
            Self {
                call_count: AtomicUsize::new(0),
            }
        }
    }

    impl NoteStore for TestNoteStore {
        async fn search(&self, query: &str) -> Result<Vec<NoteMetadata>, MemoryError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(vec![NoteMetadata {
                id: "test-1".into(),
                title: query.to_string(),
                created: datetime!(2025-01-15 0:00 UTC),
                tags: vec![],
                snippet: "A test note".into(),
                salience: 1.0,
            }])
        }

        async fn search_since(
            &self,
            query: &str,
            created_after: time::OffsetDateTime,
        ) -> Result<Vec<NoteMetadata>, MemoryError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            let meta = NoteMetadata {
                id: "test-1".into(),
                title: query.to_string(),
                created: datetime!(2025-06-15 0:00 UTC),
                tags: vec![],
                snippet: "A filtered test note".into(),
                salience: 1.0,
            };
            if meta.created >= created_after {
                Ok(vec![meta])
            } else {
                Ok(vec![])
            }
        }
    }

    #[test]
    fn test_recall_schema_has_since() {
        let recall = Recall;
        let params = recall.parameters();
        let props = params["properties"].as_object().unwrap();
        assert!(props.contains_key("since"), "schema should include 'since'");
        assert_eq!(props["since"]["type"], "string");
        assert_eq!(
            props["since"]["description"],
            "Optional RFC3339 timestamp. When set, only returns notes created at or after this time. Example: '2026-06-01T00:00:00Z'"
        );
    }

    #[tokio::test]
    async fn test_dispatch_recall_without_since() {
        let store = TestNoteStore::new();

        let result = super::dispatch_recall(json!({"query": "hello"}), &store)
            .await
            .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj["query"], "hello");
        assert_eq!(obj["count"], 1);
        let results = obj["results"].as_array().unwrap();
        assert_eq!(results[0]["title"], "hello");
    }

    #[tokio::test]
    async fn test_dispatch_recall_with_since() {
        let store = TestNoteStore::new();

        // Use a since time that is before the test note's created (2025-06-15)
        let result = super::dispatch_recall(
            json!({"query": "test", "since": "2025-06-01T00:00:00Z"}),
            &store,
        )
        .await
        .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj["count"], 1);
        assert_eq!(obj["results"][0]["snippet"], "A filtered test note");
    }

    #[tokio::test]
    async fn test_dispatch_recall_with_since_excludes_older() {
        let store = TestNoteStore::new();

        // since after the note's creation time (2025-06-15) → no results
        let result = super::dispatch_recall(
            json!({"query": "test", "since": "2025-07-01T00:00:00Z"}),
            &store,
        )
        .await
        .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj["count"], 0);
    }

    #[tokio::test]
    async fn test_dispatch_recall_invalid_since() {
        let store = TestNoteStore::new();

        let result =
            super::dispatch_recall(json!({"query": "test", "since": "not-a-timestamp"}), &store)
                .await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid since timestamp"),
            "expected error about invalid timestamp"
        );
    }

    #[test]
    fn test_require_str_missing() {
        let input = json!({"foo": "bar"});
        let result = super::require_str(&input, "query");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing required parameter"));
    }

    #[test]
    fn test_require_str_ok() {
        let input = json!({"query": "hello"});
        let result = super::require_str(&input, "query");
        assert_eq!(result.unwrap(), "hello");
    }
}
