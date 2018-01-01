
use std::ptr;
use std::os::raw::{c_int, c_ulong, c_long};
use std::sync::Arc;
use std::marker::PhantomData;
use std::ffi::CStr;

use x11::xlib;

use screen::Screen;
use visual::Visual;

pub struct DisplayHandle {
    raw_display: *mut xlib::Display,
    _marker: PhantomData<xlib::Display>,
}

impl DisplayHandle {
    pub(crate) fn new_in_arc(raw_display: *mut xlib::Display) -> Arc<DisplayHandle> {
        Arc::new(DisplayHandle {
            raw_display,
            _marker: PhantomData
        })
    }

    pub(crate) fn raw_display(&self) -> *mut xlib::Display {
        self.raw_display
    }
}

impl Drop for DisplayHandle {
    fn drop(&mut self) {
        unsafe {
            xlib::XCloseDisplay(self.raw_display);

            // TODO: check BadGC error
        }
    }
}


pub struct Display {
    display_handle: Arc<DisplayHandle>,
}

impl Display {
    /// Create new connection to X11 server.
    pub fn new() -> Result<Self, ()> {
        // TODO: display_name string support

        let raw_display = unsafe {
            xlib::XOpenDisplay(ptr::null())
        };

        if raw_display.is_null() {
            return Err(())
        }

        Ok(Self {
            display_handle: DisplayHandle::new_in_arc(raw_display),
        })
    }

    pub fn raw_display(&self) -> *mut xlib::Display {
        self.display_handle.raw_display
    }

    pub fn connection_number(&self) -> c_int {
        unsafe {
            xlib::XConnectionNumber(self.raw_display())
        }
    }

    pub fn default_screen(&self) -> Screen {
        let screen = unsafe {
            xlib::XDefaultScreenOfDisplay(self.raw_display())
        };

        Screen::new(self.display_handle.clone(), screen)
    }

    pub fn screen_of_display(&self) {
        unimplemented!()
    }

    pub fn display_string(&self) -> &CStr {
        unsafe {
            let string = xlib::XDisplayString(self.raw_display());
            CStr::from_ptr(string)
        }
    }

    /// Returns `None` if big requests extension is not supported.
    pub fn extended_max_request_size(&self) -> Option<c_long> {
        let size = unsafe {
            xlib::XExtendedMaxRequestSize(self.raw_display())
        };

        if size == 0 {
            None
        } else {
            Some(size)
        }
    }

    pub fn max_request_size(&self) -> c_long {
        unsafe {
            xlib::XMaxRequestSize(self.raw_display())
        }
    }

    pub fn last_known_request_processed(&self) -> c_ulong {
        unsafe {
            xlib::XLastKnownRequestProcessed(self.raw_display())
        }
    }

    pub fn next_request(&self) -> c_ulong {
        unsafe {
            xlib::XNextRequest(self.raw_display())
        }
    }

    pub fn protocol_version(&self) -> c_int {
        unsafe {
            xlib::XProtocolVersion(self.raw_display())
        }
    }

    pub fn protocol_revision(&self) -> c_int {
        unsafe {
            xlib::XProtocolRevision(self.raw_display())
        }
    }

    pub fn event_queue_length(&self) -> c_int {
        unsafe {
            xlib::XQLength(self.raw_display())
        }
    }

    pub fn screen_count(&self) -> c_int {
        unsafe {
            xlib::XScreenCount(self.raw_display())
        }
    }

    pub fn server_vendor(&self) -> &CStr {
        unsafe {
            let string = xlib::XServerVendor(self.raw_display());
            CStr::from_ptr(string)
        }
    }

    pub fn vendor_release(&self) -> c_int {
        unsafe {
            xlib::XVendorRelease(self.raw_display())
        }
    }

    // TODO: section "Image Format Functions and Macros" from xlib manual

    pub fn send_no_op_request(&self) {
        unsafe {
            xlib::XNoOp(self.raw_display());
        }
    }

    pub fn visual_from_id(&self, visual_id: xlib::VisualID) -> Option<Visual> {
        Visual::new(self.display_handle.clone(), visual_id)
    }


    pub fn flush_output_buffer(&self) {
        unsafe {
            xlib::XFlush(self.raw_display());
        }
    }

    pub fn sync(&self) {
        unsafe {
            xlib::XSync(self.raw_display(), xlib::False);
        }
    }
}

