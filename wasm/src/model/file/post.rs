use crate::model::file::{Chainable, File, FileBuilder, FileStore, HasRichText, HasTitle, HasUrl};
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
    ($($t:tt)*) => (log(&format!("[Page (WASM)] {}", format!($($t)*))))
}

/// Post LoroDoc contains:
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
pub struct Post {
    pub store: FileStore,
}

impl File for Post {
    fn builder() -> FileBuilder<Self> {
        FileBuilder::new("post")
    }

    async fn init(&mut self, meta: Option<&LoroMap>) -> Result<(), String> {
        self.set_type("post").await?;
        match self.initialize_richtext_document() {
            Ok(_) => (),
            Err(e) => console_log!("Richtext document was not initialized: {}", e),
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
        let title = self
            .load_string_field_with_meta(meta, "title")
            .unwrap_or(self.get_title().unwrap_or_default());
        let url = self
            .load_string_field_with_meta(meta, "url")
            .unwrap_or(self.get_url().unwrap_or_default());

        self.set_id(&id).await?;
        self.set_name(&name).await?;
        self.set_version(version).await?;
        self.set_title(&title).await?;
        self.set_url(&url).await?;
        Ok(())
    }

    async fn build_from(builder: FileBuilder<Self>) -> Result<Self, String> {
        // Ensure we have a store
        let store = builder.store.ok_or("No file store provided")?;

        let mut post = Post { store };
        post.init(None)
            .await
            .map_err(|e| format!("Failed to initialize post: {}", e))?;
        Ok(post)
    }

    fn store(&self) -> &FileStore {
        &self.store
    }

    fn mut_store(&mut self) -> &mut FileStore {
        &mut self.store
    }

    fn get_type(&self) -> String {
        "post".to_string()
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

impl HasTitle for Post {}

impl HasUrl for Post {}

impl HasRichText for Post {}

impl Serialize for Post {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Post {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;
    use loro::{LoroDoc, LoroValue};
    use serde_json::json;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn test_schema() -> crate::ProseMirrorSchema {
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
        })
        .to_string();

        crate::ProseMirrorSchema::try_from(schema_json).expect("Failed to create test schema")
    }

    #[wasm_bindgen_test]
    async fn test_post_builder() {
        let pm_schema = test_schema();
        let post = Post::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_pm_schema(pm_schema.clone())
            .expect("Failed to add pm schema")
            .build()
            .await
            .expect("Failed to build post");

        assert_eq!(post.version().unwrap(), 0);
        assert_eq!(post.schema(), pm_schema);
        assert!(!post.id().unwrap().is_empty());
    }

    #[wasm_bindgen_test]
    async fn test_post_title() {
        let mut post = Post::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_pm_schema(test_schema())
            .expect("Failed to add pm schema")
            .build()
            .await
            .expect("Failed to build post");

        // Test setting and getting title
        post.set_title("Test Post")
            .await
            .expect("Failed to set title");
        let title = post.get_title().expect("Failed to get title");
        assert_eq!(title, "Test Post");
    }

    #[wasm_bindgen_test]
    async fn test_post_url() {
        let mut post = Post::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_pm_schema(test_schema())
            .expect("Failed to add pm schema")
            .build()
            .await
            .expect("Failed to build post");

        // Test setting and getting URL
        post.set_url("/test-post").await.expect("Failed to set URL");
        let url = post.get_url().expect("Failed to get URL");
        assert_eq!(url, "/test-post");
    }

    #[wasm_bindgen_test]
    async fn test_post_init() {
        let mut post = Post::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_id("test-id".to_string())
            .expect("Failed to set id")
            .with_pm_schema(test_schema())
            .expect("Failed to add pm schema")
            .build()
            .await
            .expect("Failed to build post");

        // Create a meta map with test data
        let meta = LoroMap::new();
        meta.insert("title", "Test Title");
        meta.insert("url", "/test-url");
        meta.insert("name", "test-name");
        meta.insert("version", "1");

        post.init(Some(&meta))
            .await
            .expect("Failed to initialize post");

        assert_eq!(post.get_title().unwrap(), "Test Title");
        assert_eq!(post.get_url().unwrap(), "/test-url");
        assert_eq!(post.name().unwrap(), "test-name");
        assert_eq!(post.get_type(), "post");
    }

    #[wasm_bindgen_test]
    async fn test_post_equality() {
        let schema = test_schema();
        let post1 = Post::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_id("test-id".to_string())
            .expect("Failed to set id")
            .with_pm_schema(schema.clone())
            .expect("Failed to add pm schema")
            .build()
            .await
            .expect("Failed to build post");

        let post2 = Post::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_id("test-id".to_string())
            .expect("Failed to set id")
            .with_pm_schema(schema.clone())
            .expect("Failed to add pm schema")
            .build()
            .await
            .expect("Failed to build post");

        let post3 = Post::builder()
            .with_doc(LoroDoc::new())
            .expect("Failed to set doc")
            .with_id("different-id".to_string())
            .expect("Failed to set id")
            .with_pm_schema(schema.clone())
            .expect("Failed to add pm schema")
            .build()
            .await
            .expect("Failed to build post");

        // Posts are equal if they have the same id, regardless of other fields
        assert_eq!(post1, post2);
        assert_ne!(post1, post3);
    }
}
