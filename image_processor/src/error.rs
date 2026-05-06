use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Input file not found: {0}")]
    InputFileNotFound(PathBuf),

    #[error("Params file not found: {0}")]
    ParamsFileNotFound(PathBuf),

    #[error("Plugin directory not found: {0}")]
    PluginDirNotFound(PathBuf),

    #[error("Plugin library not found, tried: {0}")]
    PluginLibraryNotFound(String),

    #[error("Failed to load image: {0}")]
    ImageLoadError(#[from] image::ImageError),

    #[error("Params file contains a null byte: {0}")]
    ParamsContainsNullByte(PathBuf),

    #[error("Failed to read params file: {0}")]
    ParamsReadError(#[from] std::io::Error),

    #[error("Failed to load plugin library: {0}")]
    PluginLoadError(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Function 'process_image' not found in plugin: {0}")]
    PluginFnNotFound(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_file_not_found_message() {
        let err = AppError::InputFileNotFound(PathBuf::from("missing.png"));
        assert!(err.to_string().contains("missing.png"));
    }

    #[test]
    fn params_file_not_found_message() {
        let err = AppError::ParamsFileNotFound(PathBuf::from("params.json"));
        assert!(err.to_string().contains("params.json"));
    }

    #[test]
    fn plugin_dir_not_found_message() {
        let err = AppError::PluginDirNotFound(PathBuf::from("bad/dir"));
        assert!(err.to_string().contains("bad/dir"));
    }

    #[test]
    fn plugin_library_not_found_message() {
        let err = AppError::PluginLibraryNotFound("libfoo.so".into());
        assert!(err.to_string().contains("libfoo.so"));
    }

    #[test]
    fn image_load_error_from_image_error() {
        let img_err = image::ImageError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file gone",
        ));
        let err: AppError = img_err.into();
        assert!(matches!(err, AppError::ImageLoadError(_)));
    }

    #[test]
    fn params_read_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "nope");
        let err: AppError = io_err.into();
        assert!(matches!(err, AppError::ParamsReadError(_)));
    }

    #[test]
    fn params_contains_null_byte_message() {
        let err = AppError::ParamsContainsNullByte(PathBuf::from("bad.txt"));
        assert!(err.to_string().contains("bad.txt"));
    }
}
