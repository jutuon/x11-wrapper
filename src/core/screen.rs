use std::os::raw::{c_int, c_ulong};
use std::sync::Arc;

use x11::xlib;

use super::color::{DefaultColormap};
use super::display::DisplayHandle;
use super::error::{QueryError, QueryResult};
use super::visual::Visual;
use super::event::{send_event, ClientMessageEventCreator, EventMask};
use super::XlibHandle;

pub struct Screen {
    display_handle: Arc<DisplayHandle>,
    raw_screen: *mut xlib::Screen,
}

impl Screen {
    pub(crate) fn new(display_handle: Arc<DisplayHandle>, raw_screen: *mut xlib::Screen) -> Self {
        Self {
            display_handle,
            raw_screen,
        }
    }

    pub fn raw_screen(&self) -> *mut xlib::Screen {
        self.raw_screen
    }

    pub fn xlib_handle(&self) -> &XlibHandle {
        self.display_handle.xlib_handle()
    }

    pub(crate) fn display_handle(&self) -> &Arc<DisplayHandle> {
        &self.display_handle
    }

    pub fn black_pixel(&self) -> c_ulong {
        unsafe { xlib_function!(self.xlib_handle(), XBlackPixelOfScreen(self.raw_screen)) }
    }

    pub fn white_pixel(&self) -> c_ulong {
        unsafe { xlib_function!(self.xlib_handle(), XWhitePixelOfScreen(self.raw_screen)) }
    }

    pub fn colormap_cells(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XCellsOfScreen(self.raw_screen)) }
    }

    pub fn default_colormap(&self) -> Option<DefaultColormap> {
        DefaultColormap::new(unsafe { xlib_function!(self.xlib_handle(), XDefaultColormapOfScreen(self.raw_screen)) })
    }

    pub fn default_depth(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XDefaultDepthOfScreen(self.raw_screen)) }
    }

    pub fn default_visual(&self) -> Option<Visual> {
        let id = unsafe {
            let visual_ptr = xlib_function!(self.xlib_handle(), XDefaultVisualOfScreen(self.raw_screen));

            if visual_ptr.is_null() {
                return None;
            }

            xlib_function!(self.xlib_handle(), XVisualIDFromVisual(visual_ptr))
        };

        if id == 0 {
            None
        } else {
            Visual::new(self.display_handle.clone(), id)
        }
    }

    pub fn does_backing_store(&self) -> QueryResult<BackingStore> {
        let result = unsafe { xlib_function!(self.xlib_handle(), XDoesBackingStore(self.raw_screen)) };

        let result = match result {
            xlib::WhenMapped => BackingStore::WhenMapped,
            xlib::NotUseful => BackingStore::NotUseful,
            xlib::Always => BackingStore::Always,
            _ => return Err(QueryError::UnknownEnumValue),
        };

        Ok(result)
    }

    pub fn does_save_unders(&self) -> bool {
        let result = unsafe { xlib_function!(self.xlib_handle(), XDoesSaveUnders(self.raw_screen)) };

        result == xlib::True
    }

    pub fn event_mask(&self) {
        unimplemented!()
    }

    pub fn screen_number(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XScreenNumberOfScreen(self.raw_screen)) }
    }

    pub fn width_in_pixels(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XWidthOfScreen(self.raw_screen)) }
    }

    pub fn height_in_pixels(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XHeightOfScreen(self.raw_screen)) }
    }

    pub fn width_in_millimeters(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XWidthMMOfScreen(self.raw_screen)) }
    }

    pub fn height_in_millimeters(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XHeightMMOfScreen(self.raw_screen)) }
    }

    pub fn max_colormap_count(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XMaxCmapsOfScreen(self.raw_screen)) }
    }

    pub fn min_colormap_count(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XMinCmapsOfScreen(self.raw_screen)) }
    }

    pub fn planes(&self) -> c_int {
        unsafe { xlib_function!(self.xlib_handle(), XPlanesOfScreen(self.raw_screen)) }
    }

    pub fn root_window_id(&self) -> Option<xlib::Window> {
        let id = unsafe { xlib_function!(self.xlib_handle(), XRootWindowOfScreen(self.raw_screen)) };

        if id == 0 {
            None
        } else {
            Some(id)
        }
    }

    /// Send ClientMessage event to root window as
    /// defined in Extended Window Manager Hints 1.3 specification.
    ///
    /// Returns error if root window id is not found or event conversion to wire
    /// protocol fails.
    ///
    /// See also documentation for `Display::send_event`.
    pub fn send_ewmh_client_message_event(
        &self,
        client_message_event: &mut ClientMessageEventCreator,
    ) -> Result<(), ()> {
        let window_id = self.root_window_id().ok_or(())?;

        send_event(
            &self.display_handle,
            window_id,
            false,
            EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
            client_message_event,
        )
    }
}

pub enum BackingStore {
    WhenMapped,
    NotUseful,
    Always,
}
