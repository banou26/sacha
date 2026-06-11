mod common;
mod parser;

pub use parser::{parse, ParseResult};

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = parse)]
pub fn js_parse(input: &str) -> Result<JsValue, JsValue> {
    let result = parse(input).map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(js_name = format)]
pub fn js_format(input: &str) -> Result<JsValue, JsValue> {
    // format just returns the same parse result - normalization can be done in JS
    let result = parse(input).map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}
