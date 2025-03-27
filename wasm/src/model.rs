use crate::types::{
    CollectionType, ContentRecord, FieldDefinition, FieldType, ProjectType, UnparsedContentRecord,
};
use js_sys::Object;
use loro::{
    Container, ContainerID, ContainerTrait, ExportMode, LoroDoc, LoroList, LoroMap,
    LoroStringValue, LoroText, LoroTree, LoroValue, TreeID, TreeNode, ValueOrContainer,
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
    // Document storage for ProseMirror/Loro integration
    documents: RwLock<HashMap<String, Document>>,
}

// Constants for ProseMirror-Loro integration
pub const ROOT_DOC_KEY: &str = "doc";
pub const ATTRIBUTES_KEY: &str = "attributes";
pub const CHILDREN_KEY: &str = "children";
pub const NODE_NAME_KEY: &str = "nodeName";

// Document structure for ProseMirror/Loro integration
pub struct Document {
    pub id: String,
    pub version: u32,
    pub doc: LoroDoc,
}

impl Store {
    pub fn new() -> Self {
        let store = Store {
            active_theme: RwLock::new(None),
            active_site: RwLock::new(None),
            documents: RwLock::new(HashMap::new()),
        };
        store
    }

    // Document methods for ProseMirror integration

    /// Initialize a new document or get existing one
    pub fn initialize_document(&self, document_id: &str, schema_json: &str) -> Result<(), String> {
        let mut documents = self.documents.write().unwrap();

        if !documents.contains_key(document_id) {
            console_log!("Creating new document: {}", document_id);

            // Create a new Loro document
            let loro_doc = LoroDoc::new();

            // Configure text style for marks if schema is provided
            if !schema_json.is_empty() {
                self.configure_text_style(&loro_doc, schema_json)?;
            }

            // Initialize the document structure according to Loro-ProseMirror convention
            let root_map = loro_doc.get_map(ROOT_DOC_KEY);
            root_map.insert(NODE_NAME_KEY, "doc".to_string());

            // Add attributes map
            let attrs_map = LoroMap::new();
            root_map.insert_container(ATTRIBUTES_KEY, attrs_map);

            // Add children list
            let children_list = LoroList::new();
            root_map.insert_container(CHILDREN_KEY, children_list);

            // Get reference to children list
            let children = match root_map.get(CHILDREN_KEY) {
                Some(ValueOrContainer::Container(Container::List(list))) => list,
                _ => return Err("Failed to get children list".to_string()),
            };

            // Create paragraph map
            let para_map = LoroMap::new();
            para_map.insert(NODE_NAME_KEY, "paragraph".to_string());

            // Add paragraph attributes
            let para_attrs = LoroMap::new();
            para_map.insert_container(ATTRIBUTES_KEY, para_attrs);

            // Add paragraph children list
            let para_children = LoroList::new();
            para_map.insert_container(CHILDREN_KEY, para_children);

            // Add paragraph to document children
            children.insert_container(0, para_map);

            let para_map = match children.get(0) {
                Some(ValueOrContainer::Container(Container::Map(map))) => map,
                _ => return Err("Failed to get paragraph map".to_string()),
            };

            let para_children = match para_map.get(CHILDREN_KEY) {
                Some(ValueOrContainer::Container(Container::List(list))) => list,
                _ => return Err("Failed to get paragraph children list".to_string()),
            };

            // Create text node with space to avoid empty text node errors
            // In Loro-ProseMirror, text nodes are LoroText directly, not maps with text props
            let text = LoroText::new();
            text.insert(0, " "); // Space character to avoid empty text node errors

            // Add text to paragraph children
            para_children.insert_container(0, text);

            // Store the document
            let doc = Document {
                id: document_id.to_string(),
                version: 0,
                doc: loro_doc,
            };

            documents.insert(document_id.to_string(), doc);
            console_log!("Document created successfully: {}", document_id);
        } else {
            console_log!("Document already exists: {}", document_id);
        }

        Ok(())
    }

    /// Configure text style expansion behavior based on ProseMirror schema
    fn configure_text_style(&self, doc: &LoroDoc, schema_json: &str) -> Result<(), String> {
        // Parse the schema to extract mark definitions
        let schema: serde_json::Value = match serde_json::from_str(schema_json) {
            Ok(schema) => schema,
            Err(e) => {
                console_log!("Error parsing schema JSON: {}", e);
                return Err(format!("Failed to parse schema JSON: {}", e));
            }
        };

        // Extract marks from schema
        let marks = match schema.get("marks") {
            Some(marks) => marks,
            None => {
                // If no marks defined, use empty config
                return Ok(());
            }
        };

        // Build text style config map
        let mut text_style_config = HashMap::new();

        // Process each mark and build text style config
        if let Some(marks_obj) = marks.as_object() {
            for (mark_name, mark_def) in marks_obj {
                // Default to "after" for inclusive marks (matching the JS implementation)
                let expand = match mark_def.get("inclusive") {
                    Some(inclusive) if inclusive.as_bool() == Some(true) => "after",
                    _ => "none", // Non-inclusive marks default to "none"
                };

                // Store the config
                text_style_config.insert(mark_name.clone(), json!({ "expand": expand }));
            }
        }

        // Log the configuration for debugging
        console_log!("Text style config: {:?}", text_style_config);

        // In Loro, we'd configure text style directly with something like:
        // doc.config_text_style(text_style_config);
        // However, since there's no direct binding for this in the WASM API,
        // we'll simulate it by storing the config in a special metadata map
        
        // Store the text style config in a special metadata map for reference
        if !text_style_config.is_empty() {
            let meta_map = doc.get_map("__meta");
            let styles_map = LoroMap::new();
            meta_map.insert_container("textStyles", styles_map);
            
            if let Some(ValueOrContainer::Container(Container::Map(styles))) = meta_map.get("textStyles") {
                for (mark_name, style_config) in text_style_config {
                    // Convert the JSON value to a string representation
                    if let Ok(config_str) = serde_json::to_string(&style_config) {
                        styles.insert(mark_name, config_str);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a document and convert it to ProseMirror JSON format
    pub fn get_document(&self, document_id: &str) -> Result<(Value, u32), String> {
        let documents = self.documents.read().unwrap();

        match documents.get(document_id) {
            Some(document) => {
                console_log!(
                    "Getting document: {}, version: {}",
                    document_id,
                    document.version
                );

                // Convert from Loro document to ProseMirror JSON format
                let doc_json = self.loro_doc_to_pm_doc(&document.doc)?;

                Ok((doc_json, document.version))
            }
            None => Err(format!("Document not found: {}", document_id)),
        }
    }

    /// Apply ProseMirror steps to a Loro document
    pub fn apply_steps(
        &self,
        document_id: &str,
        steps: &[Value],
        client_id: &str,
        version: u32,
    ) -> Result<u32, String> {
        let mut documents = self.documents.write().unwrap();

        match documents.get_mut(document_id) {
            Some(document) => {
                console_log!(
                    "Applying steps to document: {}, current version: {}, incoming version: {}",
                    document_id,
                    document.version,
                    version
                );

                // Version check for conflict handling
                if version != document.version {
                    console_log!(
                        "Version mismatch - current: {}, received: {}",
                        document.version,
                        version
                    );
                    // For now, we'll accept the steps but warn about potential conflicts
                    // In a real-world scenario, you'd implement proper OT conflict resolution
                }

                // Apply the steps to the Loro document
                self.apply_steps_to_loro_doc(&mut document.doc, steps)
                    .map_err(|e| format!("Failed to apply steps: {:?}", e))?;

                // Increment version
                document.version += 1;

                console_log!(
                    "Steps applied successfully, new version: {}",
                    document.version
                );

                Ok(document.version)
            }
            None => Err(format!("Document not found: {}", document_id)),
        }
    }

    fn text_to_pm_node(&self, text: &LoroText) -> Vec<Value> {
        // Handle text nodes - these are LoroText objects directly
        if text.len_unicode() > 0 {
            // Get the Delta format which includes formatting
            let mut content = Vec::new();
            for delta_item in text.to_delta() {
                let insert_tuple = match delta_item.as_insert() {
                    Some(i) => i,
                    None => continue,
                };

                let (insert, attributes) = insert_tuple;

                if insert.len() == 0 {
                    continue;
                }

                // Process marks if any
                let marks = match attributes {
                    Some(attributes) => {
                        let mark_array: Vec<Value> = attributes
                            .iter()
                            .map(|(name, value)| {
                                json!({
                                    "type": name,
                                    "attrs": value
                                })
                            })
                            .collect();

                        if mark_array.is_empty() {
                            Value::Null
                        } else {
                            Value::Array(mark_array)
                        }
                    }
                    None => Value::Null,
                };

                // Add the text node
                content.push(json!({
                    "type": "text",
                    "text": insert,
                    "marks": marks
                }));
            }
            content
        } else {
            // Empty text, but add a space to avoid errors
            vec![json!({
                "type": "text",
                "text": " ",
                "marks": Value::Null
            })]
        }
    }

    /// Helper to convert Loro doc to ProseMirror JSON format
    fn loro_doc_to_pm_doc(&self, loro_doc: &LoroDoc) -> Result<Value, String> {
        console_log!("Converting Loro doc to ProseMirror format");

        // Get the root map using the constant for consistency
        let root_doc = loro_doc.get_map(ROOT_DOC_KEY);

        // Get node type (nodeName in the Loro-ProseMirror convention)
        let node_type = match root_doc.get(NODE_NAME_KEY) {
            Some(ValueOrContainer::Value(LoroValue::String(s))) => s.to_string(),
            _ => {
                return Err(format!(
                    "Document root missing '{}' attribute",
                    NODE_NAME_KEY
                ))
            }
        };

        // Get attributes map
        let attrs = match root_doc.get(ATTRIBUTES_KEY) {
            Some(ValueOrContainer::Container(Container::Map(attrs_map))) => {
                // Convert attributes to JSON object
                let mut attrs_obj = Map::new();
                attrs_map.for_each(|key, value| match value {
                    ValueOrContainer::Value(LoroValue::String(s)) => {
                        attrs_obj.insert(key.to_string(), Value::String(s.to_string()));
                    }
                    ValueOrContainer::Value(LoroValue::Bool(b)) => {
                        attrs_obj.insert(key.to_string(), Value::Bool(b));
                    }
                    ValueOrContainer::Value(LoroValue::Double(n)) => {
                        if let Some(num) = serde_json::Number::from_f64(n) {
                            attrs_obj.insert(key.to_string(), Value::Number(num));
                        }
                    }
                    ValueOrContainer::Value(LoroValue::I64(n)) => {
                        attrs_obj
                            .insert(key.to_string(), Value::Number(serde_json::Number::from(n)));
                    }
                    _ => {}
                });

                if attrs_obj.is_empty() {
                    Value::Null
                } else {
                    Value::Object(attrs_obj)
                }
            }
            _ => Value::Null, // No attributes
        };

        // Get children list
        let children_list = match root_doc.get(CHILDREN_KEY) {
            Some(ValueOrContainer::Container(Container::List(list))) => list,
            _ => return Err(format!("Document root missing '{}' list", CHILDREN_KEY)),
        };

        // Convert children items
        let mut content_json = Vec::new();

        for i in 0..children_list.len() {
            match children_list.get(i) {
                Some(ValueOrContainer::Container(Container::Map(node_map))) => {
                    // Handle regular nodes (non-text)
                    let node_json = self.convert_loro_map_to_pm_node(&node_map)?;
                    content_json.push(node_json);
                }
                Some(ValueOrContainer::Container(Container::Text(text))) => {
                    content_json.extend(self.text_to_pm_node(&text));
                }
                _ => {
                    console_log!("Skipping unsupported child type at index {}", i);
                }
            }
        }

        // Construct the root document node
        let doc_json = json!({
            "type": node_type,
            "attrs": attrs,
            "content": content_json
        });

        Ok(doc_json)
    }

    /// Helper to convert a Loro map (non-text node) to a ProseMirror node
    fn convert_loro_map_to_pm_node(&self, map: &LoroMap) -> Result<Value, String> {
        // Get node type (nodeName in the Loro-ProseMirror convention)
        let node_type = match map.get(NODE_NAME_KEY) {
            Some(ValueOrContainer::Value(LoroValue::String(s))) => s.to_string(),
            _ => return Err(format!("Node missing '{}' attribute", NODE_NAME_KEY)),
        };

        // Get attributes as (pre-)JSON map
        let attrs = match map.get(ATTRIBUTES_KEY) {
            Some(ValueOrContainer::Container(Container::Map(attrs_map))) => {
                // Convert attributes to JSON object
                let mut attrs_obj = Map::new();
                attrs_map.for_each(|key, value| match value {
                    ValueOrContainer::Value(LoroValue::String(s)) => {
                        attrs_obj.insert(key.to_string(), Value::String(s.to_string()));
                    }
                    ValueOrContainer::Value(LoroValue::Bool(b)) => {
                        attrs_obj.insert(key.to_string(), Value::Bool(b));
                    }
                    ValueOrContainer::Value(LoroValue::Double(n)) => {
                        if let Some(num) = serde_json::Number::from_f64(n) {
                            attrs_obj.insert(key.to_string(), Value::Number(num));
                        }
                    }
                    ValueOrContainer::Value(LoroValue::I64(n)) => {
                        attrs_obj
                            .insert(key.to_string(), Value::Number(serde_json::Number::from(n)));
                    }
                    _ => {}
                });

                if attrs_obj.is_empty() {
                    Value::Null
                } else {
                    Value::Object(attrs_obj)
                }
            }
            _ => Value::Null, // No attributes
        };

        // Get children list
        let children_list = match map.get(CHILDREN_KEY) {
            Some(ValueOrContainer::Container(Container::List(list))) => list,
            _ => {
                // Node has no children, return node without content
                return Ok(json!({
                    "type": node_type,
                    "attrs": attrs
                }));
            }
        };

        // Convert children
        let mut content = Vec::new();

        for i in 0..children_list.len() {
            match children_list.get(i) {
                Some(ValueOrContainer::Container(Container::Map(child_map))) => {
                    // Recurse for regular node children (do we need a max tree depth?)
                    match self.convert_loro_map_to_pm_node(&child_map) {
                        Ok(node_json) => content.push(node_json),
                        Err(e) => console_log!("Error converting child node: {}", e),
                    }
                }
                Some(ValueOrContainer::Container(Container::Text(text))) => {
                    // For text nodes
                    content.extend(self.text_to_pm_node(&text));
                }
                _ => {
                    console_log!("Skipping unsupported child type at index {}", i);
                }
            }
        }

        let content = if content.is_empty() {
            Value::Null
        } else {
            Value::Array(content)
        };

        // Return complete node
        Ok(json!({
            "type": node_type,
            "attrs": attrs,
            "content": content
        }))
    }

    /// Helper to find a text node and its position in the document based on prosemirror position
    fn find_text_at_position(&self, loro_doc: &LoroDoc, position: usize) -> Result<(LoroText, usize, usize), String> {
        // Get the root map
        let root_map = loro_doc.get_map(ROOT_DOC_KEY);
        
        // Make sure it has a children list
        let children = match root_map.get(CHILDREN_KEY) {
            Some(ValueOrContainer::Container(Container::List(list))) => list,
            _ => return Err("Document root missing children list".to_string()),
        };
        
        // Keep track of the current position as we traverse the document
        let mut current_pos = 0;
        
        // Iterate through the top-level nodes (paragraphs, etc.)
        for i in 0..children.len() {
            match children.get(i) {
                Some(ValueOrContainer::Container(Container::Map(node_map))) => {
                    // Get the node's children
                    let node_children = match node_map.get(CHILDREN_KEY) {
                        Some(ValueOrContainer::Container(Container::List(list))) => list,
                        _ => continue, // Skip nodes without children
                    };
                    
                    // Iterate through the node's children
                    for j in 0..node_children.len() {
                        match node_children.get(j) {
                            Some(ValueOrContainer::Container(Container::Text(text))) => {
                                // Calculate the start and end position of this text node
                                let text_len = text.len_unicode();
                                let text_start = current_pos;
                                let text_end = text_start + text_len;
                                
                                // Check if the position is within this text node
                                if position >= text_start && position <= text_end {
                                    // Return the text node, its start position, and relative position
                                    return Ok((text.clone(), text_start, position - text_start));
                                }
                                
                                // Move the current position forward
                                current_pos += text_len;
                            },
                            _ => {
                                // Other node types contribute to position in ProseMirror
                                // For simplicity, we're assuming 1 position unit per non-text node
                                current_pos += 1;
                            }
                        }
                    }
                    
                    // Node boundaries also contribute to position in ProseMirror
                    current_pos += 1;
                },
                Some(ValueOrContainer::Container(Container::Text(text))) => {
                    // Direct text node at the root level
                    let text_len = text.len_unicode();
                    let text_start = current_pos;
                    let text_end = text_start + text_len;
                    
                    if position >= text_start && position <= text_end {
                        return Ok((text.clone(), text_start, position - text_start));
                    }
                    
                    current_pos += text_len;
                },
                _ => {
                    // Other container types - skip
                    current_pos += 1;
                }
            }
        }
        
        Err(format!("No text node found at position {}", position))
    }

    /// Apply ProseMirror steps to a Loro document
    fn apply_steps_to_loro_doc(
        &self,
        loro_doc: &mut LoroDoc,
        steps: &[Value],
    ) -> Result<(), JsValue> {
        console_log!("Applying steps to Loro document");

        for step in steps {
            let step_type = match step.get("stepType").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => {
                    console_log!("Step missing stepType: {:?}", step);
                    continue;
                }
            };

            match step_type {
                "replace" => {
                    // Extract from/to positions and slice content
                    let from = step.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let to = step.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                    console_log!("Replace operation from {} to {}", from, to);

                    // Check if this is a deletion (from != to with no content)
                    if from != to {
                        // Try to find the text node containing this range
                        match self.find_text_at_position(loro_doc, from) {
                            Ok((text, text_start, rel_from)) => {
                                let rel_to = std::cmp::min(to, text_start + text.len_unicode()) - text_start;
                                
                                // Delete the text in this range
                                if rel_from < rel_to {
                                    let delete_len = rel_to - rel_from;
                                    console_log!("Deleting text: from={}, len={}", rel_from, delete_len);
                                    text.delete(rel_from, delete_len);
                                }
                            },
                            Err(e) => {
                                console_log!("Error finding text at position {}: {}", from, e);
                            }
                        }
                    }

                    // Check if there's content to insert
                    if let Some(slice) = step.get("slice") {
                        if let Some(content_arr) = slice.get("content").and_then(|v| v.as_array()) {
                            if !content_arr.is_empty() {
                                console_log!("Inserting content at position {}", from);
                                
                                // Try to find the text node at this position
                                match self.find_text_at_position(loro_doc, from) {
                                    Ok((text, _, rel_pos)) => {
                                        // For simple text content, extract and insert
                                        for item in content_arr {
                                            if let Some(text_obj) = item.as_object() {
                                                if let Some(text_type) = text_obj.get("type").and_then(|v| v.as_str()) {
                                                    if text_type == "text" {
                                                        if let Some(content) = text_obj.get("text").and_then(|v| v.as_str()) {
                                                            console_log!("Inserting text: '{}' at position {}", content, rel_pos);
                                                            
                                                            // Check if there are marks to apply with the text
                                                            if let Some(marks) = text_obj.get("marks").and_then(|v| v.as_array()) {
                                                                if !marks.is_empty() {
                                                                    // Create a delta for text insertion with formatting
                                                                    let mut delta = Vec::new();
                                                                    
                                                                    // First retain the text before the insertion position
                                                                    if rel_pos > 0 {
                                                                        delta.push(json!({
                                                                            "retain": rel_pos
                                                                        }));
                                                                    }
                                                                    
                                                                    // Collect all the marks' attributes
                                                                    let mut all_attributes = serde_json::Map::new();
                                                                    
                                                                    for mark in marks {
                                                                        if let Some(mark_obj) = mark.as_object() {
                                                                            if let Some(mark_type) = mark_obj.get("type").and_then(|v| v.as_str()) {
                                                                                let attrs = mark_obj.get("attrs").cloned().unwrap_or(Value::Null);
                                                                                all_attributes.insert(mark_type.to_string(), attrs);
                                                                            }
                                                                        }
                                                                    }
                                                                    
                                                                    // Add the insert operation with all marks
                                                                    delta.push(json!({
                                                                        "insert": content,
                                                                        "attributes": all_attributes
                                                                    }));
                                                                    
                                                                    // Apply the delta to insert formatted text
                                                                    console_log!("Inserting formatted text at position {}", rel_pos);
                                                                    text.apply_delta(delta);
                                                                } else {
                                                                    // Simple insert without marks
                                                                    text.insert(rel_pos, content);
                                                                }
                                                            } else {
                                                                // Simple insert without marks
                                                                text.insert(rel_pos, content);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        console_log!("Error finding text at position {}: {}", from, e);
                                    }
                                }
                            }
                        }
                    }
                }

                "addMark" => {
                    // Extract from/to positions and mark information
                    let from = step.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let to = step.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                    if let Some(mark) = step.get("mark") {
                        let mark_type = mark.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        let mark_attrs = mark.get("attrs");

                        console_log!("Adding mark '{}' from {} to {}", mark_type, from, to);

                        // Try to find the text node for this range
                        match self.find_text_at_position(loro_doc, from) {
                            Ok((text, text_start, rel_from)) => {
                                // Calculate relative end within this text node
                                let rel_to = std::cmp::min(to, text_start + text.len_unicode()) - text_start;
                                
                                if rel_from < rel_to {
                                    let length = rel_to - rel_from;
                                    
                                    // Prepare delta with formatting to apply
                                    let mut delta = Vec::new();
                                    
                                    // First retain the text before the formatting position
                                    if rel_from > 0 {
                                        delta.push(json!({
                                            "retain": rel_from
                                        }));
                                    }
                                    
                                    // Create the attributes object for the mark
                                    let mut attributes = serde_json::Map::new();
                                    
                                    // Add mark attributes if present
                                    if let Some(attrs) = mark_attrs {
                                        if let Some(attrs_obj) = attrs.as_object() {
                                            for (key, value) in attrs_obj {
                                                attributes.insert(key.clone(), value.clone());
                                            }
                                        }
                                    } else {
                                        // If no specific attributes, use an empty object
                                        attributes.insert("value".to_string(), Value::Bool(true));
                                    }
                                    
                                    // Add the retain operation with the formatting
                                    delta.push(json!({
                                        "retain": length,
                                        "attributes": {
                                            [mark_type]: attributes
                                        }
                                    }));
                                    
                                    // Apply the delta to the text
                                    console_log!("Applying format '{}' from {} to {}", mark_type, rel_from, rel_to);
                                    text.apply_delta(delta);
                                }
                            },
                            Err(e) => {
                                console_log!("Error finding text at position {}: {}", from, e);
                            }
                        }
                    }
                }

                "removeMark" => {
                    // Extract from/to positions and mark information
                    let from = step.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let to = step.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                    if let Some(mark) = step.get("mark") {
                        let mark_type = mark.get("type").and_then(|v| v.as_str()).unwrap_or("");

                        console_log!("Removing mark '{}' from {} to {}", mark_type, from, to);

                        // Try to find the text node for this range
                        match self.find_text_at_position(loro_doc, from) {
                            Ok((text, text_start, rel_from)) => {
                                // Calculate relative end within this text node
                                let rel_to = std::cmp::min(to, text_start + text.len_unicode()) - text_start;
                                
                                if rel_from < rel_to {
                                    let length = rel_to - rel_from;
                                    
                                    // Prepare delta with formatting to remove
                                    let mut delta = Vec::new();
                                    
                                    // First retain the text before the formatting position
                                    if rel_from > 0 {
                                        delta.push(json!({
                                            "retain": rel_from
                                        }));
                                    }
                                    
                                    // Add the retain operation with the formatting removal
                                    // In Loro/Quill, null attribute value means remove the attribute
                                    delta.push(json!({
                                        "retain": length,
                                        "attributes": {
                                            [mark_type]: null
                                        }
                                    }));
                                    
                                    // Apply the delta to the text
                                    console_log!("Removing format '{}' from {} to {}", mark_type, rel_from, rel_to);
                                    text.apply_delta(delta);
                                }
                            },
                            Err(e) => {
                                console_log!("Error finding text at position {}: {}", from, e);
                            }
                        }
                    }
                }

                // For other step types like replaceAround, addNodeMark, etc.
                _ => {
                    console_log!("Unsupported step type: {}", step_type);
                }
            }
        }

        // Commit the changes after all steps are applied
        loro_doc.commit();
        console_log!("Steps applied successfully");
        Ok(())
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
        main.set_title("Hello World Title!")
            .map_err(|e| format!("Failed to set page title: {}", e))?;
        main.init_body_with_content("Hello World Body!")
            .map_err(|e| format!("Failed to set page body: {}", e))?;

        // Create default post
        let post = self.create_file("post", "post")?;
        post.set_title("Hello World Title!")
            .map_err(|e| format!("Failed to set post title: {}", e))?;
        post.init_body_with_content("Hello World Body!")
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
                file.init_body()
                    .map_err(|e| format!("Failed to set body: {}", e))?;
                file.set_title(name)
                    .map_err(|e| format!("Failed to set title: {}", e))?;
                file.set_field("template", "index")
                    .map_err(|e| format!("Failed to set template: {}", e))?;
            }
            "post" => {
                file.init_body()
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

type RichtextCallback = Closure<dyn Fn(JsValue, JsValue, JsValue)>;

// File: Wrapper around a LoroTree handler and node ID that encapsulates file-specific functionality
#[derive(Clone)]
pub struct File {
    pub files: LoroTree, // Handler for the files tree in the LoroDoc
    pub id: TreeID,
    pub collection_type: String,
    // pub richtext_callback: Option<RichtextCallback>,
}

impl File {
    pub fn new(files: &LoroTree, id: TreeID, collection_type: &str) -> Self {
        File {
            files: files.clone(), // Clone the handler, not the underlying data
            id,
            collection_type: collection_type.to_string(),
            // richtext_callback: None,
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

    // pub fn register_richtext_callback(&mut self, callback: js_sys::Function) -> Result<(), String> {
    //     // For page and post types
    //     if self.collection_type == "page" || self.collection_type == "post" {
    //         let data = match self.files.get_meta(self.id) {
    //             Ok(meta) => meta,
    //             Err(_) => return Err("Node metadata not found".to_string()),
    //         };

    //         if let Ok(cb) = callback.dyn_into::<js_sys::Function>() {
    //             self.richtext_callback = Some(Closure::wrap(Box::new(
    //                 move |body: &LoroMap, doc: &LoroDoc, id: ContainerID| {
    //                     let _ = cb.call3(&JsValue::NULL, body.into(), doc.into(), &id.into());
    //                 },
    //             )
    //                 as Box<dyn Fn(JsValue, JsValue, JsValue)>));
    //         }
    //     }

    //     Ok(())
    // }

    pub fn init_body_with_content(&self, content: &str) -> Result<(), String> {
        // For page and post types
        // TODO: handle other types
        let doc = match self.files.doc() {
            Some(doc) => doc,
            None => return Err("Document not found".to_string()),
        };
        if self.collection_type == "page" || self.collection_type == "post" {
            let data = match self.files.get_meta(self.id) {
                Ok(meta) => meta,
                Err(_) => return Err("Node metadata not found".to_string()),
            };

            if !data.get("body").is_some() {
                let body_root = LoroMap::new(); // LoroMap as root node for rich text
                data.insert_container("body", body_root);
            }

            if let Some(ValueOrContainer::Container(Container::Map(body))) = data.get("body") {
                if body.get("children").is_none() {
                    body.insert_container("children", LoroList::new());
                }
                match body.get("children") {
                    Some(ValueOrContainer::Container(Container::List(list))) => {
                        match list.get(0) {
                            Some(ValueOrContainer::Container(Container::Text(text))) => {
                                // body is already initialized
                                Ok(())
                            }
                            _ => {
                                list.insert_container(0, LoroText::new());
                                Ok(())
                            }
                        }
                    }
                    _ => {
                        body.insert_container("children", LoroList::new());
                        match body.get("children") {
                            Some(ValueOrContainer::Container(Container::List(list))) => {
                                list.insert_container(0, LoroText::new());
                                if let Some(ValueOrContainer::Container(Container::Text(text))) =
                                    list.get(0)
                                {
                                    text.delete(0, text.len_utf8());
                                    text.insert(0, content);
                                    Ok(())
                                } else {
                                    Err("Body not initialized".to_string())
                                }
                            }
                            _ => Err("Body not initialized".to_string()),
                        }
                    }
                }
            } else {
                Err("Body not initialized".to_string())
            }
        } else {
            Err("Body is only available for page and post types".to_string())
        }
    }

    pub fn init_body(&self) -> Result<(), String> {
        self.init_body_with_content("")
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

fn loro_map_to_js(map: &LoroMap) -> JsValue {
    let obj = Object::new();
    // Convert map contents to JS (simplified example)
    JsValue::from(obj)
}

fn loro_doc_to_js(doc: &LoroDoc) -> JsValue {
    let obj = Object::new();
    // Convert document metadata
    JsValue::from(obj)
}

fn container_id_to_js(id: &ContainerID) -> JsValue {
    JsValue::from(id.to_string()) // Assuming ContainerID is stringifiable
}
