use crate::model::file::{Chainable, File, FileBuilder, FileStore, HasContent};
use loro::{LoroDoc, LoroError, LoroMap};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Text LoroDoc contains:
/// - meta
///   - type
///   - id
///   - name
///   - version
/// - doc
///   - content
#[derive(Debug, Clone, Default)]
pub struct Text {
    pub store: FileStore,
}

impl File for Text {
    fn builder() -> FileBuilder<Self> {
        FileBuilder::new("text")
    }

    fn init(&mut self, meta: Option<&LoroMap>) -> Result<(), String> {
        self.set_type("text")?;
        self.initialize_plaintext_document()?;

        let id = self
            .load_string_field_with_meta(meta, "id")
            .unwrap_or_default();
        let name = self
            .load_string_field_with_meta(meta, "name")
            .unwrap_or_default();
        let version = self
            .get_i64_field_with_meta(meta, "version")
            .unwrap_or_default();

        self.set_id(&id)?;
        self.set_name(&name)?;
        self.set_version(version)?;
        Ok(())
    }

    fn build_from(builder: FileBuilder<Self>) -> Result<Self, String> {
        // Ensure we have a store
        let store = builder.store.ok_or("No file store provided")?;

        let mut text = Text { store };
        text.init(None)
            .map_err(|e| format!("Failed to initialize text: {}", e))?;
        Ok(text)
    }

    fn store(&self) -> &FileStore {
        &self.store
    }

    fn mut_store(&mut self) -> &mut FileStore {
        &mut self.store
    }

    fn get_type(&self) -> String {
        "text".to_string()
    }

    fn to_json(&self) -> Result<Value, String> {
        let mut result = Map::new();
        self.add_field(&mut result, "id", &self.id()?.to_string())?;
        self.add_field(&mut result, "collection_type", &self.get_type())?;
        self.add_field_or_default(&mut result, "name", self.name())?;
        Ok(Value::Object(result))
    }
}

impl HasContent for Text {}

impl Serialize for Text {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_text_builder() {
        let text = Text::builder().build().expect("Failed to build text");

        console_log!("Text: {:?}", text);

        assert_eq!(text.version().unwrap(), 0);
        assert!(!text.id().unwrap().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_text_content() {
        let mut text = Text::builder().build().expect("Failed to build text");

        // Test inserting and getting content
        text.insert_content("Test Content", 0)
            .expect("Failed to insert content");
        let content = text.get_content().expect("Failed to get content");
        assert_eq!(content, "Test Content");

        // Test deleting content
        text.delete_content(0, 4).expect("Failed to delete content"); // Delete "Test"
        let content = text.get_content().expect("Failed to get content");
        assert_eq!(content, " Content");
    }

    #[wasm_bindgen_test]
    fn test_text_init() {
        let mut text = Text::builder()
            .with_id("test-id".to_string())
            .build()
            .expect("Failed to build text");

        // Create a meta map with test data
        let mut meta = LoroMap::new();
        meta.insert("name", "test-name");
        meta.insert("version", "1");

        text.init(Some(&meta)).expect("Failed to initialize text");

        assert_eq!(text.name().unwrap(), "test-name");
        assert_eq!(text.get_type(), "text");
    }

    #[wasm_bindgen_test]
    fn test_text_equality() {
        let text1 = Text::builder()
            .with_id("test-id".to_string())
            .build()
            .expect("Failed to build text");

        let text2 = Text::builder()
            .with_id("test-id".to_string())
            .build()
            .expect("Failed to build text");

        let text3 = Text::builder()
            .with_id("different-id".to_string())
            .build()
            .expect("Failed to build text");

        // Texts are equal if they have the same id, regardless of other fields
        assert_eq!(text1, text2);
        assert_ne!(text1, text3);
    }
}
