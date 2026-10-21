use serde::Serialize;
use wasm_bindgen::Clamped;
use wasm_bindgen::prelude::*;

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
    brightness_level: String,
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

    serde_wasm_bindgen::to_value(&result).expect("failed to serialize analysis result")
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

    0.2126 * red_linear + 0.7152 * green_linear + 0.0722 * blue_linear
}

fn contrast_ratio(first_luminance: f64, second_luminance: f64) -> f64 {
    let lighter = first_luminance.max(second_luminance);
    let darker = first_luminance.min(second_luminance);

    (lighter + 0.05) / (darker + 0.05)
}

#[wasm_bindgen]
pub fn calculate_contrast(first_luminance: f64, second_luminance: f64) -> f64 {
    contrast_ratio(first_luminance, second_luminance)
}

fn average_luminance_from_bytes(bytes: &[u8]) -> Result<f64, &'static str> {
    if bytes.len() % 4 != 0 {
        return Err("Pixel data length must be divisible by four.");
    }

    if bytes.is_empty() {
        return Err("Pixel data cannot be empty.");
    }

    let mut luminance_sum = 0.0;

    for pixel in bytes.chunks_exact(4) {
        let red = pixel[0];
        let green = pixel[1];
        let blue = pixel[2];

        luminance_sum += pixel_luminance(red, green, blue);
    }

    let pixel_count = bytes.len() / 4;

    Ok(luminance_sum / pixel_count as f64)
}

fn brightness_level(average_luminance: f64) -> &'static str {
    if average_luminance < 0.25 {
        "low"
    } else if average_luminance < 0.75 {
        "medium"
    } else {
        "high"
    }
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

    let average_luminance =
        average_luminance_from_bytes(&bytes.0).map_err(|error| JsValue::from_str(error))?;

    let rounded_luminance = (average_luminance * 10_000.0).round() / 10_000.0;

    let result = LuminanceResult {
        status: "luminance_ready".to_string(),
        width,
        height,
        pixel_count,
        average_luminance: rounded_luminance,
        brightness_level: brightness_level(rounded_luminance).to_string(),
    };

    serde_wasm_bindgen::to_value(&result).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approximately_equal(left: f64, right: f64) -> bool {
        (left - right).abs() < 0.0001
    }

    #[test]
    fn black_is_zero() {
        assert!(approximately_equal(pixel_luminance(0, 0, 0), 0.0));
    }

    #[test]
    fn white_is_one() {
        assert!(approximately_equal(pixel_luminance(255, 255, 255), 1.0));
    }

    #[test]
    fn red_matches_expected_luminance() {
        assert!(approximately_equal(pixel_luminance(255, 0, 0), 0.2126));
    }

    #[test]
    fn green_matches_expected_luminance() {
        assert!(approximately_equal(pixel_luminance(0, 255, 0), 0.7152));
    }

    #[test]
    fn blue_matches_expected_luminance() {
        assert!(approximately_equal(pixel_luminance(0, 0, 255), 0.0722));
    }

    #[test]
    fn alpha_channel_is_ignored() {
        let opaque_red = [255, 0, 0, 255];
        let transparent_red = [255, 0, 0, 0];

        let opaque_result =
            average_luminance_from_bytes(&opaque_red).expect("opaque pixel should be valid");

        let transparent_result = average_luminance_from_bytes(&transparent_red)
            .expect("transparent pixel should be valid");

        assert!(approximately_equal(opaque_result, transparent_result));
    }

    #[test]
    fn alpha_is_ignored_for_multiple_pixels() {
        let pixels = [255, 0, 0, 255, 255, 0, 0, 0];

        let result = average_luminance_from_bytes(&pixels).expect("pixel data should be valid");

        assert!(approximately_equal(result, pixel_luminance(255, 0, 0)));
    }

    #[test]
    fn rejects_incomplete_rgba_pixel() {
        let incomplete_pixel = [255, 0, 0];

        let result = average_luminance_from_bytes(&incomplete_pixel);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_pixel_data() {
        let result = average_luminance_from_bytes(&[]);

        assert!(result.is_err());
    }

    #[test]
    fn brightness_level_classifies_luminance() {
        assert_eq!(brightness_level(0.0), "low");
        assert_eq!(brightness_level(0.2499), "low");
        assert_eq!(brightness_level(0.25), "medium");
        assert_eq!(brightness_level(0.7499), "medium");
        assert_eq!(brightness_level(0.75), "high");
        assert_eq!(brightness_level(1.0), "high");
    }
    #[test]
    fn black_and_white_have_maximum_contrast() {
        assert!(approximately_equal(contrast_ratio(0.0, 1.0), 21.0));
    }

    #[test]
    fn contrast_ratio_is_order_independent() {
        assert!(approximately_equal(
            contrast_ratio(0.2, 0.8),
            contrast_ratio(0.8, 0.2)
        ));
    }

    #[test]
    fn identical_luminance_has_one_to_one_contrast() {
        assert!(approximately_equal(contrast_ratio(0.5, 0.5), 1.0));
    }
}
