//! Banners drawn by [Blip](https://github.com/sidevesh/blip): the macOS 26 Liquid Glass banner
//! that the system shows for volume and brightness.
//!
//! [`ENABLED`] is true on macOS when the Blip package was built in. Everywhere else, and when it
//! was left out (no Swift toolchain, or `OPENLOGI_NO_BLIP` set), every function here does nothing
//! and the caller draws its own window instead.
//!
//! Banners need the AppKit event loop of the calling process, which the overlay's GPUI app provides.

/// Whether banners are drawn by Blip in this build.
pub const ENABLED: bool = cfg!(blip);

/// Shows a slider banner with `level` from 0 to 1 and an icon at each end. Icons are Blip icon
/// names (`blip list-icons`) or SF Symbol names.
pub fn slider(title: &str, level: f64, leading: &str, trailing: &str) {
    #[cfg(blip)]
    ffi::slider(title, level, leading, trailing);
    #[cfg(not(blip))]
    let _ = (title, level, leading, trailing);
}

/// Shows an icon with a title and a second line. Like the system's banners, a Blip banner always
/// has both lines: an empty `detail` shows nothing.
pub fn message(title: &str, detail: &str, icon: &str) {
    #[cfg(blip)]
    ffi::message(title, detail, icon);
    #[cfg(not(blip))]
    let _ = (title, detail, icon);
}

/// Dismisses the banner, if one is showing.
pub fn dismiss() {
    #[cfg(blip)]
    ffi::dismiss();
}

#[cfg(blip)]
mod ffi {
    use std::ffi::{CString, c_char};

    #[expect(unsafe_code, reason = "declares Blip's C interface")]
    unsafe extern "C" {
        fn blip_c_slider(
            title: *const c_char,
            level: f64,
            leading: *const c_char,
            trailing: *const c_char,
        );
        fn blip_c_message(title: *const c_char, message: *const c_char, icon: *const c_char)
        -> i32;
        fn blip_c_dismiss();
    }

    /// A C string for `text`; an interior NUL (which no real title has) cuts it short.
    fn c(text: &str) -> CString {
        let end = text.find('\0').unwrap_or(text.len());
        CString::new(&text[..end]).unwrap_or_default()
    }

    #[expect(unsafe_code, reason = "calls Blip's C interface")]
    pub(super) fn slider(title: &str, level: f64, leading: &str, trailing: &str) {
        let (title, leading, trailing) = (c(title), c(leading), c(trailing));
        // SAFETY: the pointers are NUL-terminated and outlive the call, which copies them.
        unsafe { blip_c_slider(title.as_ptr(), level, leading.as_ptr(), trailing.as_ptr()) };
    }

    #[expect(unsafe_code, reason = "calls Blip's C interface")]
    pub(super) fn message(title: &str, detail: &str, icon: &str) {
        let (title, detail, icon) = (c(title), c(detail), c(icon));
        // SAFETY: as above. It returns -1, showing nothing, only for an empty detail.
        unsafe { blip_c_message(title.as_ptr(), detail.as_ptr(), icon.as_ptr()) };
    }

    #[expect(unsafe_code, reason = "calls Blip's C interface")]
    pub(super) fn dismiss() {
        // SAFETY: takes no arguments.
        unsafe { blip_c_dismiss() };
    }
}
