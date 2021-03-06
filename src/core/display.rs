use std::ptr;
use std::os::raw::{c_int, c_long, c_ulong};
use std::marker::PhantomData;
use std::ffi::CStr;

use x11::xlib;

use super::XlibHandle;
use super::screen::Screen;
use super::visual::Visual;
use super::event::{send_event, EventBuffer, EventCreator, EventMask, RawEvent};

#[cfg(feature = "multithreading")]
unsafe impl Send for DisplayHandle {}
#[cfg(feature = "multithreading")]
unsafe impl Sync for DisplayHandle {}

/// Stores `XlibHandle` and Xlib display pointer.
#[derive(Debug)]
struct DisplayHandle {
    xlib_handle: XlibHandle,
    raw_display: *mut xlib::Display,
    _marker: PhantomData<xlib::Display>,
}

impl DisplayHandle {
    fn new(
        raw_display: *mut xlib::Display,
        xlib_handle: XlibHandle,
    ) -> Self {
        Self {
            xlib_handle,
            raw_display,
            _marker: PhantomData,
        }
    }
}

impl Drop for DisplayHandle {
    /// Closes Xlib display.
    ///
    /// XCloseDisplay - BadGC
    fn drop(&mut self) {
        unsafe {
            xlib_function!(&self.xlib_handle, XCloseDisplay(Some(self.raw_display)));

            // TODO: check BadGC error
        }
    }
}

/// Connection to X11 server.
#[derive(Debug, Clone)]
pub struct X11Display {
    #[cfg(feature = "multithreading")]
    display_handle: std::sync::Arc<DisplayHandle>,
    #[cfg(not(feature = "multithreading"))]
    display_handle: std::rc::Rc<DisplayHandle>,
}

impl X11Display {
    /// Create new connection to X11 server.
    pub(crate) fn new(xlib_handle: XlibHandle) -> Result<Self, ()> {
        // TODO: display_name string support

        let raw_display = unsafe { xlib_function!(&xlib_handle, XOpenDisplay(None, ptr::null())) };

        if raw_display.is_null() {
            return Err(());
        }

        #[cfg(feature = "multithreading")]
        let display_handle = std::sync::Arc::new(DisplayHandle::new(raw_display, xlib_handle));

        #[cfg(not(feature = "multithreading"))]
        let display_handle = std::rc::Rc::new(DisplayHandle::new(raw_display, xlib_handle));

        Ok(X11Display {
            display_handle
        })
    }

    pub fn xlib_handle(&self) -> &XlibHandle {
        &self.display_handle.xlib_handle
    }

    pub fn raw_display(&self) -> *mut xlib::Display {
        self.display_handle.raw_display
    }

    /// XConnectionNumber
    pub fn connection_number(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XConnectionNumber(Some(self.raw_display()))) }
    }

    /// XDefaultScreenOfDisplay
    pub fn default_screen(&self) -> Screen {
        let screen = unsafe {
            xlib_function!(
                self.xlib_handle(),
                XDefaultScreenOfDisplay(Some(self.raw_display()))
            )
        };

        Screen::new(self.clone(), screen)
    }

    // TODO: Implement XScreenOfDisplay
    /*
    pub fn screen_of_display(&self) {
        unimplemented!()
    }
    */

    /// XDisplayString
    pub fn display_string(&self) -> &CStr {
        unsafe {
            let string = xlib_function!(self.xlib_handle(), XDisplayString(Some(self.raw_display())));
            CStr::from_ptr(string)
        }
    }

    /// Returns `None` if big requests extension is not supported.
    ///
    /// XExtendedMaxRequestSize
    pub fn extended_max_request_size(&self) -> Option<c_long> {
        let size = unsafe {
            xlib_function!(
                self.xlib_handle(),
                XExtendedMaxRequestSize(Some(self.raw_display()))
            )
        };

        if size == 0 {
            None
        } else {
            Some(size)
        }
    }

    /// XMaxRequestSize
    pub fn max_request_size(&self) -> c_long {
        unsafe { xlib_function!(self.xlib_handle(), XMaxRequestSize(Some(self.raw_display()))) }
    }

    /// XLastKnownRequestProcessed
    pub fn last_known_request_processed(&self) -> c_ulong {
        unsafe {
            xlib_function!(
                self.xlib_handle(),
                XLastKnownRequestProcessed(Some(self.raw_display()))
            )
        }
    }

    /// XNextRequest
    pub fn next_request(&self) -> c_ulong {
        unsafe { xlib_function!(self.xlib_handle(), XNextRequest(Some(self.raw_display()))) }
    }

    /// XProtocolVersion
    pub fn protocol_version(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XProtocolVersion(Some(self.raw_display()))) }
    }

    /// XProtocolRevision
    pub fn protocol_revision(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XProtocolRevision(Some(self.raw_display()))) }
    }

    /// Number of events in the event queue.
    ///
    /// XEventsQueued
    pub fn events_queued(&self, mode: EventsQueuedMode) -> c_int {
        unsafe {
            xlib_function!(
                self.xlib_handle(),
                XEventsQueued(Some(self.raw_display()), mode as c_int)
            )
        }
    }

    /// XScreenCount
    pub fn screen_count(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XScreenCount(Some(self.raw_display()))) }
    }

    /// XServerVendor
    pub fn server_vendor(&self) -> &CStr {
        unsafe {
            let string = xlib_function!(self.xlib_handle(), XServerVendor(Some(self.raw_display())));
            CStr::from_ptr(string)
        }
    }

    /// XVendorRelease
    pub fn vendor_release(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XVendorRelease(Some(self.raw_display()))) }
    }

    // TODO: section "Image Format Functions and Macros" from xlib manual

    /// XNoOp
    pub fn send_no_op_request(&self) {
        unsafe {
            xlib_function!(self.xlib_handle(), XNoOp(Some(self.raw_display())));
        }
    }

    /// XGetVisualInfo, XFree
    pub fn visual_from_id(&self, visual_id: xlib::VisualID) -> Option<Visual> {
        Visual::new(self.clone(), visual_id)
    }

    /// XFlush
    pub fn flush_output_buffer(&self) {
        unsafe {
            xlib_function!(self.xlib_handle(), XFlush(Some(self.raw_display())));
        }
    }

    /// XSync
    pub fn sync(&self) {
        unsafe {
            xlib_function!(self.xlib_handle(), XSync(Some(self.raw_display()), xlib::False));
        }
    }

    /// Try to read event from Xlib event queue to `EventBuffer`.
    ///
    /// XEventsQueued, XNextEvent
    pub fn read_event<'a>(&mut self, event_buffer: &'a mut EventBuffer) -> Option<RawEvent<'a>> {
        let mut event_count = self.events_queued(EventsQueuedMode::QueuedAlready);
        if event_count <= 0 {
            event_count = self.events_queued(EventsQueuedMode::QueuedAfterReading);

            if event_count <= 0 {
                return None;
            }
        }

        Some(self.read_event_blocking(event_buffer))
    }

    /// Blocks until event is received.
    ///
    /// XNextEvent
    pub fn read_event_blocking<'a>(&mut self, event_buffer: &'a mut EventBuffer) -> RawEvent<'a> {
        unsafe {
            xlib_function!(
                self.xlib_handle(),
                XNextEvent(Some(self.raw_display()), event_buffer.event_mut_ptr())
            );
        }

        RawEvent::new(event_buffer)
    }

    /// Sends new event.
    ///
    /// Returns error if event conversion to wire protocol format failed.
    ///
    /// X server changes event's send_event to true and serial number.
    ///
    /// This function sets event's display field.
    ///
    /// XSendEvent - BadValue, BadWindow
    pub fn send_event<T: EventCreator>(
        &self,
        window_id: xlib::Window,
        propagate: bool,
        event_mask: EventMask,
        event_creator: &mut T,
    ) -> Result<(), ()> {
        send_event(
            self,
            window_id,
            propagate,
            event_mask,
            event_creator,
        )
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
