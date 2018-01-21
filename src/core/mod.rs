//! Core Xlib functionality

pub mod window;
pub mod color;
pub mod display;
pub mod event;
pub mod error;
pub mod screen;
pub mod visual;
pub mod utils;

use std::sync::Mutex;

use x11::xlib;

use self::display::Display;

lazy_static! {
    static ref INIT_FLAG: Mutex<bool> = Mutex::new(false);
}

/// Initialization error
pub enum XlibInitError {
    AlreadyInitialized,
    /// This error can only happen if runtime library
    /// loading feature is enabled.
    LibraryLoadingError,

    /// Error in Xlib function `xlib::XInitThreads`.
    XInitThreadsError,
}

#[derive(Debug, Clone)]
pub struct XlibHandle;

impl XlibHandle {
    pub fn initialize_xlib() -> Result<Self, XlibInitError> {
        let mut guard = INIT_FLAG.lock().unwrap();

        if *guard {
            Err(XlibInitError::AlreadyInitialized)
        } else {
            let status = unsafe {
                xlib::XInitThreads()
            };

            if status == 0 {
                return Err(XlibInitError::XInitThreadsError);
            }

            error::set_xlib_error_handler();

            *guard = true;

            Ok(XlibHandle)
        }
    }

    /// Create new connection to X11 server.
    pub fn create_display(&self) -> Result<Display, ()> {
        Display::new(self.clone())
    }
}

unsafe impl Send for XlibHandle {}
unsafe impl Sync for XlibHandle {}