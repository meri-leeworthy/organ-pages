pub mod js_conversions {
    
    use crate::model::{Collection, File};
    use crate::types::{FieldType, ProjectType};
    use serde_json::{json, Value};
    
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str); // log to JS console
    }

    macro_rules! console_log {
      ($($t:tt)*) => (log(&format!("[JS Conversions (WASM)] {}", format!($($t)*))))
  }

    /// Convert a Collection to a JS-friendly JSON representation
    pub fn collection_to_json<FileType: File>(
        collection: &Collection<FileType>,
    ) -> Result<Value, String> {
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
    pub fn file_to_json<FileType: File>(file: &FileType) -> Result<Value, String> {
        file.to_json()
    }

    /// Convert a list of Files to a JS-friendly JSON array
    pub fn files_to_json<FileType: File>(files: &[FileType]) -> Result<Value, String> {
        let mut result = Vec::new();

        for file in files {
            result.push(file_to_json(file)?);
        }

        Ok(json!(result))
    }

    /// Convert a list of Collections to a JS-friendly JSON array
    pub fn collections_to_json<FileType: File>(
        collections: &[Collection<FileType>],
    ) -> Result<Value, String> {
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
