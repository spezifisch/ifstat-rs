use indexmap::IndexMap;
use libc::c_char;
use std::ffi::CString;
use std::ptr;

#[cfg(target_os = "linux")]
mod linux_impl;
#[cfg(target_os = "macos")]
mod macos_impl;
#[cfg(target_os = "windows")]
mod windows_impl;

#[cfg(target_os = "linux")]
pub use linux_impl::*;
#[cfg(target_os = "macos")]
pub use macos_impl::*;
#[cfg(target_os = "windows")]
pub use windows_impl::*;

// Helper function to convert Rust String to C string (raw pointer)
fn string_to_c_string(s: String) -> *mut c_char {
    let c_string = CString::new(s).unwrap();
    c_string.into_raw()
}

// Converts the statistics into a format that can be returned to LabVIEW
fn convert_stats_to_c(stats: IndexMap<String, (u64, u64)>) -> *mut c_char {
    let mut output = String::new();
    for (device, (received, transmitted)) in stats {
        output.push_str(&format!(
            "{},{},{}\n",
            device, received, transmitted
        ));
    }
    string_to_c_string(output)
}

#[no_mangle]
pub extern "C" fn GetNetDevStats() -> *mut c_char {
    // Call the platform-specific implementation of `get_net_dev_stats`
    let stats = get_net_dev_stats();

    // Check if the `stats` is valid (not an error)
    match stats {
        Ok(stats_map) => {
            // Convert the stats to a C string format and return the pointer
            convert_stats_to_c(stats_map)
        }
        Err(_) => {
            // On error, return a null pointer
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn FreeCString(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}
