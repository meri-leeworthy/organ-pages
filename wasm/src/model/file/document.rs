pub use document::*;

pub mod document {
    use std::{collections::HashMap, convert::TryFrom};

    use loro::{
        Container, LoroDoc, LoroError, LoroList, LoroMap, LoroText, LoroValue, TextDelta,
        ValueOrContainer,
    };
    use loro_delta::DeltaItem;
    use loro_internal::{event::TextMeta, FxHashMap, StringSlice};
    use serde_json::{json, Map, Value};
    use wasm_bindgen::prelude::*;

    use crate::{
        model::file::{File, ProseMirrorSchema},
        FileStore, PM_SCHEMA_KEY,
    };

    // Constants for ProseMirror-Loro integration
    pub const ROOT_DOC_KEY: &str = "doc";
    pub const ATTRIBUTES_KEY: &str = "attributes";
    pub const CHILDREN_KEY: &str = "children";
    pub const CONTENT_KEY: &str = "content";
    pub const NODE_NAME_KEY: &str = "nodeName"; // what is this?

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str); // log to JS console
    }

    // Helper macro for logging
    macro_rules! console_log {
        ($($t:tt)*) => (log(&format!("[Document (WASM)] {}", format!($($t)*))))
    }

    pub fn initialize_richtext_document(
        doc: &LoroDoc,
        schema: &ProseMirrorSchema,
    ) -> Result<(), LoroError> {
        // Configure text style for marks if schema is provided
        if !schema.marks.is_empty() {
            self::configure_text_style(&doc, schema);
        }

        // Initialize the document structure according to Loro-ProseMirror convention
        let root_map = doc.get_map(ROOT_DOC_KEY);
        root_map.insert(NODE_NAME_KEY, "doc".to_string())?; // what is this?

        // Add attributes map
        let attrs_map = LoroMap::new();
        root_map.get_or_create_container(ATTRIBUTES_KEY, attrs_map)?;

        // Add children list
        let children_list = LoroList::new();
        root_map.get_or_create_container(CHILDREN_KEY, children_list)?;

        // Get reference to children list
        let children = match root_map.get(CHILDREN_KEY) {
            Some(ValueOrContainer::Container(Container::List(list))) => list,
            _ => {
                return Err(LoroError::NotFoundError(
                    "Failed to get children list".into(),
                ))
            }
        };

        // Create paragraph map
        let para_map = LoroMap::new();
        para_map.insert(NODE_NAME_KEY, "paragraph".to_string());

        // Add paragraph attributes
        let para_attrs = LoroMap::new();
        para_map.get_or_create_container(ATTRIBUTES_KEY, para_attrs);

        // Add paragraph children list
        let para_children = LoroList::new();
        para_map.get_or_create_container(CHILDREN_KEY, para_children);

        // Add paragraph to document children
        children.insert_container(0, para_map);

        let para_map = match children.get(0) {
            Some(ValueOrContainer::Container(Container::Map(map))) => map,
            _ => {
                return Err(LoroError::NotFoundError(
                    "Failed to get paragraph map".into(),
                ))
            }
        };

        let para_children = match para_map.get(CHILDREN_KEY) {
            Some(ValueOrContainer::Container(Container::List(list))) => list,
            _ => {
                return Err(LoroError::NotFoundError(
                    "Failed to get paragraph children list".into(),
                ))
            }
        };

        // Create text node with space to avoid empty text node errors
        // In Loro-ProseMirror, text nodes are LoroText directly, not maps with text props
        let text = LoroText::new();
        // text.insert(0, " "); // Space character to avoid empty text node errors

        // Add text to paragraph children
        para_children.insert_container(0, text);

        // console_log!("Rich text document initialized successfully");

        Ok(())
    }

    pub fn initialize_plaintext_document(doc: &LoroDoc) -> Result<(), LoroError> {
        // Initialize the document structure according to Loro-ProseMirror convention
        let root_map = doc.get_map(ROOT_DOC_KEY);
        root_map.insert(NODE_NAME_KEY, "doc".to_string())?; // what is this?

        // Add attributes map
        let attrs_map = LoroMap::new();
        root_map.get_or_create_container(ATTRIBUTES_KEY, attrs_map)?;

        // Add children list
        let content = LoroText::new();
        root_map.get_or_create_container(CONTENT_KEY, content)?;

        // console_log!("Plain text document initialized successfully");

        Ok(())
    }

    /// Configure text style expansion behavior based on ProseMirror schema
    pub fn configure_text_style(doc: &LoroDoc, schema: &ProseMirrorSchema) -> Result<(), String> {
        // Build text style config map
        let mut text_style_config = HashMap::new();

        // Process each mark and build text style config
        for (mark_name, mark_def) in &schema.marks {
            // Default to "after" for inclusive marks (matching the JS implementation)
            let expand = if mark_def.inclusive { "after" } else { "none" };

            // Store the config
            text_style_config.insert(mark_name.clone(), json!({ "expand": expand }));
        }

        // Log the configuration for debugging
        // console_log!("Text style config: {:?}", text_style_config);

        // In Loro, we'd configure text style directly with something like:
        // doc.config_text_style(text_style_config);
        // However, since there's no direct binding for this in the WASM API,
        // we'll simulate it by storing the config in a special metadata map

        // Store the text style config in a special metadata map for reference
        if !text_style_config.is_empty() {
            let meta_map = doc.get_map("__meta");
            let styles_map = LoroMap::new();
            meta_map.insert_container("textStyles", styles_map);

            if let Some(ValueOrContainer::Container(Container::Map(styles))) =
                meta_map.get("textStyles")
            {
                for (mark_name, style_config) in text_style_config {
                    // Convert the JSON value to a string representation
                    if let Ok(config_str) = serde_json::to_string(&style_config) {
                        styles.insert(&mark_name, config_str);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn text_to_pm_node(text: &LoroText) -> Vec<Value> {
        // Handle text nodes - these are LoroText objects directly
        if text.len_unicode() > 0 {
            // Get the Delta format which includes formatting
            let mut content = Vec::new();
            for delta_item in text.to_delta() {
                // console_log!("delta_item: {:?}", delta_item);
                if let Some(insert_tuple) = delta_item.as_insert() {
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
                if let Some(delete_length) = delta_item.as_delete() {
                    console_log!("delete_length: {:?}", delete_length);
                }
                if let Some(retain_tuple) = delta_item.as_retain() {
                    let (retain_length, attributes) = retain_tuple;
                    console_log!("retain_length: {:?}", retain_length);
                    console_log!("attributes: {:?}", attributes);
                }
            }
            content
        } else {
            // Empty text, but add a space to avoid errors // or don't!
            vec![json!({
                "type": "text",
                "text": "",
                "marks": Value::Null
            })]
        }
    }

    /// Helper to convert Loro doc to ProseMirror JSON format
    pub fn loro_doc_to_pm_doc(loro_doc: &LoroDoc) -> Result<Value, String> {
        // console_log!("Converting Loro doc to ProseMirror format");

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
                    let node_json = self::convert_loro_map_to_pm_node(&node_map)?;
                    content_json.push(node_json);
                }
                Some(ValueOrContainer::Container(Container::Text(text))) => {
                    content_json.extend(self::text_to_pm_node(&text));
                }
                _ => {
                    // console_log!("Skipping unsupported child type at index {}", i);
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
    pub fn convert_loro_map_to_pm_node(map: &LoroMap) -> Result<Value, String> {
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
                    match self::convert_loro_map_to_pm_node(&child_map) {
                        Ok(node_json) => content.push(node_json),
                        Err(e) => console_log!("Error converting child node: {}", e),
                    }
                }
                Some(ValueOrContainer::Container(Container::Text(text))) => {
                    // For text nodes
                    content.extend(self::text_to_pm_node(&text));
                }
                _ => {
                    // console_log!("Skipping unsupported child type at index {}", i);
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
    pub fn find_text_at_position(
        loro_doc: &LoroDoc,
        position: usize,
    ) -> Result<(LoroText, usize, usize), String> {
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
                    // console_log!("Found Map node at index {}", i);
                    // Get the node's children
                    let node_children = match node_map.get(CHILDREN_KEY) {
                        Some(ValueOrContainer::Container(Container::List(list))) => list,
                        _ => {
                            // console_log!("Skipping Map node at index {} without children", i);
                            continue;
                        } // Skip nodes without children
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
                            }
                            _ => {
                                // Other node types contribute to position in ProseMirror
                                // For simplicity, we're assuming 1 position unit per non-text node
                                current_pos += 1;
                            }
                        }
                    }

                    // Node boundaries also contribute to position in ProseMirror
                    current_pos += 1;
                }
                Some(ValueOrContainer::Container(Container::Text(text))) => {
                    // console_log!("Found text node at index {}", i);
                    // Direct text node at the root level
                    let text_len = text.len_unicode();
                    let text_start = current_pos;
                    let text_end = text_start + text_len;

                    if position >= text_start && position <= text_end {
                        return Ok((text.clone(), text_start, position - text_start));
                    }

                    current_pos += text_len;
                }
                _ => {
                    // console_log!("Skipping unsupported child type at index {}", i);
                    // Other container types - skip
                    current_pos += 1;
                }
            }
        }

        Err(format!("No text node found at position {}", position)) // or maybe insert a text node at this position?
    }

    /// Apply ProseMirror steps to a Loro document
    pub fn apply_steps_to_loro_doc(loro_doc: &LoroDoc, steps: &[Value]) -> Result<(), JsValue> {
        // console_log!("Applying steps to Loro document");

        // Track whether we made changes that need to be committed
        let mut has_changes = false;

        for step in steps {
            let step_type = match step.get("stepType").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => {
                    // console_log!("Step missing stepType: {:?}", step);
                    continue;
                }
            };

            match step_type {
                "replace" => {
                    // Extract from/to positions and slice content
                    let from = step.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let to = step.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                    // console_log!("Replace operation from {} to {}", from, to);

                    // Check if this is a deletion (from != to)
                    if from != to {
                        // Try to find the text node containing this range
                        match self::find_text_at_position(loro_doc, from) {
                            Ok((text, text_start, rel_from)) => {
                                let rel_to =
                                    std::cmp::min(to, text_start + text.len_unicode()) - text_start;

                                // Delete the text in this range
                                if rel_from < rel_to {
                                    let delete_len = rel_to - rel_from;
                                    // console_log!(
                                    //     "Deleting text: from={}, len={}",
                                    //     rel_from,
                                    //     delete_len
                                    // );
                                    text.delete(rel_from, delete_len);
                                    has_changes = true;
                                }

                                // If the range spans multiple nodes, we need to handle that case
                                // This is a simplified approach - we only delete from the first node
                                if to > text_start + text.len_unicode() {
                                    // console_log!("Warning: Range spans multiple nodes, only deleting from first node");
                                }
                            }
                            Err(e) => {
                                // console_log!("Error finding text at position {}: {}", from, e);
                            }
                        }
                    }

                    // Check if there's content to insert
                    if let Some(slice) = step.get("slice") {
                        if let Some(content_arr) = slice.get("content").and_then(|v| v.as_array()) {
                            if !content_arr.is_empty() {
                                // console_log!("Inserting content at position {}", from);

                                // Try to find the text node at this position
                                match self::find_text_at_position(loro_doc, from) {
                                    Ok((text, _, rel_pos)) => {
                                        // For simple text content, extract and insert
                                        for item in content_arr {
                                            if let Some(text_obj) = item.as_object() {
                                                if let Some(text_type) =
                                                    text_obj.get("type").and_then(|v| v.as_str())
                                                {
                                                    if text_type == "text" {
                                                        if let Some(content) = text_obj
                                                            .get("text")
                                                            .and_then(|v| v.as_str())
                                                        {
                                                            // Skip empty content
                                                            if content.is_empty() {
                                                                continue;
                                                            }

                                                            // console_log!(
                                                            //   "Inserting text: '{}' at position {}",
                                                            //   content,
                                                            //   rel_pos
                                                            // );

                                                            // Check if there are marks to apply with the text
                                                            if let Some(marks) = text_obj
                                                                .get("marks")
                                                                .and_then(|v| v.as_array())
                                                            {
                                                                if !marks.is_empty() {
                                                                    // Create a delta for text insertion with formatting
                                                                    let mut delta: Vec<
                                                                        &DeltaItem<
                                                                            StringSlice,
                                                                            TextMeta,
                                                                        >,
                                                                    > = Vec::new();

                                                                    let retain =
                                                                        DeltaItem::Retain {
                                                                            len: rel_pos,
                                                                            attr: TextMeta::default(
                                                                            ),
                                                                        };

                                                                    // First retain the text before the insertion position
                                                                    if rel_pos > 0 {
                                                                        delta.push(&retain);
                                                                    }

                                                                    // Collect all the marks' attributes
                                                                    let mut all_attributes: FxHashMap<
                                                                  String,
                                                                  LoroValue,
                                                              > = FxHashMap::default();

                                                                    for mark in marks {
                                                                        if let Some(mark_obj) =
                                                                            mark.as_object()
                                                                        {
                                                                            if let Some(mark_type) =
                                                                                mark_obj
                                                                                    .get("type")
                                                                                    .and_then(|v| {
                                                                                        v.as_str()
                                                                                    })
                                                                            {
                                                                                let attrs = mark_obj
                                                                              .get("attrs")
                                                                              .cloned()
                                                                              .unwrap_or(
                                                                                  Value::Null,
                                                                              );
                                                                                all_attributes
                                                                                    .insert(
                                                                                    mark_type
                                                                                        .to_string(
                                                                                        ),
                                                                                    attrs.into(),
                                                                                );
                                                                            }
                                                                        }
                                                                    }

                                                                    let replace =
                                                                        DeltaItem::Replace {
                                                                            value: content.into(),
                                                                            attr: TextMeta(
                                                                                all_attributes,
                                                                            ),
                                                                            delete: 0,
                                                                        };

                                                                    // Add the insert operation with all marks
                                                                    delta.push(&replace);

                                                                    let delta: Vec<TextDelta> =
                                                                        TextDelta::from_text_diff(
                                                                            delta.into_iter(),
                                                                        );

                                                                    // Apply the delta to insert formatted text
                                                                    // console_log!("Inserting formatted text at position {}", rel_pos);
                                                                    text.apply_delta(&delta);
                                                                    has_changes = true;
                                                                } else {
                                                                    // Simple insert without marks
                                                                    text.insert(rel_pos, content);
                                                                    has_changes = true;
                                                                }
                                                            } else {
                                                                // Simple insert without marks
                                                                text.insert(rel_pos, content);
                                                                has_changes = true;
                                                            }
                                                        }
                                                    } else {
                                                        // Non-text node (paragraph, etc.)
                                                        // console_log!(
                                                        //     "Skipping non-text node insertion: {}",
                                                        //     text_type
                                                        // );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        // console_log!(
                                        //     "Error finding text at position {}: {}",
                                        //     from,
                                        //     e
                                        // );
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

                        // console_log!("Adding mark '{}' from {} to {}", mark_type, from, to);

                        // Try to find the text node for this range
                        match self::find_text_at_position(loro_doc, from) {
                            Ok((text, text_start, rel_from)) => {
                                // Calculate relative end within this text node
                                let rel_to =
                                    std::cmp::min(to, text_start + text.len_unicode()) - text_start;

                                if rel_from < rel_to {
                                    let length = rel_to - rel_from;

                                    // Prepare delta with formatting to apply
                                    let mut delta: Vec<&DeltaItem<StringSlice, TextMeta>> =
                                        Vec::new();

                                    let retain = DeltaItem::Retain {
                                        len: rel_from,
                                        attr: TextMeta::default(),
                                    };

                                    // First retain the text before the formatting position
                                    if rel_from > 0 {
                                        delta.push(&retain);
                                    }

                                    // Create the attributes object for the mark
                                    let mut attributes: FxHashMap<String, LoroValue> =
                                        FxHashMap::default();

                                    // Add mark attributes if present
                                    if let Some(attrs) = mark_attrs {
                                        if let Some(attrs_obj) = attrs.as_object() {
                                            for (key, value) in attrs_obj {
                                                attributes
                                                    .insert(key.clone(), value.clone().into());
                                            }
                                        }
                                    } else {
                                        // If no specific attributes, use an empty object
                                        attributes
                                            .insert("value".to_string(), Value::Bool(true).into());
                                    }

                                    let retain = DeltaItem::Retain {
                                        len: length,
                                        attr: TextMeta(attributes),
                                    };

                                    // Add the retain operation with the formatting
                                    delta.push(&retain);

                                    // Apply the delta to the text
                                    // console_log!(
                                    //     "Applying format '{}' from {} to {}",
                                    //     mark_type,
                                    //     rel_from,
                                    //     rel_to
                                    // );

                                    let delta: Vec<TextDelta> =
                                        TextDelta::from_text_diff(delta.into_iter());

                                    text.apply_delta(&delta);
                                    has_changes = true;

                                    // If the range spans multiple nodes, we need to handle that case
                                    if to > text_start + text.len_unicode() {
                                        // console_log!("Warning: Formatting spans multiple nodes, only applying to first node");
                                    }
                                }
                            }
                            Err(e) => {
                                // console_log!("Error finding text at position {}: {}", from, e);
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

                        // console_log!("Removing mark '{}' from {} to {}", mark_type, from, to);

                        // Try to find the text node for this range
                        match self::find_text_at_position(loro_doc, from) {
                            Ok((text, text_start, rel_from)) => {
                                // Calculate relative end within this text node
                                let rel_to =
                                    std::cmp::min(to, text_start + text.len_unicode()) - text_start;

                                if rel_from < rel_to {
                                    let length = rel_to - rel_from;

                                    // Prepare delta with formatting to remove
                                    let mut delta: Vec<&DeltaItem<StringSlice, TextMeta>> =
                                        Vec::new();

                                    let retain = DeltaItem::Retain {
                                        len: rel_from,
                                        attr: TextMeta::default(),
                                    };

                                    // First retain the text before the formatting position
                                    if rel_from > 0 {
                                        delta.push(&retain);
                                    }

                                    // Add the retain operation with the formatting removal
                                    // In Loro/Quill, null attribute value means remove the attribute

                                    let retain = DeltaItem::Retain {
                                        len: length,
                                        attr: TextMeta(FxHashMap::default()),
                                    };
                                    delta.push(&retain);

                                    // Apply the delta to the text
                                    console_log!(
                                        "Removing format '{}' from {} to {}",
                                        mark_type,
                                        rel_from,
                                        rel_to
                                    );

                                    let delta: Vec<TextDelta> =
                                        TextDelta::from_text_diff(delta.into_iter());
                                    text.apply_delta(&delta);
                                    has_changes = true;

                                    // If the range spans multiple nodes, we need to handle that case
                                    if to > text_start + text.len_unicode() {
                                        // console_log!("Warning: Removing formatting spans multiple nodes, only applying to first node");
                                    }
                                }
                            }
                            Err(e) => {
                                // console_log!("Error finding text at position {}: {}", from, e);
                            }
                        }
                    }
                }

                // For other step types like replaceAround, addNodeMark, etc.
                _ => {
                    // console_log!("Unsupported step type: {}", step_type);
                }
            }
        }

        // Only commit if we made changes
        if has_changes {
            // Commit the changes after all steps are applied
            loro_doc.commit();
            // console_log!("Steps applied successfully and changes committed");
        } else {
            // console_log!("No changes to commit");
        }

        Ok(())
    }

    pub trait HasContent: File {
        fn initialize_plaintext_document(&mut self) -> Result<(), String> {
            let store = self.store();
            let doc = match store {
                FileStore::Full(doc) => doc,
                FileStore::Cache(_) => {
                    return Err("Cannot initialize plaintext document from cache".to_string())
                }
            };
            self::initialize_plaintext_document(&doc).map_err(|e| e.to_string())
        }

        fn get_content(&self) -> Result<String, String> {
            // doc."doc" would contain "children" for richtext and "content" for plain text
            let doc = match self.store() {
                FileStore::Full(doc) => doc,
                FileStore::Cache(_) => return Err("Cannot get content from cache".to_string()),
            };
            let text = match doc.get_map("doc").get("content") {
                Some(ValueOrContainer::Container(Container::Text(text))) => text,
                _ => return Err("Content not found".to_string()),
            };
            Ok(text.to_string())
        }

        fn insert_content(&self, content: &str, position: usize) -> Result<(), String> {
            let doc = match self.store() {
                FileStore::Full(doc) => doc,
                FileStore::Cache(_) => return Err("Cannot get content from cache".to_string()),
            };
            let text = match doc.get_map("doc").get("content") {
                Some(ValueOrContainer::Container(Container::Text(text))) => text,
                _ => return Err("Content not found".to_string()),
            };
            text.insert(position, content);
            Ok(())
        }

        fn delete_content(&self, position: usize, length: usize) -> Result<(), String> {
            let doc = match self.store() {
                FileStore::Full(doc) => doc,
                FileStore::Cache(_) => return Err("Cannot get content from cache".to_string()),
            };
            let text = match doc.get_map("doc").get("content") {
                Some(ValueOrContainer::Container(Container::Text(text))) => text,
                _ => return Err("Content not found".to_string()),
            };
            text.delete(position, length);
            Ok(())
        }
    }
    pub trait HasRichText: File {
        fn schema_json(&self) -> String {
            let schema = self.schema();
            serde_json::to_string(&schema).unwrap_or_else(|_| "{}".to_string())
        }

        fn schema(&self) -> ProseMirrorSchema {
            let schema = match self.meta().get(PM_SCHEMA_KEY) {
                Some(ValueOrContainer::Value(LoroValue::String(schema))) => schema.to_string(),
                _ => "".to_string(),
            };
            let schema = ProseMirrorSchema::try_from(schema).unwrap_or_default();
            schema
        }

        fn initialize_richtext_document(&mut self) -> Result<(), String> {
            let schema = &self.schema().clone();
            let doc = match self.store() {
                FileStore::Full(doc) => doc,
                FileStore::Cache(_) => return Err("Cannot get content from cache".to_string()),
            };
            self::initialize_richtext_document(doc, schema).map_err(|e| e.to_string())
        }

        /// Apply ProseMirror steps to a Loro document
        fn apply_steps(&mut self, steps: &[Value], version: i64) -> Result<i64, String> {
            // console_log!(
            //     "Applying steps to document. Current version: {:?}, incoming version: {}",
            //     self.version(),
            //     version
            // );

            // Version check for conflict handling
            if let Ok(current_version) = self.version() {
                if version != current_version {
                    // console_log!(
                    //     "Version mismatch - current: {:?}, received: {}",
                    //     self.version(),
                    //     version
                    // );
                    // For now, we'll accept the steps but warn about potential conflicts
                }
            }

            let doc = match self.store() {
                FileStore::Full(doc) => doc,
                FileStore::Cache(_) => return Err("Cannot get content from cache".to_string()),
            };

            // Apply the steps to the Loro document
            self::apply_steps_to_loro_doc(doc, steps)
                .map_err(|e| format!("Failed to apply steps: {:?}", e))?;

            // Get current version and increment
            let new_version = self.version().unwrap_or(0) + 1;
            self.set_version(new_version)
                .map_err(|e| format!("Failed to set version: {:?}", e))?;

            // console_log!(
            //     "Steps applied successfully, new version: {:?}",
            //     self.version()
            // );

            Ok(new_version)
        }
        // fn get_rich_text(&self) -> Result<String, String> {
        //     let rich_text = self.doc().get_map("doc").get("rich_text");
        //     Ok(rich_text.to_string())
        // }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::{File, FileBuilder, FileStore, ProseMirrorSchema};

    use super::*;
    use loro::{Container, LoroDoc, LoroError, LoroMap, LoroValue, ValueOrContainer};
    use serde_json::{json, Value};
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Test implementation of File and HasRichText traits
    struct TestRichTextFile {
        store: FileStore,
    }

    impl File for TestRichTextFile {
        fn builder() -> FileBuilder<Self> {
            FileBuilder::new("test")
        }
        fn init(&mut self, _meta: Option<&LoroMap>) -> Result<(), String> {
            Ok(())
        }

        fn store(&self) -> &FileStore {
            &self.store
        }

        fn mut_store(&mut self) -> &mut FileStore {
            &mut self.store
        }

        fn get_type(&self) -> String {
            "test".to_string()
        }

        fn to_json(&self) -> Result<Value, String> {
            Ok(Value::Object(serde_json::Map::new()))
        }
    }

    impl HasRichText for TestRichTextFile {}

    // Test implementation for HasContent trait
    struct TestPlainTextFile {
        store: FileStore,
    }

    impl File for TestPlainTextFile {
        fn builder() -> FileBuilder<Self> {
            FileBuilder::new("test")
        }
        fn init(&mut self, _meta: Option<&LoroMap>) -> Result<(), String> {
            Ok(())
        }

        fn store(&self) -> &FileStore {
            &self.store
        }

        fn mut_store(&mut self) -> &mut FileStore {
            &mut self.store
        }

        fn get_type(&self) -> String {
            "test".to_string()
        }

        fn to_json(&self) -> Result<Value, String> {
            Ok(Value::Object(serde_json::Map::new()))
        }
    }

    impl HasContent for TestPlainTextFile {}

    #[wasm_bindgen_test]
    fn test_initialize_plaintext_document() {
        let mut file = TestPlainTextFile {
            store: FileStore::Full(LoroDoc::new()),
        };

        // Initialize the document
        file.initialize_plaintext_document()
            .expect("Failed to initialize");

        // Verify document structure
        let root_map = file.store.as_full().unwrap().get_map(ROOT_DOC_KEY);
        assert_eq!(
            root_map.get(NODE_NAME_KEY).and_then(|v| match v {
                ValueOrContainer::Value(LoroValue::String(s)) => Some(s.to_string()),
                _ => None,
            }),
            Some("doc".to_string())
        );

        // Verify content container exists
        assert!(matches!(
            root_map.get(CONTENT_KEY),
            Some(ValueOrContainer::Container(Container::Text(_)))
        ));
    }

    #[wasm_bindgen_test]
    fn test_plaintext_content_operations() {
        let mut file = TestPlainTextFile {
            store: FileStore::Full(LoroDoc::new()),
        };

        file.initialize_plaintext_document()
            .expect("Failed to initialize");

        // Test inserting content
        file.insert_content("Hello", 0).expect("Failed to insert");
        assert_eq!(file.get_content().unwrap(), "Hello");

        // Test inserting at specific position
        file.insert_content(" World", 5).expect("Failed to insert");
        assert_eq!(file.get_content().unwrap(), "Hello World");

        // Test deleting content
        file.delete_content(5, 6).expect("Failed to delete"); // Delete " World"
        assert_eq!(file.get_content().unwrap(), "Hello");
    }

    #[wasm_bindgen_test]
    fn test_initialize_richtext_document() {
        let schema = ProseMirrorSchema::try_from(
            r#"{
            "marks": {
                "bold": { "inclusive": true },
                "italic": { "inclusive": false }
            }
        }"#,
        )
        .unwrap();

        let mut file = TestRichTextFile {
            store: FileStore::Full(LoroDoc::new()),
        };

        // Initialize the document
        file.initialize_richtext_document()
            .expect("Failed to initialize");

        // Verify document structure
        let root_map = file.store.as_full().unwrap().get_map(ROOT_DOC_KEY);

        // Check node type
        assert_eq!(
            root_map.get(NODE_NAME_KEY).and_then(|v| match v {
                ValueOrContainer::Value(LoroValue::String(s)) => Some(s.to_string()),
                _ => None,
            }),
            Some("doc".to_string())
        );

        // Verify children list exists
        assert!(matches!(
            root_map.get(CHILDREN_KEY),
            Some(ValueOrContainer::Container(Container::List(_)))
        ));

        // Verify attributes map exists
        assert!(matches!(
            root_map.get(ATTRIBUTES_KEY),
            Some(ValueOrContainer::Container(Container::Map(_)))
        ));
    }

    #[wasm_bindgen_test]
    fn test_apply_steps_to_loro_doc() {
        let mut doc = LoroDoc::new();
        initialize_richtext_document(&mut doc, &ProseMirrorSchema::default())
            .expect("Failed to initialize");

        // Test inserting text
        let insert_step = json!({
            "stepType": "replace",
            "from": 0,
            "to": 0,
            "slice": {
                "content": [{
                    "type": "text",
                    "text": "Hello"
                }]
            }
        });

        apply_steps_to_loro_doc(&mut doc, &[insert_step]).expect("Failed to apply insert step");

        // Test adding mark
        let add_mark_step = json!({
            "stepType": "addMark",
            "from": 0,
            "to": 5,
            "mark": {
                "type": "bold"
            }
        });

        apply_steps_to_loro_doc(&mut doc, &[add_mark_step]).expect("Failed to apply mark step");

        // Convert back to ProseMirror format and verify
        let pm_doc = loro_doc_to_pm_doc(&doc).expect("Failed to convert to PM format");

        // Basic structure verification
        assert_eq!(pm_doc["type"], "doc");
        assert!(pm_doc["content"].as_array().unwrap().len() > 0);
    }

    #[wasm_bindgen_test]
    fn test_find_text_at_position() {
        let mut doc = LoroDoc::new();
        initialize_richtext_document(&mut doc, &ProseMirrorSchema::default())
            .expect("Failed to initialize");

        // Insert some text through steps
        let insert_step = json!({
            "stepType": "replace",
            "from": 0,
            "to": 0,
            "slice": {
                "content": [{
                    "type": "text",
                    "text": "Hello World"
                }]
            }
        });

        apply_steps_to_loro_doc(&mut doc, &[insert_step]).expect("Failed to apply insert step");

        // Test finding text at different positions
        let (text, start, rel_pos) =
            find_text_at_position(&doc, 0).expect("Failed to find text at position 0");
        assert_eq!(start, 0);
        assert_eq!(rel_pos, 0);

        let (text, start, rel_pos) =
            find_text_at_position(&doc, 5).expect("Failed to find text at position 5");
        assert_eq!(start, 0);
        assert_eq!(rel_pos, 5);

        // Test finding text at invalid position
        assert!(find_text_at_position(&doc, 100).is_err());
    }

    #[wasm_bindgen_test]
    fn test_configure_text_style() {
        let doc = LoroDoc::new();

        let schema = ProseMirrorSchema::try_from(json!({
            "marks": {
                "bold": { "inclusive": true },
                "italic": { "inclusive": false },
                "underline": { "inclusive": true }
            }
        }))
        .unwrap();

        configure_text_style(&doc, &schema).expect("Failed to configure text style");

        // Verify the configuration was stored
        let meta_map = doc.get_map("__meta");
        assert!(matches!(
            meta_map.get("textStyles"),
            Some(ValueOrContainer::Container(Container::Map(_)))
        ));
    }
}
