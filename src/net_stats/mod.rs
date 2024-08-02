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
