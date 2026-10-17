use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
// wasm bindgen: interpreter between rust and javascript
// prelude: convenient toolbox of common bridge components
// clamped <vec<u8>>: container for image byte data

#[derive(Serialize)]
struct AnalysisResult {
    status: String,
    message: String,
}

#[wasm_bindgen]
pub fn analyze_placeholder(width: u32, height: u32, device_pixel_ratio: u32) -> JsValue {
    let result = AnalysisResult {
        status: "placeholder".to_string(),
        message: format!(
            "Rust received image data: {} * {} at {}x.",
            width, height, device_pixel_ratio
        ),
    };

    serde_wasm_bindgen::to_value(&result)
        .expect("failed to serialize analysis result")
}

#[wasm_bindgen]
pub fn inspect_image_bytes(
    bytes: Clamped<Vec<u8>>,
    width: u32,
    height: u32,
) -> JsValue {
    let result = AnalysisResult {
        status: "bytes_received".to_string(),
        message: format!(
            "Rust received {} image bytes for a {} × {} image.",
            bytes.0.len(),
            width,
            height
        ),
    };

    serde_wasm_bindgen::to_value(&result)
        .expect("failed to serialize byte inspection result")
}
