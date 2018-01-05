//! Different X11 Windows

pub mod input;
pub mod input_output;
pub mod attribute;

use std::os::raw::{c_uint, c_int};
use std::mem;

use x11::xlib;

use self::input_output::TopLevelInputOutputWindow;
use core::screen::Screen;

/// A non root window
pub trait Window {
    fn raw_display(&self) -> *mut xlib::Display;
    fn window_id(&self) -> xlib::Window;
}

pub struct ReconfigureWindow<W: Window> {
    window: W,
    window_changes: xlib::XWindowChanges,
    value_mask: WindowChangesMask,
}

impl <W: Window> ReconfigureWindow<W> {
    pub fn new(window: W) -> Self {
        Self {
            window,
            window_changes: unsafe {
                mem::zeroed()
            },
            value_mask: WindowChangesMask::empty(),
        }
    }

    pub fn set_x(mut self, x: c_int) -> Self {
        self.window_changes.x = x;
        self.value_mask |= WindowChangesMask::X;
        self
    }

    pub fn set_y(mut self, y: c_int) -> Self {
        self.window_changes.y = y;
        self.value_mask |= WindowChangesMask::Y;
        self
    }

    /// Panics if width is zero.
    pub fn set_width(mut self, width: c_int) -> Self {
        if width == 0 { panic!("width is zero") }
        self.window_changes.width = width;
        self.value_mask |= WindowChangesMask::WIDTH;
        self
    }

    /// Panics if height is zero.
    pub fn set_height(mut self, height: c_int) -> Self {
        if height == 0 { panic!("height is zero") }

        self.window_changes.height = height;
        self.value_mask |= WindowChangesMask::HEIGHT;
        self
    }

    pub fn set_stack_mode(mut self, mode: StackMode) -> Self {
        self.window_changes.stack_mode = mode as c_int;
        self.value_mask |= WindowChangesMask::STACK_MODE;
        self
    }
}

impl ReconfigureWindow<TopLevelInputOutputWindow> {
    pub fn set_border_width(mut self, border_width: c_int) -> Self {
        self.window_changes.border_width = border_width;
        self.value_mask |= WindowChangesMask::BORDER_WIDTH;
        self
    }

    /// Sibling must really be sibling of window which will be reconfigured.
    pub fn set_sibling_and_stack_mode<S: Window>(mut self, sibling: &S, mode: StackMode) -> Self {
        self.window_changes.sibling = sibling.window_id();
        self.value_mask |= WindowChangesMask::SIBLING;

        self.set_stack_mode(mode)
    }

    pub fn configure(mut self, screen: &Screen) -> Result<TopLevelInputOutputWindow, TopLevelInputOutputWindow> {
        let status = unsafe {
            xlib::XReconfigureWMWindow(
                self.window.raw_display(),
                self.window.window_id(),
                screen.screen_number(),
                self.value_mask.bits(),
                &mut self.window_changes
            )
        };

        if status == 0 {
            Err(self.window)
        } else {
            Ok(self.window)
        }
    }
}

bitflags! {
    struct WindowChangesMask: c_uint {
        const X = xlib::CWX as c_uint;
        const Y = xlib::CWY as c_uint;
        const WIDTH = xlib::CWWidth as c_uint;
        const HEIGHT = xlib::CWHeight as c_uint;
        const BORDER_WIDTH = xlib::CWBorderWidth as c_uint;
        const SIBLING = xlib::CWSibling as c_uint;
        const STACK_MODE = xlib::CWStackMode as c_uint;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(i16)]
pub enum StackMode {
    Above = xlib::Above as i16,
    Below = xlib::Below as i16,
    TopIf = xlib::TopIf as i16,
    BottomIf = xlib::BottomIf as i16,
    Opposite = xlib::Opposite as i16,
}
