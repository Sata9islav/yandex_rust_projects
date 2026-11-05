use std::{ffi::{CStr, c_char}, panic::{AssertUnwindSafe, catch_unwind}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlurParams {
    pub radius: usize,
    pub iterations: usize,
}

impl Default for BlurParams {
    fn default() -> Self { Self { radius: 1, iterations: 1 } }
}

pub fn parse_params(text: &str) -> Result<BlurParams, String> {
    let mut result = BlurParams::default();
    for entry in text.split([',', '\n']) {
        let entry = entry.trim();
        if entry.is_empty() { continue; }
        let (key, value) = entry.split_once('=')
            .ok_or_else(|| format!("expected key=value: {entry}"))?;
        match key.trim() {
            "radius" => result.radius = value.trim().parse::<usize>()
                .map_err(|_| format!("radius must be a non-negative integer: {value}"))?,
            "iterations" => result.iterations = value.trim().parse::<usize>()
                .map_err(|_| format!("iterations must be a non-negative integer: {value}"))?,
            other => return Err(format!("unknown parameter: {other}")),
        }
    }
    if result.radius > 100 || result.iterations > 100 {
        return Err("radius and iterations must not exceed 100".into());
    }
    Ok(result)
}

/// Box blur. Edge pixels average only their existing neighbors.
pub fn blur(pixels: &mut [u8], width: usize, height: usize, params: BlurParams) -> Result<(), String> {
    let len = width.checked_mul(height).and_then(|n| n.checked_mul(4))
        .ok_or("image dimensions are too large")?;
    if pixels.len() != len { return Err("invalid RGBA buffer size".into()); }
    if params.radius == 0 || params.iterations == 0 || width == 0 || height == 0 { return Ok(()); }

    let mut next = vec![0u8; len];
    for _ in 0..params.iterations {
        for y in 0..height {
            for x in 0..width {
                let x0 = x.saturating_sub(params.radius);
                let y0 = y.saturating_sub(params.radius);
                let x1 = x.saturating_add(params.radius).min(width - 1);
                let y1 = y.saturating_add(params.radius).min(height - 1);
                let mut sums = [0u64; 4];
                let mut count = 0u64;
                for ny in y0..=y1 {
                    for nx in x0..=x1 {
                        let offset = (ny * width + nx) * 4;
                        for channel in 0..4 { sums[channel] += u64::from(pixels[offset + channel]); }
                        count += 1;
                    }
                }
                let offset = (y * width + x) * 4;
                for channel in 0..4 {
                    next[offset + channel] = (sums[channel] / count) as u8;
                }
            }
        }
        pixels.copy_from_slice(&next);
    }
    Ok(())
}

/// # Safety
/// The host must provide a writable buffer of width*height*4 bytes and a
/// valid NUL-terminated parameter string. Neither is freed by the plugin.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(width: u32, height: u32, rgba_data: *mut u8, params: *const c_char) {
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let len = (width as usize).checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or("RGBA size overflow")?;
        if rgba_data.is_null() || params.is_null() || len == 0 || len > isize::MAX as usize {
            return Err("invalid pointer or size".into());
        }
        // SAFETY: The host guarantees params is a valid C string.
        let params_str = unsafe { CStr::from_ptr(params) }.to_str()
            .map_err(|err| format!("parameters must be UTF-8: {err}"))?;
        let parsed = parse_params(params_str)?;
        // SAFETY: The host owns a writable buffer of exactly len bytes.
        let pixels = unsafe { std::slice::from_raw_parts_mut(rgba_data, len) };
        let backup = pixels.to_vec();
        match catch_unwind(AssertUnwindSafe(|| blur(pixels, width as usize, height as usize, parsed))) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => { pixels.copy_from_slice(&backup); Err(err) },
            Err(_) => { pixels.copy_from_slice(&backup); Err("panic during processing".into()) },
        }
    }));
    match outcome {
        Ok(Ok(())) => {},
        Ok(Err(err)) => eprintln!("blur: {err}; the image has not been altered"),
        Err(_) => eprintln!("blur: processing error (panic intercepted))"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_and_defaults() {
        assert_eq!(parse_params("radius=4,iterations=3").unwrap(), BlurParams { radius: 4, iterations: 3 });
        assert_eq!(parse_params("radius=2").unwrap(), BlurParams { radius: 2, iterations: 1 });
        assert_eq!(parse_params("").unwrap(), BlurParams::default());
        assert_eq!(parse_params("\n ").unwrap(), BlurParams::default());
    }
    #[test]
    fn parse_invalid() {
        for invalid in ["radius=no", "iterations=-1", "unknown=2", "radius", "radius=101", "iterations=3.5"] {
            assert!(parse_params(invalid).is_err(), "{invalid}");
        }
    }
    #[test]
    fn uniform_image_is_unchanged() {
        let mut buf = [25, 60, 90, 255].repeat(9);
        let expected = buf.clone();
        blur(&mut buf, 3, 3, BlurParams { radius: 1, iterations: 3 }).unwrap();
        assert_eq!(buf, expected);
    }
    #[test]
    fn bright_center_reaches_neighbors() {
        let mut buf = [0, 0, 0, 255].repeat(9);
        buf[4 * 4] = 255;
        blur(&mut buf, 3, 3, BlurParams { radius: 1, iterations: 1 }).unwrap();
        assert!(buf[0] > 0);
        assert!(buf[4 * 4] > 0);
        assert!(buf[8 * 4] > 0);
        assert!(buf.chunks_exact(4).all(|p| p[3] == 255));
    }
}
