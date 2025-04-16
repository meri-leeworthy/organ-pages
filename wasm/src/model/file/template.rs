use crate::model::file::{Chainable, File, FileBuilder, FileStore, HasContent};
use loro::{LoroDoc, LoroError, LoroMap};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Template LoroDoc contains:
/// - meta
///   - type
///   - id
///   - name
///   - version
/// - doc
///   - content
#[derive(Debug, Clone, Default)]
pub struct Template {
    pub store: FileStore,
}

impl File for Template {
    fn builder() -> FileBuilder<Self> {
        FileBuilder::new("template")
    }

    fn init(&mut self, meta: Option<&LoroMap>) -> Result<(), String> {
        self.set_type("template")?;
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

        let mut template = Template { store };
        template
            .init(None)
            .map_err(|e| format!("Failed to initialize template: {}", e))?;
        Ok(template)
    }

    fn store(&self) -> &FileStore {
        &self.store
    }

    fn mut_store(&mut self) -> &mut FileStore {
        &mut self.store
    }

    fn get_type(&self) -> String {
        "template".to_string()
    }

    fn to_json(&self) -> Result<Value, String> {
        let mut result = Map::new();
        self.add_field(&mut result, "id", &self.id()?.to_string())?;
        self.add_field(&mut result, "collection_type", &self.get_type())?;
        self.add_field_or_default(&mut result, "name", self.name())?;
        Ok(Value::Object(result))
    }
}

impl HasContent for Template {}

impl Serialize for Template {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Template {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl PartialEq for Template {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loro::LoroValue;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_template_builder() {
        let template = Template::builder()
            .build()
            .expect("Failed to build template");

        assert_eq!(template.version().unwrap(), 0);
        assert!(!template.id().unwrap().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_template_content() {
        let mut template = Template::builder()
            .build()
            .expect("Failed to build template");

        // Test inserting and getting content
        template
            .insert_content("Test Content", 0)
            .expect("Failed to insert content");
        let content = template.get_content().expect("Failed to get content");
        assert_eq!(content, "Test Content");

        // Test deleting content
        template
            .delete_content(0, 4)
            .expect("Failed to delete content"); // Delete "Test"
        let content = template.get_content().expect("Failed to get content");
        assert_eq!(content, " Content");
    }

    #[wasm_bindgen_test]
    fn test_template_init() {
        let mut template = Template::builder()
            .with_id("test-id".to_string())
            .build()
            .expect("Failed to build template");

        // Create a meta map with test data
        let meta = LoroMap::new();
        meta.insert("name", "test-name");
        meta.insert("version", "1");

        template
            .init(Some(&meta))
            .expect("Failed to initialize template");

        assert_eq!(template.name().unwrap(), "test-name");
        assert_eq!(template.get_type(), "template");
    }

    #[wasm_bindgen_test]
    fn test_template_equality() {
        let template1 = Template::builder()
            .with_id("test-id".to_string())
            .build()
            .expect("Failed to build template");

        let template2 = Template::builder()
            .with_id("test-id".to_string())
            .build()
            .expect("Failed to build template");

        let template3 = Template::builder()
            .with_id("different-id".to_string())
            .build()
            .expect("Failed to build template");

        // Templates are equal if they have the same id, regardless of other fields
        assert_eq!(template1, template2);
        assert_ne!(template1, template3);
    }
}
