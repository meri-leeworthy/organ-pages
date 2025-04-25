use std::collections::HashMap;

use crate::model::collection::Collection;
use crate::model::file::{Asset, File, Page, Partial, Post, Template, Text};
use crate::model::lib::Model;
use crate::model::{HasContent, HasTitle};
use crate::types::{FieldDefinition, FieldType, ProjectType};
use crate::ProseMirrorSchema;

use loro::{Container, ExportMode, LoroDoc, LoroError, LoroMap, LoroValue, ValueOrContainer};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // log to JS console
}

// Helper macro for logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!("[Project (WASM)] {}", format!($($t)*))))
}

const DEFAULT_STYLE: &str = r#"* {
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

const TEMPLATE_CONTENT: &str = r#"<!DOCTYPE html>
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

// Project: Wrapper around a LoroDocument that encapsulates project-specific functionality
#[derive(Clone, Debug)]
pub struct Project {
    id: String,
    project_type: ProjectType,
    created: f64, // Store as timestamp
    updated: f64, // Store as timestamp
    doc: LoroDoc,
}

impl Project {
    pub async fn new(
        project_type: ProjectType,
        theme_id: Option<String>,
    ) -> Result<Project, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis() as f64;
        let doc = LoroDoc::new();

        // Initialize collections map
        doc.get_map("collections");

        // Initialize metadata map
        let meta_map = doc.get_map(crate::META_KEY);
        meta_map
            .insert("id", id.clone())
            .map_err(|e| format!("Failed to set project ID: {}", e))?;

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
                        .await
                        .map_err(|e| format!("Failed to initialize site: {}", e))?;
                } else {
                    return Err("Theme ID is required for site creation".to_string());
                }
            }
            ProjectType::Theme => {
                project
                    .init_default_theme()
                    .await
                    .map_err(|e| format!("Failed to initialize theme: {}", e))?;
            }
        }

        Ok(project)
    }

    pub fn project_type(&self) -> ProjectType {
        self.project_type.clone()
    }

    pub fn created(&self) -> f64 {
        self.created
    }

    pub fn updated(&self) -> f64 {
        self.updated
    }

    pub fn analyse(&self) -> Result<(), String> {
        let analysis = self.doc.analyze();
        console_log!("Analysis: {:#?}", analysis);
        Ok(())
    }

    pub fn import(
        data: Vec<u8>,
        id: String,
        project_type: ProjectType,
        created: f64,
        updated: f64,
    ) -> Result<Project, String> {
        let doc = LoroDoc::new();
        doc.import(&data[..])
            .map_err(|e| format!("Failed to import project: {}", e))?;

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
    async fn init_default_theme(&mut self) -> Result<(), String> {
        let meta = self.meta();
        meta.insert("name", "New Theme".to_string())
            .map_err(|e| format!("Failed to set theme name: {}", e))?;

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
        self.add_collection::<Template>("template", template_model)?;

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
        self.add_collection::<Partial>("partial", partial_model)?;

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
        self.add_collection::<Text>("text", text_model)?;

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
        self.add_collection::<Asset>("asset", asset_model)?;

        // Create default template
        let template_builder: crate::FileBuilder<Template> =
            self.create_file("index", "template", crate::FileStore::Full(LoroDoc::new()))?;

        let template: Template = self.attach_file(template_builder).await?;
        template
            .insert_content(TEMPLATE_CONTENT, 0)
            .map_err(|e| format!("Failed to set template content: {}", e))?;

        // Create default style
        let style_builder: crate::FileBuilder<Text> =
            self.create_file("style", "text", crate::FileStore::Full(LoroDoc::new()))?;
        let style: Text = self.attach_file(style_builder).await?;
        style
            .insert_content(DEFAULT_STYLE, 0)
            .map_err(|e| format!("Failed to set style content: {}", e))?;

        self.updated = chrono::Utc::now().timestamp_millis() as f64;

        Ok(())
    }

    // Helper method to initialize a default site
    async fn init_default_site(&mut self, theme_id: &str) -> Result<(), String> {
        let meta = self.meta();
        meta.insert("name", "New Site".to_string())
            .map_err(|e| format!("(init_default_site) Failed to set site name: {}", e))?;
        meta.insert("themeId", theme_id.to_string())
            .map_err(|e| format!("(init_default_site) Failed to set theme ID: {}", e))?;

        // Add page collection
        let mut page_model = Model::new();
        page_model
            .insert(
                "template",
                FieldDefinition {
                    name: "template".to_string(),
                    field_type: FieldType::String,
                    required: true,
                },
            )
            .insert(
                "title",
                FieldDefinition {
                    name: "title".to_string(),
                    field_type: FieldType::String,
                    required: true,
                },
            )
            .insert(
                "body",
                FieldDefinition {
                    name: "body".to_string(),
                    field_type: FieldType::RichText,
                    required: true,
                },
            );

        console_log!(
            "(init_default_site) Adding Page collection {:?}",
            page_model
        );
        self.add_collection::<Page>("page", page_model)?;

        // Add post collection
        let mut post_model = Model::new();
        post_model
            .insert(
                "title",
                FieldDefinition {
                    name: "title".to_string(),
                    field_type: FieldType::String,
                    required: true,
                },
            )
            .insert(
                "body",
                FieldDefinition {
                    name: "body".to_string(),
                    field_type: FieldType::RichText,
                    required: true,
                },
            );

        console_log!(
            "(init_default_site) Adding Post collection {:?}",
            post_model
        );
        self.add_collection::<Post>("post", post_model)?;

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
        console_log!(
            "(init_default_site) Adding Asset collection {:?}",
            asset_model
        );
        self.add_collection::<Asset>("asset", asset_model)?;

        let pm_schema = ProseMirrorSchema {
            marks: HashMap::new(),
            nodes: HashMap::new(),
            top_node: "doc".to_string(),
        };
        // Create default page
        let main_builder: crate::FileBuilder<Page> =
            match self.create_file("main", "page", crate::FileStore::Full(LoroDoc::new())) {
                Ok(builder) => builder.with_pm_schema(pm_schema)?,
                Err(e) => return Err(e),
            };

        let main: Page = self.attach_file(main_builder).await?;
        main.set_title("Hello World Title!")
            .await
            .map_err(|e| format!("(init_default_site) Failed to set page title: {}", e))?;
        console_log!("(init_default_site) Added Page {:?}", main);

        let pm_schema = ProseMirrorSchema {
            marks: HashMap::new(),
            nodes: HashMap::new(),
            top_node: "doc".to_string(),
        };
        // Create default post
        let post_builder: crate::FileBuilder<Post> =
            match self.create_file("test_post", "post", crate::FileStore::Full(LoroDoc::new())) {
                Ok(builder) => builder.with_pm_schema(pm_schema)?,
                Err(e) => return Err(e),
            };

        let post: Post = self.attach_file(post_builder).await?;
        post.set_title("Hello World Title!")
            .await
            .map_err(|e| format!("(init_default_site) Failed to set post title: {}", e))?;
        console_log!("(init_default_site) Added Post {:?}", post);

        self.updated = chrono::Utc::now().timestamp_millis() as f64;

        Ok(())
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn meta(&self) -> LoroMap {
        self.doc.get_map(crate::META_KEY)
    }

    pub fn name(&self) -> Result<String, String> {
        match self.meta().get("name") {
            Some(ValueOrContainer::Value(LoroValue::String(name))) => Ok(name.clone().to_string()),
            _ => Err("Name not found".to_string()),
        }
    }

    pub fn set_name(&mut self, name: &str) -> Result<(), LoroError> {
        let meta = self.meta();
        meta.insert("name", name.to_string())?;
        self.updated = chrono::Utc::now().timestamp_millis() as f64;
        self.doc.commit();
        Ok(())
    }

    pub fn theme_id(&self) -> Option<String> {
        let meta = self.meta();
        match meta.get("themeId") {
            Some(ValueOrContainer::Value(LoroValue::String(theme_id))) => {
                Some(theme_id.clone().to_string())
            }
            _ => None,
        }
    }

    pub fn set_theme_id(&mut self, theme_id: &str) -> Result<(), LoroError> {
        let meta = self.meta();
        meta.insert("themeId", theme_id.to_string())?;
        self.updated = chrono::Utc::now().timestamp_millis() as f64;
        self.doc.commit();
        Ok(())
    }

    // Create a new collection with the specified model
    pub fn add_collection<FileType: File + Default>(
        &mut self,
        name: &str,
        model: Model,
    ) -> Result<Collection<FileType>, String> {
        let collections = self.doc.get_map("collections");
        let collection_map = LoroMap::new();
        collection_map
            .insert("name", name.to_string())
            .expect("Failed to insert name");
        collections
            .insert_container(name, collection_map)
            .expect("Failed to insert collection");

        // Get the inserted map
        match collections.get(name) {
            Some(ValueOrContainer::Container(Container::Map(map))) => map,
            _ => {
                return Err(format!(
                    "Failed to get collection map after insertion: {}",
                    name
                ))
            }
        };

        // Create a builder and add all fields from the model
        let mut builder = Collection::<FileType>::builder(name.to_string()).unwrap();
        for (field_name, field_value) in model.fields.iter() {
            builder = builder.add_field(
                field_name,
                field_value.field_type.clone(),
                field_value.required,
            );
        }

        // Build the collection
        let collection = builder.build(&self.doc)?;

        self.doc.commit();
        self.updated = chrono::Utc::now().timestamp_millis() as f64;

        Ok(collection)
    }

    pub fn get_collection<FileType: File + Default>(
        &self,
        name: &str,
    ) -> Result<Collection<FileType>, String> {
        let collections = self.doc.get_map("collections");
        match collections.get(name) {
            Some(ValueOrContainer::Container(Container::Map(map))) => {
                let collection_builder = Collection::builder(name.to_string())?.with_map(map);
                let collection = collection_builder.build(&self.doc)?;
                Ok(collection)
            }
            _ => Err(format!("Collection not found: {}", name)),
        }
    }

    pub fn get_collections(&self) -> Result<Vec<(String, LoroMap)>, String> {
        let collections = self.doc.get_map("collections");
        let mut result: Vec<(String, LoroMap)> = Vec::new();

        for value in collections.values() {
            if let ValueOrContainer::Container(Container::Map(map)) = value {
                match map.get("name") {
                    Some(ValueOrContainer::Value(LoroValue::String(name))) => {
                        result.push((name.to_string(), map));
                    }
                    _ => {}
                }
            }
        }

        Ok(result)
    }

    pub fn create_file<TFile: File + Default>(
        &mut self,
        name: &str,
        collection_type: &str,
        store: crate::FileStore,
    ) -> Result<crate::FileBuilder<TFile>, String> {
        // Get the collection
        let collection = self.get_collection::<TFile>(collection_type)?;
        // Create a file in the collection
        collection.create_file(name, collection_type, store)
    }

    /// Returns a file that is attached to the collection
    pub async fn attach_file<TFile: File + Default>(
        &mut self,
        file_builder: crate::FileBuilder<TFile>,
    ) -> Result<TFile, String> {
        let collection = self.get_collection::<TFile>(&file_builder.collection_type())?;
        let file = collection
            .attach_file(file_builder)
            .await
            .expect("Failed to attach file");

        self.doc.commit();
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn test_new_theme_project() {
        let project = Project::new(ProjectType::Theme, None).await.unwrap();

        // Check basic properties
        assert!(matches!(project.project_type(), ProjectType::Theme));
        assert!(project.id().len() > 0);
        assert_eq!(project.name().unwrap(), "New Theme");

        // Check collections were created
        let collections = project.get_collections().unwrap();
        let collection_names: Vec<String> =
            collections.iter().map(|(name, _)| name.clone()).collect();
        assert!(collection_names.contains(&"template".to_string()));
        assert!(collection_names.contains(&"partial".to_string()));
        assert!(collection_names.contains(&"text".to_string()));
        assert!(collection_names.contains(&"asset".to_string()));

        // Check default template was created
        let template_collection = project.get_collection::<Template>("template").unwrap();
        let templates = template_collection.get_files("template").await.unwrap();
        assert_eq!(templates.len(), 1);

        // Check default style was created
        let text_collection = project.get_collection::<Text>("text").unwrap();
        let texts = text_collection.get_files("text").await.unwrap();
        assert_eq!(texts.len(), 1);
    }

    #[wasm_bindgen_test]
    async fn test_new_site_project() {
        let theme_id = "test-theme-id";
        let project = Project::new(ProjectType::Site, Some(theme_id.to_string()))
            .await
            .unwrap();

        // Check basic properties
        assert!(matches!(project.project_type(), ProjectType::Site));
        assert!(project.id().len() > 0);
        assert_eq!(project.name().unwrap(), "New Site");
        assert_eq!(project.theme_id().unwrap(), theme_id);

        // Check collections were created
        let collections = project.get_collections().unwrap();
        let collection_names: Vec<String> =
            collections.iter().map(|(name, _)| name.clone()).collect();
        assert!(collection_names.contains(&"page".to_string()));
        assert!(collection_names.contains(&"post".to_string()));
        assert!(collection_names.contains(&"asset".to_string()));

        // Check default page was created
        let page_collection = project.get_collection::<Page>("page").unwrap();
        let pages = page_collection.get_files("page").await.unwrap();
        assert_eq!(pages.len(), 1);

        // Check default post was created
        let post_collection = project.get_collection::<Post>("post").unwrap();
        let posts = post_collection.get_files("post").await.unwrap();
        assert_eq!(posts.len(), 1);
    }

    #[wasm_bindgen_test]
    async fn test_project_metadata() {
        let mut project = Project::new(ProjectType::Theme, None).await.unwrap();

        // Test name update
        let new_name = "Updated Theme";
        project.set_name(new_name).unwrap();
        assert_eq!(project.name().unwrap(), new_name);

        // Test timestamps
        let created = project.created();
        let initial_updated = project.updated();
        assert!(created > 0.0);
        assert!(initial_updated >= created);

        // Make a change and check updated timestamp
        std::thread::sleep(std::time::Duration::from_millis(10));
        project.set_name("Another name").unwrap();
        assert!(project.updated() > initial_updated);
    }

    #[wasm_bindgen_test]
    async fn test_collection_operations() {
        let mut project = Project::new(ProjectType::Theme, None).await.unwrap();

        // Test adding a new collection
        let mut model = Model::new();
        model.insert(
            "test_field",
            FieldDefinition {
                name: "test_field".to_string(),
                field_type: FieldType::String,
                required: true,
            },
        );

        let collection = project
            .add_collection::<Text>("test_collection", model)
            .unwrap();
        assert_eq!(collection.name(), "test_collection");

        // Test retrieving the collection
        let retrieved = project.get_collection::<Text>("test_collection").unwrap();
        assert_eq!(retrieved.name(), "test_collection");

        // Test listing collections
        let collections = project.get_collections().unwrap();
        assert!(collections
            .iter()
            .any(|(name, _)| name == "test_collection"));
    }

    #[wasm_bindgen_test]
    async fn test_file_operations() {
        let mut project = Project::new(ProjectType::Theme, None).await.unwrap();

        // Create and attach a new text file
        let file_builder: crate::FileBuilder<Text> = project
            .create_file("test_text", "text", crate::FileStore::Cache(LoroMap::new()))
            .unwrap();
        let text_file = project.attach_file(file_builder).await.unwrap();

        // Verify the file exists in the collection
        let text_collection = project.get_collection::<Text>("text").unwrap();
        let files = text_collection.get_files("text").await.unwrap();
        assert!(files.iter().any(|f| f.name().unwrap() == "test_text"));
    }

    #[wasm_bindgen_test]
    async fn test_export_import() {
        let original_project = Project::new(ProjectType::Theme, None).await.unwrap();
        let exported_data = original_project.export().unwrap();

        // Import the exported data
        let imported_project = Project::import(
            exported_data,
            original_project.id(),
            original_project.project_type(),
            original_project.created(),
            original_project.updated(),
        )
        .unwrap();

        // Verify the imported project matches the original
        assert_eq!(imported_project.id(), original_project.id());
        assert_eq!(
            imported_project.name().unwrap(),
            original_project.name().unwrap()
        );
        assert_eq!(
            imported_project.project_type(),
            original_project.project_type()
        );
        assert_eq!(imported_project.created(), original_project.created());
        assert_eq!(imported_project.updated(), original_project.updated());
    }

    #[wasm_bindgen_test]
    async fn test_error_cases() {
        // Test creating site without theme ID
        assert!(Project::new(ProjectType::Site, None).await.is_err());

        // Test getting non-existent collection
        let project = Project::new(ProjectType::Theme, None).await.unwrap();
        assert!(project.get_collection::<Text>("non_existent").is_err());
    }
}
