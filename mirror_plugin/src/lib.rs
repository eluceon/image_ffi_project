use std::ffi::{c_char, c_uint, CStr};

use serde::Deserialize;

#[derive(Deserialize)]
struct MirrorParams {
    #[serde(default)]
    horizontal: bool,
    #[serde(default)]
    vertical: bool,
}

/// Process image by mirroring horizontally and/or vertically.
///
/// Modifies `rgba_data` in-place.
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
        MirrorParams {
            horizontal: false,
            vertical: false,
        }
    } else {
        // SAFETY: params is a valid null-terminated C string provided by the caller.
        let c_str = unsafe { CStr::from_ptr(params) };
        let json_str = c_str.to_str().unwrap_or("{}");
        serde_json::from_str(json_str).unwrap_or(MirrorParams {
            horizontal: false,
            vertical: false,
        })
    };

    let Some(len) = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
    else {
        return;
    };

    // SAFETY: The caller guarantees rgba_data points to a valid buffer of size
    // width * height * 4. We stay within bounds.
    let pixels = unsafe { std::slice::from_raw_parts_mut(rgba_data, len) };

    if params.horizontal {
        mirror_horizontal(pixels, width, height);
    }
    if params.vertical {
        mirror_vertical(pixels, width, height);
    }
}

fn mirror_horizontal(pixels: &mut [u8], width: usize, height: usize) {
    let row_bytes = width * 4;
    for y in 0..height {
        let row_start = y * row_bytes;
        for x in 0..width / 2 {
            let left = row_start + x * 4;
            let right = row_start + (width - 1 - x) * 4;
            let (front, back) = pixels.split_at_mut(right);
            front[left..left + 4].swap_with_slice(&mut back[..4]);
        }
    }
}

fn mirror_vertical(pixels: &mut [u8], width: usize, height: usize) {
    let row_bytes = width * 4;
    for y in 0..height / 2 {
        let top = y * row_bytes;
        let bottom = (height - 1 - y) * row_bytes;
        let (front, back) = pixels.split_at_mut(bottom);
        front[top..top + row_bytes].swap_with_slice(&mut back[..row_bytes]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_horizontal() {
        // 2x2 image: pixel(0,0)=R, pixel(1,0)=G, pixel(0,1)=B, pixel(1,1)=W
        // R = [255, 0, 0, 255], G = [0, 255, 0, 255]
        // B = [0, 0, 255, 255], W = [255, 255, 255, 255]
        let mut data = vec![
            255, 0, 0, 255, // (0,0) R
            0, 255, 0, 255, // (1,0) G
            0, 0, 255, 255, // (0,1) B
            255, 255, 255, 255, // (1,1) W
        ];
        mirror_horizontal(&mut data, 2, 2);
        // Row 0: G, R
        assert_eq!(&data[0..4], &[0, 255, 0, 255]); // was G
        assert_eq!(&data[4..8], &[255, 0, 0, 255]); // was R
                                                    // Row 1: W, B
        assert_eq!(&data[8..12], &[255, 255, 255, 255]); // was W
        assert_eq!(&data[12..16], &[0, 0, 255, 255]); // was B
    }

    #[test]
    fn test_mirror_vertical() {
        let mut data = vec![
            255, 0, 0, 255, // (0,0) R
            0, 255, 0, 255, // (1,0) G
            0, 0, 255, 255, // (0,1) B
            255, 255, 255, 255, // (1,1) W
        ];
        mirror_vertical(&mut data, 2, 2);
        // Row 0: B, W
        assert_eq!(&data[0..4], &[0, 0, 255, 255]); // was B
        assert_eq!(&data[4..8], &[255, 255, 255, 255]); // was W
                                                        // Row 1: R, G
        assert_eq!(&data[8..12], &[255, 0, 0, 255]); // was R
        assert_eq!(&data[12..16], &[0, 255, 0, 255]); // was G
    }
}
