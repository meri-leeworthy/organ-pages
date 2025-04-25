use loro::LoroMap;
use serde_json::Value;
#[allow(unused)]
// use handlebars::Handlebars;
// use pulldown_cmark::{html, Options, Parser};
// use serde_json::Value;
use wasm_bindgen::prelude::*;
// use wasm_bindgen::JsValue;

mod events;
mod js_conversions;
mod messages;
mod model;
mod store;
mod types;

// Re-export our public components
pub use events::EventEmitter;

// Re-export LoroDoc type
pub use loro::LoroDoc;
pub use model::*;
pub use store::*;
pub use types::*;

use js_sys::{JsString, Promise};
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // log to JS console
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format!("[Lib (WASM)] {}", format!($($t)*))))
}

// Import the JavaScript functions
#[wasm_bindgen(module = "/indexeddb.js")]
extern "C" {
    fn saveToIndexedDB(db_name: &str, store_name: &str, key: &str, value: &JsValue) -> Promise;
    fn loadFromIndexedDB(db_name: &str, store_name: &str, key: &str) -> Promise;
}

// // Asynchronous function to save data to IndexedDB
#[wasm_bindgen]
pub async fn save_data(
    store_name: &str,
    key: &str,
    value: js_sys::Uint8Array,
) -> Result<(), JsValue> {
    let promise = saveToIndexedDB(IDB_DB_NAME, store_name, key, &value);
    JsFuture::from(promise).await?;
    console_log!("Saved data to IndexedDB: {:?}", value);
    Ok(())
}

// // Asynchronous function to load data from IndexedDB
#[wasm_bindgen]
pub async fn load_data(store_name: &str, key: &str) -> Result<JsString, JsValue> {
    let promise = loadFromIndexedDB(IDB_DB_NAME, store_name, key);
    let result = JsFuture::from(promise).await?;
    console_log!("Loaded data from IndexedDB: {:?}", result);
    Ok(result.into())
}

#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

pub trait ApplyMap {
    fn apply_map(&self, map: &serde_json::Map<String, Value>) -> Result<(), String>;
}

impl ApplyMap for LoroMap {
    fn apply_map(&self, map: &serde_json::Map<String, Value>) -> Result<(), String> {
        for (k, v) in map {
            match v {
                serde_json::Value::String(s) => self.insert(k, s.clone()),
                serde_json::Value::Bool(b) => self.insert(k, *b),
                _ => return Err("Unsupported field value type".to_string()),
            }
            .map_err(|e| format!("Failed to insert field: {}", e))?;
        }
        Ok(())
    }
}

// fn get_content_and_type(data: &serde_json::Map<String, Value>) -> Result<(String, String), String> {
//     let body = data
//         .get("body")
//         .and_then(|body| body.as_object())
//         .ok_or_else(|| "No body object found".to_string())?;

//     let content = body
//         .get("content")
//         .and_then(|content| content.as_str())
//         .ok_or_else(|| "No content found in body".to_string())?
//         .to_string();

//     let content_type = body
//         .get("type")
//         .and_then(|t| t.as_str())
//         .ok_or_else(|| "No type found in body".to_string())?
//         .to_string();

//     Ok((content, content_type))
// }

// Should return a string of rendered HTML for the current filename
// #[wasm_bindgen]
// pub fn render(current_file_id: i32, context: &JsValue) -> Result<String, JsValue> {
//     let context = validate_context(context).map_err(|e| e.to_string())?;
//     let context = parse_context(context);

//     log(&format!("context: {:?}", context));

//     let current_file = context
//         .get(&current_file_id.to_string())
//         .ok_or_else(|| JsValue::from_str("Current file not found in context"))?;

//     // Get the template filename from the current file data
//     let template_id = match current_file
//         .data
//         .get("template")
//         .expect("'Template' not found in associated data for current file")
//         .clone()
//     {
//         Value::Number(n) => n.to_string(),
//         Value::String(s) => s,
//         _ => return Err(JsValue::from_str("Template property not found")),
//     };

//     // Get the template content from the context
//     let template = context
//         .get(&template_id)
//         .ok_or_else(|| JsValue::from_str("Template not found in context"))?;

//     let (template_content, _) = get_content_and_type(&template.data)
//         .map_err(|e| JsValue::from_str(&format!("Failed to get template content: {}", e)))?;

//     // get images from context
//     let images: ContentRecord = ContentRecord::new_with_content(
//         context
//             .iter()
//             .filter(|(_, v)| matches!(v.file_type, FileType::Asset))
//             .map(|(k, v)| (k.clone(), v.clone()))
//             .collect(),
//     );

//     // get template assets from context
//     let template_assets: ContentRecord = ContentRecord::new_with_content(
//         context
//             .iter()
//             .filter(|(_, v)| matches!(v.file_type, FileType::TemplateAsset))
//             .map(|(k, v)| (k.clone(), v.clone()))
//             .collect(),
//     );

//     // get partials from context
//     let partials: ContentRecord = ContentRecord::new_with_content(
//         context
//             .iter()
//             .filter(|(_, v)| matches!(v.file_type, FileType::Partial))
//             .map(|(k, v)| (k.clone(), v.clone()))
//             .collect(),
//     );

//     // Get content and content type from current file
//     let (content, content_type) = get_content_and_type(&current_file.data)
//         .map_err(|e| JsValue::from_str(&format!("Failed to get file content: {}", e)))?;

//     // Convert content based on type
//     let html_output = match content_type.as_str() {
//         "html" => content,
//         "plaintext" => content,
//         "markdown" | _ => match markdown_to_html(&content, &images, &template_assets) {
//             Ok(output) => output,
//             Err(e) => return Err(e.into()),
//         },
//         _ => return Err(JsValue::from_str("Invalid content type")),
//     };

//     // convert ContentRecord to JSON and add "content" key with "html_output" as value
//     let mut render_context: Value = json!({});
//     render_context["content"] = json!(html_output);

//     for (key, value) in &current_file.data {
//         render_context[key] = value.clone();
//     }

//     if current_file.file_type == FileType::Page {
//         // add posts to render context
//         let posts: Vec<ContentData> = context
//             .iter()
//             .filter(|(_, v)| matches!(v.file_type, FileType::Post))
//             .map(|(_, v)| v.clone())
//             .collect();
//         render_context["posts"] = json!(posts);
//     }

//     // Render the template with the context
//     let mut rendered_template = match render_template(&template_content, &partials, &render_context)
//     {
//         Ok(output) => output,
//         Err(e) => return Err(JsValue::from_str(&e)),
//     };

//     // Replace template asset URLs
//     for (_id, file) in template_assets.iter() {
//         rendered_template = rendered_template.replace(
//             &format!("href=\"{}\"", file.name),
//             &format!("href=\"{}\"", file.url),
//         );
//     }

//     Ok(rendered_template)
// }

// fn render_template(
//     template_content: &str,
//     partials: &ContentRecord,
//     render_context: &Value,
// ) -> Result<String, String> {
//     let mut handlebars = Handlebars::new();

//     // Register the partials
//     for (id, file) in partials.iter() {
//         let (content, _) = get_content_and_type(&file.data)
//             .map_err(|e| format!("Failed to get partial content: {}", e))?;
//         handlebars
//             .register_partial(id, &content)
//             .map_err(|e| format!("Failed to register partial {}: {}", id, e))?;
//     }

//     // Register the template from the string
//     if let Err(e) = handlebars.register_template_string("template", template_content) {
//         return Err(format!("Template error: {}", e));
//     }

//     log(&format!("render_context: {:?}", render_context));

//     // Render the template
//     match handlebars.render("template", render_context) {
//         Ok(output) => Ok(output),
//         Err(e) => Err(format!("Rendering error: {}", e)),
//     }
// }

// fn markdown_to_html(
//     markdown_input: &str,
//     images: &ContentRecord,
//     template_assets: &ContentRecord,
// ) -> Result<String, String> {
//     let options = Options::empty();

//     // Render the Markdown to HTML
//     let parser = Parser::new_ext(markdown_input, options);
//     let mut html_output = String::new();
//     html::push_html(&mut html_output, parser);

//     // Replace image `src` attributes with the provided URLs
//     for (_id, file) in images.iter() {
//         html_output = html_output.replace(
//             &format!("src=\"{}\"", file.name),
//             &format!("src=\"{}\"", file.url),
//         );
//     }

//     Ok(html_output)
// }

// fn validate_context(value: &JsValue) -> Result<UnparsedContentRecord, serde_wasm_bindgen::Error> {
//     serde_wasm_bindgen::from_value(value.clone())
// }

// fn parse_file(file: &UnparsedContentData) -> Result<ContentData, serde_json::Error> {
//     let data = serde_json::from_str(&file.data)?;

//     let file_type = match file.file_type.as_str() {
//         "asset" => FileType::Asset,
//         "template" => FileType::Template,
//         "page" => FileType::Page,
//         "templateAsset" => FileType::TemplateAsset,
//         "partial" => FileType::Partial,
//         "post" => FileType::Post,
//         _ => {
//             return Err(serde_json::Error::custom(format!(
//                 "Invalid file type for file: {:?}",
//                 file,
//             )))
//         }
//     };
//     Ok(ContentData {
//         name: file.name.clone(),
//         file_type,
//         data,
//         url: file.url.clone(),
//     })
// }

// fn parse_context(context: UnparsedContentRecord) -> ContentRecord {
//     let parsed_content: HashMap<String, ContentData> = context
//         .into_iter()
//         .map(|(filename, file)| {
//             parse_file(&file).map(|parsed_file| (filename.clone(), parsed_file))
//         })
//         .collect::<Result<_, _>>()
//         .map_err(|e| serde_wasm_bindgen::Error::new(&format!("Failed to parse context: {}", e)))
//         .expect("Failed to parse context");

//     ContentRecord::new_with_content(parsed_content)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use wasm_bindgen_test::*;
//     use serde_json::json;
//     use serde_wasm_bindgen::{from_value, to_value};
//     use std::collections::HashMap;

//     pub use model::*;
//     pub use store::*;
//     pub use types::*;

//     pub use model::file::*;

//     #[wasm_bindgen_test]
//     fn test_markdown_to_html() {
//         let markdown = "# Hello World\nThis is a test.";
//         let images: ContentRecord = ContentRecord::new();
//         let template_assets: ContentRecord = ContentRecord::new();
//         let expected = "<h1>Hello World</h1>\n<p>This is a test.</p>\n";
//         let result =
//             markdown_to_html(markdown, &images, &template_assets).expect("Markdown to html failed");
//         print!("{}", result);
//         assert_eq!(result, expected);
//     }

//     #[wasm_bindgen_test]
//     fn test_validate_content_record() {
//         // ... rest of the test code ...
//     }
// }
