
use std::ptr;
use std::os::raw::{c_int, c_ulong, c_long};
use std::sync::Arc;
use std::marker::PhantomData;
use std::ffi::CStr;

use x11::xlib;

use screen::Screen;
use visual::Visual;
use event::{EventBuffer, EventCreator, EventMask, send_event};

#[derive(Debug)]
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
    event_buffer: EventBuffer,
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
            event_buffer: EventBuffer::zeroed(),
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

    /// Number of events in the event queue.
    pub fn events_queued(&self, mode: EventsQueuedMode) -> c_int {
        unsafe {
            xlib::XEventsQueued(self.raw_display(), mode as c_int)
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

    /// Returns true if function copied new event to `event_ptr`.
    pub fn read_event(&mut self) -> Option<&EventBuffer> {
        let mut event_count = self.events_queued(EventsQueuedMode::QueuedAlready);
        if event_count <= 0 {
            event_count = self.events_queued(EventsQueuedMode::QueuedAfterReading);

            if event_count <= 0 {
                return None;
            }
        }

        Some(self.read_event_blocking())
    }

    /// Blocks until event is received.
    pub fn read_event_blocking(&mut self) -> &EventBuffer {
        unsafe {
            xlib::XNextEvent(self.raw_display(), self.event_buffer.event_mut_ptr());
        }

        &self.event_buffer
    }

    /// Sends new event.
    ///
    /// Returns error if event conversion to wire protocol format failed.
    ///
    /// X server changes event's send_event to true and serial number.
    ///
    /// This function sets event's display field.
    pub fn send_event<T: EventCreator>(&self, window_id: xlib::Window, propagate: bool, event_mask: EventMask, event_creator: &mut T) -> Result<(), ()> {
        send_event(self.raw_display(), window_id, propagate, event_mask, event_creator)
    }
}

/// Enum values from Xlib.h file.
#[repr(i8)]
pub enum EventsQueuedMode {
    /// Behavior equals to function `xlib::XQLength`.
    QueuedAlready = 0,
    /// Try read more events from X11 connection before
    /// returning queue's event count.
    QueuedAfterReading = 1,
    /// Behavior equals to function `xlib::XPending`.
    QueuedAfterFlush = 2,
}