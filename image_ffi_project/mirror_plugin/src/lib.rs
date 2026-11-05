use std::{ffi::{CStr, c_char}, panic::{AssertUnwindSafe, catch_unwind}};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MirrorParams {
    pub horizontal: bool,
    pub vertical: bool,
}

/// Optional parameters separated by commas or newlines, e.g.
/// `horizontal=true,vertical=false`. Missing keys default to false.
pub fn parse_params(text: &str) -> Result<MirrorParams, String> {
    let mut result = MirrorParams::default();
    for entry in text.split([',', '\n']) {
        let entry = entry.trim();
        if entry.is_empty() { continue; }
        let (key, value) = entry.split_once('=')
            .ok_or_else(|| format!("expected key=value: {entry}"))?;
        let boolean = match value.trim() {
            "true" => true,
            "false" => false,
            _ => return Err(format!("invalid boolean value: {}", value.trim())),
        };
        match key.trim() {
            "horizontal" => result.horizontal = boolean,
            "vertical" => result.vertical = boolean,
            other => return Err(format!("unknown parameter: {other}")),
        }
    }
    Ok(result)
}

pub fn mirror(pixels: &mut [u8], width: usize, height: usize, params: MirrorParams) -> Result<(), String> {
    let required = width.checked_mul(height).and_then(|n| n.checked_mul(4))
        .ok_or("image dimensions are too large")?;
    if pixels.len() != required { return Err("invalid RGBA buffer size".into()); }
    if !params.horizontal && !params.vertical { return Ok(()); }

    for y in 0..height {
        for x in 0..width {
            let target_y = if params.vertical { height - 1 - y } else { y };
            let target_x = if params.horizontal { width - 1 - x } else { x };
            let a = (y * width + x) * 4;
            let b = (target_y * width + target_x) * 4;
            if a < b {
                // Splitting prevents overlapping mutable borrows.
                let (left, right) = pixels.split_at_mut(b);
                left[a..a + 4].swap_with_slice(&mut right[..4]);
            }
        }
    }
    Ok(())
}

/// # Safety
/// The host must provide a writable buffer of width*height*4 bytes and a
/// valid NUL-terminated parameter string. Neither is freed by the plugin.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(width: u32, height: u32, rgba_data: *mut u8, params: *const c_char) {
    // All fallible work and all mutations happen inside this boundary.
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let len = (width as usize).checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or("RGBA size overflow")?;
        if rgba_data.is_null() || params.is_null() || len == 0 || len > isize::MAX as usize {
            return Err("invalid pointer or size".into());
        }
        // SAFETY: The host guarantees that params is NUL-terminated.
        let params_str = unsafe { CStr::from_ptr(params) }.to_str()
            .map_err(|err| format!("parameters must be UTF-8: {err}"))?;
        let parsed = parse_params(params_str)?;
        // SAFETY: The host guarantees exclusive access to a buffer of len bytes.
        let pixels = unsafe { std::slice::from_raw_parts_mut(rgba_data, len) };
        // Keep a backup so unexpected Rust panics never leave a partial result.
        let backup = pixels.to_vec();
        match catch_unwind(AssertUnwindSafe(|| mirror(pixels, width as usize, height as usize, parsed))) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => { pixels.copy_from_slice(&backup); Err(err) }
            Err(_) => { pixels.copy_from_slice(&backup); Err("panic during processing".into()) }
        }
    }));
    match outcome {
        Ok(Ok(())) => {},
        Ok(Err(err)) => eprintln!("mirror: {err}; the image has not been altered"),
        Err(_) => eprintln!("mirror: processing error (panic intercepted))"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Vec<u8> {
        [1u8, 2, 3, 4].into_iter().flat_map(|id| [id, 0, 0, 255]).collect()
    }
    fn pixels(ids: &[u8]) -> Vec<u8> {
        ids.iter().flat_map(|&id| [id, 0, 0, 255]).collect()
    }
    #[test]
    fn parse_valid_and_defaults() {
        assert_eq!(parse_params("horizontal=true,vertical=false").unwrap(), MirrorParams { horizontal: true, vertical: false });
        assert_eq!(parse_params("vertical=true").unwrap(), MirrorParams { horizontal: false, vertical: true });
        assert_eq!(parse_params("").unwrap(), MirrorParams::default());
        assert_eq!(parse_params(" \n ").unwrap(), MirrorParams::default());
    }
    #[test]
    fn parse_invalid() {
        for invalid in ["x=true", "horizontal=yes", "vertical=7", "horizontal", "vertical=true,foo=false"] {
            assert!(parse_params(invalid).is_err(), "{invalid}");
        }
    }
    #[test]
    fn horizontal() {
        let mut buf = input();
        mirror(&mut buf, 2, 2, MirrorParams { horizontal: true, vertical: false }).unwrap();
        assert_eq!(buf, pixels(&[2, 1, 4, 3]));
    }
    #[test]
    fn vertical() {
        let mut buf = input();
        mirror(&mut buf, 2, 2, MirrorParams { horizontal: false, vertical: true }).unwrap();
        assert_eq!(buf, pixels(&[3, 4, 1, 2]));
    }
    #[test]
    fn horizontal_and_vertical() {
        let mut buf = input();
        mirror(&mut buf, 2, 2, MirrorParams { horizontal: true, vertical: true }).unwrap();
        assert_eq!(buf, pixels(&[4, 3, 2, 1]));
    }
}
