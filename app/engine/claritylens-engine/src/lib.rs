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
    pixel_count: u64,

    average_luminance: f64,
    brightness_level: String,

    sampled_color_pixel_count: u64,
    average_red: f64,
    average_green: f64,
    average_blue: f64,

    average_red_8bit: u8,
    average_green_8bit: u8,
    average_blue_8bit: u8,
}

#[derive(Serialize)]
struct ContrastResult {
    status: String,
    foreground_luminance: f64,
    background_luminance: f64,
    contrast_ratio: f64,
    normal_text_passes: bool,
    large_text_passes: bool,
}

#[derive(Serialize)]
struct AverageColorResult {
    status: String,
    sampled_pixel_count: u64,
    average_red: f64,
    average_green: f64,
    average_blue: f64,
    average_red_8bit: u8,
    average_green_8bit: u8,
    average_blue_8bit: u8,
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

fn normalized_to_8bit(value: f64) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
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

fn brightness_level(average_luminance: f64) -> &'static str {
    if average_luminance < 0.25 {
        "low"
    } else if average_luminance < 0.75 {
        "medium"
    } else {
        "high"
    }
}

fn average_luminance_from_bytes(bytes: &[u8]) -> Result<f64, &'static str> {
    if bytes.len() % 4 != 0 {
        return Err("Pixel data length must be divisible by four.");
    }

    if bytes.is_empty() {
        return Err("Pixel data cannot be empty.");
    }

    let mut luminance_sum = 0.0;
    let mut visible_pixel_count = 0u64;

    for pixel in bytes.chunks_exact(4) {
        let red = pixel[0];
        let green = pixel[1];
        let blue = pixel[2];
        let alpha = pixel[3];

        if alpha == 0 {
            continue;
        }

        luminance_sum += pixel_luminance(red, green, blue);

        visible_pixel_count += 1;
    }

    if visible_pixel_count == 0 {
        return Err("No visible pixels were found.");
    }

    Ok(luminance_sum / visible_pixel_count as f64)
}

fn average_color(pixels: &[u8]) -> Result<(u64, f64, f64, f64), String> {
    if pixels.len() % 4 != 0 {
        return Err("Pixel buffer length must be divisible by four.".to_string());
    }

    if pixels.is_empty() {
        return Err("Pixel buffer cannot be empty.".to_string());
    }

    let mut red_total: u64 = 0;
    let mut green_total: u64 = 0;
    let mut blue_total: u64 = 0;
    let mut sampled_pixel_count: u64 = 0;

    for pixel in pixels.chunks_exact(4) {
        let red = pixel[0];
        let green = pixel[1];
        let blue = pixel[2];
        let alpha = pixel[3];

        if alpha == 0 {
            continue;
        }

        red_total += u64::from(red);
        green_total += u64::from(green);
        blue_total += u64::from(blue);
        sampled_pixel_count += 1;
    }

    if sampled_pixel_count == 0 {
        return Err("No visible pixels were found.".to_string());
    }

    let count = sampled_pixel_count as f64;

    Ok((
        sampled_pixel_count,
        red_total as f64 / count / 255.0,
        green_total as f64 / count / 255.0,
        blue_total as f64 / count / 255.0,
    ))
}

#[wasm_bindgen]
pub fn calculate_contrast(first_luminance: f64, second_luminance: f64) -> f64 {
    contrast_ratio(first_luminance, second_luminance)
}

#[wasm_bindgen]
pub fn inspect_contrast(
    foreground_red: u8,
    foreground_green: u8,
    foreground_blue: u8,
    background_red: u8,
    background_green: u8,
    background_blue: u8,
) -> Result<JsValue, JsValue> {
    let foreground_luminance = pixel_luminance(foreground_red, foreground_green, foreground_blue);

    let background_luminance = pixel_luminance(background_red, background_green, background_blue);

    let ratio = contrast_ratio(foreground_luminance, background_luminance);

    let result = ContrastResult {
        status: "contrast_ready".to_string(),
        foreground_luminance: (foreground_luminance * 10_000.0).round() / 10_000.0,
        background_luminance: (background_luminance * 10_000.0).round() / 10_000.0,
        contrast_ratio: (ratio * 100.0).round() / 100.0,
        normal_text_passes: ratio >= 4.5,
        large_text_passes: ratio >= 3.0,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn inspect_average_color(pixels: &[u8]) -> Result<JsValue, JsValue> {
    let (sampled_pixel_count, average_red, average_green, average_blue) =
        average_color(pixels).map_err(|error| JsValue::from_str(&error))?;

    let average_red_8bit = normalized_to_8bit(average_red);

    let average_green_8bit = normalized_to_8bit(average_green);

    let average_blue_8bit = normalized_to_8bit(average_blue);

    let result = AverageColorResult {
        status: "average_color_ready".to_string(),
        sampled_pixel_count,
        average_red,
        average_green,
        average_blue,
        average_red_8bit,
        average_green_8bit,
        average_blue_8bit,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn inspect_image_bytes(
    bytes: Clamped<Vec<u8>>,
    width: u32,
    height: u32,
) -> Result<JsValue, JsValue> {
    if width == 0 || height == 0 {
        return Err(JsValue::from_str(
            "Image width and height must be greater than zero.",
        ));
    }

    let expected_length = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| JsValue::from_str("Image dimensions are too large."))?
        as usize;

    if bytes.0.len() != expected_length {
        return Err(JsValue::from_str(
            "Pixel data length does not match image dimensions.",
        ));
    }

    let pixel_count = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or_else(|| JsValue::from_str("Pixel count is too large."))?;

    let average_luminance = average_luminance_from_bytes(&bytes.0).map_err(JsValue::from_str)?;

    let (sampled_color_pixel_count, average_red, average_green, average_blue) =
        average_color(&bytes.0).map_err(|error| JsValue::from_str(&error))?;

    let rounded_luminance = (average_luminance * 10_000.0).round() / 10_000.0;

    let average_red_8bit = normalized_to_8bit(average_red);

    let average_green_8bit = normalized_to_8bit(average_green);

    let average_blue_8bit = normalized_to_8bit(average_blue);

    let result = LuminanceResult {
        status: "luminance_ready".to_string(),
        width,
        height,
        pixel_count,
        average_luminance: rounded_luminance,
        brightness_level: brightness_level(rounded_luminance).to_string(),

        sampled_color_pixel_count,
        average_red,
        average_green,
        average_blue,
        average_red_8bit,
        average_green_8bit,
        average_blue_8bit,
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
    fn rejects_all_transparent_pixels() {
        let transparent_red = [255, 0, 0, 0];

        assert!(average_luminance_from_bytes(&transparent_red).is_err());
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

        assert!(average_luminance_from_bytes(&incomplete_pixel).is_err());
    }

    #[test]
    fn rejects_empty_pixel_data() {
        assert!(average_luminance_from_bytes(&[]).is_err());
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

    #[test]
    fn black_and_white_contrast_passes_both_text_sizes() {
        let ratio = contrast_ratio(0.0, 1.0);

        assert!(ratio >= 4.5);
        assert!(ratio >= 3.0);
    }

    #[test]
    fn low_contrast_pair_fails_normal_text() {
        let ratio = contrast_ratio(0.4, 0.6);

        assert!(ratio < 4.5);
    }

    #[test]
    fn large_text_threshold_is_lower() {
        let ratio = contrast_ratio(0.0, 0.14);

        assert!(approximately_equal(ratio, 3.8));
        assert!(ratio >= 3.0);
        assert!(ratio < 4.5);
    }

    #[test]
    fn average_color_of_black_pixels_is_black() {
        let pixels = [0, 0, 0, 255, 0, 0, 0, 255];

        let result = average_color(&pixels).unwrap();

        assert_eq!(result.0, 2);
        assert!(approximately_equal(result.1, 0.0));
        assert!(approximately_equal(result.2, 0.0));
        assert!(approximately_equal(result.3, 0.0));
    }

    #[test]
    fn average_color_of_white_pixels_is_white() {
        let pixels = [255, 255, 255, 255, 255, 255, 255, 255];

        let result = average_color(&pixels).unwrap();

        assert_eq!(result.0, 2);
        assert!(approximately_equal(result.1, 1.0));
        assert!(approximately_equal(result.2, 1.0));
        assert!(approximately_equal(result.3, 1.0));
    }

    #[test]
    fn average_color_ignores_fully_transparent_pixels() {
        let pixels = [255, 0, 0, 0, 0, 0, 255, 255];

        let result = average_color(&pixels).unwrap();

        assert_eq!(result.0, 1);
        assert!(approximately_equal(result.1, 0.0));
        assert!(approximately_equal(result.2, 0.0));
        assert!(approximately_equal(result.3, 1.0));
    }

    #[test]
    fn average_color_mixes_visible_pixels() {
        let pixels = [255, 0, 0, 255, 0, 0, 255, 255];

        let result = average_color(&pixels).unwrap();

        assert_eq!(result.0, 2);
        assert!(approximately_equal(result.1, 0.5));
        assert!(approximately_equal(result.2, 0.0));
        assert!(approximately_equal(result.3, 0.5));
    }

    #[test]
    fn average_color_of_red_and_blue_is_purple() {
        let pixels = [255, 0, 0, 255, 0, 0, 255, 255];

        let result = average_color(&pixels).unwrap();

        assert_eq!(result.0, 2);
        assert!(approximately_equal(result.1, 0.5));
        assert!(approximately_equal(result.2, 0.0));
        assert!(approximately_equal(result.3, 0.5));
    }
}
