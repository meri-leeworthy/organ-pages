use js_sys::Uint8Array;
use loro::{Container, LoroDoc, LoroError, LoroMap, LoroValue, TreeID, ValueOrContainer};
use serde_json::{Map, Value};
use std::{convert::TryFrom, marker::PhantomData};
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use super::ProseMirrorSchema;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // log to JS console
}

// Helper macro for logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!("[File (WASM)] {}", format!($($t)*))))
}

/// Files are created in a few different ways:
/// - Totally new, from scratch
/// - Shell files contain metadata, but no content
/// - Loaded files contain metadata and content
///
/// Files can also be attached or unattached
///
/// Files can be in different states:
/// - Empty, unattached
/// - Initialised (not loaded), unattached
/// - Metadata loaded, unattached
/// - Metadata loaded, attached
/// - Content loaded, unattached
/// - Content loaded, attached
/// - Invalid
///

#[derive(Debug, Clone)]
pub enum FileStore {
    Cache(LoroMap),
    Full(LoroDoc),
}

impl FileStore {
    pub fn as_full(&self) -> Option<&LoroDoc> {
        match self {
            FileStore::Full(doc) => Some(doc),
            FileStore::Cache(_) => None,
        }
    }

    pub fn as_cache(&self) -> Option<&LoroMap> {
        match self {
            FileStore::Full(_) => None,
            FileStore::Cache(cache) => Some(cache),
        }
    }

    pub fn is_full(&self) -> bool {
        matches!(self, FileStore::Full(_))
    }

    pub fn is_cache(&self) -> bool {
        matches!(self, FileStore::Cache(_))
    }

    pub fn meta(&self) -> LoroMap {
        match self {
            FileStore::Full(doc) => doc.get_map(META_KEY),
            FileStore::Cache(cache) => cache.clone(), // does this work??
        }
    }
}

impl Default for FileStore {
    fn default() -> Self {
        FileStore::Cache(LoroMap::new())
    }
}

// pub struct Page {
// 	file_store: Option<FileStore>,
// }

pub const META_KEY: &str = "meta";
pub const ID_KEY: &str = "id";
pub const NAME_KEY: &str = "name";
pub const PM_SCHEMA_KEY: &str = "pm_schema";
pub const TITLE_KEY: &str = "title";
pub const VERSION_KEY: &str = "version";
pub const TYPE_KEY: &str = "type";
pub const URL_KEY: &str = "url";
pub const ALT_KEY: &str = "alt";
pub const MIME_TYPE_KEY: &str = "mime_type";

pub trait File {
    fn builder_for(collection_type: &str) -> FileBuilder<Self>
    where
        Self: Sized,
    {
        FileBuilder::new(collection_type)
    }

    fn builder() -> FileBuilder<Self>
    where
        Self: Sized;

    fn build_from(builder: FileBuilder<Self>) -> Result<Self, String>
    where
        Self: Sized,
    {
        builder.build()
    }

    fn store(&self) -> &FileStore;
    fn mut_store(&mut self) -> &mut FileStore;

    /// Initialize the file from a meta map
    /// The meta map is optional, and if not provided, the file will be initialized from the root doc
    fn init(&mut self, meta: Option<&LoroMap>) -> Result<(), String>;

    /// Used in init() to load a string field from the root doc or (external) meta
    /// The meta is first preference, then the current field value, then the root doc
    /// this should be the reverse of normal read behaviour
    fn load_string_field_with_meta(
        &mut self,
        meta_ext: Option<&LoroMap>,
        field: &str,
    ) -> Result<String, String> {
        if let Some(meta) = meta_ext {
            match meta.get(field) {
                Some(ValueOrContainer::Value(LoroValue::String(value))) => {
                    return Ok(value.to_string())
                }
                Some(ValueOrContainer::Value(LoroValue::I64(value))) => {
                    return Ok(value.to_string())
                }
                _ => (),
            };
        }

        match self.meta().get(field) {
            Some(ValueOrContainer::Value(LoroValue::String(value))) => return Ok(value.to_string()),
            Some(ValueOrContainer::Container(Container::Text(value))) => {
                return Ok(value.to_string())
            }
            Some(ValueOrContainer::Value(LoroValue::I64(value))) => return Ok(value.to_string()),
            _ => (),
        };

        Err(format!("Field {} not found in doc or meta", field))
    }

    fn get_i64_field_with_meta(
        &mut self,
        meta: Option<&LoroMap>,
        field: &str,
    ) -> Result<i64, String> {
        // console_log!("Getting i64 field with meta: {:?}", field);
        let value = self.load_string_field_with_meta(meta, field);
        // console_log!("Value: {:?}", value);
        match value {
            Ok(value) => Ok(value.parse::<i64>().unwrap()),
            Err(e) => Err(e),
        }
    }

    fn log(&self) -> Result<(), String> {
        let type_name = self.get_type();
        let version = self.version()?;
        let name = self.name()?;
        console_log!("[{}] {} (v{})", type_name, name, version);
        Ok(())
    }

    fn id(&self) -> Result<String, String> {
        match self.meta().get(ID_KEY) {
            Some(ValueOrContainer::Value(LoroValue::String(id))) => Ok(id.to_string()),
            _ => Err("ID not found".to_string()),
        }
    }
    fn version(&self) -> Result<i64, String> {
        match self.meta().get(VERSION_KEY) {
            Some(ValueOrContainer::Value(LoroValue::I64(version))) => Ok(version),
            Some(ValueOrContainer::Value(LoroValue::String(version))) => Ok(version
                .parse::<i64>()
                .map_err(|_| "Failed to parse version".to_string())?),
            _ => Err("Version not found".to_string()),
        }
    }

    fn get_type(&self) -> String;

    fn meta(&self) -> LoroMap {
        match self.store() {
            FileStore::Full(doc) => doc.get_map(META_KEY),
            FileStore::Cache(cache) => cache.clone(), // does this work??
        }
    }

    fn name(&self) -> Result<String, String> {
        match self.meta().get(NAME_KEY) {
            Some(ValueOrContainer::Value(LoroValue::String(name))) => Ok(name.clone().to_string()),
            _ => Err("Name not found".to_string()),
        }
    }

    fn set_name(&mut self, name: &str) -> Result<(), String> {
        match self.meta().insert(NAME_KEY, name.to_string()) {
            Ok(_) => Ok(()),
            Err(_) => Err("Node metadata not found".to_string()),
        }
    }

    fn set_field(&self, field: &str, value: &str) -> Result<(), String> {
        self.meta()
            .insert(field, value.to_string())
            .map_err(|e| e.to_string())
    }

    fn set_type(&mut self, doc_type: &str) -> Result<(), String> {
        self.set_field("type", doc_type).map_err(|e| e.to_string())
    }

    fn set_id(&mut self, id: &str) -> Result<(), String> {
        self.set_field("id", id).map_err(|e| e.to_string())
    }

    fn set_version(&mut self, version: i64) -> Result<(), String> {
        self.set_field("version", &version.to_string())
            .map_err(|e| e.to_string())
    }

    fn get_field(&self, field: &str) -> Result<Value, String> {
        match self.meta().get(field) {
            Some(ValueOrContainer::Value(LoroValue::String(value))) => {
                Ok(Value::String(value.to_string()))
            }
            Some(ValueOrContainer::Value(LoroValue::Bool(value))) => Ok(Value::Bool(value)),
            Some(ValueOrContainer::Value(LoroValue::Double(value))) => {
                Ok(Value::Number(serde_json::Number::from_f64(value).unwrap()))
            }
            Some(ValueOrContainer::Value(LoroValue::I64(value))) => {
                Ok(Value::Number(serde_json::Number::from(value)))
            }
            Some(ValueOrContainer::Container(Container::Text(text))) => {
                Ok(Value::String(text.to_string()))
            }
            Some(ValueOrContainer::Container(Container::List(list))) => {
                // Convert LoroList to Value array
                let mut array = Vec::new();
                for item in list.to_vec().iter() {
                    match item {
                        LoroValue::String(s) => {
                            array.push(Value::String(s.to_string()));
                        }
                        LoroValue::Bool(b) => {
                            array.push(Value::Bool(*b));
                        }
                        LoroValue::Double(n) => {
                            if let Some(num) = serde_json::Number::from_f64(*n) {
                                array.push(Value::Number(num));
                            }
                        }
                        LoroValue::I64(n) => {
                            array.push(Value::Number(serde_json::Number::from(*n)));
                        }
                        _ => return Err("Unsupported list item type".to_string()),
                    }
                }
                Ok(Value::Array(array))
            }
            None => Err(format!("Field not found: {}", field)),
            _ => Err(format!("Unsupported field type: {}", field)),
        }
    }

    fn to_json(&self) -> Result<Value, String>;
}

#[derive(Debug, Clone)]
pub struct FileBuilder<T: File> {
    pub id: Option<String>,
    pub store: Option<FileStore>,
    pub tree_id: Option<TreeID>,
    collection_type: String,
    phantom: PhantomData<T>,
}

impl<T: File> FileBuilder<T> {
    pub fn new(collection_type: &str) -> Self {
        // console_log!("Creating file builder: {:?}", collection_type);
        FileBuilder {
            id: None,
            store: None,
            tree_id: None,
            collection_type: collection_type.to_string(),
            phantom: PhantomData,
        }
    }

    pub fn collection_type(&self) -> String {
        self.collection_type.clone()
    }

    pub fn with_doc(mut self, doc: LoroDoc) -> Self {
        self.store = Some(FileStore::Full(doc));
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_version(self, version: i64) -> Result<Self, String> {
        let meta = self.meta()?;
        meta.insert(VERSION_KEY, version)
            .map_err(|e| format!("Failed to insert version: {}", e))?;
        Ok(self)
    }

    pub fn version(&self) -> Option<i64> {
        self.store
            .as_ref()
            .and_then(|store| match store.meta().get(VERSION_KEY) {
                Some(ValueOrContainer::Value(LoroValue::I64(version))) => Some(version),
                _ => None,
            })
    }

    pub fn meta(&self) -> Result<LoroMap, String> {
        match &self.store {
            Some(FileStore::Full(doc)) => Ok(doc.get_map(META_KEY)),
            Some(FileStore::Cache(cache)) => Ok(cache.clone()),
            None => Err("Store not found".to_string()),
        }
    }

    pub fn with_pm_schema(self, pm_schema: ProseMirrorSchema) -> Result<Self, String> {
        let meta = self.meta()?;
        meta.insert(PM_SCHEMA_KEY, pm_schema.to_string())
            .map_err(|e| format!("Failed to insert pm_schema: {}", e))?;
        Ok(self)
    }

    pub fn pm_schema(&self) -> Option<ProseMirrorSchema> {
        self.store
            .as_ref()
            .and_then(|store| match store.meta().get(PM_SCHEMA_KEY) {
                Some(ValueOrContainer::Value(LoroValue::String(pm_schema))) => {
                    Some(ProseMirrorSchema::try_from(pm_schema.to_string()).unwrap())
                }
                _ => None,
            })
    }

    pub fn with_name(self, name: String) -> Result<Self, String> {
        let meta = self.meta()?;
        meta.insert(NAME_KEY, name)
            .map_err(|e| format!("Failed to insert name: {}", e))?;
        Ok(self)
    }

    pub fn with_meta(mut self, meta: LoroMap) -> Result<Self, String> {
        // Create a new meta map if we don't have one yet
        if self.store.is_none() {
            self.store = Some(FileStore::Cache(LoroMap::new()));
        }

        let builder_meta = self.meta()?;

        // get id from meta
        if let Some(ValueOrContainer::Value(LoroValue::String(id))) = meta.get(ID_KEY) {
            builder_meta
                .insert(ID_KEY, LoroValue::String(id.clone()))
                .map_err(|e| format!("Failed to insert id: {}", e))?;
            self.id = Some(id.to_string());
        } else {
            return Err("File ID not found".to_string());
        }

        // get name from meta
        if let Some(ValueOrContainer::Value(LoroValue::String(name))) = meta.get(NAME_KEY) {
            builder_meta
                .insert(NAME_KEY, LoroValue::String(name.clone()))
                .map_err(|e| format!("Failed to insert name: {}", e))?;
        } else {
            return Err("File name not found".to_string());
        }

        // get version from meta
        let version = match meta.get(VERSION_KEY) {
            Some(ValueOrContainer::Value(LoroValue::I64(version))) => {
                builder_meta
                    .insert(VERSION_KEY, LoroValue::I64(version))
                    .map_err(|e| format!("Failed to insert version: {}", e))?;
                version
            }
            Some(ValueOrContainer::Value(LoroValue::String(version))) => {
                let parsed_version = version
                    .parse::<i64>()
                    .map_err(|_| "Failed to parse version".to_string())?;
                builder_meta
                    .insert(VERSION_KEY, LoroValue::I64(parsed_version))
                    .map_err(|e| format!("Failed to insert version: {}", e))?;
                parsed_version
            }
            _ => return Err("File version not found".to_string()),
        };

        // get pm_schema from meta
        if let Some(ValueOrContainer::Value(LoroValue::String(pm_schema))) = meta.get(PM_SCHEMA_KEY)
        {
            builder_meta
                .insert(PM_SCHEMA_KEY, LoroValue::String(pm_schema.clone()))
                .map_err(|_| "Failed to insert pm_schema".to_string())?;

            // Try to convert pm_schema string to ProseMirrorSchema
            if let Ok(schema) = ProseMirrorSchema::try_from(pm_schema.to_string()) {
                // Nothing to do here, we've already stored the string representation
            }
        }

        // No need to call with_version since we've already set it directly
        Ok(self)
    }

    pub fn build(self) -> Result<T, String> {
        // Create a default store if one doesn't exist
        let store = match self.store {
            Some(store) => store,
            None => {
                // Create a new cache if we don't have a store
                let mut cache = LoroMap::new();

                // Generate a new UUID if no ID was provided
                let id = match &self.id {
                    Some(id) => id.clone(),
                    None => Uuid::new_v4().to_string(),
                };

                // Set default metadata
                cache
                    .insert(ID_KEY, id)
                    .map_err(|e| format!("Failed to insert id: {}", e))?;

                // Default version is 0
                cache
                    .insert(VERSION_KEY, 0i64)
                    .map_err(|e| format!("Failed to insert version: {}", e))?;

                FileStore::Cache(cache)
            }
        };

        // Create the file
        let builder = FileBuilder {
            id: self.id,
            store: Some(store),
            tree_id: self.tree_id,
            collection_type: self.collection_type,
            phantom: self.phantom,
        };

        T::build_from(builder)
    }

    /// Asynchronous build method that can load file data from IndexedDB if needed
    pub async fn build_async(self) -> Result<T, String> {
        // If we have both an ID and no store, try to load from IndexedDB
        if let (Some(id), None) = (&self.id, &self.store) {
            // Try to load file data from IndexedDB
            match self.load_from_indexeddb(id).await {
                Ok(loaded_store) => {
                    // Build with the loaded store
                    let builder = FileBuilder {
                        id: self.id,
                        store: Some(loaded_store),
                        tree_id: self.tree_id,
                        collection_type: self.collection_type,
                        phantom: self.phantom,
                    };
                    return T::build_from(builder);
                }
                Err(e) => {
                    console_log!("Failed to load file from IndexedDB: {}", e);
                    // Continue with building a new file
                }
            }
        }

        // Fall back to the synchronous build method
        self.build()
    }

    /// Internal method to load file data from IndexedDB
    async fn load_from_indexeddb(&self, id: &str) -> Result<FileStore, String> {
        // Configure the database parameters
        let db_name = "organ_db".to_string();
        let store_name = "files".to_string();

        // Load the file data from IndexedDB
        let result = crate::load_data(db_name, store_name, id.to_string())
            .await
            .map_err(|e| format!("IndexedDB error: {:?}", e))?;

        // Convert the JS string to a byte array
        let result_str = result.as_string().ok_or("Invalid data format")?;
        let bytes = result_str.as_bytes();

        // Create a new LoroDoc - it appears LoroDoc::new() returns a LoroDoc directly
        let doc = LoroDoc::new();

        // Import the snapshot into the document
        doc.import(bytes)
            .map_err(|e| format!("Failed to import data: {}", e))?;

        Ok(FileStore::Full(doc))
    }

    /// Save a file to IndexedDB
    pub async fn save_to_indexeddb(&self) -> Result<(), String> {
        // We need both a store and an ID to save
        if let (Some(id), Some(FileStore::Full(doc))) = (&self.id, &self.store) {
            // Export the LoroDoc to bytes
            let export_data = doc
                .export(loro::ExportMode::all_updates())
                .map_err(|e| format!("Failed to export LoroDoc: {}", e))?;

            // Convert to Uint8Array for JS
            let uint8_array = Uint8Array::from(&export_data[..]);

            // Configure the database parameters
            let db_name = "organ_db".to_string();
            let store_name = "files".to_string();

            // Save to IndexedDB
            crate::save_data(db_name, store_name, id.to_string(), uint8_array)
                .await
                .map_err(|e| format!("IndexedDB error: {:?}", e))?;

            Ok(())
        } else {
            Err("Cannot save: missing ID or full document".to_string())
        }
    }
}

pub trait HasTitle: File {
    fn set_title(&self, title: &str) -> Result<(), String> {
        self.meta()
            .insert(TITLE_KEY, title.to_string())
            .map_err(|e| e.to_string())
    }

    fn get_title(&self) -> Result<String, String> {
        let title = self.meta().get(TITLE_KEY);
        match title {
            Some(ValueOrContainer::Value(LoroValue::String(title))) => {
                Ok(title.clone().to_string())
            }
            _ => Err("Title not found".to_string()),
        }
    }
}

pub trait HasUrl: File {
    fn get_url(&self) -> Result<String, String> {
        let url = self.meta().get(URL_KEY);
        match url {
            Some(ValueOrContainer::Value(LoroValue::String(url))) => Ok(url.clone().to_string()),
            _ => Err("URL not found".to_string()),
        }
    }
    fn set_url(&self, url: &str) -> Result<(), String> {
        self.meta()
            .insert(URL_KEY, url.to_string())
            .map_err(|e| e.to_string())
    }
}

pub trait HasAlt: File {
    fn get_alt(&self) -> Result<String, String> {
        let alt = self.meta().get(ALT_KEY);
        match alt {
            Some(ValueOrContainer::Value(LoroValue::String(alt))) => Ok(alt.clone().to_string()),
            _ => Err("Alt text not found".to_string()),
        }
    }
    fn set_alt(&self, alt: &str) -> Result<(), String> {
        self.meta()
            .insert(ALT_KEY, alt.to_string())
            .map_err(|e| e.to_string())
    }
}

pub trait HasMimeType: File {
    fn get_mime_type(&self) -> Result<String, String> {
        let mime_type = self.meta().get(MIME_TYPE_KEY);
        match mime_type {
            Some(ValueOrContainer::Value(LoroValue::String(mime_type))) => {
                Ok(mime_type.clone().to_string())
            }
            _ => Err("Mime type not found".to_string()),
        }
    }
    fn set_mime_type(&self, mime_type: &str) -> Result<(), String> {
        self.meta()
            .insert(MIME_TYPE_KEY, mime_type.to_string())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // A simple test implementation of the File trait
    struct TestFile {
        store: FileStore,
    }

    impl File for TestFile {
        fn builder() -> FileBuilder<Self> {
            FileBuilder::new("test")
        }

        fn init(&mut self, meta: Option<&LoroMap>) -> Result<(), String> {
            self.set_type("test")?;

            if let Some(meta) = meta {
                if let Some(ValueOrContainer::Value(LoroValue::String(id))) = meta.get("id") {
                    self.set_id(&id)?;
                }
                if let Some(ValueOrContainer::Value(LoroValue::String(name))) = meta.get("name") {
                    self.set_name(&name)?;
                }
                if let Some(ValueOrContainer::Value(LoroValue::String(version))) =
                    meta.get("version")
                {
                    self.set_version(version.parse().unwrap_or(0))?;
                }
            }
            Ok(())
        }

        fn build_from(builder: FileBuilder<Self>) -> Result<Self, String> {
            // Ensure we have a store
            let store = builder.store.ok_or("No file store provided")?;

            let mut file = TestFile { store };
            file.init(None)
                .map_err(|e| format!("Failed to initialize test file: {}", e))?;
            Ok(file)
        }

        fn store(&self) -> &FileStore {
            &self.store
        }

        fn mut_store(&mut self) -> &mut FileStore {
            &mut self.store
        }

        fn get_type(&self) -> String {
            "test".to_string()
        }

        fn to_json(&self) -> Result<Value, String> {
            Ok(Value::Object(serde_json::Map::new()))
        }
    }

    // Implement helper traits for testing
    impl HasTitle for TestFile {}
    impl HasUrl for TestFile {}
    impl HasAlt for TestFile {}
    impl HasMimeType for TestFile {}

    #[wasm_bindgen_test]
    fn test_file_builder() {
        let file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");
        assert_eq!(file.version().unwrap(), 0);
        assert!(!file.id().unwrap().is_empty());
        assert_eq!(file.get_type(), "test");
    }

    #[wasm_bindgen_test]
    fn test_file_builder_with_meta() {
        let mut meta = LoroMap::new();
        meta.insert("id", "test-id");
        meta.insert("name", "test-name");
        meta.insert("version", 2);

        let file = TestFile::builder_for("test")
            .with_meta(meta)
            .expect("Failed to apply metadata")
            .build()
            .expect("Failed to build test file");

        assert_eq!(file.id().unwrap(), "test-id");
        assert_eq!(file.name().unwrap(), "test-name");
        assert_eq!(file.version().unwrap(), 2);
    }

    #[wasm_bindgen_test]
    fn test_get_string_field_with_meta() {
        let mut file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");
        let mut meta = LoroMap::new();
        meta.insert("test_field", "test_value");

        let result = file.load_string_field_with_meta(Some(&meta), "test_field");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_value");

        // Test field not found
        let result = file.load_string_field_with_meta(Some(&meta), "nonexistent_field");
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_get_u32_field_with_meta() {
        let mut file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");
        let mut meta = LoroMap::new();
        meta.insert("test_number", "42");

        let result = file.get_i64_field_with_meta(Some(&meta), "test_number");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Test field not found
        let result = file.get_i64_field_with_meta(Some(&meta), "nonexistent_field");
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_has_title_trait() {
        let file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");

        // Test setting and getting title
        file.set_title("Test Title").expect("Failed to set title");
        let title = file.get_title().expect("Failed to get title");
        assert_eq!(title, "Test Title");

        // Test getting nonexistent title
        let file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");
        assert!(file.get_title().is_err());
    }

    #[wasm_bindgen_test]
    fn test_has_url_trait() {
        let file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");

        // Test setting and getting URL
        file.set_url("/test-url").expect("Failed to set URL");
        let url = file.get_url().expect("Failed to get URL");
        assert_eq!(url, "/test-url");

        // Test getting nonexistent URL
        let file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");
        assert!(file.get_url().is_err());
    }

    #[wasm_bindgen_test]
    fn test_has_alt_trait() {
        let file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");

        // Test setting and getting alt text
        file.set_alt("Test Alt").expect("Failed to set alt text");
        let alt = file.get_alt().expect("Failed to get alt text");
        assert_eq!(alt, "Test Alt");

        // Test getting nonexistent alt text
        let file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");
        assert!(file.get_alt().is_err());
    }

    #[wasm_bindgen_test]
    fn test_has_mime_type_trait() {
        let file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");

        // Test setting and getting mime type
        file.set_mime_type("text/plain")
            .expect("Failed to set mime type");
        let mime_type = file.get_mime_type().expect("Failed to get mime type");
        assert_eq!(mime_type, "text/plain");

        // Test getting nonexistent mime type
        let file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");
        assert!(file.get_mime_type().is_err());
    }

    #[wasm_bindgen_test]
    fn test_meta_operations() {
        let mut file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");

        // Test setting and getting name
        file.set_name("test-name").expect("Failed to set name");
        assert_eq!(file.name().unwrap(), "test-name");

        // Test setting and getting custom field
        file.set_field("custom_field", "custom_value")
            .expect("Failed to set custom field");
        let value = file
            .get_field("custom_field")
            .expect("Failed to get custom field");
        assert_eq!(value.as_str().unwrap(), "custom_value");

        // Test getting nonexistent field
        assert!(file.get_field("nonexistent_field").is_err());
    }

    #[wasm_bindgen_test]
    fn test_file_meta_operations_edge_cases() {
        let mut file = TestFile::builder_for("test")
            .build()
            .expect("Failed to build test file");

        // Test empty values
        file.set_name("").expect("Failed to set empty name");
        assert_eq!(file.name().unwrap(), "");

        // Test special characters
        file.set_name("test/name:with@special#chars")
            .expect("Failed to set name with special chars");
        assert_eq!(file.name().unwrap(), "test/name:with@special#chars");

        // Test very long values
        let long_name = "a".repeat(1000);
        file.set_name(&long_name).expect("Failed to set long name");
        assert_eq!(file.name().unwrap(), long_name);
    }

    #[wasm_bindgen_test]
    fn test_file_builder_with_doc() {
        let doc = LoroDoc::new();
        doc.get_map(META_KEY).insert("id", "doc-id").unwrap();
        doc.get_map(META_KEY).insert("name", "doc-name").unwrap();
        doc.get_map(META_KEY).insert("version", 3i64).unwrap();

        let file = TestFile::builder()
            .with_doc(doc)
            .build()
            .expect("Failed to build test file with doc");

        assert_eq!(file.id().unwrap(), "doc-id");
        assert_eq!(file.name().unwrap(), "doc-name");
        assert_eq!(file.version().unwrap(), 3);
    }

    #[wasm_bindgen_test]
    fn test_file_builder_with_version() {
        let file = TestFile::builder()
            .with_version(42)
            .expect("Failed to set version")
            .build()
            .expect("Failed to build test file with version");

        assert_eq!(file.version().unwrap(), 42);
    }

    #[wasm_bindgen_test]
    fn test_file_builder_with_name() {
        let file = TestFile::builder()
            .with_name("custom-name".to_string())
            .expect("Failed to set name")
            .build()
            .expect("Failed to build test file with name");

        assert_eq!(file.name().unwrap(), "custom-name");
    }

    #[wasm_bindgen_test]
    fn test_file_builder_with_pm_schema() {
        let schema = ProseMirrorSchema::try_from("basic".to_string()).expect("Invalid schema");

        let file = TestFile::builder()
            .with_pm_schema(schema)
            .expect("Failed to set schema")
            .build()
            .expect("Failed to build test file with schema");

        assert_eq!(
            file.get_field("pm_schema").unwrap().as_str().unwrap(),
            "basic"
        );
    }

    #[wasm_bindgen_test]
    fn test_get_field_types() {
        let file = TestFile::builder()
            .build()
            .expect("Failed to build test file");

        // Test boolean field
        file.meta()
            .insert("bool_field", true)
            .expect("Failed to insert bool");
        let bool_val = file
            .get_field("bool_field")
            .expect("Failed to get bool field");
        assert!(bool_val.is_boolean());
        assert_eq!(bool_val.as_bool().unwrap(), true);

        // Test number field (i64)
        file.meta()
            .insert("int_field", 123i64)
            .expect("Failed to insert int");
        let int_val = file
            .get_field("int_field")
            .expect("Failed to get int field");
        assert!(int_val.is_number());
        assert_eq!(int_val.as_i64().unwrap(), 123);

        // Test floating point field
        file.meta()
            .insert("float_field", 3.14)
            .expect("Failed to insert float");
        let float_val = file
            .get_field("float_field")
            .expect("Failed to get float field");
        assert!(float_val.is_number());
        assert!((float_val.as_f64().unwrap() - 3.14).abs() < f64::EPSILON);
    }

    #[wasm_bindgen_test]
    fn test_to_json_with_chainable() {
        let mut file = TestFile::builder()
            .build()
            .expect("Failed to build test file");
        file.set_name("test-name").expect("Failed to set name");

        // Create a JSON object using Chainable methods
        let mut json_map = Map::new();
        file.add_field(&mut json_map, "custom_key", "custom_value")
            .expect("Failed to add field")
            .add_field_or_default(&mut json_map, "name", file.name())
            .expect("Failed to add name field")
            .add_field_or_default(&mut json_map, "nonexistent", Err::<String, _>("Not found"))
            .expect("Failed to add default field");

        assert_eq!(
            json_map.get("custom_key").unwrap().as_str().unwrap(),
            "custom_value"
        );
        assert_eq!(json_map.get("name").unwrap().as_str().unwrap(), "test-name");
        assert_eq!(json_map.get("nonexistent").unwrap().as_str().unwrap(), "");
    }

    #[wasm_bindgen_test]
    fn test_error_handling() {
        // Test handling of invalid field access
        let file = TestFile::builder()
            .build()
            .expect("Failed to build test file");

        // Test getting non-existent field
        let result = file.get_field("nonexistent_field");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        // Test invalid ID
        let result = file.id();
        if let Ok(id) = result {
            assert!(!id.is_empty(), "ID should not be empty");
        }

        // Test initialization with invalid metadata
        let mut invalid_meta = LoroMap::new();
        invalid_meta.insert("invalid_key", "invalid_value");

        // This should not fail because our TestFile implementation handles missing fields gracefully
        let mut file = TestFile::builder()
            .build()
            .expect("Failed to build test file");
        assert!(file.init(Some(&invalid_meta)).is_ok());
    }

    // This is a mock implementation that would need to be adapted
    // for real async testing with wasm-bindgen-test
    #[wasm_bindgen_test]
    fn test_file_operations() {
        // Test basic file operations in sequence
        let mut file = TestFile::builder()
            .with_name("initial-name".to_string())
            .expect("Failed to set initial name")
            .build()
            .expect("Failed to build test file");

        // Verify initial state
        assert_eq!(file.name().unwrap(), "initial-name");

        // Update file
        file.set_name("updated-name")
            .expect("Failed to update name");
        assert_eq!(file.name().unwrap(), "updated-name");

        // Set multiple fields
        file.set_field("field1", "value1")
            .expect("Failed to set field1");
        file.set_field("field2", "value2")
            .expect("Failed to set field2");

        // Verify fields
        let val1 = file.get_field("field1").expect("Failed to get field1");
        let val2 = file.get_field("field2").expect("Failed to get field2");

        assert_eq!(val1.as_str().unwrap(), "value1");
        assert_eq!(val2.as_str().unwrap(), "value2");
    }
}

pub trait Chainable {
    /// Helper functions for to_json
    fn add_field(
        &self,
        result: &mut Map<String, Value>,
        key: &str,
        value: &str,
    ) -> Result<&Self, String>
    where
        Self: Sized,
    {
        result.insert(key.to_string(), Value::String(value.to_string()));
        Ok(self)
    }

    /// Add a field to the JSON object, or a default value if the field is not found
    fn add_field_or_default<T>(
        &self,
        result: &mut Map<String, Value>,
        key: &str,
        value: Result<String, T>,
    ) -> Result<&Self, String>
    where
        Self: Sized,
    {
        if let Ok(value) = value {
            self.add_field(result, key, &value)?;
        } else {
            self.add_field(result, key, "")?;
        }
        Ok(self)
    }
}

impl<T: File> Chainable for T {}
