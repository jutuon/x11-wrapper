pub extern crate x11;

#[macro_use]
extern crate bitflags;

#[link(name = "X11")]
extern "C" {}

pub mod core;
pub mod protocol;
pub mod property;
