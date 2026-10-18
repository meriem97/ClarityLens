use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

#[derive(Serialize)]
struct AnalysisResult {
    status: String,
    message: String,
}

#[derive(Serialize)]
struct LuminanceResult {
    status: String,
    width: u32,
    height: u32,
    pixel_count: u32,
    average_luminance: f64,
}

#[wasm_bindgen]
pub fn analyze_placeholder(
    width: u32,
    height: u32,
    device_pixel_ratio: u32,
) -> JsValue {
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

fn srgb_to_linear(channel: u8) -> f64 {
    let value = channel as f64 / 255.0;

    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn pixel_luminance(red: u8, green: u8, blue: u8) -> f64 {
    let red_linear = srgb_to_linear(red);
    let green_linear = srgb_to_linear(green);
    let blue_linear = srgb_to_linear(blue);

    0.2126 * red_linear
        + 0.7152 * green_linear
        + 0.0722 * blue_linear
}

#[wasm_bindgen]
pub fn inspect_image_bytes(
    bytes: Clamped<Vec<u8>>,
    width: u32,
    height: u32,
) -> Result<JsValue, JsValue> {
    let expected_length = width
        .checked_mul(height)
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| JsValue::from_str("Image dimensions are too large."))?
        as usize;

    if width == 0 || height == 0 {
        return Err(JsValue::from_str(
            "Image width and height must be greater than zero.",
        ));
    }

    if bytes.0.len() != expected_length {
        return Err(JsValue::from_str(
            "Pixel data length does not match image dimensions.",
        ));
    }

    let pixel_count = width
        .checked_mul(height)
        .ok_or_else(|| JsValue::from_str("Pixel count is too large."))?;

    let mut luminance_sum = 0.0;

    for pixel in bytes.0.chunks_exact(4) {
        let red = pixel[0];
        let green = pixel[1];
        let blue = pixel[2];

        luminance_sum += pixel_luminance(red, green, blue);
    }

    let average_luminance = luminance_sum / pixel_count as f64;

    let result = LuminanceResult {
        status: "luminance_ready".to_string(),
        width,
        height,
        pixel_count,
        average_luminance: (average_luminance * 10_000.0).round() / 10_000.0,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}
#[cfg(test)]
mod tests {
    use super::*;

    fn approximately_equal(left: f64, right: f64) -> bool {
        (left - right).abs() < 0.0001
    }

    #[test]
    fn black_is_zero() {
        assert!(approximately_equal(
            pixel_luminance(0, 0, 0),
            0.0
        ));
    }

    #[test]
    fn white_is_one() {
        assert!(approximately_equal(
            pixel_luminance(255, 255, 255),
            1.0
        ));
    }

    #[test]
    fn red_matches_expected_luminance() {
        assert!(approximately_equal(
            pixel_luminance(255, 0, 0),
            0.2126
        ));
    }

    #[test]
    fn green_matches_expected_luminance() {
        assert!(approximately_equal(
            pixel_luminance(0, 255, 0),
            0.7152
        ));
    }

    #[test]
    fn blue_matches_expected_luminance() {
        assert!(approximately_equal(
            pixel_luminance(0, 0, 255),
            0.0722
        ));
    }
}