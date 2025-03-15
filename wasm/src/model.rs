use crate::types::{
    CollectionType, ContentRecord, FieldDefinition, FieldType, ProjectType, UnparsedContentRecord,
};
use loro::{
    Container, ExportMode, LoroDoc, LoroList, LoroMap, LoroStringValue, LoroText, LoroTree,
    LoroValue, TreeID, TreeNode, ValueOrContainer,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // log to JS console
}

// Helper macro for logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!("[model.rs] {}", format!($($t)*))))
}

#[derive(Debug, Clone)]
pub struct Model {
    pub fields: HashMap<String, FieldDefinition>,
}

impl Model {
    pub fn new() -> Self {
        Model {
            fields: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: FieldDefinition) {
        self.fields.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<FieldDefinition> {
        self.fields.get(key).cloned()
    }
}

// Store: The main entry point for our data model
pub struct Store {
    active_theme: RwLock<Option<Project>>,
    active_site: RwLock<Option<Project>>,
}

impl Store {
    pub fn new() -> Self {
        let store = Store {
            active_theme: RwLock::new(None),
            active_site: RwLock::new(None),
        };
        store
    }

    pub fn init_default(&self) -> Result<(), String> {
        console_log!("Initializing default projects");
        // Create default theme
        let default_theme = Project::new(ProjectType::Theme, None)
            .map_err(|e| format!("Failed to create default theme: {}", e))?;
        let theme_id = default_theme.id();

        // Create default site
        let default_site = Project::new(ProjectType::Site, Some(theme_id))
            .map_err(|e| format!("Failed to create default site: {}", e))?;

        // Set active projects
        {
            let mut active_theme = self.active_theme.write().unwrap();
            *active_theme = Some(default_theme);
        }
        {
            let mut active_site = self.active_site.write().unwrap();
            *active_site = Some(default_site);
        }

        Ok(())
    }

    pub fn get_active_theme(&self) -> Option<Project> {
        self.active_theme.read().unwrap().clone()
    }

    pub fn get_active_site(&self) -> Option<Project> {
        self.active_site.read().unwrap().clone()
    }

    pub fn set_active_theme(&self, theme: Project) {
        let mut active_theme = self.active_theme.write().unwrap();
        *active_theme = Some(theme);
    }

    pub fn set_active_site(&self, site: Project) {
        let mut active_site = self.active_site.write().unwrap();
        *active_site = Some(site);
    }

    pub fn export_to_json(&self) -> Result<Value, String> {
        // TODO: Implement export
        Ok(serde_json::json!({}))
    }

    pub fn import_from_json(&self, _data: Value) -> Result<(), String> {
        // TODO: Implement import
        Ok(())
    }
}

// Project: Wrapper around a LoroDocument that encapsulates project-specific functionality
#[derive(Clone)]
pub struct Project {
    id: String,
    project_type: ProjectType,
    created: f64, // Store as timestamp
    updated: f64, // Store as timestamp
    doc: LoroDoc,
}

impl Project {
    pub fn new(project_type: ProjectType, theme_id: Option<String>) -> Result<Project, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis() as f64;
        let doc = LoroDoc::new();

        // Initialize collections map
        let collections_map = doc.get_map("collections");

        // Initialize metadata map
        let meta_map = doc.get_map("meta");
        meta_map.insert("id", id.clone());

        let mut project = Project {
            id,
            project_type: project_type.clone(),
            created: now,
            updated: now,
            doc,
        };

        // Initialize based on project type
        match project_type {
            ProjectType::Site => {
                if let Some(theme_id) = theme_id {
                    project
                        .init_default_site(&theme_id)
                        .map_err(|e| format!("Failed to initialize site: {}", e))?;
                } else {
                    return Err("Theme ID is required for site creation".to_string());
                }
            }
            ProjectType::Theme => {
                project
                    .init_default_theme()
                    .map_err(|e| format!("Failed to initialize theme: {}", e))?;
            }
        }

        Ok(project)
    }

    pub fn import(
        data: Vec<u8>,
        id: String,
        project_type: ProjectType,
        created: f64,
        updated: f64,
    ) -> Result<Project, String> {
        let doc = LoroDoc::new();
        doc.import(&data[..]);

        let project = Project {
            id,
            project_type,
            created,
            updated,
            doc,
        };

        Ok(project)
    }

    // Helper method to initialize a default theme
    fn init_default_theme(&mut self) -> Result<(), String> {
        let meta = self.doc.get_map("meta");
        meta.insert("name", "New Theme".to_string());

        // Add template collection
        let mut template_model = Model::new();
        template_model.insert(
            "content",
            FieldDefinition {
                name: "content".to_string(),
                field_type: FieldType::Text,
                required: true,
            },
        );
        self.add_collection("template", template_model)?;

        // Add partial collection
        let mut partial_model = Model::new();
        partial_model.insert(
            "content",
            FieldDefinition {
                name: "content".to_string(),
                field_type: FieldType::Text,
                required: true,
            },
        );
        self.add_collection("partial", partial_model)?;

        // Add text collection
        let mut text_model = Model::new();
        text_model.insert(
            "content",
            FieldDefinition {
                name: "content".to_string(),
                field_type: FieldType::Text,
                required: true,
            },
        );
        self.add_collection("text", text_model)?;

        // Add asset collection
        let mut asset_model = Model::new();
        asset_model.insert(
            "mime_type",
            FieldDefinition {
                name: "mime_type".to_string(),
                field_type: FieldType::String,
                required: true,
            },
        );
        self.add_collection("asset", asset_model)?;

        // Create default template
        let template = self.create_file("index", "template")?;
        let template_content = r#"<!DOCTYPE html>
<html lang="en">

<head>
<link rel="stylesheet" href="style.css" />
<title>{{title}}</title>
</head>

<body>
<h1>{{title}}</h1>
{{{content}}}
</body>
</html>"#;
        template
            .set_content(template_content)
            .map_err(|e| format!("Failed to set template content: {}", e))?;

        // Create default style
        let style = self.create_file("style", "text")?;
        let style_content = r#"* {
  font-family: sans-serif;
}

h1 {
  font-size: 2rem;
  font-weight: bold;
}

h2 {
  font-size: 1.5rem;
  font-weight: bold;
}
  
img {
  width: 80%;
}"#;
        style
            .set_content(style_content)
            .map_err(|e| format!("Failed to set style content: {}", e))?;

        self.updated = chrono::Utc::now().timestamp_millis() as f64;

        Ok(())
    }

    // Helper method to initialize a default site
    fn init_default_site(&mut self, theme_id: &str) -> Result<(), String> {
        let meta = self.doc.get_map("meta");
        meta.insert("name", "New Site".to_string());
        meta.insert("themeId", theme_id.to_string());

        // Add page collection
        let mut page_model = Model::new();
        page_model.insert(
            "template",
            FieldDefinition {
                name: "template".to_string(),
                field_type: FieldType::String,
                required: true,
            },
        );
        page_model.insert(
            "title",
            FieldDefinition {
                name: "title".to_string(),
                field_type: FieldType::String,
                required: true,
            },
        );
        page_model.insert(
            "body",
            FieldDefinition {
                name: "body".to_string(),
                field_type: FieldType::RichText,
                required: true,
            },
        );
        self.add_collection("page", page_model)?;

        // Add post collection
        let mut post_model = Model::new();
        post_model.insert(
            "title",
            FieldDefinition {
                name: "title".to_string(),
                field_type: FieldType::String,
                required: true,
            },
        );
        post_model.insert(
            "body",
            FieldDefinition {
                name: "body".to_string(),
                field_type: FieldType::RichText,
                required: true,
            },
        );

        console_log!("Adding post collection {:?}", post_model);
        self.add_collection("post", post_model)?;

        // Add asset collection
        let mut asset_model = Model::new();
        asset_model.insert(
            "mime_type",
            FieldDefinition {
                name: "mime_type".to_string(),
                field_type: FieldType::String,
                required: true,
            },
        );
        self.add_collection("asset", asset_model)?;

        // Create default page
        let main = self.create_file("main", "page")?;
        main.set_title("Hello World")
            .map_err(|e| format!("Failed to set page title: {}", e))?;
        main.set_body("Hello World")
            .map_err(|e| format!("Failed to set page body: {}", e))?;

        // Create default post
        let post = self.create_file("post", "post")?;
        post.set_title("Hello World")
            .map_err(|e| format!("Failed to set post title: {}", e))?;
        post.set_body("Hello World")
            .map_err(|e| format!("Failed to set post body: {}", e))?;

        self.updated = chrono::Utc::now().timestamp_millis() as f64;

        Ok(())
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn name(&self) -> Result<String, String> {
        match self.doc.get_map("meta").get("name") {
            Some(ValueOrContainer::Value(LoroValue::String(name))) => Ok(name.clone().to_string()),
            _ => Err("Name not found".to_string()),
        }
    }

    pub fn set_name(&mut self, name: &str) {
        let meta = self.doc.get_map("meta");
        meta.insert("name", name.to_string());
        self.updated = chrono::Utc::now().timestamp_millis() as f64;
    }

    pub fn theme_id(&self) -> Option<String> {
        let meta = self.doc.get_map("meta");
        match meta.get("themeId") {
            Some(ValueOrContainer::Value(LoroValue::String(theme_id))) => {
                Some(theme_id.clone().to_string())
            }
            _ => None,
        }
    }

    pub fn set_theme_id(&mut self, theme_id: &str) {
        let meta = self.doc.get_map("meta");
        meta.insert("themeId", theme_id.to_string());
        self.updated = chrono::Utc::now().timestamp_millis() as f64;
    }

    // Removed active_file methods

    // Create a new collection with the specified model
    pub fn add_collection(&mut self, name: &str, model: Model) -> Result<Collection, String> {
        let collections = self.doc.get_map("collections");
        let collection_map = LoroMap::new();

        collection_map.insert("name", name.to_string());

        let files_tree = LoroTree::new();
        collection_map.insert_container("files", files_tree);

        let fields_map = LoroMap::new();
        collection_map.insert_container("fields", fields_map);

        collections.insert_container(name, collection_map);

        // Get the inserted map
        let map = match collections.get(name) {
            Some(ValueOrContainer::Container(Container::Map(map))) => map,
            _ => {
                return Err(format!(
                    "Failed to get collection map after insertion: {}",
                    name
                ))
            }
        };

        let collection = Collection::new(name.to_string(), &map);

        for (field_name, field_value) in model.fields.iter() {
            collection
                .add_field(
                    &field_name,
                    field_value.field_type.clone(),
                    field_value.required,
                )
                .map_err(|e| format!("Failed to add field: {}", e))?;
        }

        self.updated = chrono::Utc::now().timestamp_millis() as f64;

        Ok(collection)
    }

    pub fn get_collection(&self, name: &str) -> Result<Collection, String> {
        let collections = self.doc.get_map("collections");
        match collections.get(name) {
            Some(ValueOrContainer::Container(Container::Map(map))) => {
                let collection = Collection::new(name.to_string(), &map);
                Ok(collection)
            }
            _ => Err(format!("Collection not found: {}", name)),
        }
    }

    pub fn get_collections(&self) -> Result<Vec<Collection>, String> {
        let collections = self.doc.get_map("collections");
        let mut result: Vec<Collection> = Vec::new();

        for value in collections.values() {
            if let ValueOrContainer::Container(Container::Map(map)) = value {
                match map.get("name") {
                    Some(ValueOrContainer::Value(LoroValue::String(name))) => {
                        let collection = Collection::new(name.to_string().clone(), &map);
                        result.push(collection);
                    }
                    _ => {}
                }
            }
        }

        Ok(result)
    }

    pub fn create_file(&mut self, name: &str, collection_type: &str) -> Result<File, String> {
        // Get the collection
        let collection = self.get_collection(collection_type)?;

        // Create a file in the collection
        let file = collection
            .create_file(name)
            .map_err(|e| format!("Failed to create file: {}", e))?;

        self.updated = chrono::Utc::now().timestamp_millis() as f64;

        Ok(file)
    }

    pub fn save(&mut self) -> Result<(), String> {
        // This would be implemented to export project data
        self.updated = chrono::Utc::now().timestamp_millis() as f64;
        Ok(())
    }

    pub fn export(&self) -> Result<Vec<u8>, String> {
        let export = self.doc.export(ExportMode::all_updates());
        match export {
            Ok(export) => Ok(export),
            Err(e) => Err(format!("Failed to export: {}", e)),
        }
    }
}

// Collection: Wrapper around a LoroMap handler that encapsulates collection-specific functionality
pub struct Collection {
    pub name: String,
    pub map: LoroMap, // Handler for collection data in the LoroDoc
}

impl Collection {
    pub fn new(name: String, map: &LoroMap) -> Self {
        Collection {
            name,
            map: map.clone(), // Clone the handler, not the underlying data
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn add_field(
        &self,
        name: &str,
        field_type: FieldType,
        required: bool,
    ) -> Result<(), String> {
        let fields = match self.map.get("fields") {
            Some(ValueOrContainer::Container(Container::Map(fields_map))) => fields_map,
            _ => return Err("Fields not initialized".to_string()),
        };

        console_log!("Adding field {:?} to collection {:?}", name, self.name);

        let field_value = serde_json::json!({
            "name": name,
            "field_type": field_type.to_string(),
            "required": required,
        });

        // Convert to LoroValue
        if let Some(field_obj) = field_value.as_object() {
            let mut field_map = LoroMap::new();
            for (k, v) in field_obj {
                match v {
                    serde_json::Value::String(s) => field_map.insert(k, s.clone()),
                    serde_json::Value::Bool(b) => field_map.insert(k, *b),
                    _ => return Err("Unsupported field value type".to_string()),
                };
            }
            fields.insert_container(name, field_map);
        }

        Ok(())
    }

    pub fn get_field(&self, name: &str) -> Result<Value, String> {
        let fields = match self.map.get("fields") {
            Some(ValueOrContainer::Container(Container::Map(fields_map))) => fields_map,
            _ => return Err("Fields not initialized".to_string()),
        };

        match fields.get(name) {
            Some(ValueOrContainer::Container(Container::Map(field_map))) => {
                let mut result = Map::new();

                // Extract field type
                if let Some(ValueOrContainer::Value(LoroValue::String(type_str))) =
                    field_map.get("type")
                {
                    result.insert("type".to_string(), Value::String(type_str.to_string()));
                }

                // Extract required flag
                if let Some(ValueOrContainer::Value(LoroValue::Bool(required))) =
                    field_map.get("required")
                {
                    result.insert("required".to_string(), Value::Bool(required));
                }

                Ok(Value::Object(result))
            }
            _ => Err(format!("Field not found: {}", name)),
        }
    }

    pub fn get_fields(&self) -> Result<Vec<FieldDefinition>, String> {
        let fields = match self.map.get("fields") {
            Some(ValueOrContainer::Container(Container::Map(fields_map))) => fields_map,
            _ => return Err("Fields not initialized".to_string()),
        };

        let mut result: Vec<FieldDefinition> = Vec::new();

        for value in fields.values() {
            console_log!("Field Value for {:?}: {:?}", self.name, value);
            if let ValueOrContainer::Container(Container::Map(field_map)) = value {
                let name = match field_map.get("name") {
                    //right key?
                    Some(ValueOrContainer::Value(LoroValue::String(name))) => name.to_string(),
                    _ => return Err("Name not found".to_string()),
                };

                // Extract field type
                let field_type = match field_map.get("field_type") {
                    Some(ValueOrContainer::Value(LoroValue::String(type_str))) => {
                        match type_str.as_str() {
                            "richtext" => FieldType::RichText,
                            "text" => FieldType::Text,
                            "list" => FieldType::List,
                            "map" => FieldType::Map,
                            "datetime" => FieldType::DateTime,
                            "string" => FieldType::String,
                            "number" => FieldType::Number,
                            "object" => FieldType::Object,
                            "array" => FieldType::Array,
                            "blob" => FieldType::Blob,
                            _ => return Err(format!("Unknown field type: {:?}", type_str)),
                        }
                    }
                    _ => return Err("Field type not found".to_string()),
                };

                // Extract required flag
                let required = match field_map.get("required") {
                    Some(ValueOrContainer::Value(LoroValue::Bool(required))) => required,
                    _ => false,
                };

                console_log!(
                    "Field Definition: {:?}",
                    FieldDefinition {
                        name: name.to_string(),
                        field_type,
                        required,
                    }
                );

                result.push(FieldDefinition {
                    name: name.to_string(),
                    field_type,
                    required,
                });
            }
        }

        Ok(result)
    }

    pub fn create_file(&self, name: &str) -> Result<File, String> {
        let files = match self.map.get("files") {
            Some(ValueOrContainer::Container(Container::Tree(tree))) => tree,
            _ => return Err("Files tree not initialized".to_string()),
        };

        let node_id = files.create(None);
        let file = match node_id {
            Ok(node) => File::new(&files, node, &self.name),
            Err(e) => return Err(format!("Failed to create file: {}", e)),
        };

        file.set_name(name)
            .map_err(|e| format!("Failed to set file name: {}", e))?;

        // Initialize file based on collection type
        match self.name.as_str() {
            "page" => {
                file.set_body("")
                    .map_err(|e| format!("Failed to set body: {}", e))?;
                file.set_title(name)
                    .map_err(|e| format!("Failed to set title: {}", e))?;
                file.set_field("template", "index")
                    .map_err(|e| format!("Failed to set template: {}", e))?;
            }
            "post" => {
                file.set_body("")
                    .map_err(|e| format!("Failed to set body: {}", e))?;
                file.set_title(name)
                    .map_err(|e| format!("Failed to set title: {}", e))?;
                let date = chrono::Utc::now().to_rfc3339();
                file.set_field("date", &date)
                    .map_err(|e| format!("Failed to set date: {}", e))?;
            }
            "template" | "partial" | "text" => {
                file.set_content("")
                    .map_err(|e| format!("Failed to set content: {}", e))?;
            }
            "asset" => {
                file.set_field("mime_type", "image/png")
                    .map_err(|e| format!("Failed to set mime_type: {}", e))?;
                file.set_field("url", "")
                    .map_err(|e| format!("Failed to set url: {}", e))?;
                file.set_field("alt", "")
                    .map_err(|e| format!("Failed to set alt: {}", e))?;
            }
            _ => {}
        }

        Ok(file)
    }

    pub fn get_files(&self) -> Result<Vec<File>, String> {
        let files = match self.map.get("files") {
            Some(ValueOrContainer::Container(Container::Tree(tree))) => tree,
            _ => return Err("Files tree not initialized".to_string()),
        };

        let mut result = Vec::new();

        for node in files.get_nodes(false) {
            let file = File::new(&files, node.id, &self.name);
            result.push(file);
        }

        Ok(result)
    }
}

// File: Wrapper around a LoroTree handler and node ID that encapsulates file-specific functionality
#[derive(Clone)]
pub struct File {
    pub files: LoroTree, // Handler for the files tree in the LoroDoc
    pub id: TreeID,
    pub collection_type: String,
}

impl File {
    pub fn new(files: &LoroTree, id: TreeID, collection_type: &str) -> Self {
        File {
            files: files.clone(), // Clone the handler, not the underlying data
            id,
            collection_type: collection_type.to_string(),
        }
    }

    pub fn name(&self) -> Result<String, String> {
        match self.files.get_meta(self.id).unwrap().get("name") {
            Some(ValueOrContainer::Value(LoroValue::String(name))) => Ok(name.clone().to_string()),
            _ => Err("Name not found".to_string()),
        }
    }

    pub fn set_name(&self, name: &str) -> Result<(), String> {
        match self.files.get_meta(self.id) {
            Ok(meta) => {
                meta.insert("name", name.to_string());
                Ok(())
            }
            Err(_) => Err("Node metadata not found".to_string()),
        }
    }

    pub fn collection_type(&self) -> String {
        self.collection_type.clone()
    }

    pub fn set_body(&self, content: &str) -> Result<(), String> {
        // For page and post types
        if self.collection_type == "page" || self.collection_type == "post" {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            if !data.get("body").is_some() {
                let body_text = LoroText::new();
                data.insert_container("body", body_text);
            }

            if let Some(ValueOrContainer::Container(Container::Text(body))) = data.get("body") {
                body.delete(0, body.len_utf8());
                body.insert(0, content);
                Ok(())
            } else {
                Err("Body not initialized".to_string())
            }
        } else {
            Err("Body is only available for page and post types".to_string())
        }
    }

    pub fn get_body(&self) -> Result<String, String> {
        // For page and post types
        if self.collection_type == "page" || self.collection_type == "post" {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            if let Some(ValueOrContainer::Container(Container::Text(body))) = data.get("body") {
                Ok(body.to_string())
            } else {
                Err("Body not initialized".to_string())
            }
        } else {
            Err("Body is only available for page and post types".to_string())
        }
    }

    pub fn set_content(&self, content: &str) -> Result<(), String> {
        // For template, partial, and text types
        if self.collection_type == "template"
            || self.collection_type == "partial"
            || self.collection_type == "text"
        {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            if let None = data.get("content") {
                let content_text = LoroText::new();
                data.insert_container("content", content_text);
            }

            if let Some(ValueOrContainer::Container(Container::Text(content_text))) =
                data.get("content")
            {
                content_text.delete(0, content_text.len_utf8());
                content_text.insert(0, content);
                Ok(())
            } else {
                Err("Content not initialized".to_string())
            }
        } else {
            Err("Content is only available for template, partial, and text types".to_string())
        }
    }

    pub fn get_content(&self) -> Result<String, String> {
        // For template, partial, and text types
        if self.collection_type == "template"
            || self.collection_type == "partial"
            || self.collection_type == "text"
        {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            if let Some(ValueOrContainer::Container(Container::Text(content))) = data.get("content")
            {
                Ok(content.to_string())
            } else {
                Err("Content not initialized".to_string())
            }
        } else {
            Err("Content is only available for template, partial, and text types".to_string())
        }
    }

    pub fn set_title(&self, title: &str) -> Result<(), String> {
        // For page and post types
        if self.collection_type == "page" || self.collection_type == "post" {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            data.insert("title", title.to_string());
            Ok(())
        } else {
            Err("Title is only available for page and post types".to_string())
        }
    }

    pub fn get_title(&self) -> Result<String, String> {
        // For page and post types
        if self.collection_type == "page" || self.collection_type == "post" {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            match data.get("title") {
                Some(ValueOrContainer::Value(LoroValue::String(title))) => {
                    Ok(title.clone().to_string())
                }
                _ => Err("Title not found".to_string()),
            }
        } else {
            Err("Title is only available for page and post types".to_string())
        }
    }

    pub fn set_field(&self, field: &str, value: &str) -> Result<(), String> {
        let data = match self.files.get_meta(self.id) {
            Ok(meta) => meta,
            Err(_) => return Err("Node metadata not found".to_string()),
        };

        data.insert(field, value.to_string());
        Ok(())
    }

    pub fn get_field(&self, field: &str) -> Result<Value, String> {
        let data = match self.files.get_meta(self.id) {
            Ok(meta) => meta,
            Err(_) => return Err("Node metadata not found".to_string()),
        };

        match data.get(field) {
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

    pub fn set_url(&self, url: &str) -> Result<(), String> {
        let data = match self.files.get_meta(self.id) {
            Ok(meta) => meta,
            Err(_) => return Err("Node metadata not found".to_string()),
        };

        data.insert("url", url.to_string());
        Ok(())
    }

    pub fn get_url(&self) -> Result<String, String> {
        let data = match self.files.get_meta(self.id) {
            Ok(meta) => meta,
            Err(_) => return Err("Node metadata not found".to_string()),
        };

        match data.get("url") {
            Some(ValueOrContainer::Value(LoroValue::String(url))) => Ok(url.clone().to_string()),
            _ => Err("URL not found".to_string()),
        }
    }

    // For asset type
    pub fn set_mime_type(&self, mime_type: &str) -> Result<(), String> {
        if self.collection_type == "asset" {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            data.insert("mime_type", mime_type.to_string());
            Ok(())
        } else {
            Err("Mime type is only available for asset type".to_string())
        }
    }

    pub fn get_mime_type(&self) -> Result<String, String> {
        if self.collection_type == "asset" {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            match data.get("mime_type") {
                Some(ValueOrContainer::Value(LoroValue::String(mime_type))) => {
                    Ok(mime_type.clone().to_string())
                }
                _ => Err("Mime type not found".to_string()),
            }
        } else {
            Err("Mime type is only available for asset type".to_string())
        }
    }

    pub fn set_alt(&self, alt: &str) -> Result<(), String> {
        if self.collection_type == "asset" {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            data.insert("alt", alt.to_string());
            Ok(())
        } else {
            Err("Alt text is only available for asset type".to_string())
        }
    }

    pub fn get_alt(&self) -> Result<String, String> {
        if self.collection_type == "asset" {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            match data.get("alt") {
                Some(ValueOrContainer::Value(LoroValue::String(alt))) => {
                    Ok(alt.clone().to_string())
                }
                _ => Err("Alt text not found".to_string()),
            }
        } else {
            Err("Alt text is only available for asset type".to_string())
        }
    }

    pub fn to_json(&self) -> Result<Value, String> {
        let data = match self.files.get_meta(self.id) {
            Ok(meta) => meta,
            Err(_) => return Err("Node metadata not found".to_string()),
        };

        let mut result = Map::new();

        // Add id
        result.insert("id".to_string(), Value::String(self.id.to_string()));

        // Add collection type
        result.insert(
            "collection_type".to_string(),
            Value::String(self.collection_type.clone()),
        );

        // Add name if available
        if let Ok(name) = self.name() {
            result.insert("name".to_string(), Value::String(name));
        }

        // Add specific fields based on collection type
        match self.collection_type.as_str() {
            "page" | "post" => {
                if let Ok(body) = self.get_body() {
                    result.insert("body".to_string(), Value::String(body));
                }
                if let Ok(title) = self.get_title() {
                    result.insert("title".to_string(), Value::String(title));
                }
            }
            "template" | "partial" | "text" => {
                if let Ok(content) = self.get_content() {
                    result.insert("content".to_string(), Value::String(content));
                }
            }
            "asset" => {
                if let Ok(url) = self.get_url() {
                    result.insert("url".to_string(), Value::String(url));
                }
                if let Ok(mime_type) = self.get_mime_type() {
                    result.insert("mime_type".to_string(), Value::String(mime_type));
                }
                if let Ok(alt) = self.get_alt() {
                    result.insert("alt".to_string(), Value::String(alt));
                }
            }
            _ => {}
        }

        Ok(Value::Object(result))
    }
}
