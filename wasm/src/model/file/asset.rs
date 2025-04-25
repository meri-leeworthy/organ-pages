use crate::model::file::{Chainable, File, FileBuilder, FileStore, HasAlt, HasMimeType, HasUrl};
use loro::LoroMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Asset LoroDoc contains:
/// - meta
///   - type
///   - id
///   - name
///   - version
///   - mime_type
///   - url
///   - alt
#[derive(Debug, Clone, Default)]
pub struct Asset {
    pub store: FileStore,
}

impl File for Asset {
    fn builder() -> FileBuilder<Self> {
        FileBuilder::new("asset")
    }

    async fn init(&mut self, meta: Option<&LoroMap>) -> Result<(), String> {
        self.set_type("asset").await?;

        let id = self
            .load_string_field_with_meta(meta, "id")
            .unwrap_or_default();
        let name = self
            .load_string_field_with_meta(meta, "name")
            .unwrap_or_default();
        let version = self
            .get_i64_field_with_meta(meta, "version")
            .unwrap_or_default();
        let mime_type = self
            .load_string_field_with_meta(meta, "mime_type")
            .unwrap_or(self.get_mime_type().unwrap_or_default());
        let url = self
            .load_string_field_with_meta(meta, "url")
            .unwrap_or(self.get_url().unwrap_or_default());
        let alt = self
            .load_string_field_with_meta(meta, "alt")
            .unwrap_or(self.get_alt().unwrap_or_default());

        self.set_id(&id).await?;
        self.set_name(&name).await?;
        self.set_version(version).await?;
        self.set_mime_type(&mime_type).await?;
        self.set_url(&url).await?;
        self.set_alt(&alt).await?;
        Ok(())
    }

    async fn build_from(builder: FileBuilder<Self>) -> Result<Self, String> {
        // Ensure we have a store
        let store = builder.store.ok_or("No file store provided")?;

        let mut asset = Asset { store };
        asset
            .init(None)
            .await
            .map_err(|e| format!("Failed to initialize asset: {}", e))?;
        Ok(asset)
    }

    fn store(&self) -> &FileStore {
        &self.store
    }

    fn mut_store(&mut self) -> &mut FileStore {
        &mut self.store
    }

    fn get_type(&self) -> String {
        "asset".to_string()
    }

    fn to_json(&self) -> Result<Value, String> {
        let mut result = Map::new();
        self.add_field(&mut result, "id", &self.id()?.to_string())?;
        self.add_field(&mut result, "collection_type", &self.get_type())?;
        self.add_field_or_default(&mut result, "name", self.name())?;
        self.add_field_or_default(&mut result, "url", self.get_url())?;
        self.add_field_or_default(&mut result, "mime_type", self.get_mime_type())?;
        self.add_field_or_default(&mut result, "alt", self.get_alt())?;
        Ok(Value::Object(result))
    }
}

impl HasMimeType for Asset {}

impl HasUrl for Asset {}

impl HasAlt for Asset {}

impl Serialize for Asset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Asset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl PartialEq for Asset {
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
    async fn test_asset_builder() {
        let asset = Asset::builder()
            .with_meta(LoroMap::new())
            .expect("Failed to set meta")
            .build()
            .await
            .expect("Failed to build asset");

        assert_eq!(asset.version().unwrap(), 0);
        assert!(!asset.id().unwrap().is_empty());
    }

    #[wasm_bindgen_test]
    async fn test_asset_url() {
        let mut asset = Asset::builder()
            .with_meta(LoroMap::new())
            .expect("Failed to set meta")
            .build()
            .await
            .expect("Failed to build asset");

        // Test setting and getting URL
        asset
            .set_url("/test-asset.jpg")
            .await
            .expect("Failed to set URL");
        let url = asset.get_url().expect("Failed to get URL");
        assert_eq!(url, "/test-asset.jpg");
    }

    #[wasm_bindgen_test]
    async fn test_asset_mime_type() {
        let mut asset = Asset::builder()
            .with_meta(LoroMap::new())
            .expect("Failed to set meta")
            .build()
            .await
            .expect("Failed to build asset");

        // Test setting and getting mime type
        asset
            .set_mime_type("image/jpeg")
            .await
            .expect("Failed to set mime type");
        let mime_type = asset.get_mime_type().expect("Failed to get mime type");
        assert_eq!(mime_type, "image/jpeg");
    }

    #[wasm_bindgen_test]
    async fn test_asset_alt() {
        let asset = Asset::builder()
            .with_meta(LoroMap::new())
            .expect("Failed to set meta")
            .build()
            .await
            .expect("Failed to build asset");

        // Test setting and getting alt text
        asset
            .set_alt("Test image")
            .await
            .expect("Failed to set alt text");
        let alt = asset.get_alt().expect("Failed to get alt text");
        assert_eq!(alt, "Test image");
    }

    #[wasm_bindgen_test]
    async fn test_asset_init() {
        let mut asset = Asset::builder()
            .with_meta(LoroMap::new())
            .expect("Failed to set meta")
            .with_id("test-id".to_string())
            .expect("Failed to set id")
            .build()
            .await
            .expect("Failed to build asset");

        // Create a meta map with test data
        let meta = LoroMap::new();
        meta.insert("name", "test-name");
        meta.insert("version", "1");
        meta.insert("mime_type", "image/jpeg");
        meta.insert("url", "/test-asset.jpg");
        meta.insert("alt", "Test image");

        asset
            .init(Some(&meta))
            .await
            .expect("Failed to initialize asset");

        assert_eq!(asset.name().unwrap(), "test-name");
        assert_eq!(asset.get_type(), "asset");
        assert_eq!(asset.get_mime_type().unwrap(), "image/jpeg");
        assert_eq!(asset.get_url().unwrap(), "/test-asset.jpg");
        assert_eq!(asset.get_alt().unwrap(), "Test image");
    }

    #[wasm_bindgen_test]
    async fn test_asset_equality() {
        let asset1 = Asset::builder()
            .with_meta(LoroMap::new())
            .expect("Failed to set meta")
            .with_id("test-id".to_string())
            .expect("Failed to set id")
            .build()
            .await
            .expect("Failed to build asset");

        let asset2 = Asset::builder()
            .with_meta(LoroMap::new())
            .expect("Failed to set meta")
            .with_id("test-id".to_string())
            .expect("Failed to set id")
            .build()
            .await
            .expect("Failed to build asset");

        let asset3 = Asset::builder()
            .with_meta(LoroMap::new())
            .expect("Failed to set meta")
            .with_id("different-id".to_string())
            .expect("Failed to set id")
            .build()
            .await
            .expect("Failed to build asset");

        // Assets are equal if they have the same id, regardless of other fields
        assert_eq!(asset1, asset2);
        assert_ne!(asset1, asset3);
    }

    #[wasm_bindgen_test]
    async fn test_asset_mime_type_validation() {
        let asset = Asset::builder()
            .build()
            .await
            .expect("Failed to build asset");

        // Test valid mime types
        asset
            .set_mime_type("image/jpeg")
            .await
            .expect("Failed to set mime type");
        assert_eq!(asset.get_mime_type().unwrap(), "image/jpeg");

        asset
            .set_mime_type("application/json")
            .await
            .expect("Failed to set mime type");
        assert_eq!(asset.get_mime_type().unwrap(), "application/json");

        // Test empty mime type
        asset
            .set_mime_type("")
            .await
            .expect("Failed to set empty mime type");
        assert_eq!(asset.get_mime_type().unwrap(), "");

        // Test invalid mime type format
        asset
            .set_mime_type("invalid-mime-type")
            .await
            .expect("Failed to set invalid mime type");
        assert_eq!(asset.get_mime_type().unwrap(), "invalid-mime-type");
    }

    #[wasm_bindgen_test]
    async fn test_asset_url_edge_cases() {
        let asset = Asset::builder()
            .build()
            .await
            .expect("Failed to build asset");

        // Test empty URL
        asset.set_url("").await.expect("Failed to set empty URL");
        assert_eq!(asset.get_url().unwrap(), "");

        // Test URL with special characters
        asset
            .set_url("/path/with spaces and #special @chars")
            .await
            .expect("Failed to set URL with special chars");
        assert_eq!(
            asset.get_url().unwrap(),
            "/path/with spaces and #special @chars"
        );

        // Test very long URL
        let long_url = format!("/{}", "a".repeat(1000));
        asset
            .set_url(&long_url)
            .await
            .expect("Failed to set long URL");
        assert_eq!(asset.get_url().unwrap(), long_url);
    }

    #[wasm_bindgen_test]
    async fn test_asset_alt_edge_cases() {
        let asset = Asset::builder()
            .build()
            .await
            .expect("Failed to build asset");

        // Test empty alt text
        asset
            .set_alt("")
            .await
            .expect("Failed to set empty alt text");
        assert_eq!(asset.get_alt().unwrap(), "");

        // Test alt text with special characters
        asset
            .set_alt("Alt text with 特殊文字 and symbols @#$%")
            .await
            .expect("Failed to set alt text with special chars");
        assert_eq!(
            asset.get_alt().unwrap(),
            "Alt text with 特殊文字 and symbols @#$%"
        );

        // Test very long alt text
        let long_alt = "a".repeat(1000);
        asset
            .set_alt(&long_alt)
            .await
            .expect("Failed to set long alt text");
        assert_eq!(asset.get_alt().unwrap(), long_alt);
    }

    // #[wasm_bindgen_test]
    // fn test_asset_to_json() {
    //     let mut asset = Asset::builder(FileStore::Cache(LoroMap::new())).build().expect("Failed to build asset");

    //     asset
    //         .set_mime_type("image/png")
    //         .expect("Failed to set mime type");
    //     asset.set_url("/test.png").expect("Failed to set URL");
    //     asset.set_alt("Test image").expect("Failed to set alt text");

    //     let json = asset.to_json().expect("Failed to convert to JSON");
    //     let obj = json.as_object().expect("JSON should be an object");

    //     // Verify the JSON structure matches expectations
    //     assert!(obj.contains_key("id"));
    //     assert!(obj.contains_key("mime_type"));
    //     assert!(obj.contains_key("url"));
    //     assert!(obj.contains_key("alt"));
    // }
}
