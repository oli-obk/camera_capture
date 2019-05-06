extern crate image;

#[cfg(unix)]
pub use linux::*;
#[cfg(windows)]
pub use windows::*;

pub mod error;

#[cfg(unix)]
pub mod linux;
#[cfg(windows)]
pub mod windows;
