use crate::model::file::{Asset, Page, Partial, Post, Template, Text};
use enum_dispatch::enum_dispatch;
use loro::{LoroMap, LoroValue, ValueOrContainer};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

// Type definitions to match the TypeScript interfaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Site,
    Theme,
}

impl Display for ProjectType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ProjectType::Site => write!(f, "site"),
            ProjectType::Theme => write!(f, "theme"),
        }
    }
}

impl PartialEq for ProjectType {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

#[enum_dispatch(File)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FileType {
    Asset(Asset),
    Template(Template),
    Page(Page),
    Text(Text),
    Partial(Partial),
    Post(Post),
    // UserModel,
}

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            FileType::Asset(_asset) => write!(f, "asset"),
            FileType::Template(_template) => write!(f, "template"),
            FileType::Page(_page) => write!(f, "page"),
            FileType::Text(_text) => write!(f, "text"),
            FileType::Partial(_partial) => write!(f, "partial"),
            FileType::Post(_post) => write!(f, "post"),
            // FileType::UserModel => write!(f, "userModel"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
}

impl TryFrom<LoroMap> for FieldDefinition {
    type Error = String;

    fn try_from(map: LoroMap) -> Result<Self, String> {
        let name = match map.get("name") {
            Some(ValueOrContainer::Value(LoroValue::String(name))) => name.to_string(),
            _ => return Err("Field name not found".to_string()),
        };

        let field_type = match map.get("field_type") {
            Some(ValueOrContainer::Value(LoroValue::String(type_str))) => type_str.to_string(),
            _ => return Err("Field type not found".to_string()),
        };

        // Extract required flag
        let required = match map.get("required") {
            Some(ValueOrContainer::Value(LoroValue::Bool(required))) => required,
            _ => false,
        };

        Ok(FieldDefinition {
            name,
            field_type: FieldType::try_from(field_type)?,
            required,
        })
    }
}

impl Into<Value> for FieldDefinition {
    fn into(self) -> Value {
        let mut map = Map::new();
        map.insert("name".to_string(), self.name.into());
        map.insert("field_type".to_string(), self.field_type.to_string().into());
        map.insert("required".to_string(), self.required.into());
        map.into()
    }
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

impl TryFrom<String> for FieldType {
    type Error = String;

    fn try_from(s: String) -> Result<Self, String> {
        match s.as_str() {
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
            _ => Err("Invalid field type".to_string()),
        }
    }
}

impl PartialEq for FieldType {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
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
    pub file_type: FileType,
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
