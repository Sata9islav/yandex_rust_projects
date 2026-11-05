//! Shared C ABI function signature used by the application and the plugins.

use std::ffi::c_char;

/// Plugins modify `width * height * 4` RGBA bytes in place.
///
/// # Safety
/// The caller must supply a valid, writable RGBA buffer of the required length
/// and a valid NUL-terminated string. Both must remain alive for the call.
pub type ProcessImageFn = unsafe extern "C" fn(u32, u32, *mut u8, *const c_char);
