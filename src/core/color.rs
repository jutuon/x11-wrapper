use std::sync::Arc;

use x11::xlib;

use super::screen::Screen;
use super::display::DisplayHandle;
use super::visual::Visual;

pub struct DefaultColormap(xlib::XID);

impl DefaultColormap {
    pub(crate) fn new(id: xlib::XID) -> Option<Self> {
        if id == 0 {
            None
        } else {
            Some(DefaultColormap(id))
        }
    }
}

pub trait ColormapID {
    fn id(&self) -> xlib::XID;
}

impl ColormapID for DefaultColormap {
    fn id(&self) -> xlib::XID {
        self.0
    }
}

#[derive(Debug)]
pub struct CreatedColormap {
    display_handle: Arc<DisplayHandle>,
    colormap: xlib::Colormap,
}

impl CreatedColormap {
    /// Returns error if `Screen` does not support `Visual`.
    ///
    /// XCreateColormap - BadAlloc, BadMatch, BadValue, BadWindow
    pub(crate) fn create(
        display_handle: Arc<DisplayHandle>,
        screen: &Screen,
        visual: &Visual,
    ) -> Result<CreatedColormap, ()> {
        let root_window_id = match screen.root_window_id() {
            Some(id) => id,
            None => return Err(()),
        };

        let colormap = unsafe {
            xlib_function!(
                display_handle.xlib_handle(),
                XCreateColormap(
                    Some(display_handle.raw_display()),
                    root_window_id,
                    visual.raw_visual(),
                    xlib::AllocNone
                )
            )
        };
        //TODO: colormap AllocAll
        if colormap == 0 {
            // TODO: check errors

            Err(())
        } else {
            Ok(Self {
                display_handle,
                colormap,
            })
        }
    }
}

impl ColormapID for CreatedColormap {
    fn id(&self) -> xlib::XID {
        self.colormap
    }
}

impl Drop for CreatedColormap {
    /// XFreeColormap - BadColor
    fn drop(&mut self) {
        // TODO: check error

        unsafe {
            xlib_function!(
                self.display_handle.xlib_handle(),
                XFreeColormap(Some(self.display_handle.raw_display()), self.colormap)
            );
        }
    }
}
