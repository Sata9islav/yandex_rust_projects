use std::{ffi::CString, path::{Path, PathBuf}};

use libloading::{Library, Symbol};
use plugin_interface::ProcessImageFn;

use crate::error::{AppError, Result};

pub fn plugin_filename(name: &str) -> Result<String> {
    if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err(AppError::InvalidPluginName(name.to_owned()));
    }

    let filename = if cfg!(target_os = "windows") {
        format!("{name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{name}.dylib")
    } else {
        format!("lib{name}.so")
    };
    Ok(filename)
}

pub fn plugin_file_path(directory: &Path, name: &str) -> Result<PathBuf> {
    Ok(directory.join(plugin_filename(name)?))
}

/// Calls an untrusted dynamic library. Only load plugins you trust: the
/// declared C ABI cannot guarantee memory safety of third-party code.
pub fn run_plugin(path: &Path, width: u32, height: u32, rgba: &mut [u8], params: &str) -> Result<()> {
    let expected_len = (width as usize)
        .checked_mul(height as usize)
        .and_then(|len| len.checked_mul(4))
        .ok_or(AppError::InvalidBuffer)?;
    if rgba.len() != expected_len {
        return Err(AppError::InvalidBuffer);
    }
    let params = CString::new(params)?;

    // SAFETY: The library is a trusted cdylib built against our interface.
    // The Library, Symbol, CString and RGBA buffer remain alive throughout the call.
    // The plugins are responsible for validating raw pointers and not unwinding
    // across this extern "C" boundary.
    unsafe {
        let lib = Library::new(path).map_err(|source| AppError::LoadPlugin {
            path: path.to_path_buf(), source,
        })?;
        let process: Symbol<'_, ProcessImageFn> = lib.get(b"process_image\0")
            .map_err(|source| AppError::MissingSymbol {
                path: path.to_path_buf(), source,
            })?;
        process(width, height, rgba.as_mut_ptr(), params.as_ptr());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_is_os_specific() {
        let file = plugin_filename("mirror").unwrap();
        assert_eq!(file, if cfg!(target_os = "windows") { "mirror.dll" }
            else if cfg!(target_os = "macos") { "libmirror.dylib" }
            else { "libmirror.so" });
    }

    #[test]
    fn rejects_paths_as_plugin_names() {
        assert!(plugin_filename("../mirror").is_err());
        assert!(plugin_filename("libmirror.so").is_err());
        assert!(plugin_filename("").is_err());
    }

    #[test]
    fn invalid_rgba_buffer_is_rejected_before_loading() {
        let mut pixels = vec![0; 3];
        assert!(matches!(run_plugin(Path::new("missing.so"), 1, 1, &mut pixels, ""),
            Err(AppError::InvalidBuffer)));
    }
}
