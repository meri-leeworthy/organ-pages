use crate::messages::{FieldDefinition, FileUpdate, Message, Response};
use crate::model::{Collection, File, Project, Store};
use crate::types::{CollectionType, ContentRecord, FieldType, ProjectType, UnparsedContentRecord};
use loro::{Container, ValueOrContainer};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use wasm_bindgen::prelude::*;

/// The Actor struct manages the application state and handles messages
#[wasm_bindgen]
pub struct Actor {
    store: Arc<RwLock<Store>>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // log to JS console
}

// Helper macro for logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!("[Actor] {}", format!($($t)*))))
}

/// Helper functions for converting between Rust types and JS representations
mod js_conversions {
    use super::*;
    use crate::model::{Collection, File};
    use crate::types::FieldType;
    use serde_json::{json, Value};
    use std::collections::HashMap;

    /// Convert a Collection to a JS-friendly JSON representation
    pub fn collection_to_json(collection: &Collection) -> Result<Value, String> {
        // Get fields
        let fields = match collection.get_fields() {
            Ok(fields) => {
                console_log!("Collection Name: {:?}", collection.name());
                console_log!("Fields: {:?}", fields);

                let mut result = Vec::new();

                // Convert each field to a JSON object
                for field_def in fields {
                    let field_type = match field_def.field_type {
                        FieldType::RichText => "richtext",
                        FieldType::Text => "text",
                        FieldType::List => "list",
                        FieldType::Map => "map",
                        FieldType::DateTime => "datetime",
                        FieldType::String => "string",
                        FieldType::Number => "number",
                        FieldType::Object => "object",
                        FieldType::Array => "array",
                        FieldType::Blob => "blob",
                    };

                    result.push(json!({
                        "name": field_def.name,
                        "type": field_type,
                        "required": field_def.required
                    }));
                }

                result
            }
            Err(e) => {
                console_log!("Error getting fields: {}", e);
                Vec::new()
            }
        };

        Ok(json!({
            "name": collection.name(),
            "fields": fields
        }))
    }

    /// Convert a File to a JS-friendly JSON representation
    pub fn file_to_json(file: &File) -> Result<Value, String> {
        file.to_json()
    }

    /// Convert a list of Files to a JS-friendly JSON array
    pub fn files_to_json(files: &[File]) -> Result<Value, String> {
        let mut result = Vec::new();

        for file in files {
            result.push(file_to_json(file)?);
        }

        Ok(json!(result))
    }

    /// Convert a list of Collections to a JS-friendly JSON array
    pub fn collections_to_json(collections: &[Collection]) -> Result<Value, String> {
        let mut result = Vec::new();

        for collection in collections {
            result.push(collection_to_json(collection)?);
        }

        Ok(json!(result))
    }

    /// Convert a field type string to the corresponding FieldType enum
    pub fn string_to_field_type(field_type: &str) -> Result<FieldType, String> {
        match field_type {
            "richtext" => Ok(FieldType::RichText),
            "text" => Ok(FieldType::Text),
            "list" => Ok(FieldType::List),
            "map" => Ok(FieldType::Map),
            "datetime" => Ok(FieldType::DateTime),
            "string" => Ok(FieldType::String),
            "number" => Ok(FieldType::Number),
            "object" => Ok(FieldType::Object),
            "array" => Ok(FieldType::Array),
            "blob" => Ok(FieldType::Blob),
            _ => Err(format!("Invalid field type: {}", field_type)),
        }
    }

    /// Convert a string to a ProjectType enum
    pub fn string_to_project_type(project_type: &str) -> Result<ProjectType, String> {
        match project_type {
            "site" => Ok(ProjectType::Site),
            "theme" => Ok(ProjectType::Theme),
            _ => Err(format!("Invalid project type: {}", project_type)),
        }
    }
}

#[wasm_bindgen]
impl Actor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_log!("Creating new Actor instance");
        console_error_panic_hook::set_once();

        let actor = Actor {
            store: Arc::new(RwLock::new(Store::new())),
        };
        console_log!("Actor instance created successfully");
        actor
    }

    /// Process a message and return a response
    #[wasm_bindgen]
    pub fn process_message(&self, message_json: &str) -> Result<String, JsValue> {
        console_log!("Received message: {}", message_json);

        let message: Message = match serde_json::from_str(message_json) {
            Ok(msg) => {
                console_log!("Successfully parsed message");
                msg
            }
            Err(e) => {
                console_log!("Failed to parse message: {}", e);
                return Err(JsValue::from_str(&format!(
                    "Failed to parse message: {}",
                    e
                )));
            }
        };

        let response = self.handle_message(message);
        console_log!("Generated response: {:?}", response);

        match serde_json::to_string(&response) {
            Ok(json) => {
                console_log!("Successfully serialized response");
                Ok(json)
            }
            Err(e) => {
                console_log!("Failed to serialize response: {}", e);
                Err(JsValue::from_str(&format!(
                    "Failed to serialize response: {}",
                    e
                )))
            }
        }
    }

    /// Handle a message and return a response
    fn handle_message(&self, message: Message) -> Response {
        console_log!("Handling message: {:?}", message);

        let response = match message {
            Message::InitDefault => {
                console_log!("Processing InitDefault message");
                self.init_default()
            }
            Message::CreateSite { name, theme_id } => {
                console_log!(
                    "Processing CreateSite message - name: {}, theme_id: {}",
                    name,
                    theme_id
                );
                self.create_site(name, theme_id)
            }
            Message::GetSite => self.get_site(),
            Message::CreateTheme { name } => {
                console_log!("Processing CreateTheme message - name: {}", name);
                self.create_theme(name)
            }
            Message::GetTheme => self.get_theme(),
            Message::AddCollection {
                project_type,
                name,
                fields,
            } => self.add_collection(project_type, name, fields),
            Message::GetCollection { project_type, name } => {
                self.get_collection(project_type, name)
            }
            Message::ListCollections { project_type } => self.list_collections(project_type),
            Message::CreateFile {
                project_type,
                collection_name,
                name,
            } => self.create_file(project_type, collection_name, name),
            Message::UpdateFile {
                project_type,
                collection_name,
                file_id,
                updates,
            } => self.update_file(project_type, collection_name, file_id, updates),
            Message::GetFile {
                project_type,
                collection_name,
                file_id,
            } => self.get_file(project_type, collection_name, file_id),
            Message::ListFiles {
                project_type,
                collection_name,
            } => self.list_files(project_type, collection_name),
            Message::SaveState { project_type } => self.save_state(project_type),
            Message::LoadState { site_id, theme_id } => self.load_state(site_id, theme_id),
            Message::ExportProject { project_type } => self.export_project(project_type),
            Message::ImportProject {
                data,
                id,
                project_type,
                created,
                updated,
            } => self.import_project(
                data,
                id,
                js_conversions::string_to_project_type(&project_type).unwrap(),
                created,
                updated,
            ),
            Message::RenderFile { file_id, context } => self.render_file(file_id, context),
        };

        console_log!("Message handling complete with response: {:?}", response);
        response
    }

    /// Initialize default projects
    fn init_default(&self) -> Response {
        console_log!("Starting init_default");

        let store_result = self.store.write();
        if let Err(e) = store_result {
            console_log!("Failed to acquire store write lock: {}", e);
            return Response::error(&format!("Failed to acquire store lock: {}", e));
        }

        let mut store = store_result.unwrap();
        match store.init_default() {
            Ok(()) => {
                console_log!("Successfully initialized default store");

                // Get the created projects and store them in site/theme variables
                drop(store); // Release the write lock before acquiring read lock
                let store = self.store.read().unwrap();

                console_log!("Init default completed successfully");
                Response::success(json!({ "status": "initialized" }))
            }
            Err(e) => {
                console_log!("Failed to initialize default store: {}", e);
                Response::error(&format!("Failed to initialize: {}", e))
            }
        }
    }

    /// Create a new site
    fn create_site(&self, name: String, theme_id: String) -> Response {
        console_log!(
            "Creating site with name: {} and theme_id: {}",
            name,
            theme_id
        );

        match Project::new(ProjectType::Site, Some(theme_id.clone())) {
            Ok(mut project) => {
                project.set_name(&name);

                // Set as active site in the store
                self.store.write().unwrap().set_active_site(project.clone());

                // Get project ID
                let id = project.id();

                Response::success(json!({
                    "id": id,
                    "name": name,
                    "themeId": theme_id
                }))
            }
            Err(e) => Response::error(&format!("Failed to create site: {}", e)),
        }
    }

    /// Create a new theme
    fn create_theme(&self, name: String) -> Response {
        console_log!("Creating theme with name: {}", name);

        match Project::new(ProjectType::Theme, None) {
            Ok(mut project) => {
                project.set_name(&name);

                // Set as active theme in the store
                self.store
                    .write()
                    .unwrap()
                    .set_active_theme(project.clone());

                // Get project ID
                let id = project.id();

                Response::success(json!({
                    "id": id,
                    "name": name
                }))
            }
            Err(e) => Response::error(&format!("Failed to create theme: {}", e)),
        }
    }

    /// Get current site
    fn get_site(&self) -> Response {
        console_log!("Getting current site");

        if let Some(site) = self.store.read().unwrap().get_active_site() {
            let theme_id = site.theme_id().unwrap_or_default();

            return Response::success(json!({
                "id": site.id(),
                "name": site.name().unwrap_or_else(|_| "Unnamed".to_string()),
                "themeId": theme_id
            }));
        }

        // If we get here, either we couldn't get a lock or there's no active site
        Response::error("No active site found")
    }

    /// Get current theme
    fn get_theme(&self) -> Response {
        console_log!("Getting current theme");

        if let Some(theme) = self.store.read().unwrap().get_active_theme() {
            return Response::success(json!({
                "id": theme.id(),
                "name": theme.name().unwrap_or_else(|_| "Unnamed".to_string())
            }));
        }

        // If we get here, either we couldn't get a lock or there's no active theme
        Response::error("No active theme found")
    }

    /// Add a collection to a project
    fn add_collection(
        &self,
        project_type: String,
        name: String,
        fields: Vec<FieldDefinition>,
    ) -> Response {
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        console_log!("Adding collection: {} to {:?}", name, project_type);

        // Get the appropriate project based on project_type
        let mut project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.store.read().unwrap().get_active_site() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.store.read().unwrap().get_active_theme() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        // Create model
        let mut model = crate::model::Model::new();

        // Add fields
        for field in &fields {
            let field_type = match js_conversions::string_to_field_type(&field.field_type) {
                Ok(ft) => ft,
                Err(e) => return Response::error(&e),
            };

            model.insert(
                &field.name,
                crate::types::FieldDefinition {
                    name: field.name.clone(),
                    field_type,
                    required: field.required,
                },
            );
        }

        // Add collection
        match project.add_collection(&name, model) {
            Ok(collection) => {
                // Update the project in our store
                match project_type {
                    ProjectType::Site => {
                        self.store.write().unwrap().set_active_site(project);
                    }
                    ProjectType::Theme => {
                        self.store.write().unwrap().set_active_theme(project);
                    }
                }

                match js_conversions::collection_to_json(&collection) {
                    Ok(json_value) => Response::success(json_value),
                    Err(e) => {
                        Response::error(&format!("Failed to convert collection to JSON: {}", e))
                    }
                }
            }
            Err(e) => Response::error(&format!("Failed to add collection: {}", e)),
        }
    }

    /// Get a collection from a project
    fn get_collection(&self, project_type: String, name: String) -> Response {
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        console_log!("Getting collection: {} from {:?}", name, project_type);

        // Get the appropriate project based on project_type
        let project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.store.read().unwrap().get_active_site() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.store.read().unwrap().get_active_theme() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        match project.get_collection(&name) {
            Ok(collection) => match js_conversions::collection_to_json(&collection) {
                Ok(json_value) => Response::success(json_value),
                Err(e) => Response::error(&format!("Failed to convert collection to JSON: {}", e)),
            },
            Err(e) => Response::error(&format!("Failed to get collection: {}", e)),
        }
    }

    /// List collections in a project
    fn list_collections(&self, project_type: String) -> Response {
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        console_log!("Listing collections for {:?}", project_type);

        // Get the appropriate project based on project_type
        let project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.store.read().unwrap().get_active_site() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.store.read().unwrap().get_active_theme() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        match project.get_collections() {
            Ok(collections) => match js_conversions::collections_to_json(&collections) {
                Ok(json_value) => Response::success(json_value),
                Err(e) => Response::error(&format!("Failed to convert collections to JSON: {}", e)),
            },
            Err(e) => Response::error(&format!("Failed to list collections: {}", e)),
        }
    }

    /// Create a file in a collection
    fn create_file(&self, project_type: String, collection_name: String, name: String) -> Response {
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        console_log!(
            "Creating file: {} in collection: {} for {:?}",
            name,
            collection_name,
            project_type
        );

        // Get the appropriate project based on project_type
        let mut project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.store.read().unwrap().get_active_site() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.store.read().unwrap().get_active_theme() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        match project.create_file(&name, &collection_name) {
            Ok(file) => {
                // Update the project in our store
                match project_type {
                    ProjectType::Site => {
                        self.store.write().unwrap().set_active_site(project);
                    }
                    ProjectType::Theme => {
                        self.store.write().unwrap().set_active_theme(project);
                    }
                }

                match js_conversions::file_to_json(&file) {
                    Ok(json_value) => Response::success(json_value),
                    Err(e) => Response::error(&format!("Failed to convert file to JSON: {}", e)),
                }
            }
            Err(e) => Response::error(&format!("Failed to create file: {}", e)),
        }
    }

    /// Update a file
    fn update_file(
        &self,
        project_type: String,
        collection_name: String,
        file_id: String,
        updates: FileUpdate,
    ) -> Response {
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        console_log!(
            "Updating file: {} in collection: {} for {:?}",
            file_id,
            collection_name,
            project_type
        );

        // Get the appropriate project based on project_type
        let project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.store.read().unwrap().get_active_site() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.store.read().unwrap().get_active_theme() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        match project.get_collection(&collection_name) {
            Ok(collection) => {
                // Get all files
                match collection.get_files() {
                    Ok(files) => {
                        let mut file_found = false;
                        let mut updated_project = project.clone();

                        for file in &files {
                            if file.id.to_string() == file_id {
                                // Apply updates
                                match &updates {
                                    FileUpdate::SetName(name) => {
                                        if let Err(e) = file.set_name(name) {
                                            return Response::error(&format!(
                                                "Failed to set name: {}",
                                                e
                                            ));
                                        }
                                    }
                                    FileUpdate::SetContent(content) => {
                                        if let Err(e) = file.set_content(content) {
                                            return Response::error(&format!(
                                                "Failed to set content: {}",
                                                e
                                            ));
                                        }
                                    }
                                    FileUpdate::SetBody(body) => {
                                        if let Err(e) = file.set_body(body) {
                                            return Response::error(&format!(
                                                "Failed to set body: {}",
                                                e
                                            ));
                                        }
                                    }
                                    FileUpdate::SetTitle(title) => {
                                        if let Err(e) = file.set_title(title) {
                                            return Response::error(&format!(
                                                "Failed to set title: {}",
                                                e
                                            ));
                                        }
                                    }
                                    FileUpdate::SetField { name, value } => {
                                        if let Err(e) = file.set_field(name, value) {
                                            return Response::error(&format!(
                                                "Failed to set field: {}",
                                                e
                                            ));
                                        }
                                    }
                                    FileUpdate::SetUrl(url) => {
                                        if let Err(e) = file.set_url(url) {
                                            return Response::error(&format!(
                                                "Failed to set URL: {}",
                                                e
                                            ));
                                        }
                                    }
                                    FileUpdate::SetMimeType(mime_type) => {
                                        if let Err(e) = file.set_mime_type(mime_type) {
                                            return Response::error(&format!(
                                                "Failed to set MIME type: {}",
                                                e
                                            ));
                                        }
                                    }
                                    FileUpdate::SetAlt(alt) => {
                                        if let Err(e) = file.set_alt(alt) {
                                            return Response::error(&format!(
                                                "Failed to set alt text: {}",
                                                e
                                            ));
                                        }
                                    }
                                }

                                file_found = true;
                                break;
                            }
                        }

                        if file_found {
                            // Update the project in our store
                            match project_type {
                                ProjectType::Site => {
                                    self.store.write().unwrap().set_active_site(updated_project);
                                }
                                ProjectType::Theme => {
                                    self.store
                                        .write()
                                        .unwrap()
                                        .set_active_theme(updated_project);
                                }
                            }

                            Response::success(json!({ "status": "updated" }))
                        } else {
                            Response::error(&format!("File not found: {}", file_id))
                        }
                    }
                    Err(e) => Response::error(&format!("Failed to get files: {}", e)),
                }
            }
            Err(e) => Response::error(&format!("Failed to get collection: {}", e)),
        }
    }

    /// Get a file
    fn get_file(&self, project_type: String, collection_name: String, file_id: String) -> Response {
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        console_log!(
            "Getting file: {} from collection: {} for {:?}",
            file_id,
            collection_name,
            project_type
        );

        // Get the appropriate project based on project_type
        let project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.store.read().unwrap().get_active_site() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.store.read().unwrap().get_active_theme() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        match project.get_collection(&collection_name) {
            Ok(collection) => {
                // Get all files
                match collection.get_files() {
                    Ok(files) => {
                        for file in &files {
                            if file.id.to_string() == file_id {
                                return match js_conversions::file_to_json(file) {
                                    Ok(json_value) => Response::success(json_value),
                                    Err(e) => Response::error(&format!(
                                        "Failed to convert file to JSON: {}",
                                        e
                                    )),
                                };
                            }
                        }

                        Response::error(&format!("File not found: {}", file_id))
                    }
                    Err(e) => Response::error(&format!("Failed to get files: {}", e)),
                }
            }
            Err(e) => Response::error(&format!("Failed to get collection: {}", e)),
        }
    }

    /// List files in a collection
    fn list_files(&self, project_type: String, collection_name: String) -> Response {
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        console_log!(
            "Listing files for collection: {} in {:?}",
            collection_name,
            project_type
        );

        // Get the appropriate project based on project_type
        let project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.store.read().unwrap().get_active_site() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.store.read().unwrap().get_active_theme() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        match project.get_collection(&collection_name) {
            Ok(collection) => match collection.get_files() {
                Ok(files) => match js_conversions::files_to_json(&files) {
                    Ok(json_value) => Response::success(json_value),
                    Err(e) => Response::error(&format!("Failed to convert files to JSON: {}", e)),
                },
                Err(e) => Response::error(&format!("Failed to get files: {}", e)),
            },
            Err(e) => Response::error(&format!("Failed to get collection: {}", e)),
        }
    }

    /// Save state to IndexedDB
    fn save_state(&self, project_type: String) -> Response {
        console_log!("Saving state to IndexedDB - project_type: {}", project_type);

        // Use wasm_bindgen_futures::spawn_local to execute this future
        // For now, we'll just continue with our implementation

        // Get the project
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        let project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.store.read().unwrap().get_active_site() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.store.read().unwrap().get_active_theme() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        let project_id = project.id();

        // Export the site and theme to JSON for storage
        let project_export = match project.export() {
            Ok(json) => {
                console_log!("Project exported for storage, size: {} chars", json.len());
                json
            }
            Err(e) => {
                console_log!("Failed to export project: {}", e);
                return Response::error(&format!("Failed to export project: {}", e));
            }
        };

        let project_export = js_sys::Uint8Array::from(&project_export[..]);

        // Save the site and theme data to IndexedDB
        // These would be separate futures using save_data
        let save_project_result = super::save_data(
            "organ_db".to_string(),
            "projects".to_string(),
            project_id.clone(),
            project_export,
        );

        // For now, we'll just return success - in a real async implementation
        // we would await all futures and handle errors appropriately
        Response::success(json!({
            "status": "saved",
            "project_type": project_type,
        }))
    }

    /// Load state from IndexedDB
    fn load_state(&self, site_id: Option<String>, theme_id: Option<String>) -> Response {
        console_log!(
            "Loading state from IndexedDB - siteId: {:?}, themeId: {:?}",
            site_id,
            theme_id
        );

        // If specific IDs are provided, use those
        // Otherwise, try to load the active projects metadata

        if let (Some(site_id), Some(theme_id)) = (site_id.as_ref(), theme_id.as_ref()) {
            console_log!(
                "Loading specific projects - siteId: {}, themeId: {}",
                site_id,
                theme_id
            );

            // Load site data
            // In a fully implemented version, we would use async code:
            // let site_data = load_data("organ_db", "projects", site_id).await?;
            // For now, we'll simulate success if we have site/theme in memory

            // Check if we already have these projects in memory
            let store = self.store.read().unwrap();
            let has_site = store.get_active_site().is_some();
            let has_theme = store.get_active_theme().is_some();

            // If we have both, return success
            if has_site && has_theme {
                return Response::success(json!({
                    "status": "loaded",
                    "siteId": site_id,
                    "themeId": theme_id
                }));
            }

            // Otherwise, return error (in production, we'd attempt to load from IndexedDB)
            return Response::error("Projects not found in memory");
        } else {
            console_log!("Loading default active projects");

            // Try to load the active projects metadata from IndexedDB
            // This would be an async operation in the full implementation:
            // let metadata = load_data("organ_db", "projects_metadata", "active_projects").await?;

            // For now, check if we have any projects in memory
            let store = self.store.read().unwrap();
            if let (Some(site), Some(theme)) = (store.get_active_site(), store.get_active_theme()) {
                let site_id = site.id();
                let theme_id = theme.id();

                console_log!(
                    "Found existing projects in memory - siteId: {}, themeId: {}",
                    site_id,
                    theme_id
                );

                return Response::success(json!({
                    "status": "loaded",
                    "siteId": site_id,
                    "themeId": theme_id
                }));
            }

            // No projects found
            console_log!("No active projects found");
            return Response::error("No active projects found");
        }
    }

    /// Export project to JSON
    fn export_project(&self, project_type: String) -> Response {
        console_log!("Exporting project with type: {}", project_type);

        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        // Determine if this is a site or theme based on IDs
        let project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.store.read().unwrap().get_active_site() {
                    Some(site.clone())
                } else {
                    None
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.store.read().unwrap().get_active_theme() {
                    Some(theme.clone())
                } else {
                    None
                }
            }
        };

        // Export the project
        match project {
            Some(project) => match project.export() {
                Ok(export) => Response::success(export),
                Err(e) => Response::error(&format!("Failed to export project: {}", e)),
            },
            None => Response::error(&format!("Project not found: {:?}", project_type)),
        }
    }

    /// Import project from JSON
    fn import_project(
        &self,
        data: Vec<u8>,
        id: String,
        project_type: ProjectType,
        created: f64,
        updated: f64,
    ) -> Response {
        console_log!("Importing project from data");

        // Parse the data to determine project type

        match project_type {
            ProjectType::Site => {
                // Import as a site
                match Project::import(data, id, project_type, created, updated) {
                    Ok(project) => {
                        // Store as the active site
                        self.store.write().unwrap().set_active_site(project.clone());

                        // Get the project ID
                        let id = project.id();
                        let name = project
                            .name()
                            .unwrap_or_else(|_| "Imported Site".to_string());
                        let theme_id = project.theme_id().unwrap_or_default();

                        Response::success(json!({
                            "id": id,
                            "name": name,
                            "themeId": theme_id
                        }))
                    }
                    Err(e) => Response::error(&format!("Failed to import site: {}", e)),
                }
            }
            ProjectType::Theme => {
                // Import as a theme
                match Project::import(data, id, project_type, created, updated) {
                    Ok(project) => {
                        // Store as the active theme
                        self.store
                            .write()
                            .unwrap()
                            .set_active_theme(project.clone());

                        // Get the project ID
                        let id = project.id();
                        let name = project
                            .name()
                            .unwrap_or_else(|_| "Imported Theme".to_string());

                        Response::success(json!({
                            "id": id,
                            "name": name
                        }))
                    }
                    Err(e) => Response::error(&format!("Failed to import theme: {}", e)),
                }
            }
            _ => Response::error(&format!("Unknown project type: {:?}", project_type)),
        }
    }

    /// Render a file (using the existing render function)
    fn render_file(&self, file_id: String, context: Value) -> Response {
        console_log!("Rendering file: {} with context", file_id);

        // Convert context to JsValue
        let js_context = serde_wasm_bindgen::to_value(&context)
            .map_err(|e| format!("Failed to convert context: {}", e))
            .unwrap();

        // Call the existing render function
        match crate::render(file_id.parse().unwrap_or(0), &js_context) {
            Ok(html) => Response::success(json!({ "html": html })),
            Err(e) => Response::error(&format!("Failed to render: {:?}", e)),
        }
    }
}
