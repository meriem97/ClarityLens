use serde::Serialize;
use wasm_bindgen::prelude::*;

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
            "Rust received image data: {} × {} at {}x.",
            width, height, device_pixel_ratio
        ),
    };

    serde_wasm_bindgen::to_value(&result)
        .expect("failed to serialize analysis result")
}
