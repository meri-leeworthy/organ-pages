use crate::types::FieldDefinition;
// use js_sys::Object;
// use loro::{ContainerID, LoroDoc, LoroMap};
use std::collections::HashMap;
// use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = console)]
//     fn log(s: &str); // log to JS console
// }

// // Helper macro for logging
// macro_rules! console_log {
//   ($($t:tt)*) => (log(&format!("[Model Lib (WASM)] {}", format!($($t)*))))
// }

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

    pub fn insert(&mut self, key: &str, value: FieldDefinition) -> &mut Self {
        self.fields.insert(key.to_string(), value);
        self
    }

    pub fn get(&self, key: &str) -> Option<FieldDefinition> {
        self.fields.get(key).cloned()
    }
}

// type RichtextCallback = Closure<dyn Fn(JsValue, JsValue, JsValue)>;

// fn loro_map_to_js(map: &LoroMap) -> JsValue {
//     let obj = Object::new();
//     // Convert map contents to JS (simplified example)
//     JsValue::from(obj)
// }

// fn loro_doc_to_js(doc: &LoroDoc) -> JsValue {
//     let obj = Object::new();
//     // Convert document metadata
//     JsValue::from(obj)
// }

// fn container_id_to_js(id: &ContainerID) -> JsValue {
//     JsValue::from(id.to_string()) // Assuming ContainerID is stringifiable
// }
