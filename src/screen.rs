
use std::os::raw::{c_int, c_ulong};
use std::sync::Arc;

use x11::xlib;

use color::{ DefaultColormap, CreatedColormap };
use display::DisplayHandle;
use error::{ QueryResult, QueryError };
use visual::Visual;
use window::WindowBuilder;
use event::{ ClientMessageEventCreator, send_event, EventMask };

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

    pub fn black_pixel(&self) -> c_ulong {
        unsafe {
            xlib::XBlackPixelOfScreen(self.raw_screen)
        }
    }

    pub fn white_pixel(&self) -> c_ulong {
        unsafe {
            xlib::XWhitePixelOfScreen(self.raw_screen)
        }
    }

    pub fn colormap_cells(&self) -> c_int {
        unsafe {
            xlib::XCellsOfScreen(self.raw_screen)
        }
    }

    pub fn default_colormap(&self) -> Option<DefaultColormap> {
        DefaultColormap::new(unsafe {
            xlib::XDefaultColormapOfScreen(self.raw_screen)
        })
    }

    pub fn default_depth(&self) -> c_int {
        unsafe {
            xlib::XDefaultDepthOfScreen(self.raw_screen)
        }
    }

    pub fn default_visual(&self) -> Option<Visual> {
        let id = unsafe {
            let visual_ptr = xlib::XDefaultVisualOfScreen(self.raw_screen);

            if visual_ptr.is_null() {
                return None;
            }

            xlib::XVisualIDFromVisual(visual_ptr)
        };

        if id == 0 {
            None
        } else {
            Visual::new(self.display_handle.clone(), id)
        }
    }

    pub fn does_backing_store(&self) -> QueryResult<BackingStore> {
        let result = unsafe {
            xlib::XDoesBackingStore(self.raw_screen)
        };


        let result = match result {
            xlib::WhenMapped => BackingStore::WhenMapped,
            xlib::NotUseful => BackingStore::NotUseful,
            xlib::Always => BackingStore::Always,
            _ => return Err(QueryError::UnknownEnumValue),
        };

        Ok(result)
    }

    pub fn does_save_unders(&self) -> bool {
        let result = unsafe {
            xlib::XDoesSaveUnders(self.raw_screen)
        };

        result == xlib::True
    }

    pub fn event_mask(&self) {
        unimplemented!()
    }

    pub fn screen_number(&self) -> c_int {
        unsafe {
            xlib::XScreenNumberOfScreen(self.raw_screen)
        }
    }

    pub fn width_in_pixels(&self) -> c_int {
        unsafe {
            xlib::XWidthOfScreen(self.raw_screen)
        }
    }

    pub fn height_in_pixels(&self) -> c_int {
        unsafe {
            xlib::XHeightOfScreen(self.raw_screen)
        }
    }

    pub fn width_in_millimeters(&self) -> c_int {
        unsafe {
            xlib::XWidthMMOfScreen(self.raw_screen)
        }
    }

    pub fn height_in_millimeters(&self) -> c_int {
        unsafe {
            xlib::XHeightMMOfScreen(self.raw_screen)
        }
    }

    pub fn max_colormap_count(&self) -> c_int {
        unsafe {
            xlib::XMaxCmapsOfScreen(self.raw_screen)
        }
    }

    pub fn min_colormap_count(&self) -> c_int {
        unsafe {
            xlib::XMinCmapsOfScreen(self.raw_screen)
        }
    }

    pub fn planes(&self) -> c_int {
        unsafe {
            xlib::XPlanesOfScreen(self.raw_screen)
        }
    }

    pub fn root_window_id(&self) -> Option<xlib::Window> {
        let id = unsafe {
            xlib::XRootWindowOfScreen(self.raw_screen)
        };

        if id == 0 {
            None
        } else {
            Some(id)
        }
    }

    /// Parent of created window will be root window of this screen.
    ///
    /// Returns error if this Screen does not support `visual`.
    pub fn create_window_builder(&self, visual: Visual) -> Result<WindowBuilder, ()> {

        let created_colormap = CreatedColormap::create(self.display_handle.clone(), self, &visual)?;

        let window_builder = WindowBuilder::new(self.display_handle.clone(), self.root_window_id().unwrap_or(0), true)?
            .set_colormap_and_visual(created_colormap, visual);

        Ok(window_builder)
    }

    /// Send ClientMessage event to root window as
    /// defined in Extended Window Manager Hints 1.3 specification.
    ///
    /// Returns error if root window id is not found or event conversion to wire
    /// protocol fails.
    ///
    /// See also documentation for `Display::send_event`.
    pub fn send_ewmh_client_message_event(&self, client_message_event: &mut ClientMessageEventCreator) -> Result<(), ()> {
        let window_id = self.root_window_id().ok_or(())?;

        send_event(
            self.display_handle.raw_display(),
            window_id,
            false,
            EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
            client_message_event
        )
    }
}


pub enum BackingStore {
    WhenMapped,
    NotUseful,
    Always,
}

