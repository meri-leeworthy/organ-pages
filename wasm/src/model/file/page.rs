use std::collections::HashMap;
use std::convert::TryFrom;

use crate::model::file::schema::ProseMirrorSchema;
use crate::model::file::{Chainable, File, FileBuilder, FileStore, HasRichText, HasTitle, HasUrl};
use loro::{LoroDoc, LoroError, LoroMap, LoroValue, ValueOrContainer};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use wasm_bindgen::prelude::wasm_bindgen;

use super::PM_SCHEMA_KEY;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // log to JS console
}

// Helper macro for logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!("[Page (WASM)] {}", format!($($t)*))))
}

/// Page LoroDoc contains:
/// - meta
///   - type
///   - id
///   - name
///   - version
///   - title
///   - url
/// - doc
///   - children
#[derive(Debug, Clone, Default)]
pub struct Page {
    pub store: FileStore,
}

impl File for Page {
    fn builder() -> FileBuilder<Self> {
        FileBuilder::new("page")
    }

    fn init(&mut self, meta: Option<&LoroMap>) -> Result<(), String> {
        self.set_type("page")?;
        self.initialize_richtext_document()?;

        let id = self
            .load_string_field_with_meta(meta, "id")
            .unwrap_or_default();
        let name = self
            .load_string_field_with_meta(meta, "name")
            .unwrap_or_default();
        let version = self
            .get_i64_field_with_meta(meta, "version")
            .unwrap_or_default();
        let title = self
            .load_string_field_with_meta(meta, "title")
            .unwrap_or(self.get_title().unwrap_or_default());
        let url = self
            .load_string_field_with_meta(meta, "url")
            .unwrap_or(self.get_url().unwrap_or_default());

        self.set_id(&id)?;
        self.set_name(&name)?;
        self.set_version(version)?;
        self.set_title(&title)?;
        self.set_url(&url)?;
        Ok(())
    }

    fn build_from(builder: FileBuilder<Self>) -> Result<Self, String> {
        // Ensure we have a store
        let store = builder.store.ok_or("No file store provided")?;

        // console_log!("Building page from builder: {:?}", builder);
        let mut page = Page { store };
        page.init(None)
            .map_err(|e| format!("Failed to initialize page: {}", e))?;
        Ok(page)
    }

    fn store(&self) -> &FileStore {
        &self.store
    }

    fn mut_store(&mut self) -> &mut FileStore {
        &mut self.store
    }

    fn get_type(&self) -> String {
        "page".to_string()
    }

    fn to_json(&self) -> Result<Value, String> {
        let mut result = Map::new();
        self.add_field(&mut result, "id", &self.id()?.to_string())?;
        self.add_field(&mut result, "collection_type", &self.get_type())?;
        self.add_field_or_default(&mut result, "name", self.name())?;
        self.add_field_or_default(&mut result, "title", self.get_title())?;
        self.add_field_or_default(&mut result, "url", self.get_url())?;
        Ok(Value::Object(result))
    }
}

impl HasTitle for Page {}

impl HasUrl for Page {}

impl HasRichText for Page {
    fn schema_json(&self) -> String {
        let schema = self.schema();
        serde_json::to_string(&schema).unwrap_or_else(|_| "{}".to_string())
    }

    fn schema(&self) -> ProseMirrorSchema {
        let schema = match self.meta().get(PM_SCHEMA_KEY) {
            Some(ValueOrContainer::Value(LoroValue::String(schema))) => schema.to_string(),
            _ => "".to_string(),
        };
        let schema = ProseMirrorSchema::try_from(schema).unwrap_or_default();
        schema
    }
}

impl Serialize for Page {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Page {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl PartialEq for Page {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;
    use serde_json::json;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn test_schema() -> ProseMirrorSchema {
        let schema_json = json!({
            "marks": {
                "bold": { "inclusive": true },
                "italic": { "inclusive": false }
            },
            "nodes": {
                "doc": {
                    "content": "block+"
                },
                "paragraph": {
                    "content": "inline*",
                    "group": "block"
                }
            }
        });

        ProseMirrorSchema::try_from(schema_json).expect("Failed to create test schema")
    }

    #[wasm_bindgen_test]
    fn test_page_builder() {
        let pm_schema = test_schema();
        let page = Page::builder()
            .with_pm_schema(pm_schema.clone())
            .expect("Failed to add pm schema")
            .build()
            .expect("Failed to build page");

        assert_eq!(page.version().unwrap(), 0);
        assert_eq!(page.schema(), pm_schema);
        assert!(!page.id().unwrap().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_page_title() {
        let mut page = Page::builder()
            .with_pm_schema(test_schema())
            .expect("Failed to add pm schema")
            .build()
            .expect("Failed to build page");

        // Test setting and getting title
        page.set_title("Test Page").expect("Failed to set title");
        let title = page.get_title().expect("Failed to get title");
        assert_eq!(title, "Test Page");
    }

    #[wasm_bindgen_test]
    fn test_page_url() {
        let mut page = Page::builder()
            .with_pm_schema(test_schema())
            .expect("Failed to add pm schema")
            .build()
            .expect("Failed to build page");

        // Test setting and getting URL
        page.set_url("/test-page").expect("Failed to set URL");
        let url = page.get_url().expect("Failed to get URL");
        assert_eq!(url, "/test-page");
    }

    #[wasm_bindgen_test]
    fn test_page_to_json() {
        let mut page = Page::builder()
            .with_pm_schema(test_schema())
            .expect("Failed to add pm schema")
            .build()
            .expect("Failed to build page");

        page.set_title("Test Page").expect("Failed to set title");
        page.set_name("test-page").expect("Failed to set name");

        let json = page.to_json().expect("Failed to convert to JSON");
        let obj = json.as_object().expect("JSON should be an object");

        assert_eq!(obj["title"].as_str().unwrap(), "Test Page");
        assert_eq!(obj["name"].as_str().unwrap(), "test-page");
        assert_eq!(obj["collection_type"].as_str().unwrap(), "page");
    }

    #[wasm_bindgen_test]
    fn test_page_init() {
        let mut page = Page::builder()
            .with_id("test-id".to_string())
            .with_pm_schema(test_schema())
            .expect("Failed to add pm schema")
            .build()
            .expect("Failed to build page");

        // Create a meta map with test data
        let meta = LoroMap::new();
        meta.insert("title", "Test Title");
        meta.insert("url", "/test-url");
        meta.insert("name", "test-name");
        meta.insert("version", "1");

        page.init(Some(&meta)).expect("Failed to initialize page");

        assert_eq!(page.get_title().unwrap(), "Test Title");
        assert_eq!(page.get_url().unwrap(), "/test-url");
        assert_eq!(page.name().unwrap(), "test-name");
        assert_eq!(page.get_type(), "page");
    }

    #[wasm_bindgen_test]
    fn test_page_equality() {
        let schema = test_schema();
        let page1 = Page::builder()
            .with_id("test-id".to_string())
            .with_pm_schema(schema.clone())
            .expect("Failed to add pm schema")
            .build()
            .expect("Failed to build page");

        let page2 = Page::builder()
            .with_id("test-id".to_string())
            .with_pm_schema(schema.clone())
            .expect("Failed to add pm schema")
            .build()
            .expect("Failed to build page");

        let page3 = Page::builder()
            .with_id("different-id".to_string())
            .with_pm_schema(schema.clone())
            .expect("Failed to add pm schema")
            .build()
            .expect("Failed to build page");

        // Pages are equal if they have the same id, regardless of other fields
        assert_eq!(page1, page2);
        assert_ne!(page1, page3);
    }

    #[wasm_bindgen_test]
    fn test_page_schema_validation() {
        // Test with invalid schema
        let invalid_schema = ProseMirrorSchema {
            marks: HashMap::new(),
            nodes: HashMap::new(),
            top_node: "invalid".to_string(),
        };

        let result = Page::builder()
            .with_pm_schema(invalid_schema)
            .expect("Failed to add pm schema")
            .build();

        // Should still build since schema validation is not strict
        assert!(result.is_ok());

        // Test with empty schema
        let empty_schema = ProseMirrorSchema::default();
        let result = Page::builder()
            .with_pm_schema(empty_schema)
            .expect("Failed to add pm schema")
            .build();
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_page_rich_text_operations() {
        let schema = test_schema();
        let mut page = Page::builder()
            .with_pm_schema(schema.clone())
            .expect("Failed to add pm schema")
            .build()
            .expect("Failed to build page");

        // Test applying steps
        let steps = vec![json!({
            "stepType": "replace",
            "from": 0,
            "to": 0,
            "slice": {
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": "Hello world"
                    }]
                }]
            }
        })];

        let new_version = page.apply_steps(&steps, 0).expect("Failed to apply steps");
        assert_eq!(new_version, 1);

        // Test version handling
        let result = page.apply_steps(&steps, 0);
        assert!(
            result.is_ok(),
            "Should accept steps even with version mismatch"
        );
    }

    #[wasm_bindgen_test]
    fn test_page_schema_json() {
        let schema = test_schema();
        let page = Page::builder()
            .with_pm_schema(schema.clone())
            .expect("Failed to add pm schema")
            .build()
            .expect("Failed to build page");

        let schema_json = page.schema_json();
        let parsed_schema: Value =
            serde_json::from_str(&schema_json).expect("Failed to parse schema JSON");

        // Verify schema structure
        let obj = parsed_schema.as_object().unwrap();
        assert!(obj.contains_key("marks"));
        assert!(obj.contains_key("nodes"));
        assert!(obj.contains_key("top_node"));

        // Verify marks
        let marks = obj["marks"].as_object().unwrap();
        assert!(marks.contains_key("bold"));
        assert!(marks.contains_key("italic"));

        // Verify nodes
        let nodes = obj["nodes"].as_object().unwrap();
        assert!(nodes.contains_key("doc"));
        assert!(nodes.contains_key("paragraph"));
    }
}
