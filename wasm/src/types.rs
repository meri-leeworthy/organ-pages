use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

// Type definitions to match the TypeScript interfaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Site,
    Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectionType {
    Page,
    Post,
    Template,
    Partial,
    Asset,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum FieldType {
    RichText,
    Text,
    List,
    Map,
    DateTime,
    String,
    Number,
    Object,
    Array,
    Blob,
}

impl FieldType {
    pub fn to_string(&self) -> String {
        match self {
            FieldType::RichText => "richtext".to_string(),
            FieldType::Text => "text".to_string(),
            FieldType::List => "list".to_string(),
            FieldType::Map => "map".to_string(),
            FieldType::DateTime => "datetime".to_string(),
            FieldType::String => "string".to_string(),
            FieldType::Number => "number".to_string(),
            FieldType::Object => "object".to_string(),
            FieldType::Array => "array".to_string(),
            FieldType::Blob => "blob".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Collection {
    Asset,
    Template,
    Page,
    TemplateAsset,
    Partial,
    Post,
}

impl Display for Collection {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Collection::Asset => write!(f, "asset"),
            Collection::Template => write!(f, "template"),
            Collection::Page => write!(f, "page"),
            Collection::TemplateAsset => write!(f, "templateAsset"),
            Collection::Partial => write!(f, "partial"),
            Collection::Post => write!(f, "post"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnparsedContentData {
    pub name: String,
    pub file_type: String,
    pub data: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentData {
    pub name: String,
    pub file_type: Collection,
    pub data: Map<String, Value>,
    pub url: String,
}

// Alias for the expected record type
// note that the key is serialised as a string, but it functions as primary key id
// and is interpreted as an integer in the JS context
pub type UnparsedContentRecord = HashMap<String, UnparsedContentData>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContentRecord {
    pub content: HashMap<String, ContentData>,
}

impl ContentRecord {
    pub fn new() -> Self {
        ContentRecord {
            content: HashMap::new(),
        }
    }

    pub fn new_with_content(content: HashMap<String, ContentData>) -> Self {
        ContentRecord { content }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ContentData)> {
        self.content.iter()
    }

    pub fn get(&self, key: &String) -> Option<&ContentData> {
        self.content.get(key)
    }
}
