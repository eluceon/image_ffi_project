use std::ffi::{c_char, c_uint, CString};
use std::path::{Path, PathBuf};

use libloading::{Library, Symbol};

use crate::error::AppError;

type ProcessImageFn =
    unsafe extern "C" fn(width: c_uint, height: c_uint, rgba_data: *mut u8, params: *const c_char);

pub struct Plugin {
    _lib: Library,
    process_image: ProcessImageFn,
}

impl Plugin {
    pub fn load(plugin_dir: &Path, plugin_name: &str) -> Result<Self, AppError> {
        if !plugin_dir.exists() {
            return Err(AppError::PluginDirNotFound(plugin_dir.to_path_buf()));
        }

        let lib_path = find_library(plugin_dir, plugin_name)?;

        // SAFETY: Loading a dynamic library. We trust the plugin to be a valid cdylib
        // with the correct process_image symbol. The library stays loaded for the lifetime
        // of Plugin (held in _lib).
        let lib =
            unsafe { Library::new(&lib_path).map_err(|e| AppError::PluginLoadError(Box::new(e)))? };

        // SAFETY: We load the symbol "process_image" from the library. The symbol must exist
        // with the C ABI signature: fn(u32, u32, *mut u8, *const c_char).
        // We copy the function pointer out of the Symbol before the borrow on `lib` ends,
        // allowing `lib` to be moved into the Plugin struct.
        let process_image: ProcessImageFn = unsafe {
            let symbol: Symbol<ProcessImageFn> = lib
                .get(b"process_image")
                .map_err(|e| AppError::PluginFnNotFound(Box::new(e)))?;
            *symbol
        };

        Ok(Plugin {
            _lib: lib,
            process_image,
        })
    }

    /// Call the plugin's process_image function.
    /// The plugin modifies `rgba_data` in-place.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `rgba_data` points to a valid buffer of `width * height * 4` bytes.
    /// - `params` is a valid null-terminated C string.
    /// - The buffer remains valid for the duration of the call.
    pub unsafe fn process(
        &self,
        width: u32,
        height: u32,
        rgba_data: *mut u8,
        params: *const c_char,
    ) {
        (self.process_image)(width, height, rgba_data, params);
    }
}

fn find_library(plugin_dir: &Path, plugin_name: &str) -> Result<PathBuf, AppError> {
    let candidates = if cfg!(target_os = "windows") {
        vec![format!("{}.dll", plugin_name)]
    } else if cfg!(target_os = "macos") {
        vec![
            format!("lib{}.dylib", plugin_name),
            format!("lib{}.so", plugin_name),
        ]
    } else {
        vec![format!("lib{}.so", plugin_name)]
    };

    for candidate in &candidates {
        let path = plugin_dir.join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    let attempted: Vec<String> = candidates
        .iter()
        .map(|c| plugin_dir.join(c).display().to_string())
        .collect();
    Err(AppError::PluginLibraryNotFound(attempted.join(", ")))
}

/// Read plugin params from a file and convert to a null-terminated C string.
pub fn read_params(path: &Path) -> Result<CString, AppError> {
    if !path.exists() {
        return Err(AppError::ParamsFileNotFound(path.to_path_buf()));
    }
    let content = std::fs::read_to_string(path)?;
    CString::new(content).map_err(|_| AppError::ParamsContainsNullByte(path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_path(name: &str) -> PathBuf {
        let salt: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        if let Some((stem, ext)) = name.rsplit_once('.') {
            std::env::temp_dir().join(format!("plugin_loader_test_{}_{}.{}", stem, salt, ext))
        } else {
            std::env::temp_dir().join(format!("plugin_loader_test_{}_{}", name, salt))
        }
    }

    #[test]
    fn read_params_valid_json() {
        let path = temp_path("valid.json");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"{\"radius\": 5}").unwrap();
        drop(f);

        let cstring = read_params(&path).unwrap();
        assert_eq!(cstring.to_str().unwrap(), "{\"radius\": 5}");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn read_params_file_not_found() {
        let err = read_params(Path::new("/nonexistent/params.json")).unwrap_err();
        assert!(matches!(err, AppError::ParamsFileNotFound(_)));
    }

    #[test]
    fn read_params_empty_file() {
        let path = temp_path("empty.json");
        std::fs::File::create(&path).unwrap();

        let cstring = read_params(&path).unwrap();
        assert_eq!(cstring.to_str().unwrap(), "");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn read_params_with_null_byte() {
        let path = temp_path("null.json");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"param\x00value").unwrap();
        drop(f);

        let err = read_params(&path).unwrap_err();
        assert!(matches!(err, AppError::ParamsContainsNullByte(_)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn plugin_load_dir_not_found() {
        match Plugin::load(Path::new("/nonexistent/dir"), "foo") {
            Err(AppError::PluginDirNotFound(_)) => {}
            other => panic!("expected PluginDirNotFound, got {:?}", other.as_ref().err()),
        }
    }

    #[test]
    fn plugin_load_library_not_found() {
        let dir = temp_path("plugin_dir");
        std::fs::create_dir_all(&dir).unwrap();

        match Plugin::load(&dir, "nonexistent_plugin_xyz") {
            Err(AppError::PluginLibraryNotFound(_)) => {}
            other => panic!(
                "expected PluginLibraryNotFound, got {:?}",
                other.as_ref().err()
            ),
        }
        std::fs::remove_dir_all(&dir).ok();
    }
}
