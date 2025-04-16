use crate::types::{FieldDefinition, FieldType};
use crate::{ApplyMap, ID_KEY, VERSION_KEY};
use loro::{
    Container, ContainerTrait, LoroDoc, LoroMap, LoroTree, LoroValue, TreeID, ValueOrContainer,
};

use serde_json::Value;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::marker::PhantomData;

use wasm_bindgen::prelude::*;

// Add wasm_bindgen_test imports
#[cfg(test)]
use wasm_bindgen_test::*;

#[cfg(test)]
wasm_bindgen_test_configure!(run_in_browser);

// Use the full path to avoid duplicate imports and missing FileState
use crate::model::file::{File, FileBuilder, FileStore, HasTitle, HasUrl};
use crate::{Model, Project, ProjectType};

pub const COLLECTIONS_KEY: &str = "collections";
pub const FIELDS_KEY: &str = "fields";
pub const FILES_KEY: &str = "files";
pub const TYPE_KEY: &str = "type";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // log to JS console
}

// Helper macro for logging
macro_rules! console_log {
  ($($t:tt)*) => (log(&format!("[Collection (WASM)] {}", format!($($t)*))))
}

/// Builder for creating a Collection with immutable fields
#[derive(Debug, Clone)]
pub struct CollectionBuilder<TFile: File> {
    name: String,
    map: LoroMap,
    fields: Vec<FieldDefinition>,
    file_type: PhantomData<TFile>,
}

impl<TFile: File> CollectionBuilder<TFile> {
    pub fn new(name: String) -> Result<Self, String> {
        let map = LoroMap::new();
        let fields_map = LoroMap::new();
        let files_tree = LoroTree::new();
        map.insert_container(FIELDS_KEY, fields_map)
            .map_err(|e| format!("Failed to insert fields: {}", e))?;
        map.insert_container(FILES_KEY, files_tree)
            .map_err(|e| format!("Failed to insert files: {}", e))?;

        Ok(CollectionBuilder {
            name,
            map,
            fields: Vec::new(),
            file_type: PhantomData,
        })
    }

    pub fn add_field(mut self, name: &str, field_type: FieldType, required: bool) -> Self {
        self.fields.push(FieldDefinition {
            name: name.to_string(),
            field_type,
            required,
        });
        self
    }

    pub fn with_map(mut self, map: LoroMap) -> Self {
        self.map = map;
        self
    }

    /// First check if the collection exists in the document
    /// If it does, then we can just update the map
    /// If it doesn't, then we can create a new collection
    pub fn build(self, doc: &LoroDoc) -> Result<Collection<TFile>, String> {
        let collections = doc.get_map(COLLECTIONS_KEY);

        if let None = collections.get(&self.name) {
            console_log!("(build) Collection not found: {:#?}", self.name);
            console_log!("(build) Inserting map with id: {:#?}", self.map.id());
            collections
                .insert_container(&self.name.clone(), self.map)
                .map_err(|e| format!("Failed to insert collection: {}", e))?;
            console_log!(
                "(build) Collections after inserting {}: {:#?}",
                self.name,
                collections.get_deep_value()
            );
        }

        let collection = match collections.get(&self.name) {
            Some(ValueOrContainer::Container(Container::Map(map))) => map,
            _ => return Err("Collection not found".to_string()),
        };

        // Initialise fields map if it doesn't exist
        let fields_map = match collection.get(FIELDS_KEY) {
            Some(ValueOrContainer::Container(Container::Map(fields_map))) => fields_map,
            _ => {
                console_log!(
                    "(build) Fields not initialized, initializing: {:#?}",
                    self.name
                );
                collection
                    .insert_container(FIELDS_KEY, LoroMap::new())
                    .map_err(|e| format!("Failed to insert fields: {}", e))?;
                match collection.get(FIELDS_KEY) {
                    Some(ValueOrContainer::Container(Container::Map(fields_map))) => fields_map,
                    _ => return Err("Fields not initialized".to_string()),
                }
            }
        };

        for field in self.fields {
            let field_name = field.name.clone();

            if let Some(ValueOrContainer::Container(Container::Map(_field_map))) =
                fields_map.get(field_name.as_str())
            {
                console_log!("(build) Field already exists: {:#?}", field_name);
            } else {
                console_log!("(build) Field doesn't exist, inserting: {:#?}", field_name);
                let field_value: Value = field.into();

                // Convert to LoroValue
                if let Some(field_obj) = field_value.as_object() {
                    let field_map = LoroMap::new();
                    field_map
                        .apply_map(field_obj)
                        .map_err(|e| format!("Failed to insert field: {}", e))?;
                    fields_map
                        .insert_container(&field_name, field_map)
                        .map_err(|e| format!("Failed to insert field: {}", e))?;
                }
            }
        }

        // Initialise files tree if it doesn't exist
        match collection.get(FILES_KEY) {
            Some(ValueOrContainer::Container(Container::Tree(files_tree))) => files_tree,
            _ => {
                console_log!(
                    "(build) Files tree not initialized, initializing now: {:#?}",
                    self.name
                );
                collection
                    .insert_container(FILES_KEY, LoroTree::new())
                    .map_err(|e| format!("Failed to insert files: {}", e))?;
                match collection.get(FILES_KEY) {
                    Some(ValueOrContainer::Container(Container::Tree(files_tree))) => files_tree,
                    _ => return Err("Files tree didn't initialize correctly".to_string()),
                }
            }
        };

        console_log!("Built attached Collection: {:?}", self.name);

        let collection = Collection {
            name: self.name,
            map: collection,
            file_type: PhantomData,
        };

        Ok(collection)
    }

    pub fn build_detached(self) -> Result<Collection<TFile>, String> {
        let fields_map = match self.map.get(FIELDS_KEY) {
            Some(ValueOrContainer::Container(Container::Map(fields_map))) => fields_map,
            _ => return Err("Fields not initialized".to_string()),
        };

        // Add all fields to the collection
        for field in self.fields {
            let field_name = field.name.clone();
            let field_value: Value = field.into();

            // Convert to LoroValue
            if let Some(field_obj) = field_value.as_object() {
                let field_map = LoroMap::new();
                field_map
                    .apply_map(field_obj)
                    .map_err(|e| format!("Failed to insert field: {}", e))?;
                fields_map
                    .insert_container(&field_name, field_map)
                    .map_err(|e| format!("Failed to insert field: {}", e))?;
            }
        }

        Ok(Collection {
            name: self.name,
            map: self.map,
            file_type: PhantomData,
        })
    }
}

/// Wrapper around a LoroMap handler that encapsulates collection-specific functionality.
/// The LoroMap contains a "fields" map and a "files" tree.
#[derive(Debug)]
pub struct Collection<TFile: File> {
    name: String,
    map: LoroMap, // Handler for collection data in the LoroDoc
    file_type: PhantomData<TFile>,
}

impl<TFile: File> Collection<TFile> {
    pub fn builder(name: String) -> Result<CollectionBuilder<TFile>, String> {
        CollectionBuilder::new(name)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    fn files_tree(&self) -> Result<LoroTree, String> {
        let files = match self.map.get_attached().unwrap().get(FILES_KEY) {
            Some(ValueOrContainer::Container(Container::Tree(files_tree))) => files_tree,
            _ => return Err("Files tree not initialized".to_string()),
        };
        Ok(files)
    }

    fn fields_map(&self) -> Result<LoroMap, String> {
        let fields = match self.map.get(FIELDS_KEY) {
            Some(ValueOrContainer::Container(Container::Map(fields_map))) => fields_map,
            _ => return Err("Fields not initialized".to_string()),
        };
        Ok(fields)
    }

    pub fn get_field(&self, name: &str) -> Result<FieldDefinition, String> {
        let fields = self.fields_map()?;

        match fields.get(name) {
            Some(ValueOrContainer::Container(Container::Map(field_map))) => {
                // Extract field type
                FieldDefinition::try_from(field_map)
            }
            _ => Err(format!("Field not found: {}", name)),
        }
    }

    pub fn get_fields(&self) -> Result<Vec<FieldDefinition>, String> {
        let fields = self.fields_map()?;

        let mut result: Vec<FieldDefinition> = Vec::new();

        for value in fields.values() {
            // console_log!("Field Value for {:?}: {:?}", self.name, value);
            if let ValueOrContainer::Container(Container::Map(field_map)) = value {
                let field_definition = FieldDefinition::try_from(field_map);
                if let Ok(field_definition) = field_definition {
                    result.push(field_definition);
                } else {
                    return Err(format!(
                        "Failed to get field definition: {}",
                        field_definition.err().unwrap()
                    ));
                }
            }
        }

        Ok(result)
    }

    pub fn create_file(
        &self,
        name: &str,
        collection_type: &str,
    ) -> Result<FileBuilder<TFile>, String> {
        console_log!("Creating file: {:?}", name);
        TFile::builder_for(collection_type).with_name(name.to_string())
    }

    pub fn attach_file(&self, file_builder: FileBuilder<TFile>) -> Result<TFile, String> {
        self.attach_file_with_parent(file_builder, None)
    }

    /// Attach a file to the collection file tree, with a parent file
    ///
    /// The file tree meta map is a cache of file metadata. The file
    /// LoroDoc meta map itself is the source of truth.
    ///
    pub fn attach_file_with_parent(
        &self,
        file_builder: FileBuilder<TFile>,
        parent_id: Option<TreeID>,
    ) -> Result<TFile, String> {
        // Get the files tree
        let files_tree = self.files_tree()?;

        // Create a temp file to get the metadata
        let file_meta = file_builder.meta().map_err(|e| e.to_string())?;

        // Check if we have an ID
        let id = match file_meta.get(ID_KEY) {
            Some(ValueOrContainer::Value(LoroValue::String(id))) => id.to_string(),
            _ => return Err("(attach_file) File ID not found".to_string()),
        };

        // Create a new node
        let node_id = files_tree
            .create(parent_id)
            .map_err(|e| format!("Failed to create node: {}", e))?;

        let node_meta = files_tree
            .get_meta(node_id)
            .map_err(|e| format!("Failed to get node meta: {}", e))?;

        node_meta
            .insert("id", id.clone())
            .map_err(|e| format!("Failed to insert ID: {}", e))?;

        // other properties?

        // Build the file
        let file = file_builder
            .build()
            .map_err(|e| format!("Failed to build file: {}", e))?;

        // Return the file - FileState is no longer set here
        Ok(file)
    }

    /// Get a file metadata from the collection file tree
    ///
    /// The file tree meta map is a cache of file metadata. The file
    /// LoroDoc meta map itself is the source of truth.
    /// This method gets the (cached) file metadata from the file tree meta map.
    pub fn get_file(&self, file_id: &str, collection_type: &str) -> Result<TFile, String> {
        // Check if the file tree is attached
        if !self.map.is_attached() {
            return Err("(get_file) Map is not attached".to_string());
        }

        // Get the file tree
        let files_tree = self.files_tree()?;

        // Find the file
        let nodes = files_tree.get_nodes(true);
        let file_node = nodes.iter().find(|node| {
            if let Ok(meta) = files_tree.get_meta(node.id) {
                if let Some(ValueOrContainer::Value(LoroValue::String(id))) = meta.get("id") {
                    return id == file_id.into();
                }
            }
            false
        });

        let file_node = match file_node {
            Some(node) => node,
            None => return Err(format!("(get_file) File {} not found", file_id)),
        };

        // Get the node metadata
        let node_meta = match files_tree.get_meta(file_node.id) {
            Ok(meta) => meta,
            Err(e) => {
                return Err(format!(
                    "(get_file) Meta for file {} not found: {}",
                    file_id, e
                ))
            }
        };

        // Create a file builder
        let file_builder = TFile::builder_for(collection_type);

        // Initialize with meta
        let target_file = file_builder
            .with_meta(node_meta.clone())
            .map_err(|e| format!("(get_file) with_meta: {}", e))?
            .build()
            .map_err(|e| format!("(get_file) build: {}", e))?;

        // FileState is no longer set here
        Ok(target_file)
    }

    pub fn get_files(&self, collection_type: &str) -> Result<Vec<TFile>, String> {
        // Check if the file tree is attached
        if !self.map.is_attached() {
            return Err("(get_files) Map is not attached".to_string());
        }

        // Get the file tree
        let files_tree = self.files_tree()?;

        // Get all nodes
        let nodes = files_tree.get_nodes(false);
        let mut result = Vec::new();

        for node in nodes {
            // Get the metadata
            let meta = match files_tree.get_meta(node.id) {
                Ok(meta) => meta,
                Err(e) => {
                    console_log!("Missing meta for node: {:?}", e);
                    continue;
                }
            };

            // Check the file type
            if let Some(ValueOrContainer::Value(LoroValue::String(doc_type))) = meta.get(TYPE_KEY) {
                if doc_type != collection_type.into() {
                    continue;
                }
            } else {
                // If no type is specified, we'll allow it through
                // This is hacky, but it's a workaround for now
            }

            // Create a file builder
            let file_builder = match TFile::builder_for(collection_type)
                .with_meta(meta.clone())
                .map_err(|e| format!("(get_files) Meta Map not valid: {}", e))
            {
                Ok(builder) => builder.build(),
                Err(e) => return Err(format!("(get_files) Meta Map not valid: {}", e)),
            };

            match file_builder {
                Ok(mut file) => {
                    // FileState is no longer set here
                    result.push(file);
                }
                Err(e) => {
                    console_log!("Failed to build file from metadata: {}", e);
                    continue;
                }
            }
        }

        console_log!("Returning {} files", result.len());
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    // Helper macro for logging
    macro_rules! console_log {
    ($($t:tt)*) => (log(&format!("[Collection Tests (WASM)] {}", format!($($t)*))))
  }

    #[derive(Default, Debug, Clone)]
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
                } else if let Some(ValueOrContainer::Value(LoroValue::I64(version))) =
                    meta.get("version")
                {
                    self.set_version(version)?;
                }
            }
            Ok(())
        }

        fn build_from(builder: FileBuilder<Self>) -> Result<Self, String> {
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

    impl HasTitle for TestFile {}
    impl HasUrl for TestFile {}

    fn setup_test_collection() -> Collection<TestFile> {
        let mut project = Project::new(ProjectType::Site, Some("test_theme".to_string()))
            .expect("Failed to create project");
        console_log!("Adding Collection {:?}", "test");
        let collection = project
            .add_collection::<TestFile>(
                "test",
                Model {
                    fields: HashMap::new(),
                },
            )
            .expect("Failed to add collection");
        console_log!("Added Collection {:?}", collection);
        let map = &collection.map;
        console_log!("Map is attached: {:?}", map.is_attached());
        collection
    }

    #[wasm_bindgen_test]
    fn test_collection_creation() {
        let collection = setup_test_collection();
        assert_eq!(collection.name(), "test");
    }

    #[wasm_bindgen_test]
    fn test_add_and_get_field() {
        let collection = CollectionBuilder::<TestFile>::new("test".to_string())
            .expect("Failed to create collection builder")
            .add_field("title", FieldType::Text, true)
            .build_detached()
            .expect("Failed to build collection");

        // Test getting the field
        let field = collection.get_field("title").expect("Failed to get field");
        assert_eq!(field.field_type, FieldType::Text);
        assert!(field.required);
    }

    #[wasm_bindgen_test]
    fn test_get_fields() {
        let collection = CollectionBuilder::<TestFile>::new("test".to_string())
            .expect("Failed to create collection builder")
            .add_field("title", FieldType::Text, true)
            .add_field("content", FieldType::Text, false)
            .build_detached()
            .expect("Failed to build collection");

        let fields = collection.get_fields().expect("Failed to get fields");
        assert_eq!(fields.len(), 2);

        let title_field = fields.iter().find(|f| f.name == "title").unwrap();
        assert!(matches!(title_field.field_type, FieldType::Text));
        assert!(title_field.required);

        let content_field = fields.iter().find(|f| f.name == "content").unwrap();
        assert!(matches!(content_field.field_type, FieldType::Text));
        assert!(!content_field.required);
    }

    #[wasm_bindgen_test]
    fn test_create_file() {
        let collection = setup_test_collection();

        // Test creating a file
        let file_builder = collection
            .create_file("test_file", "test")
            .expect("Failed to create file");

        let file = collection
            .attach_file(file_builder)
            .expect("Failed to attach file");

        assert_eq!(file.get_type(), "test");
        assert!(!file.id().unwrap().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_create_file_with_parent() {
        let collection = setup_test_collection();

        // Create parent file first
        let parent_file_builder = collection
            .create_file("parent_file", "test")
            .expect("Failed to create parent file");

        let parent_file = collection
            .attach_file(parent_file_builder)
            .expect("Failed to attach parent file");

        console_log!("Parent file: {:?}", parent_file);

        // Get the tree ID of the parent
        let files = match collection.map.get(FILES_KEY) {
            Some(ValueOrContainer::Container(Container::Tree(tree))) => tree,
            _ => panic!("Files tree not found"),
        };

        console_log!("Files: {:?}", files);

        let files_nodes = files.get_nodes(false);

        let parent_node = files_nodes
            .iter()
            .find(|n| {
                let meta = files.get_meta(n.id).unwrap();
                let id = match meta.get("id") {
                    Some(ValueOrContainer::Value(LoroValue::String(id))) => id,
                    _ => return false,
                };
                id.to_string() == parent_file.id().unwrap()
            })
            .unwrap();

        console_log!("Parent node: {:?}", parent_node);

        // Create child file
        let child_file_builder = collection
            .create_file("child_file", "test")
            .expect("Failed to create child file");

        let child_file = collection
            .attach_file_with_parent(child_file_builder, Some(parent_node.id))
            .expect("Failed to attach child file");

        assert_eq!(child_file.get_type(), "test");
        assert!(!child_file.id().unwrap().is_empty());
        assert_ne!(child_file.id(), parent_file.id());
    }

    #[wasm_bindgen_test]
    fn test_get_file() {
        let collection = setup_test_collection();

        // Create a file first
        let created_file = collection
            .create_file("test_file", "test")
            .expect("Failed to create file");

        let file = collection
            .attach_file(created_file)
            .expect("Failed to attach file");

        console_log!("Created file: {:?}", file);
        let file_id = file.id().unwrap();

        // Test getting the file
        let retrieved_file = collection
            .get_file(&file_id, "test")
            .expect("Failed to get file");
        assert_eq!(retrieved_file.id().unwrap(), file_id);
        assert_eq!(retrieved_file.get_type(), "test");
    }

    #[wasm_bindgen_test]
    fn test_get_files() {
        let collection = setup_test_collection();

        console_log!("Created Collection. Creating file builders");

        // Create multiple files
        let file_builder1 = collection
            .create_file("file1", "test")
            .expect("Failed to create file1");
        let file_builder2 = collection
            .create_file("file2", "test")
            .expect("Failed to create file2");

        console_log!("Attaching files");

        let file1 = collection
            .attach_file(file_builder1)
            .expect("Failed to attach file1");
        let file2 = collection
            .attach_file(file_builder2)
            .expect("Failed to attach file2");

        console_log!("Getting files");

        let files = collection.get_files("test").expect("Failed to get files");

        console_log!("Files: {:?}", files);

        assert_eq!(files.len(), 2);

        let file_ids: Vec<String> = files.iter().map(|f| f.id().unwrap()).collect();
        assert!(file_ids.contains(&file1.id().unwrap()));
        assert!(file_ids.contains(&file2.id().unwrap()));
    }

    #[wasm_bindgen_test]
    fn test_file_state_transitions_in_collection() {
        let collection = setup_test_collection();

        // Create a new file
        let file_builder = collection
            .create_file("test_file", "test")
            .expect("Failed to create file");

        // After building but before attachment
        let file = file_builder.clone().build().expect("Failed to build file");

        // After attachment - just verify the file was attached successfully
        let attached_file = collection
            .attach_file(file_builder)
            .expect("Failed to attach file");

        // Verify we can retrieve the file
        let retrieved_file = collection
            .get_file(&attached_file.id().unwrap(), "test")
            .expect("Failed to get file");

        // Instead verify that it's the same file
        assert_eq!(retrieved_file.id().unwrap(), attached_file.id().unwrap());
    }

    #[wasm_bindgen_test]
    fn test_file_state_transitions_with_parent() {
        let collection = setup_test_collection();

        // Create and attach parent file
        let parent_builder = collection
            .create_file("parent", "test")
            .expect("Failed to create parent file");
        let parent = collection
            .attach_file(parent_builder)
            .expect("Failed to attach parent file");

        // Get parent's tree ID - test still should work, just no state assertions
        let files_tree = collection.files_tree().expect("Failed to get files tree");
        let files_nodes = files_tree.get_nodes(false);
        let parent_node = files_nodes
            .iter()
            .find(|n| {
                let meta = files_tree.get_meta(n.id).unwrap();
                let id = match meta.get("id") {
                    Some(ValueOrContainer::Value(LoroValue::String(id))) => id,
                    _ => return false,
                };
                id.to_string() == parent.id().unwrap()
            })
            .expect("Failed to find parent node");

        // Create and attach child file
        let child_builder = collection
            .create_file("child", "test")
            .expect("Failed to create child file");
        let child = collection
            .attach_file_with_parent(child_builder, Some(parent_node.id))
            .expect("Failed to attach child file");

        // Just verify the ids are different
        assert_ne!(child.id(), parent.id());
    }

    #[wasm_bindgen_test]
    fn test_collection_file_operations() {
        let collection = setup_test_collection();

        // Test file creation without state transitions
        let file_builder = collection
            .create_file("test_file", "test")
            .expect("Failed to create file");

        let file = collection
            .attach_file(file_builder)
            .expect("Failed to attach file");

        // Just verify it has a valid ID
        assert!(!file.id().unwrap().is_empty());

        // Test retrieving files - verify length and ID only
        let files = collection.get_files("test").expect("Failed to get files");
        assert_eq!(files.len(), 1);

        // Verify the file ID matches
        assert_eq!(files[0].id(), file.id());
    }

    #[wasm_bindgen_test]
    fn test_collection_error_handling() {
        let collection = setup_test_collection();

        // Test getting non-existent file
        let result = collection.get_file("nonexistent_id", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        // Test getting files with wrong type
        let result = collection.get_files("nonexistent_type");
        assert_eq!(result.unwrap().len(), 0); // Should return empty list, not error
    }

    #[wasm_bindgen_test]
    fn test_collection_builder_errors() {
        // Test building with invalid name (empty)
        let result = CollectionBuilder::<TestFile>::new("".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name"));

        // Test adding duplicate field
        let result = CollectionBuilder::<TestFile>::new("test".to_string())
            .expect("Failed to create collection builder")
            .add_field("field1", FieldType::Text, true)
            .add_field("field1", FieldType::Number, false); // Same field name

        let collection_result = result.build_detached();
        assert!(collection_result.is_ok()); // Should overwrite the first field
        let fields = collection_result
            .expect("Failed to build collection")
            .get_fields()
            .expect("Failed to get fields");

        let field = fields.iter().find(|f| f.name == "field1").unwrap();
        assert!(matches!(field.field_type, FieldType::Number)); // Should be the second definition
        assert!(!field.required); // Should be the second definition
    }

    #[wasm_bindgen_test]
    fn test_collection_creation_options() {
        // Test collection creation with detailed model
        let mut model = Model {
            fields: HashMap::new(),
        };

        model.fields.insert(
            "title".to_string(),
            FieldDefinition {
                name: "title".to_string(),
                field_type: FieldType::Text,
                required: true,
            },
        );

        model.fields.insert(
            "description".to_string(),
            FieldDefinition {
                name: "description".to_string(),
                field_type: FieldType::Text, // Changed from Markdown to Text
                required: false,
            },
        );

        let mut project = Project::new(ProjectType::Site, Some("test_theme".to_string()))
            .expect("Failed to create project");

        let collection = project
            .add_collection::<TestFile>("custom_model", model)
            .expect("Failed to add collection with custom model");

        // Verify model was correctly applied
        let fields = collection.get_fields().expect("Failed to get fields");
        assert_eq!(fields.len(), 2);

        let title_field = fields.iter().find(|f| f.name == "title").unwrap();
        assert!(matches!(title_field.field_type, FieldType::Text));
        assert!(title_field.required);

        let desc_field = fields.iter().find(|f| f.name == "description").unwrap();
        assert!(matches!(desc_field.field_type, FieldType::Text)); // Changed from Markdown to Text
        assert!(!desc_field.required);
    }
}
