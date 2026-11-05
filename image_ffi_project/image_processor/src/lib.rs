pub mod error;
pub mod plugin_loader;

use std::{fs, path::Path};
use error::{AppError, Result};

pub fn process_image(input: &Path, output: &Path, plugin_name: &str, params_path: &Path, plugin_dir: &Path) -> Result<()> {
    for path in [input, params_path] {
        if !path.is_file() {
            return Err(AppError::FileNotFound(path.to_path_buf()));
        }
    }

    let plugin_path = plugin_loader::plugin_file_path(plugin_dir, plugin_name)?;
    if !plugin_path.is_file() {
        return Err(AppError::FileNotFound(plugin_path));
    }
    let params = fs::read_to_string(params_path).map_err(|source| AppError::ReadParams {
        path: params_path.to_path_buf(), source,
    })?;
    let decoded = image::open(input).map_err(|source| AppError::OpenImage {
        path: input.to_path_buf(), source,
    })?;
    let mut rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();

    plugin_loader::run_plugin(&plugin_path, width, height, rgba.as_mut(), &params)?;
    rgba.save_with_format(output, image::ImageFormat::Png)
        .map_err(|source| AppError::SaveImage { path: output.to_path_buf(), source })?;
    Ok(())
}
