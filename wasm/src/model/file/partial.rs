use crate::model::file::{Chainable, File, FileBuilder, FileStore, HasContent};
use loro::LoroMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // log to JS console
}

// Helper macro for logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!("[Partial (WASM)] {}", format!($($t)*))))
}

/// Partial LoroDoc contains:
/// - meta
///   - type
///   - id
///   - name
///   - version
/// - doc
///   - content
#[derive(Debug, Clone, Default)]
pub struct Partial {
    pub store: FileStore,
}

impl File for Partial {
    fn builder() -> FileBuilder<Self> {
        FileBuilder::new("partial")
    }

    async fn init(&mut self, meta: Option<&LoroMap>) -> Result<(), String> {
        self.set_type("partial").await?;
        match self.initialize_plaintext_document() {
            Ok(_) => (),
            Err(e) => console_log!("Plaintext document was not initialized: {}", e),
        }

        let id = self
            .load_string_field_with_meta(meta, "id")
            .unwrap_or_default();
        let name = self
            .load_string_field_with_meta(meta, "name")
            .unwrap_or_default();
        let version = self
            .get_i64_field_with_meta(meta, "version")
            .unwrap_or_default();

        self.set_id(&id).await?;
        self.set_name(&name).await?;
        self.set_version(version).await?;
        Ok(())
    }

    async fn build_from(builder: FileBuilder<Self>) -> Result<Self, String> {
        // Ensure we have a store
        let store = builder.store.ok_or("No file store provided")?;

        let mut partial = Partial { store };
        partial
            .init(None)
            .await
            .map_err(|e| format!("Failed to initialize partial: {}", e))?;
        Ok(partial)
    }

    fn store(&self) -> &FileStore {
        &self.store
    }

    fn mut_store(&mut self) -> &mut FileStore {
        &mut self.store
    }

    fn get_type(&self) -> String {
        "partial".to_string()
    }

    fn to_json(&self) -> Result<Value, String> {
        let mut result = Map::new();
        self.add_field(&mut result, "id", &self.id()?.to_string())?;
        self.add_field(&mut result, "collection_type", &self.get_type())?;
        self.add_field_or_default(&mut result, "name", self.name())?;
        Ok(Value::Object(result))
    }
}

impl HasContent for Partial {}

impl Serialize for Partial {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Partial {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl PartialEq for Partial {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loro::LoroDoc;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_partial_builder() {
        let partial = Partial::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .build()
            .await
            .expect("Failed to build partial");

        assert_eq!(partial.version().unwrap(), 0);
        assert!(!partial.id().unwrap().is_empty());
    }

    #[wasm_bindgen_test]
    async fn test_partial_content() {
        let mut partial = Partial::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .build()
            .await
            .expect("Failed to build partial");

        // Test inserting and getting content
        partial
            .insert_content("Test Content", 0)
            .expect("Failed to insert content");
        let content = partial.get_content().expect("Failed to get content");
        assert_eq!(content, "Test Content");

        // Test deleting content
        partial
            .delete_content(0, 4)
            .expect("Failed to delete content"); // Delete "Test"
        let content = partial.get_content().expect("Failed to get content");
        assert_eq!(content, " Content");
    }

    #[wasm_bindgen_test]
    async fn test_partial_init() {
        let mut partial = Partial::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_id("test-id".to_string())
            .expect("Failed to set id")
            .build()
            .await
            .expect("Failed to build partial");

        // Create a meta map with test data
        let meta = LoroMap::new();
        meta.insert("name", "test-name");
        meta.insert("version", "1");

        partial
            .init(Some(&meta))
            .await
            .expect("Failed to initialize partial");

        assert_eq!(partial.name().unwrap(), "test-name");
        assert_eq!(partial.get_type(), "partial");
    }

    #[wasm_bindgen_test]
    async fn test_partial_equality() {
        let partial1 = Partial::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_id("test-id".to_string())
            .expect("Failed to set id")
            .build()
            .await
            .expect("Failed to build partial");

        let partial2 = Partial::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_id("test-id".to_string())
            .expect("Failed to set id")
            .build()
            .await
            .expect("Failed to build partial");

        let partial3 = Partial::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_id("different-id".to_string())
            .expect("Failed to set id")
            .build()
            .await
            .expect("Failed to build partial");

        // Partials are equal if they have the same id, regardless of other fields
        assert_eq!(partial1, partial2);
        assert_ne!(partial1, partial3);
    }
}
