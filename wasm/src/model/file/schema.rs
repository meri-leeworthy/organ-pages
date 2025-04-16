use core::fmt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Display;

/// Represents a ProseMirror schema mark definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkDefinition {
    /// Whether the mark is inclusive (affects text style expansion)
    #[serde(default)]
    pub inclusive: bool,
    /// Additional attributes for the mark
    #[serde(default)]
    pub attrs: HashMap<String, Value>,
}

/// Represents a ProseMirror schema node definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    /// Content expression defining valid child nodes
    pub content: Option<String>,
    /// Group name this node belongs to
    pub group: Option<String>,
    /// Whether inline content is allowed
    #[serde(default)]
    pub inline: bool,
    /// Whether the node can be selected
    #[serde(default)]
    pub selectable: bool,
    /// Whether the node is draggable
    #[serde(default)]
    pub draggable: bool,
    /// Node attributes
    #[serde(default)]
    pub attrs: HashMap<String, Value>,
}

/// Represents a complete ProseMirror schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProseMirrorSchema {
    /// Map of mark names to their definitions
    #[serde(default)]
    pub marks: HashMap<String, MarkDefinition>,
    /// Map of node names to their definitions
    #[serde(default)]
    pub nodes: HashMap<String, NodeDefinition>,
    /// Top level node type
    #[serde(default = "default_top_node")]
    pub top_node: String,
}

fn default_top_node() -> String {
    "doc".to_string()
}

impl Default for ProseMirrorSchema {
    fn default() -> Self {
        ProseMirrorSchema {
            marks: HashMap::new(),
            nodes: HashMap::new(),
            top_node: default_top_node(),
        }
    }
}

impl TryFrom<&str> for ProseMirrorSchema {
    type Error = String;

    fn try_from(schema_json: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(schema_json)
            .map_err(|e| format!("Failed to parse ProseMirror schema: {}", e))
    }
}

impl TryFrom<String> for ProseMirrorSchema {
    type Error = String;

    fn try_from(schema_json: String) -> Result<Self, Self::Error> {
        Self::try_from(schema_json.as_str())
    }
}

impl TryFrom<Value> for ProseMirrorSchema {
    type Error = String;

    fn try_from(schema_json: Value) -> Result<Self, Self::Error> {
        Self::try_from(schema_json.to_string())
    }
}

impl TryFrom<&ProseMirrorSchema> for Value {
    type Error = String;

    fn try_from(schema: &ProseMirrorSchema) -> Result<Self, Self::Error> {
        Ok(serde_json::to_value(schema).unwrap())
    }
}

impl ToString for ProseMirrorSchema {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

impl PartialEq for ProseMirrorSchema {
    fn eq(&self, other: &Self) -> bool {
        let self_json: Value = match Value::try_from(self) {
            Ok(json) => json,
            Err(_) => return false,
        };
        let other_json: Value = match Value::try_from(other) {
            Ok(json) => json,
            Err(_) => return false,
        };
        self_json == other_json
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_schema_parsing() {
        let schema_json = json!({
            "marks": {
                "bold": { "inclusive": true },
                "italic": { "inclusive": false }
            },
            "nodes": {
                "doc": {
                    "content": "block+"
                },
                "paragraph": {
                    "content": "inline*",
                    "group": "block",
                    "draggable": false
                }
            }
        })
        .to_string();

        let schema = ProseMirrorSchema::try_from(schema_json).expect("Failed to parse schema");

        // Test marks
        assert!(schema.marks.get("bold").unwrap().inclusive);
        assert!(!schema.marks.get("italic").unwrap().inclusive);

        // Test nodes
        let doc = schema.nodes.get("doc").unwrap();
        assert_eq!(doc.content.as_deref(), Some("block+"));

        let para = schema.nodes.get("paragraph").unwrap();
        assert_eq!(para.content.as_deref(), Some("inline*"));
        assert_eq!(para.group.as_deref(), Some("block"));
        assert!(!para.draggable);
    }

    #[wasm_bindgen_test]
    fn test_schema_defaults() {
        let schema = ProseMirrorSchema::default();
        assert_eq!(schema.top_node, "doc");
        assert!(schema.marks.is_empty());
        assert!(schema.nodes.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_invalid_schema() {
        let invalid_json = "{ invalid json }";
        assert!(ProseMirrorSchema::try_from(invalid_json).is_err());
    }
}
