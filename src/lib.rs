
extern crate x11;

#[link(name = "X11")]
extern "C" {}

pub mod display;
pub mod window;
pub mod screen;
pub mod utils;
pub mod error;
pub mod visual;
pub mod color;