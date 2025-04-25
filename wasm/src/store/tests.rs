#[allow(unused)]
#[cfg(test)]
mod tests {
    use crate::{
        messages::{FileUpdate, Response},
        ProjectType, StoreInner, ID_KEY,
    };

    use super::*;
    use serde_json::json;
    use std::sync::Once;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    static INIT: Once = Once::new();

    fn setup_panic_hook() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        });
    }

    #[wasm_bindgen_test]
    fn test_store_creation() {
        setup_panic_hook();
        let store = StoreInner::new();
        assert!(store.active_site.lock().unwrap().is_none());
        assert!(store.active_theme.lock().unwrap().is_none());
        assert!(store.active_file.lock().unwrap().is_none());
    }

    #[wasm_bindgen_test]
    async fn test_init_default() {
        setup_panic_hook();
        let store = StoreInner::new();
        let response = store.init_default().await;

        match response {
            Response::Success(_) => {
                assert!(store.active_site.lock().unwrap().is_some());
                assert!(store.active_theme.lock().unwrap().is_some());
            }
            Response::Error(e) => panic!("Failed to initialize default store: {}", e),
        }
    }

    #[wasm_bindgen_test]
    async fn test_create_theme() {
        setup_panic_hook();
        let store = StoreInner::new();
        let theme_name = "Test Theme";
        let response = store.create_theme(theme_name.to_string()).await;

        match response {
            Response::Success(value) => {
                let theme = store.active_theme.lock().unwrap();
                assert!(theme.is_some());
                let theme = theme.as_ref().unwrap();
                assert_eq!(theme.name().unwrap(), theme_name);

                // Check JSON response
                let json = value.as_object().unwrap();
                assert_eq!(json["name"].as_str().unwrap(), theme_name);
                assert!(json.contains_key("id"));
            }
            Response::Error(e) => panic!("Failed to create theme: {}", e),
        }
    }

    #[wasm_bindgen_test]
    async fn test_create_site() {
        setup_panic_hook();
        let store = StoreInner::new();

        // First create a theme
        let theme_response = store.create_theme("Test Theme".to_string()).await;
        let theme_id = match theme_response {
            Response::Success(value) => value["id"].as_str().unwrap().to_string(),
            Response::Error(e) => panic!("Failed to create theme: {}", e),
        };

        // Then create a site
        let site_name = "Test Site";
        let response = store
            .create_site(site_name.to_string(), theme_id.clone())
            .await;

        match response {
            Response::Success(value) => {
                let site = store.active_site.lock().unwrap();
                assert!(site.is_some());
                let site = site.as_ref().unwrap();
                assert_eq!(site.name().unwrap(), site_name);

                // Check JSON response
                let json = value.as_object().unwrap();
                assert_eq!(json["name"].as_str().unwrap(), site_name);
                assert_eq!(json["themeId"].as_str().unwrap(), theme_id);
                assert!(json.contains_key("id"));
            }
            Response::Error(e) => panic!("Failed to create site: {}", e),
        }
    }

    #[wasm_bindgen_test]
    async fn test_get_site() {
        setup_panic_hook();
        let store = StoreInner::new();

        // First create a theme and site
        let theme_response = store.create_theme("Test Theme".to_string()).await;
        let theme_id = match theme_response {
            Response::Success(value) => value["id"].as_str().unwrap().to_string(),
            Response::Error(e) => panic!("Failed to create theme: {}", e),
        };

        let site_name = "Test Site";
        store.create_site(site_name.to_string(), theme_id.clone());

        // Test get_site
        let response = store.get_site();
        match response {
            Response::Success(value) => {
                let json = value.as_object().unwrap();
                assert_eq!(json["name"].as_str().unwrap(), site_name);
                assert!(json.contains_key("id"));
                assert_eq!(json["themeId"].as_str().unwrap(), theme_id);
            }
            Response::Error(e) => panic!("Failed to get site: {}", e),
        }
    }

    #[wasm_bindgen_test]
    fn test_get_theme() {
        setup_panic_hook();
        let store = StoreInner::new();

        // Create a theme
        let theme_name = "Test Theme";
        store.create_theme(theme_name.to_string());

        // Test get_theme
        let response = store.get_theme();
        match response {
            Response::Success(value) => {
                let json = value.as_object().unwrap();
                assert_eq!(json["name"].as_str().unwrap(), theme_name);
                assert!(json.contains_key("id"));
            }
            Response::Error(e) => panic!("Failed to get theme: {}", e),
        }
    }

    #[wasm_bindgen_test]
    async fn test_create_and_get_file() {
        setup_panic_hook();
        let store = StoreInner::new();
        store.init_default();

        // Create a page file
        let file_name = "test-page";
        let response = store
            .create_file(
                "site".to_string(),
                "page".to_string(),
                file_name.to_string(),
            )
            .await;

        // Check the response
        let file_id = match response {
            Response::Success(value) => {
                let json = value.as_object().unwrap();
                console_log!("File ID: {:?}", json);
                json[ID_KEY].as_str().unwrap().to_string()
            }
            Response::Error(e) => panic!("Failed to create file: {}", e),
        };

        // Get the file
        let response = store
            .get_file("site".to_string(), "page".to_string(), file_id.clone())
            .await;
        match response {
            Response::Success(value) => {
                let json = value.as_object().unwrap();
                assert_eq!(
                    json["name"].as_str().unwrap(),
                    file_name,
                    "File name {} does not match expected name {}",
                    file_name,
                    json["name"].as_str().unwrap()
                );
                assert_eq!(json["id"].as_str().unwrap(), file_id);
            }
            Response::Error(e) => panic!("Failed to get file: {} for {:?}", e, file_id),
        }
    }

    /// my concern is that get_file doesn't return an
    /// attached handler such that update_file can apply updates
    ///
    /// let's check if the container id is the same from creation
    /// to get_file in store.update_file
    ///
    /// Depends on:
    /// - store.init_default()
    /// - store.create_file()
    /// - store.update_file()
    ///     - collection.get_file()
    ///
    #[wasm_bindgen_test]
    async fn test_update_file() {
        setup_panic_hook();
        let store = StoreInner::new();
        store.init_default(); //

        // Create a page file
        let file_name = "test-page";
        let response = store
            .create_file(
                "site".to_string(),
                "page".to_string(),
                file_name.to_string(),
            )
            .await;

        // Check the response
        let file_id = match response {
            Response::Success(value) => {
                let json = value.as_object().unwrap();
                console_log!(
                    "[Store (WASM)] (test_update_file) Success! Created File 📁: {:?}",
                    json
                );
                json[ID_KEY].as_str().unwrap().to_string()
            }
            Response::Error(e) => panic!("Failed to create file: {}", e),
        };

        // Update the file name
        let new_name = "updated-page";
        let update = FileUpdate::SetName(new_name.to_string());
        let response = store
            .update_file(
                "site".to_string(),
                "page".to_string(),
                file_id.clone(),
                update,
            )
            .await;

        match response {
            Response::Success(_) => {
                // Verify the update
                let response = store
                    .get_file("site".to_string(), "page".to_string(), file_id)
                    .await;
                match response {
                    Response::Success(value) => {
                        let json = value.as_object().unwrap();
                        assert_eq!(
                            json["name"].as_str().expect("File name is required"),
                            new_name
                        );
                    }
                    Response::Error(e) => panic!("Failed to get updated file: {}", e),
                }
            }
            Response::Error(e) => panic!("Failed to update file: {} for {:?}", e, file_id),
        }
    }

    #[wasm_bindgen_test]
    async fn test_list_files() {
        setup_panic_hook();
        let store = StoreInner::new();
        store.init_default();

        // Create a few page files
        let file_names = vec!["page1", "page2", "page3"];
        for name in &file_names {
            let response = store
                .create_file("site".to_string(), "page".to_string(), name.to_string())
                .await;
            console_log!("(test_list_files) Create file Response: {:?}", response);
            assert!(
                matches!(response, Response::Success(_)),
                "Failed to create file {}",
                name
            );
        }

        // List the files
        let response = store
            .list_files("site".to_string(), "page".to_string())
            .await;
        match response {
            Response::Success(value) => {
                let files = value.as_array().expect("Files must be an array");
                assert_eq!(
                    files.len(),
                    file_names.len() + 1,
                    "Expected {} files, got {}",
                    file_names.len() + 1,
                    files.len()
                );

                // Verify all created files are in the list
                let names: Vec<&str> = files
                    .iter()
                    .map(|f| f["name"].as_str().expect("File name is required"))
                    .collect();
                for name in file_names {
                    assert!(names.contains(&name), "File {} not found in list", name);
                }
            }
            Response::Error(e) => panic!("Failed to list files: {}", e),
        }
    }

    // #[wasm_bindgen_test]
    // fn test_export_import_project() {
    //     let store = Store::new();

    //     // Create and setup a theme
    //     let theme_name = "Export Test Theme";
    //     store.create_theme(theme_name.to_string());

    //     // Export the theme
    //     let response = store.export_project("theme".to_string());
    //     let exported_data = match response {
    //         Response::Success(value) => value.to_string().into_bytes(),
    //         Response::Error(e) => panic!("Failed to export theme: {}", e),
    //     };

    //     // Create a new store
    //     let new_store = Store::new();

    //     // Import the theme
    //     let theme_id = "test-theme-id".to_string();
    //     let response = new_store.import_project(
    //         exported_data,
    //         theme_id.clone(),
    //         ProjectType::Theme,
    //         0.0,
    //         0.0,
    //     );

    //     match response {
    //         Response::Success(value) => {
    //             let json = value.as_object().unwrap();
    //             assert_eq!(json["name"].as_str().unwrap(), theme_name);
    //             assert_eq!(json["id"].as_str().unwrap(), theme_id);

    //             // Verify the theme was imported correctly
    //             let theme = new_store.active_theme.read().unwrap();
    //             assert!(theme.is_some());
    //             assert_eq!(theme.as_ref().unwrap().name().unwrap(), theme_name);
    //         }
    //         Response::Error(e) => panic!("Failed to import theme: {}", e),
    //     }
    // }
}
