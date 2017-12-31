
use std::ptr;
use std::os::raw::{c_int, c_ulong};
use std::sync::Arc;

use x11::xlib;

use utils::Colormap;
use display::DisplayHandle;
use error::{ QueryResult, QueryError };


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

    pub fn default_colormap(&self) -> Colormap {
        Colormap::new(unsafe {
            xlib::XDefaultColormapOfScreen(self.raw_screen)
        })
    }

    pub fn default_depth(&self) -> c_int {
        unsafe {
            xlib::XDefaultDepthOfScreen(self.raw_screen)
        }
    }

    pub fn default_visual(&self) {
        unimplemented!()
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

    pub fn root_window(&self) {
        unimplemented!()
    }
}


pub enum BackingStore {
    WhenMapped,
    NotUseful,
    Always,
}

