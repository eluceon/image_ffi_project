use std::ffi::{c_char, c_uint, CStr};

use serde::Deserialize;

#[derive(Deserialize)]
struct BlurParams {
    #[serde(default = "default_radius")]
    radius: usize,
    #[serde(default = "default_iterations")]
    iterations: usize,
}

fn default_radius() -> usize {
    3
}
fn default_iterations() -> usize {
    1
}

/// Apply weighted box blur to the image in-place.
///
/// Modifies `rgba_data` in-place. Each pixel's new color is a weighted average
/// of neighboring pixels within `radius`, where weight = 1 / (distance + 1).
///
/// # Safety
///
/// `rgba_data` must point to a valid buffer of `width * height * 4` bytes.
/// `params` must be a valid null-terminated C string, or null.
#[no_mangle]
pub unsafe extern "C" fn process_image(
    width: c_uint,
    height: c_uint,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    let width = width as usize;
    let height = height as usize;

    if rgba_data.is_null() {
        return;
    }

    let params = if params.is_null() {
        BlurParams {
            radius: default_radius(),
            iterations: default_iterations(),
        }
    } else {
        // SAFETY: params is a valid null-terminated C string provided by the caller.
        let c_str = unsafe { CStr::from_ptr(params) };
        let json_str = c_str.to_str().unwrap_or("{}");
        serde_json::from_str(json_str).unwrap_or(BlurParams {
            radius: default_radius(),
            iterations: default_iterations(),
        })
    };

    let len = width * height * 4;

    // SAFETY: The caller guarantees rgba_data points to a valid buffer of size
    // width * height * 4. We stay within bounds.
    let pixels = unsafe { std::slice::from_raw_parts_mut(rgba_data, len) };

    let mut buf = vec![0u8; len];
    for i in 0..params.iterations {
        if i % 2 == 0 {
            blur_pass(&mut buf, pixels, width, height, params.radius);
        } else {
            blur_pass(pixels, &buf, width, height, params.radius);
        }
    }
    if params.iterations % 2 == 1 {
        pixels.copy_from_slice(&buf);
    }
}

fn blur_pass(pixels: &mut [u8], original: &[u8], width: usize, height: usize, radius: usize) {
    let radius = radius.min(width.min(height)); // clamp to image dimensions

    for y in 0..height {
        for x in 0..width {
            let mut sum_r: f64 = 0.0;
            let mut sum_g: f64 = 0.0;
            let mut sum_b: f64 = 0.0;
            let mut total_weight: f64 = 0.0;

            let y_min = y.saturating_sub(radius);
            let y_max = (y + radius).min(height - 1);
            let x_min = x.saturating_sub(radius);
            let x_max = (x + radius).min(width - 1);

            for ny in y_min..=y_max {
                for nx in x_min..=x_max {
                    let dx = nx.abs_diff(x);
                    let dy = ny.abs_diff(y);
                    let distance = dx.max(dy) as f64;
                    let weight = 1.0 / (distance + 1.0);

                    let idx = (ny * width + nx) * 4;
                    sum_r += original[idx] as f64 * weight;
                    sum_g += original[idx + 1] as f64 * weight;
                    sum_b += original[idx + 2] as f64 * weight;
                    total_weight += weight;
                }
            }

            let idx = (y * width + x) * 4;
            pixels[idx] = (sum_r / total_weight).round() as u8;
            pixels[idx + 1] = (sum_g / total_weight).round() as u8;
            pixels[idx + 2] = (sum_b / total_weight).round() as u8;
            // Alpha channel is preserved from original
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blur_identity_small_radius() {
        // 2x2 image, radius 0 should not change anything
        let original = vec![
            255, 0, 0, 255, // R
            0, 255, 0, 255, // G
            0, 0, 255, 255, // B
            255, 255, 255, 255, // W
        ];
        let mut data = original.clone();
        let snapshot = original.clone();
        blur_pass(&mut data, &snapshot, 2, 2, 0);
        assert_eq!(data, original);
    }

    #[test]
    fn test_blur_uniform_image() {
        // All pixels same color — blur should not change them
        let original = vec![100u8; 4 * 4 * 4]; // 4x4 image, all gray
        let mut data = original.clone();
        blur_pass(&mut data, &original, 4, 4, 2);
        assert_eq!(data, original);
    }
}
