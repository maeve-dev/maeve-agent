use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

/// Errors that can occur during memory store operations.
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("store error: {0}")]
    Store(String),

    #[error("note not found")]
    NotFound,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Metadata about a stored note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub id: String,
    pub title: String,
    pub created: OffsetDateTime,
    pub tags: Vec<String>,
    pub snippet: String,
    pub salience: f64,
}

// ---------------------------------------------------------------------------
// NoteStore trait
// ---------------------------------------------------------------------------

/// A persistent store for memory notes.
#[allow(async_fn_in_trait)]
pub trait NoteStore: Send + Sync {
    /// Search notes by query string (title / tag / alias matching).
    async fn search(&self, query: &str) -> Result<Vec<NoteMetadata>, MemoryError>;

    /// Search notes created after a given timestamp.
    async fn search_since(
        &self,
        query: &str,
        created_after: OffsetDateTime,
    ) -> Result<Vec<NoteMetadata>, MemoryError> {
        let _ = (query, created_after);
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------
// LocalNoteStore – simple in‑memory implementation backed by a JSON file
// ---------------------------------------------------------------------------

/// A note store backed by a local directory of JSON files.
pub struct LocalNoteStore {
    /// Path to the storage directory.
    path: std::path::PathBuf,
}

impl LocalNoteStore {
    /// Create a new `LocalNoteStore` rooted at `path`.
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl NoteStore for LocalNoteStore {
    async fn search(&self, query: &str) -> Result<Vec<NoteMetadata>, MemoryError> {
        // Simple in‑memory search: load all notes and filter
        let notes = load_all_notes(&self.path).await?;
        let query_lower = query.to_lowercase();

        Ok(notes
            .into_iter()
            .filter(|n| {
                n.title.to_lowercase().contains(&query_lower)
                    || n.tags.iter().any(|t| t.to_lowercase() == query_lower)
            })
            .collect())
    }

    async fn search_since(
        &self,
        query: &str,
        created_after: OffsetDateTime,
    ) -> Result<Vec<NoteMetadata>, MemoryError> {
        let notes = load_all_notes(&self.path).await?;
        let query_lower = query.to_lowercase();

        Ok(notes
            .into_iter()
            .filter(|n| {
                (n.title.to_lowercase().contains(&query_lower)
                    || n.tags.iter().any(|t| t.to_lowercase() == query_lower))
                    && n.created >= created_after
            })
            .collect())
    }
}

/// Load all note files from the given directory.
async fn load_all_notes(path: &std::path::Path) -> Result<Vec<NoteMetadata>, MemoryError> {
    let mut notes = Vec::new();
    let mut dir = tokio::fs::read_dir(path).await?;

    while let Some(entry) = dir.next_entry().await? {
        let file_path = entry.path();
        if file_path.extension().is_some_and(|e| e == "json") {
            let content = tokio::fs::read_to_string(&file_path).await?;
            if let Ok(note) = serde_json::from_str::<NoteMetadata>(&content) {
                notes.push(note);
            }
        }
    }

    Ok(notes)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[tokio::test]
    async fn test_search_since_default_trait_method() {
        // The default implementation returns an empty vec.
        struct EmptyStore;

        impl NoteStore for EmptyStore {
            async fn search(&self, _query: &str) -> Result<Vec<NoteMetadata>, MemoryError> {
                Ok(vec![])
            }
        }

        let store = EmptyStore;
        let since = datetime!(2024-01-01 0:00 UTC);
        let results = store.search_since("test", since).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_local_store_search_since() {
        // Create a temp directory and write some test note files.
        let dir = std::env::temp_dir().join(format!("maeve_test_{}", std::process::id()));
        let _ = tokio::fs::create_dir_all(&dir).await;

        let note1 = NoteMetadata {
            id: "1".into(),
            title: "alpha".into(),
            created: datetime!(2024-06-01 0:00 UTC),
            tags: vec!["important".into()],
            snippet: "First note".into(),
            salience: 0.9,
        };
        let note2 = NoteMetadata {
            id: "2".into(),
            title: "beta".into(),
            created: datetime!(2024-07-01 0:00 UTC),
            tags: vec!["trivial".into()],
            snippet: "Second note".into(),
            salience: 0.5,
        };

        tokio::fs::write(
            dir.join("note1.json"),
            serde_json::to_string_pretty(&note1).unwrap(),
        )
        .await
        .unwrap();
        tokio::fs::write(
            dir.join("note2.json"),
            serde_json::to_string_pretty(&note2).unwrap(),
        )
        .await
        .unwrap();

        let store = LocalNoteStore::new(&dir);
        let since = datetime!(2024-06-15 0:00 UTC);

        let results = store.search_since("alpha", since).await.unwrap();
        // note1 has created 2024-06-01 which is before since, so no match
        assert!(results.is_empty());

        let results = store.search_since("beta", since).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "2");

        // Clean up
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
