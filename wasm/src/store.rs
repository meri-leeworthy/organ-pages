use crate::messages::{FileUpdate, Message, Response};
use crate::model::collection::Collection;
use crate::model::file::{File, HasTitle, HasUrl};
use crate::model::project::Project;
use crate::model::{Asset, Page, Partial, Post, Template, Text};
use crate::types::{FileType, ProjectType};
use crate::{js_conversions::*, FileStore, ProseMirrorSchema};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

mod tests;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // log to JS console
}

// Helper macro for logging
macro_rules! console_log {
($($t:tt)*) => (log(&format!("[Store (WASM)] {}", format!($($t)*))))
}

/// Store: The main entry point for our data model
#[wasm_bindgen]
pub struct Store {
    active_site: Arc<Mutex<Option<Project>>>,
    active_theme: Arc<Mutex<Option<Project>>>,
    active_file: Arc<Mutex<Option<FileType>>>,
}

#[wasm_bindgen]
impl Store {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_log!("Creating new Store instance");
        console_error_panic_hook::set_once();

        let actor = Store {
            active_theme: Arc::new(Mutex::new(None)),
            active_site: Arc::new(Mutex::new(None)),
            active_file: Arc::new(Mutex::new(None)),
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
            // Message::AddCollection {
            //     project_type,
            //     name,
            //     fields,
            // } => self.add_collection(project_type, name, fields),
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
            // Message::RenderFile { file_id, context } => self.render_file(file_id, context),

            // Document operations for ProseMirror integration
            // TODO: Rename to GetActiveFile
            Message::GetDocument { document_id } => {
                console_log!("Processing GetDocument message - id: {}", document_id);
                self.get_active_file()
            } // Message::ApplySteps {
              //     document_id,
              //     steps,
              //     version,
              // } => {
              //     console_log!(
              //         "Processing ApplySteps message - id: {}, version: {}",
              //         document_id,
              //         version
              //     );
              //     self.apply_steps(document_id, steps, version)
              // }
        };

        console_log!("Message handling complete with response: {:?}", response);
        response
    }

    /// Initialize default projects
    fn init_default(&self) -> Response {
        console_log!("Initializing default projects");
        // Create default theme
        let default_theme = match Project::new(ProjectType::Theme, None) {
            Ok(theme) => theme,
            Err(error) => {
                return Response::Error(format!("Failed to create default theme: {}", error))
            }
        };

        let theme_id = default_theme.id();

        // Create default site
        let default_site = match Project::new(ProjectType::Site, Some(theme_id)) {
            Ok(site) => site,
            Err(error) => {
                return Response::Error(format!("Failed to create default site: {}", error))
            }
        };

        // Set active projects
        {
            let mut active_theme = self.active_theme.lock().unwrap();
            *active_theme = Some(default_theme);
        }
        {
            let mut active_site = self.active_site.lock().unwrap();
            *active_site = Some(default_site);
        }

        console_log!("Successfully initialized default store");
        Response::success(json!({ "status": "initialized" }))
    }

    fn set_theme(&self, theme: Project) -> Result<(), String> {
        console_log!("Setting theme: {:?}", theme);
        let mut active_theme = match self.active_theme.lock() {
            Ok(lock) => lock,
            Err(poisoned) => {
                console_log!("Lock is poisoned: {:?}", poisoned);
                return Err("Failed to acquire lock".to_string());
            }
        };
        *active_theme = Some(theme);
        drop(active_theme); // Explicitly release the lock here
        Ok(())
    }

    fn set_site(&self, site: Project) -> Result<(), String> {
        console_log!("Setting site: {:?}", site);
        let mut active_site = match self.active_site.lock() {
            Ok(lock) => lock,
            Err(poisoned) => {
                console_log!("Lock is poisoned: {:?}", poisoned);
                return Err("Failed to acquire lock".to_string());
            }
        };
        *active_site = Some(site);
        drop(active_site); // Explicitly release the lock here
        Ok(())
    }

    fn export_to_json(&self) -> Result<Value, String> {
        // TODO: Implement export
        Ok(serde_json::json!({}))
    }

    fn import_from_json(&self, _data: Value) -> Result<(), String> {
        // TODO: Implement import
        Ok(())
    }

    /// ACTOR Create a new site
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
                self.set_site(project.clone());

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

    /// ACTOR Create a new theme
    fn create_theme(&self, name: String) -> Response {
        console_log!("Creating theme with name: {}", name);

        match Project::new(ProjectType::Theme, None) {
            Ok(mut project) => {
                project.set_name(&name);

                // Set as active theme in the store
                self.set_theme(project.clone());

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

    /// ACTOR Get current site
    fn get_site(&self) -> Response {
        console_log!("Getting current site");

        if let Some(site) = self.active_site.lock().unwrap().clone() {
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

    /// ACTOR Get current theme
    fn get_theme(&self) -> Response {
        console_log!("Getting current theme");

        if let Some(theme) = self.active_theme.lock().unwrap().clone() {
            return Response::success(json!({
                "id": theme.id(),
                "name": theme.name().unwrap_or_else(|_| "Unnamed".to_string())
            }));
        }

        // If we get here, either we couldn't get a lock or there's no active theme
        Response::error("No active theme found")
    }

    /// ACTOR add a collection to a project
    // fn add_collection(
    //     &self,
    //     project_type: String,
    //     name: String,
    //     fields: Vec<FieldDefinition>,
    // ) -> Response {
    //     let project_type = match js_conversions::string_to_project_type(&project_type) {
    //         Ok(pt) => pt,
    //         Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
    //     };
    //     console_log!("Adding collection: {} to {:?}", name, project_type);
    //     // Get the appropriate project based on project_type
    //     let mut project = match project_type {
    //         ProjectType::Site => {
    //             if let Some(site) = self.active_site.read().unwrap().clone() {
    //                 site.clone()
    //             } else {
    //                 return Response::error("Failed to acquire site lock");
    //             }
    //         }
    //         ProjectType::Theme => {
    //             if let Some(theme) = self.active_theme.read().unwrap().clone() {
    //                 theme.clone()
    //             } else {
    //                 return Response::error("Failed to acquire theme lock");
    //             }
    //         }
    //     };
    //     // Create model
    //     let mut model = crate::model::Model::new();
    //     // Add fields
    //     for field in &fields {
    //         let field_type =
    //             match js_conversions::string_to_field_type(&field.field_type.to_string()) {
    //                 Ok(ft) => ft,
    //                 Err(e) => return Response::error(&e),
    //             };
    //         model.insert(
    //             &field.name,
    //             crate::types::FieldDefinition {
    //                 name: field.name.clone(),
    //                 field_type,
    //                 required: field.required,
    //             },
    //         );
    //     }
    //     // Add collection
    //     match project.add_collection(&name, model) {
    //         Ok(collection) => {
    //             // Update the project in our store
    //             match project_type {
    //                 ProjectType::Site => {
    //                     self.set_site(project);
    //                 }
    //                 ProjectType::Theme => {
    //                     self.set_theme(project);
    //                 }
    //             }
    //             match js_conversions::collection_to_json(&collection) {
    //                 Ok(json_value) => Response::success(json_value),
    //                 Err(e) => {
    //                     Response::error(&format!("Failed to convert collection to JSON: {}", e))
    //                 }
    //             }
    //         }
    //         Err(e) => Response::error(&format!("Failed to add collection: {}", e)),
    //     }
    // }

    /// ACTOR get a collection from a project
    fn get_collection(&self, project_type: String, name: String) -> Response {
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        console_log!("Getting collection: {} from {:?}", name, project_type);

        // Get the appropriate project based on project_type
        let project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.active_site.lock().unwrap().clone() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.active_theme.lock().unwrap().clone() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        fn get_collection_generic<T: File + Default>(project: &Project, name: &str) -> Response {
            let collection = match project.get_collection::<T>(name) {
                Ok(collection) => collection,
                Err(e) => return Response::error(&format!("Failed to get collection: {}", e)),
            };

            match js_conversions::collection_to_json(&collection) {
                Ok(json_value) => Response::success(json_value),
                Err(e) => Response::error(&format!("Failed to convert collection to JSON: {}", e)),
            }
        }

        match name.as_str() {
            "page" => get_collection_generic::<Page>(&project, &name),
            "post" => get_collection_generic::<Post>(&project, &name),
            "asset" => get_collection_generic::<Asset>(&project, &name),
            "template" => get_collection_generic::<Template>(&project, &name),
            "partial" => get_collection_generic::<Partial>(&project, &name),
            "text" => get_collection_generic::<Text>(&project, &name),
            _ => Response::error(&format!("Collection not found: {}", name)),
        }
    }

    /// ACTOR List collections in a project
    ///
    /// Returns a JSON list of collections in the project
    fn list_collections(&self, project_type: String) -> Response {
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        console_log!("Listing collections for {:?}", project_type);

        // Get the appropriate project based on project_type
        let project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.active_site.lock().unwrap().clone() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.active_theme.lock().unwrap().clone() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        let collections = match project.get_collections() {
            Ok(collections) => collections,
            Err(e) => return Response::error(&format!("Failed to get collections: {}", e)),
        };

        let collections: Vec<Result<Value, String>> = collections
            .iter()
            .map(|(name, map)| match name.as_str() {
                "page" => {
                    let collection = Collection::<Page>::builder(name.clone())?.build_detached()?;
                    js_conversions::collection_to_json(&collection)
                }
                "post" => {
                    let collection = Collection::<Post>::builder(name.clone())?.build_detached()?;
                    js_conversions::collection_to_json(&collection)
                }
                "asset" => {
                    let collection =
                        Collection::<Asset>::builder(name.clone())?.build_detached()?;
                    js_conversions::collection_to_json(&collection)
                }
                "template" => {
                    let collection =
                        Collection::<Template>::builder(name.clone())?.build_detached()?;
                    js_conversions::collection_to_json(&collection)
                }
                "partial" => {
                    let collection =
                        Collection::<Partial>::builder(name.clone())?.build_detached()?;
                    js_conversions::collection_to_json(&collection)
                }
                "text" => {
                    let collection = Collection::<Text>::builder(name.clone())?.build_detached()?;
                    js_conversions::collection_to_json(&collection)
                }
                _ => Err(format!("Collection not found: {}", name)),
            })
            .collect();

        return Response::success(json!(collections));

        // match js_conversions::collections_to_json(&collections) {
        //     Ok(json_value) => Response::success(json_value),
        //     Err(e) => Response::error(&format!("Failed to convert collections to JSON: {}", e)),
        // }
    }

    fn create_file_generic<T: File + Default + Debug>(
        &self,
        project_type: ProjectType,
        collection_name: &str,
        name: &str,
        pm_schema: Option<ProseMirrorSchema>,
    ) -> Response {
        let mut guard = match project_type {
            ProjectType::Site => self
                .active_site
                .lock()
                .expect("Failed to acquire write lock"),
            ProjectType::Theme => self
                .active_theme
                .lock()
                .expect("Failed to acquire write lock"),
        };

        if let Some(ref mut project) = *guard {
            // Perform all operations on project while holding single lock
            let mut file_builder = match project.create_file::<T>(&name, &collection_name) {
                Ok(file) => file,
                Err(e) => return Response::error(&format!("Failed to create file: {}", e)),
            };

            if let Some(pm_schema) = pm_schema {
                file_builder = match file_builder.with_pm_schema(pm_schema) {
                    Ok(builder) => builder,
                    Err(e) => return Response::error(&format!("Failed to set PM schema: {}", e)),
                };
            }

            let file = match project.attach_file(file_builder) {
                Ok(f) => f,
                Err(_) => return Response::error("Failed to attach file"),
            };

            match js_conversions::file_to_json::<T>(&file) {
                Ok(json_value) => Response::success(json_value),
                Err(e) => Response::error(&format!("Failed to convert file to JSON: {}", e)),
            }
        } else {
            Response::error("No active project")
        }
    }

    /// ACTOR Create a file in a collection
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

        match collection_name.as_str() {
            "page" => self.create_file_generic::<Page>(
                project_type,
                &collection_name,
                &name,
                Some(ProseMirrorSchema::default()),
            ),
            "post" => self.create_file_generic::<Post>(
                project_type,
                &collection_name,
                &name,
                Some(ProseMirrorSchema::default()),
            ),
            "asset" => {
                self.create_file_generic::<Asset>(project_type, &collection_name, &name, None)
            }
            "template" => {
                self.create_file_generic::<Template>(project_type, &collection_name, &name, None)
            }
            "partial" => {
                self.create_file_generic::<Partial>(project_type, &collection_name, &name, None)
            }
            "text" => self.create_file_generic::<Text>(project_type, &collection_name, &name, None),
            _ => Response::error(&format!("Collection not found: {}", collection_name)),
        }
    }

    /// What this needs to do (or maybe what File needs to do)
    /// is update data directly on the File LoroDoc (which is kept in store)
    /// and to also update the metadata cache on the collection files tree
    ///
    /// it might make sense to have a method for updating the active file only
    /// rather than being able to update any arbitrary file
    ///
    fn update_file(
        &self,
        project_type: String,
        collection_name: String,
        file_id: String,
        update: FileUpdate,
    ) -> Response {
        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        console_log!(
            "⏳ Updating file: {} in collection: {} for {:?} with update: {:?}",
            file_id,
            collection_name,
            project_type,
            update
        );

        // Get the appropriate project based on project_type
        // Get a reference to the project instead of cloning
        let guard = match project_type {
            ProjectType::Site => self.active_site.lock().unwrap(),
            ProjectType::Theme => self.active_theme.lock().unwrap(),
        };

        let project = match &*guard {
            Some(project) => project,
            None => return Response::error("No active project"),
        };

        match collection_name.as_str() {
            "page" => {
                let collection = project.get_collection::<Page>("page");
                let collection = match collection {
                    Ok(collection) => collection,
                    Err(e) => return Response::error(&format!("Failed to get collection: {}", e)),
                };

                let mut file = match collection.get_file(&file_id, &collection_name) {
                    Ok(file) => file,
                    Err(e) => return Response::error(&format!("Failed to get file: {}", e)),
                };

                match update {
                    FileUpdate::SetName(name) => {
                        if let Err(e) = file.set_name(&name) {
                            return Response::error(&format!("Failed to set name: {}", e));
                        }
                    }
                    FileUpdate::SetTitle(title) => {
                        if let Err(e) = file.set_title(&title) {
                            return Response::error(&format!("Failed to set title: {}", e));
                        }
                    }
                    FileUpdate::SetUrl(url) => {
                        if let Err(e) = file.set_url(&url) {
                            return Response::error(&format!("Failed to set URL: {}", e));
                        }
                    }
                    FileUpdate::SetField { name, value } => {
                        if let Err(e) = file.set_field(&name, &value) {
                            return Response::error(&format!("Failed to set field: {}", e));
                        }
                    }
                    _ => return Response::error(&format!("Unsupported update: {:?}", update)),
                }
            }
            "post" => {
                let collection = project.get_collection::<Post>("post");
                let collection = match collection {
                    Ok(collection) => collection,
                    Err(e) => return Response::error(&format!("Failed to get collection: {}", e)),
                };

                let mut file = match collection.get_file(&file_id, &collection_name) {
                    Ok(file) => file,
                    Err(e) => return Response::error(&format!("Failed to get file: {}", e)),
                };

                match update {
                    FileUpdate::SetName(name) => {
                        if let Err(e) = file.set_name(&name) {
                            return Response::error(&format!("Failed to set name: {}", e));
                        }
                    }
                    FileUpdate::SetTitle(title) => {
                        if let Err(e) = file.set_title(&title) {
                            return Response::error(&format!("Failed to set title: {}", e));
                        }
                    }
                    FileUpdate::SetUrl(url) => {
                        if let Err(e) = file.set_url(&url) {
                            return Response::error(&format!("Failed to set URL: {}", e));
                        }
                    }
                    FileUpdate::SetField { name, value } => {
                        if let Err(e) = file.set_field(&name, &value) {
                            return Response::error(&format!("Failed to set field: {}", e));
                        }
                    }
                    _ => return Response::error(&format!("Unsupported update: {:?}", update)),
                }
            }
            _ => return Response::error(&format!("Collection not found: {}", collection_name)),
        }

        Response::success(json!({
            "status": "updated",
            "project_type": project_type,
        }))
    }

    // Add this helper function before the get_file method
    fn get_file_generic<T: File + Default>(
        &self,
        project: &Project,
        collection_name: &str,
        file_id: &str,
    ) -> Response {
        let collection = match project.get_collection::<T>(collection_name) {
            Ok(collection) => collection,
            Err(e) => return Response::error(&format!("Failed to get collection: {}", e)),
        };

        let file = match collection.get_file(file_id, collection_name) {
            Ok(file) => file,
            Err(e) => return Response::error(&format!("Failed to get file: {}", e)),
        };

        match js_conversions::file_to_json(&file) {
            Ok(json_value) => Response::success(json_value),
            Err(e) => Response::error(&format!("Failed to convert file to JSON: {}", e)),
        }
    }

    /// ACTOR Get a file
    ///
    /// File metadata is cached in a files tree. First narrow by project,
    /// then by collection, then by file id.
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
                if let Some(site) = self.active_site.lock().unwrap().clone() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.active_theme.lock().unwrap().clone() {
                    theme.clone()
                } else {
                    return Response::error("Failed to acquire theme lock");
                }
            }
        };

        match collection_name.as_str() {
            "page" => self.get_file_generic::<Page>(&project, &collection_name, &file_id),
            "post" => self.get_file_generic::<Post>(&project, &collection_name, &file_id),
            "asset" => self.get_file_generic::<Asset>(&project, &collection_name, &file_id),
            "template" => self.get_file_generic::<Template>(&project, &collection_name, &file_id),
            "partial" => self.get_file_generic::<Partial>(&project, &collection_name, &file_id),
            "text" => self.get_file_generic::<Text>(&project, &collection_name, &file_id),
            _ => Response::error(&format!("Collection not found: {}", collection_name)),
        }
    }

    // Add this helper function before the list_files method
    fn list_files_generic<T: File + Default>(
        &self,
        project: &Project,
        collection_name: &str,
    ) -> Response {
        let collection = match project.get_collection::<T>(collection_name) {
            Ok(collection) => collection,
            Err(e) => return Response::error(&format!("Failed to get collection: {}", e)),
        };

        let files = match collection.get_files(collection_name) {
            Ok(files) => files,
            Err(e) => return Response::error(&format!("Failed to get files: {}", e)),
        };

        match js_conversions::files_to_json(&files) {
            Ok(json_value) => Response::success(json_value),
            Err(e) => Response::error(&format!("Failed to convert files to JSON: {}", e)),
        }
    }

    /// ACTOR List files in a collection
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

        // Get a reference to the project instead of cloning
        let guard = match project_type {
            ProjectType::Site => self.active_site.lock().unwrap(),
            ProjectType::Theme => self.active_theme.lock().unwrap(),
        };

        let project = match &*guard {
            Some(project) => project,
            None => return Response::error("No active project"),
        };

        match collection_name.as_str() {
            "page" => self.list_files_generic::<Page>(project, &collection_name),
            "post" => self.list_files_generic::<Post>(project, &collection_name),
            "asset" => self.list_files_generic::<Asset>(project, &collection_name),
            "template" => self.list_files_generic::<Template>(project, &collection_name),
            "partial" => self.list_files_generic::<Partial>(project, &collection_name),
            "text" => self.list_files_generic::<Text>(project, &collection_name),
            _ => Response::error(&format!("Collection not found: {}", collection_name)),
        }
    }

    /// ACTOR Save state to IndexedDB
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
                if let Some(site) = self.active_site.lock().unwrap().clone() {
                    site.clone()
                } else {
                    return Response::error("Failed to acquire site lock");
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.active_theme.lock().unwrap().clone() {
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

    /// ACTOR Load state from IndexedDB
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
            let has_site = self.active_site.lock().unwrap().is_some();
            let has_theme = self.active_theme.lock().unwrap().is_some();

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
            if let (Some(site), Some(theme)) = (
                self.active_site.lock().unwrap().clone(),
                self.active_theme.lock().unwrap().clone(),
            ) {
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

    /// ACTOR Export project to JSON
    fn export_project(&self, project_type: String) -> Response {
        console_log!("Exporting project with type: {}", project_type);

        let project_type = match js_conversions::string_to_project_type(&project_type) {
            Ok(pt) => pt,
            Err(e) => return Response::error(&format!("Failed to convert project type: {}", e)),
        };

        // Determine if this is a site or theme based on IDs
        let project = match project_type {
            ProjectType::Site => {
                if let Some(site) = self.active_site.lock().unwrap().clone() {
                    Some(site.clone())
                } else {
                    None
                }
            }
            ProjectType::Theme => {
                if let Some(theme) = self.active_theme.lock().unwrap().clone() {
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

    /// ACTOR Import project from JSON
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
                        self.set_site(project.clone());

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
                        self.set_theme(project.clone());

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
        }
    }

    /// ACTOR render a file (using the existing render function)
    // fn render_file(&self, file_id: String, context: Value) -> Response {
    //     console_log!("Rendering file: {} with context", file_id);
    //     // Convert context to JsValue
    //     let js_context = serde_wasm_bindgen::to_value(&context)
    //         .map_err(|e| format!("Failed to convert context: {}", e))
    //         .unwrap();
    //     // Call the existing render function
    //     match crate::render(file_id.parse().unwrap_or(0), &js_context) {
    //         Ok(html) => Response::success(json!({ "html": html })),
    //         Err(e) => Response::error(&format!("Failed to render: {:?}", e)),
    //     }
    // }

    /// ACTOR Get the active file
    fn get_active_file(&self) -> Response {
        console_log!("Getting active file");

        match self.get_active_file_json() {
            Ok((content, version)) => {
                console_log!("Document retrieved successfully, version: {}", version);
                Response::success(json!({
                    "content": content,
                    "version": version
                }))
            }
            Err(e) => {
                console_log!("Failed to get document: {}", e);
                Response::error(&format!("Failed to get document: {}", e))
            }
        }
    }

    /// ACTOR Apply steps to a document
    // fn apply_steps(&self, document_id: String, steps: Vec<Value>, version: u32) -> Response {
    //     console_log!(
    //         "Applying steps to document: {}, version: {}",
    //         document_id,
    //         version
    //     );
    //     let active_file = match self.active_file.read().unwrap() {
    //         Some(file) => file,
    //         None => return Response::error("No active file found"),
    //     };
    //     match active_file.apply_steps(&steps, version) {
    //         Ok(new_version) => {
    //             console_log!("Steps applied successfully, new version: {}", new_version);
    //             Response::success(json!({
    //                 "success": true,
    //                 "version": new_version
    //             }))
    //         }
    //         Err(e) => {
    //             console_log!("Failed to apply steps: {}", e);
    //             Response::error(&format!("Failed to apply steps: {}", e))
    //         }
    //     }
    // }

    // /// Get the active file and convert it to ProseMirror JSON format
    fn get_active_file_json(&self) -> Result<(Value, i64), String> {
        let file = &self.active_file.lock().unwrap();
        let file = file.as_ref().unwrap().get_file();

        console_log!(
            "Getting active file: {}, version: {}",
            file.id().unwrap_or_default(),
            file.version().unwrap_or_default()
        );

        let doc = match file.store() {
            FileStore::Full(doc) => doc,
            FileStore::Cache(_) => return Err("File is not full".to_string()),
        };

        // Convert from Loro document to ProseMirror JSON format
        let doc_json = crate::model::file::loro_doc_to_pm_doc(&doc)?;

        Ok((doc_json, file.version().unwrap()))
    }
}
