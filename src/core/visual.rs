use std::os::raw::{c_int, c_void};
use std::sync::Arc;
use std::mem;

use x11::xlib;

use super::display::DisplayHandle;

#[derive(Debug)]
pub struct Visual {
    display_handle: Arc<DisplayHandle>,
    visual_info: xlib::XVisualInfo,
}

impl Visual {
    /// XGetVisualInfo, XFree
    pub(crate) fn new(
        display_handle: Arc<DisplayHandle>,
        visual_id: xlib::VisualID,
    ) -> Option<Self> {
        let mut template: xlib::XVisualInfo = unsafe { mem::zeroed() };

        template.visualid = visual_id;
        let mut count = 0;

        let visual_info_list = unsafe {
            xlib_function!(
                display_handle.xlib_handle(),
                XGetVisualInfo(
                    Some(display_handle.raw_display()),
                    xlib::VisualIDMask,
                    &mut template,
                    &mut count
                )
            )
        };

        if visual_info_list.is_null() {
            return None;
        } else {
            let visual_info = xlib::XVisualInfo {
                ..unsafe { *visual_info_list }
            };

            unsafe {
                xlib_function!(
                    display_handle.xlib_handle(),
                    XFree(None, visual_info_list as *mut c_void)
                );
            }

            Some(Self {
                display_handle,
                visual_info,
            })
        }
    }

    pub fn raw_visual(&self) -> *mut xlib::Visual {
        self.visual_info.visual
    }

    pub fn depth(&self) -> c_int {
        self.visual_info.depth
    }
}
