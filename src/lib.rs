mod combinators;
mod common;
pub mod parser;

use wasm_bindgen::prelude::*;

fn vec_to_js_array(v: &[String]) -> js_sys::Array {
    let arr = js_sys::Array::new();
    for s in v {
        arr.push(&JsValue::from_str(s));
    }
    arr
}

fn set_field(obj: &js_sys::Object, key: &str, opt: &Option<Vec<String>>) {
    if let Some(v) = opt {
        js_sys::Reflect::set(obj, &JsValue::from_str(key), &vec_to_js_array(v)).unwrap_or(false);
    }
}

#[wasm_bindgen]
pub fn parse(input: &str) -> JsValue {
    let result = parser::parse(input);
    let obj = js_sys::Object::new();

    set_field(&obj, "titles", &result.titles);
    set_field(&obj, "groups", &result.groups);
    set_field(&obj, "episodeTerms", &result.episode_terms);
    set_field(&obj, "containerTerms", &result.container_terms);
    set_field(&obj, "videoCodecTerms", &result.video_codec_terms);
    set_field(&obj, "audioCodecTerms", &result.audio_codec_terms);
    set_field(&obj, "audioLanguageTerms", &result.audio_language_terms);
    set_field(&obj, "versionTerms", &result.version_terms);
    set_field(&obj, "batchTerms", &result.batch_terms);
    set_field(&obj, "subtitleTerms", &result.subtitle_terms);
    set_field(&obj, "typeTerms", &result.type_terms);
    set_field(&obj, "subtitleLanguageTerms", &result.subtitle_language_terms);
    set_field(&obj, "audioTerms", &result.audio_terms);
    set_field(&obj, "videoTerms", &result.video_terms);
    set_field(&obj, "seasonTerms", &result.season_terms);
    set_field(&obj, "sourceTerms", &result.source_terms);
    set_field(&obj, "dates", &result.dates);
    set_field(&obj, "resolutionTerms", &result.resolution_terms);

    obj.into()
}

// Re-export for native Rust usage
pub use parser::{parse as parse_native, ParseResult};
