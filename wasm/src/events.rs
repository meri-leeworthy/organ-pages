use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

// Simple callback system to mimic the EventEmitter pattern in TypeScript
#[derive(Clone)]
#[wasm_bindgen]
pub struct EventEmitter {
    listeners: Arc<Mutex<HashMap<String, Vec<js_sys::Function>>>>,
}

#[wasm_bindgen]
impl EventEmitter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        EventEmitter {
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[wasm_bindgen]
    pub fn on(&self, event_name: &str, callback: js_sys::Function) -> js_sys::Function {
        let mut listeners = self.listeners.lock().unwrap();
        let event_listeners = listeners
            .entry(event_name.to_string())
            .or_insert_with(Vec::new);
        event_listeners.push(callback.clone());
        callback
    }

    #[wasm_bindgen]
    pub fn off(&self, event_name: &str, callback: &js_sys::Function) {
        let mut listeners = self.listeners.lock().unwrap();
        if let Some(event_listeners) = listeners.get_mut(event_name) {
            event_listeners.retain(|cb| !cb.eq(callback));
        }
    }

    #[wasm_bindgen]
    pub fn emit(&self, event_name: &str, args: JsValue) {
        let listeners = self.listeners.lock().unwrap();
        if let Some(event_listeners) = listeners.get(event_name) {
            for callback in event_listeners {
                let _ = callback.call1(&JsValue::null(), &args);
            }
        }
    }
}

// Helper function to create an event emitter that can be inherited by other objects
pub fn create_event_emitter() -> EventEmitter {
    EventEmitter::new()
}
