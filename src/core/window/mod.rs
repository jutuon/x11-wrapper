//! Different X11 Windows

pub mod input;
pub mod input_output;
pub mod attribute;

use x11::xlib;

pub trait Window {
    fn raw_display(&self) -> *mut xlib::Display;
    fn window_id(&self) -> xlib::Window;
}