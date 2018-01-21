pub extern crate x11;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

#[link(name = "X11")]
extern "C" {}

pub mod core;
pub mod protocol;
pub mod property;


pub use core::XlibHandle;
pub use core::error::check_error;