use crate::messages::{FileUpdate, Message, Response};
use crate::model::collection::Collection;
use crate::model::file::{File, HasTitle, HasUrl};
use crate::model::project::Project;
use crate::model::{Asset, Page, Partial, Post, Template, Text};
use crate::types::{FileType, ProjectType};
use crate::{js_conversions::*, FileStore, ProseMirrorSchema};
use loro::{LoroDoc, LoroMap};
use serde_json::{json, Value};
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

pub const IDB_DB_NAME: &str = "organ_db";
pub const IDB_PROJECTS_STORE: &str = "projects";
pub const IDB_FILES_STORE: &str = "files";

#[wasm_bindgen]
pub struct Store {
    inner: Arc<StoreInner>,
}

#[wasm_bindgen]
impl Store {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Store {
        console_log!("Creating StoreWrapper");

        Store {
            inner: Arc::new(StoreInner {
                active_site: Arc::new(Mutex::new(None)),
                active_theme: Arc::new(Mutex::new(None)),
                active_file: Arc::new(Mutex::new(None)),
            }),
        }
    }

    /// Process a message and return a response
    #[wasm_bindgen]
    pub fn process_message(&self, message_json: &str) -> Result<js_sys::Promise, JsValue> {
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

        let store = self.inner.clone(); // ✅ Arc<Store>
        let fut = async move {
            let response = store.handle_message(message).await;
            console_log!("Generated response: {:?}", response);

            match serde_json::to_string(&response) {
                Ok(json) => {
                    console_log!("Successfully serialized response");
                    Ok(JsValue::from_str(&json))
                }
                Err(e) => {
                    console_log!("Failed to serialize response: {}", e);
                    Err(JsValue::from_str(&format!(
                        "Failed to serialize response: {}",
                        e
                    )))
                }
            }
        };

        Ok(wasm_bindgen_futures::future_to_promise(fut))
    }
}

/// Store: The main entry point for our data model
#[wasm_bindgen]
#[derive(Clone)]
pub struct StoreInner {
    active_site: Arc<Mutex<Option<Project>>>,
    active_theme: Arc<Mutex<Option<Project>>>,
    active_file: Arc<Mutex<Option<FileType>>>,
}

#[wasm_bindgen]
impl StoreInner {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_log!("Creating new Store instance");
        console_error_panic_hook::set_once();

        let actor = StoreInner {
            active_theme: Arc::new(Mutex::new(None)),
            active_site: Arc::new(Mutex::new(None)),
            active_file: Arc::new(Mutex::new(None)),
        };
        console_log!("Actor instance created successfully");
        actor
    }

    /// Handle a message and return a response
    async fn handle_message(&self, message: Message) -> Response {
        console_log!("Handling message: {:?}", message);

        let response = match message {
            Message::InitDefault => {
                console_log!("Processing InitDefault message");
                self.init_default().await
            }
            Message::CreateSite { name, theme_id } => {
                console_log!(
                    "Processing CreateSite message - name: {}, theme_id: {}",
                    name,
                    theme_id
                );
                self.create_site(name, theme_id).await
            }
            Message::GetSite => self.get_site(),
            Message::CreateTheme { name } => {
                console_log!("Processing CreateTheme message - name: {}", name);
                self.create_theme(name).await
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
            } => self.create_file(project_type, collection_name, name).await,
            Message::UpdateFile {
                project_type,
                collection_name,
                file_id,
                updates,
            } => {
                self.update_file(project_type, collection_name, file_id, updates)
                    .await
            }
            Message::GetFile {
                project_type,
                collection_name,
                file_id,
            } => self.get_file(project_type, collection_name, file_id).await,
            Message::ListFiles {
                project_type,
                collection_name,
            } => self.list_files(project_type, collection_name).await,
            Message::SaveState { project_type } => self.save_state(project_type).await,
            Message::LoadState { site_id, theme_id } => self.load_state(site_id, theme_id).await,
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
    async fn init_default(&self) -> Response {
        console_log!("Initializing default projects");
        // Create default theme
        let default_theme = match Project::new(ProjectType::Theme, None).await {
            Ok(theme) => theme,
            Err(error) => {
                return Response::Error(format!("Failed to create default theme: {}", error))
            }
        };

        let theme_id = default_theme.id();

        // Create default site
        let default_site = match Project::new(ProjectType::Site, Some(theme_id)).await {
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
    async fn create_site(&self, name: String, theme_id: String) -> Response {
        console_log!(
            "Creating site with name: {} and theme_id: {}",
            name,
            theme_id
        );

        match Project::new(ProjectType::Site, Some(theme_id.clone())).await {
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
    async fn create_theme(&self, name: String) -> Response {
        console_log!("Creating theme with name: {}", name);

        match Project::new(ProjectType::Theme, None).await {
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
        let mut guard = match project_type {
            ProjectType::Site => self
                .active_site
                .lock()
                .expect("Failed to acquire site lock"),
            ProjectType::Theme => self
                .active_theme
                .lock()
                .expect("Failed to acquire theme lock"),
        };

        if let Some(ref mut project) = *guard {
            let collections = match project.get_collections() {
                Ok(collections) => collections,
                Err(e) => return Response::error(&format!("Failed to get collections: {}", e)),
            };

            console_log!("Collections: {:#?}", collections);

            let collections: Vec<Value> = collections
                .iter()
                .map(|(name, map)| match name.as_str() {
                    "page" => {
                        let collection =
                            Collection::<Page>::builder(name.clone())?.build_detached()?;
                        js_conversions::collection_to_json(&collection)
                    }
                    "post" => {
                        let collection =
                            Collection::<Post>::builder(name.clone())?.build_detached()?;
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
                        let collection =
                            Collection::<Text>::builder(name.clone())?.build_detached()?;
                        js_conversions::collection_to_json(&collection)
                    }
                    _ => Err(format!("Collection not found: {}", name)),
                })
                .map(|c| c.unwrap())
                .collect();

            return Response::success(json!(collections));
        } else {
            return Response::error("No active project");
        }

        // match js_conversions::collections_to_json(&collections) {
        //     Ok(json_value) => Response::success(json_value),
        //     Err(e) => Response::error(&format!("Failed to convert collections to JSON: {}", e)),
        // }
    }

    async fn create_file_generic<T: File + Default + Debug>(
        &self,
        project_type: ProjectType,
        collection_name: &str,
        name: &str,
        pm_schema: Option<ProseMirrorSchema>,
        store: crate::FileStore,
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
            let mut file_builder = match project.create_file::<T>(&name, &collection_name, store) {
                Ok(file) => file,
                Err(e) => return Response::error(&format!("Failed to create file: {}", e)),
            };

            if let Some(pm_schema) = pm_schema {
                file_builder = match file_builder.with_pm_schema(pm_schema) {
                    Ok(builder) => builder,
                    Err(e) => return Response::error(&format!("Failed to set PM schema: {}", e)),
                };
            }

            let file = match project.attach_file(file_builder).await {
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
    async fn create_file(
        &self,
        project_type: String,
        collection_name: String,
        name: String,
    ) -> Response {
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
            "page" => {
                self.create_file_generic::<Page>(
                    project_type,
                    &collection_name,
                    &name,
                    Some(ProseMirrorSchema::default()),
                    crate::FileStore::Full(LoroDoc::new()),
                )
                .await
            }
            "post" => {
                self.create_file_generic::<Post>(
                    project_type,
                    &collection_name,
                    &name,
                    Some(ProseMirrorSchema::default()),
                    crate::FileStore::Full(LoroDoc::new()),
                )
                .await
            }
            "asset" => {
                self.create_file_generic::<Asset>(
                    project_type,
                    &collection_name,
                    &name,
                    None,
                    crate::FileStore::Cache(LoroMap::new()),
                )
                .await
            }
            "template" => {
                self.create_file_generic::<Template>(
                    project_type,
                    &collection_name,
                    &name,
                    None,
                    crate::FileStore::Full(LoroDoc::new()),
                )
                .await
            }
            "partial" => {
                self.create_file_generic::<Partial>(
                    project_type,
                    &collection_name,
                    &name,
                    None,
                    crate::FileStore::Full(LoroDoc::new()),
                )
                .await
            }
            "text" => {
                self.create_file_generic::<Text>(
                    project_type,
                    &collection_name,
                    &name,
                    None,
                    crate::FileStore::Full(LoroDoc::new()),
                )
                .await
            }
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
    async fn update_file(
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

        // only supports page and post for now?
        match collection_name.as_str() {
            "page" => {
                let collection = project.get_collection::<Page>("page");
                let collection = match collection {
                    Ok(collection) => collection,
                    Err(e) => return Response::error(&format!("Failed to get collection: {}", e)),
                };

                let mut file = match collection.get_file(&file_id, &collection_name).await {
                    Ok(file) => file,
                    Err(e) => return Response::error(&format!("Failed to get file: {}", e)),
                };

                match update {
                    FileUpdate::SetName(name) => {
                        if let Err(e) = file.set_name(&name).await {
                            return Response::error(&format!("Failed to set name: {}", e));
                        }
                    }
                    FileUpdate::SetTitle(title) => {
                        if let Err(e) = file.set_title(&title).await {
                            return Response::error(&format!("Failed to set title: {}", e));
                        }
                    }
                    FileUpdate::SetUrl(url) => {
                        if let Err(e) = file.set_url(&url).await {
                            return Response::error(&format!("Failed to set URL: {}", e));
                        }
                    }
                    FileUpdate::SetField { name, value } => {
                        if let Err(e) = file.set_field(&name, &value).await {
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

                let mut file = match collection.get_file(&file_id, &collection_name).await {
                    Ok(file) => file,
                    Err(e) => return Response::error(&format!("Failed to get file: {}", e)),
                };

                match update {
                    FileUpdate::SetName(name) => {
                        if let Err(e) = file.set_name(&name).await {
                            return Response::error(&format!("Failed to set name: {}", e));
                        }
                    }
                    FileUpdate::SetTitle(title) => {
                        if let Err(e) = file.set_title(&title).await {
                            return Response::error(&format!("Failed to set title: {}", e));
                        }
                    }
                    FileUpdate::SetUrl(url) => {
                        if let Err(e) = file.set_url(&url).await {
                            return Response::error(&format!("Failed to set URL: {}", e));
                        }
                    }
                    FileUpdate::SetField { name, value } => {
                        if let Err(e) = file.set_field(&name, &value).await {
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
    async fn get_file_generic<T: File + Default>(
        &self,
        project: &Project,
        collection_name: &str,
        file_id: &str,
    ) -> Response {
        let collection = match project.get_collection::<T>(collection_name) {
            Ok(collection) => collection,
            Err(e) => return Response::error(&format!("Failed to get collection: {}", e)),
        };

        let file = match collection.get_file(file_id, collection_name).await {
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
    async fn get_file(
        &self,
        project_type: String,
        collection_name: String,
        file_id: String,
    ) -> Response {
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
            "page" => {
                self.get_file_generic::<Page>(&project, &collection_name, &file_id)
                    .await
            }
            "post" => {
                self.get_file_generic::<Post>(&project, &collection_name, &file_id)
                    .await
            }
            "asset" => {
                self.get_file_generic::<Asset>(&project, &collection_name, &file_id)
                    .await
            }
            "template" => {
                self.get_file_generic::<Template>(&project, &collection_name, &file_id)
                    .await
            }
            "partial" => {
                self.get_file_generic::<Partial>(&project, &collection_name, &file_id)
                    .await
            }
            "text" => {
                self.get_file_generic::<Text>(&project, &collection_name, &file_id)
                    .await
            }
            _ => Response::error(&format!("Collection not found: {}", collection_name)),
        }
    }

    // Add this helper function before the list_files method
    async fn list_files_generic<T: File + Default>(
        &self,
        project: &Project,
        collection_name: &str,
    ) -> Response {
        let collection = match project.get_collection::<T>(collection_name) {
            Ok(collection) => collection,
            Err(e) => return Response::error(&format!("Failed to get collection: {}", e)),
        };

        let files = match collection.get_files(collection_name).await {
            Ok(files) => files,
            Err(e) => return Response::error(&format!("Failed to get files: {}", e)),
        };

        match js_conversions::files_to_json(&files) {
            Ok(json_value) => Response::success(json_value),
            Err(e) => Response::error(&format!("Failed to convert files to JSON: {}", e)),
        }
    }

    /// ACTOR List files in a collection
    async fn list_files(&self, project_type: String, collection_name: String) -> Response {
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
            "page" => {
                self.list_files_generic::<Page>(project, &collection_name)
                    .await
            }
            "post" => {
                self.list_files_generic::<Post>(project, &collection_name)
                    .await
            }
            "asset" => {
                self.list_files_generic::<Asset>(project, &collection_name)
                    .await
            }
            "template" => {
                self.list_files_generic::<Template>(project, &collection_name)
                    .await
            }
            "partial" => {
                self.list_files_generic::<Partial>(project, &collection_name)
                    .await
            }
            "text" => {
                self.list_files_generic::<Text>(project, &collection_name)
                    .await
            }
            _ => Response::error(&format!("Collection not found: {}", collection_name)),
        }
    }

    /// ACTOR Save state to IndexedDB
    async fn save_state(&self, project_type: String) -> Response {
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
        console_log!("Saving project data to IndexedDB: {}", project_id);

        let length = &project_export.length();
        match crate::save_data(IDB_PROJECTS_STORE, &project_id, project_export).await {
            Ok(_) => {
                console_log!("Site data saved, size: {} bytes", length);
            }
            Err(e) => {
                console_log!("Failed to save site data: {:#?}", e);
                return Response::Error(format!("Failed to save site: {:#?}", e));
            }
        };

        Response::success(json!({
            "status": "saved",
            "project_type": project_type,
        }))
    }

    /// Helper function to convert JS Uint8Array to Rust Vec<u8>
    fn to_vec_u8(array: &js_sys::Uint8Array) -> Result<Vec<u8>, String> {
        let mut result = vec![0; array.length() as usize];
        array.copy_to(&mut result);
        Ok(result)
    }

    /// ACTOR Load state from IndexedDB
    async fn load_state(&self, site_id: Option<String>, theme_id: Option<String>) -> Response {
        console_log!(
            "Loading state from IndexedDB - site_id: {:?}, theme_id: {:?}",
            site_id,
            theme_id
        );

        // Check if we have projects in memory first
        let has_site_in_memory = self.active_site.lock().unwrap().is_some();
        let has_theme_in_memory = self.active_theme.lock().unwrap().is_some();

        // If specific IDs are provided, use those
        if let (Some(site_id), Some(theme_id)) = (site_id.as_ref(), theme_id.as_ref()) {
            console_log!(
                "Loading specific projects - site_id: {}, theme_id: {}",
                site_id,
                theme_id
            );

            // For now, if we have the projects in memory, return success
            if has_site_in_memory && has_theme_in_memory {
                let site = self.active_site.lock().unwrap().clone().unwrap();
                let theme = self.active_theme.lock().unwrap().clone().unwrap();

                if site.id() == *site_id && theme.id() == *theme_id {
                    return Response::success(json!({
                        "status": "loaded",
                        "site_id": site_id,
                        "theme_id": theme_id
                    }));
                }
            }

            let store_clone = self.clone();

            // Load site data
            console_log!("Loading site data from IndexedDB: {}", site_id);
            let site_data = match crate::load_data(IDB_PROJECTS_STORE, &site_id).await {
                Ok(data) => {
                    console_log!("Site data loaded, size: {} bytes", data.length());
                    data
                }
                Err(e) => {
                    console_log!("Failed to load site data: {:#?}", e);
                    return Response::Error(format!("Failed to load site: {:#?}", e));
                }
            };

            // Load theme data
            console_log!("Loading theme data from IndexedDB: {}", theme_id);
            let theme_data = match crate::load_data(IDB_PROJECTS_STORE, &theme_id).await {
                Ok(data) => {
                    console_log!("Theme data loaded, size: {} bytes", data.length());
                    data
                }
                Err(e) => {
                    console_log!("Failed to load theme data: {:#?}", e);
                    return Response::Error(format!("Failed to load theme: {:#?}", e));
                }
            };
            // Convert JavaScript Uint8Array to Rust Vec<u8>
            let site_data_value: wasm_bindgen::JsValue = site_data.into();
            let site_bytes = StoreInner::to_vec_u8(&js_sys::Uint8Array::from(site_data_value))
                .map_err(|e| Response::Error(format!("Failed to convert site data: {}", e)));
            let site_bytes = match site_bytes {
                Ok(bytes) => bytes,
                Err(err) => return err,
            };

            let theme_data_value: wasm_bindgen::JsValue = theme_data.into();
            let theme_bytes = StoreInner::to_vec_u8(&js_sys::Uint8Array::from(theme_data_value))
                .map_err(|e| Response::Error(format!("Failed to convert theme data: {}", e)));

            let theme_bytes = match theme_bytes {
                Ok(bytes) => bytes,
                Err(err) => return err,
            };

            // Import the site
            console_log!("Importing site from loaded data");
            let site = match Project::import(
                site_bytes,
                site_id.clone(),
                ProjectType::Site,
                0.0, // We could retrieve created time from metadata if needed
                0.0, // We could retrieve updated time from metadata if needed
            ) {
                Ok(project) => project,
                Err(e) => {
                    console_log!("Failed to import site: {}", e);
                    return Response::Error(format!("Failed to import site: {}", e));
                }
            };

            // Import the theme
            console_log!("Importing theme from loaded data");
            let theme = match Project::import(
                theme_bytes,
                theme_id.clone(),
                ProjectType::Theme,
                0.0, // We could retrieve created time from metadata if needed
                0.0, // We could retrieve updated time from metadata if needed
            ) {
                Ok(project) => project,
                Err(e) => {
                    console_log!("Failed to import theme: {}", e);
                    return Response::Error(format!("Failed to import theme: {}", e));
                }
            };

            // Set the loaded projects in the store
            console_log!("Setting loaded projects in store");
            if let Err(e) = store_clone.set_site(site) {
                console_log!("Failed to set site: {}", e);
                return Response::Error(format!("Failed to set site: {}", e));
            }

            if let Err(e) = store_clone.set_theme(theme) {
                console_log!("Failed to set theme: {}", e);
                return Response::Error(format!("Failed to set theme: {}", e));
            }

            // Return success with the loaded project IDs
            console_log!("Projects loaded successfully");

            Response::Success(json!({
                "status": "loaded",
                "siteId": site_id,
                "themeId": theme_id
            }))
        } else {
            console_log!("Loading default active projects");

            // Check if we have any projects in memory
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

        let file = &self.active_file.lock().unwrap();
        let file = match file.as_ref().unwrap() {
            FileType::Asset(asset) => self.get_active_file_json_generic::<Asset>(asset),
            FileType::Template(template) => self.get_active_file_json_generic::<Template>(template),
            FileType::Page(page) => self.get_active_file_json_generic::<Page>(page),
            FileType::Text(text) => self.get_active_file_json_generic::<Text>(text),
            FileType::Partial(partial) => self.get_active_file_json_generic::<Partial>(partial),
            FileType::Post(post) => self.get_active_file_json_generic::<Post>(post),
        };

        match file {
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
    fn get_active_file_json_generic<T: File + Default>(
        &self,
        file: &T,
    ) -> Result<(Value, i64), String> {
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
