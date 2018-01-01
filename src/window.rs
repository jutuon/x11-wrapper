
use std::os::raw::{c_int, c_ulong, c_uint, c_long};
use std::sync::Arc;

use x11::xlib;

use display::DisplayHandle;
use color::{CreatedColormap, ColormapID};
use visual::Visual;
use event::EventMask;

pub struct WindowBuilder {
    display_handle: Arc<DisplayHandle>,
    attributes: xlib::XSetWindowAttributes,
    colormap_and_visual: Option<(CreatedColormap, Visual)>,
    parent_window_id: xlib::Window,
    x: c_int,
    y: c_int,
    width: c_uint,
    height: c_uint,
}

impl WindowBuilder {
    /// Returns error if parent window id is zero.
    pub(crate) fn new(display_handle: Arc<DisplayHandle>, parent_window_id: xlib::Window) -> Result<Self, ()> {

        if parent_window_id == 0 {
            return Err(());
        }

        // default attributes
        let attributes = xlib::XSetWindowAttributes {
            background_pixmap: 0,
            background_pixel: 0, // default undefined
            border_pixmap: xlib::CopyFromParent as xlib::Pixmap,
            border_pixel: 0, // default undefined
            bit_gravity: xlib::ForgetGravity,
            win_gravity: xlib::NorthWestGravity,
            backing_store: xlib::NotUseful,
            backing_planes: c_ulong::max_value(),
            backing_pixel: 0,
            save_under: xlib::False,
            event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: xlib::False,
            colormap: xlib::CopyFromParent as xlib::Colormap,
            cursor: 0,
        };

        Ok(Self {
            attributes,
            display_handle,
            colormap_and_visual: None,
            parent_window_id,
            x: 0,
            y: 0,
            width: 640,
            height: 480,
        })
    }

    pub(crate) fn set_colormap_and_visual(mut self, colormap: CreatedColormap, visual: Visual) -> Self {
        self.attributes.colormap = colormap.id();
        self.colormap_and_visual = Some((colormap, visual));
        self
    }

    /// Default values: x = 0, y = 0
    pub fn set_x_y(mut self, x: c_int, y: c_int) -> Self {
        self.x = x;
        self.y = y;

        self
    }

    /// Default values: width = 640, height = 480
    ///
    /// Panics if width or height is zero.
    pub fn set_width_height(mut self, width: c_uint, height: c_uint) -> Self {
        if width == 0 {
            panic!("WindowBuilder width is zero");
        }

        if height == 0 {
            panic!("WindowBuilder height is zero");
        }

        self.width = width;
        self.height = height;

        self
    }


    pub fn build_input_output_window(mut self) -> Result<InputOutputWindow, ()> {
        let (valuemask, visual, depth, colormap) = if let Some((colormap, visual)) = self.colormap_and_visual {
            (xlib::CWColormap, visual.raw_visual(), visual.depth(), Some(colormap))
        } else {
            (0, xlib::CopyFromParent as *mut xlib::Visual, xlib::CopyFromParent, None)
        };

        let window_id = unsafe {
            xlib::XCreateWindow(
                self.display_handle.raw_display(),
                self.parent_window_id,
                self.x,
                self.y,
                self.width,
                self.height,
                0,
                depth,
                xlib::InputOutput as c_uint,
                visual,
                valuemask,
                &mut self.attributes,
            )
        };

        if window_id == 0 {
            Err(())
        } else {
            Ok(InputOutputWindow {
                display_handle: self.display_handle,
                colormap,
                window_id,
            })
        }
    }
}

pub struct InputOutputWindow {
    display_handle: Arc<DisplayHandle>,
    colormap: Option<CreatedColormap>,
    window_id: xlib::Window,
}

impl InputOutputWindow {
    pub fn map_window(&mut self) {
        // TODO: check errors

        unsafe {
            xlib::XMapWindow(self.display_handle.raw_display(), self.window_id);
        }
    }

    pub fn select_input(&mut self, event_mask: EventMask) {
        // TODO: check errors

        unsafe {
            xlib::XSelectInput(self.display_handle.raw_display(), self.window_id, event_mask.bits());
        }
    }
}

impl Drop for InputOutputWindow {
    fn drop(&mut self) {
        unsafe {
            // TODO: check errors
            xlib::XDestroyWindow(self.display_handle.raw_display(), self.window_id);
        }
    }
}


pub trait CommonAttributes {
    // win-gravity, event-mask, do-not-propagate-mask
    // override-redirect, cursor
}

