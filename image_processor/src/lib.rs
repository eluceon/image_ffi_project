pub mod error;
pub mod plugin_loader;

use image::RgbaImage;
use std::path::Path;

use error::AppError;

/// Load a PNG image and return its dimensions and RGBA pixel data.
pub fn load_image(path: &Path) -> Result<(u32, u32, Vec<u8>), AppError> {
    if !path.exists() {
        return Err(AppError::InputFileNotFound(path.to_path_buf()));
    }

    let img = image::open(path)?;
    let rgba: RgbaImage = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    let data = rgba.into_raw();
    Ok((width, height, data))
}

/// Save RGBA pixel data as a PNG image.
pub fn save_image(path: &Path, width: u32, height: u32, data: Vec<u8>) -> Result<(), AppError> {
    let img = RgbaImage::from_raw(width, height, data).ok_or_else(|| {
        image::ImageError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "buffer size does not match width * height * 4",
        ))
    })?;
    img.save(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let salt: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        if let Some((stem, ext)) = name.rsplit_once('.') {
            std::env::temp_dir().join(format!("image_processor_test_{}_{}.{}", stem, salt, ext))
        } else {
            std::env::temp_dir().join(format!("image_processor_test_{}_{}", name, salt))
        }
    }

    #[test]
    fn load_valid_png() {
        let img = RgbaImage::from_pixel(3, 2, image::Rgba([10, 20, 30, 255]));
        let path = temp_path("valid.png");
        img.save(&path).unwrap();

        let (w, h, data) = load_image(&path).unwrap();
        assert_eq!(w, 3);
        assert_eq!(h, 2);
        assert_eq!(data.len(), 3 * 2 * 4);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_nonexistent_file() {
        let err = load_image(Path::new("/nonexistent/foo.png")).unwrap_err();
        assert!(matches!(err, AppError::InputFileNotFound(_)));
    }

    #[test]
    fn load_not_a_png() {
        let path = temp_path("not_a_png.png");
        std::fs::write(&path, b"this is not a png file").unwrap();

        let err = load_image(&path).unwrap_err();
        assert!(matches!(err, AppError::ImageLoadError(_)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn save_and_reload_roundtrip() {
        let data = vec![
            255, 0, 0, 255, // R
            0, 255, 0, 255, // G
            0, 0, 255, 255, // B
            255, 255, 255, 255, // W
        ];
        let path = temp_path("roundtrip.png");
        save_image(&path, 2, 2, data.clone()).unwrap();

        let (w, h, reloaded) = load_image(&path).unwrap();
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        assert_eq!(reloaded, data);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn save_with_wrong_buffer_size() {
        let data = vec![0u8; 10]; // not enough for 2x2x4 = 16
        let path = temp_path("wrong_size.png");
        let err = save_image(&path, 2, 2, data).unwrap_err();
        assert!(matches!(err, AppError::ImageLoadError(_)));
    }
}
