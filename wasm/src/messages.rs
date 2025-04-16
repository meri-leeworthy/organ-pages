use js_sys::Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::types::FieldDefinition;

/// Messages that can be sent to the Actor system.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    // Project operations
    CreateSite {
        name: String,
        theme_id: String,
    },
    GetSite,
    CreateTheme {
        name: String,
    },
    GetTheme,

    // Collection operations
    // AddCollection {
    //     project_type: String,
    //     name: String,
    //     fields: Vec<FieldDefinition>,
    // },
    GetCollection {
        project_type: String,
        name: String,
    },
    ListCollections {
        project_type: String,
    },

    // File operations
    CreateFile {
        project_type: String,
        collection_name: String,
        name: String,
    },
    UpdateFile {
        project_type: String,
        collection_name: String,
        file_id: String,
        updates: FileUpdate,
    },
    GetFile {
        project_type: String,
        collection_name: String,
        file_id: String,
    },
    ListFiles {
        project_type: String,
        collection_name: String,
    },

    // Storage operations
    SaveState {
        project_type: String,
    },
    LoadState {
        site_id: Option<String>,
        theme_id: Option<String>,
    },
    ExportProject {
        project_type: String,
    },
    ImportProject {
        data: Vec<u8>,
        id: String,
        project_type: String,
        created: f64,
        updated: f64,
    },

    // Rendering operations
    // RenderFile {
    //     file_id: String,
    //     context: serde_json::Value,
    // },

    // Initialization
    InitDefault,

    // Document operations for ProseMirror integration
    GetDocument {
        document_id: String,
    },
    // ApplySteps {
    //     document_id: String,
    //     steps: Vec<serde_json::Value>, // Serialized ProseMirror steps
    //     version: u32,
    // },
}

/// File update operations
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileUpdate {
    SetField { name: String, value: String },
    SetName(String),
    SetContent(String),
    SetBody(String),
    SetTitle(String),
    SetUrl(String),
    SetMimeType(String),
    SetAlt(String),
}

/// Response from the Actor system
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Success(serde_json::Value),
    Error(String),
}

impl Response {
    pub fn success<T: Serialize>(value: T) -> Self {
        match serde_json::to_value(value) {
            Ok(json) => Response::Success(json),
            Err(err) => Response::Error(format!("Serialization error: {}", err)),
        }
    }

    pub fn error(message: &str) -> Self {
        Response::Error(message.to_string())
    }
}
