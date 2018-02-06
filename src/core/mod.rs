//! Core Xlib functionality
//!
//! [Xlib documentation](https://www.x.org/releases/X11R7.7/doc/libX11/libX11/libX11.html)

pub mod window;
pub mod color;
pub mod display;
pub mod event;
pub mod error;
pub mod screen;
pub mod visual;
pub mod utils;

use std::sync::Mutex;
use std::fmt;

use self::display::Display;

lazy_static! {
    static ref INIT_FLAG: Mutex<bool> = Mutex::new(false);
}

#[derive(Debug)]
/// Initialization error
pub enum XlibInitError {
    AlreadyInitialized,
    /// This error can only happen if runtime library
    /// loading feature is enabled.
    LibraryLoadingError(String),

    /// Error in Xlib function `xlib::XInitThreads`.
    XInitThreadsError,
}

#[cfg(not(feature = "runtime-linking"))]
#[derive(Clone)]
pub struct XlibHandle;


#[cfg(feature = "runtime-linking")]
#[derive(Clone)]
pub struct XlibHandle {
    pub(crate) functions: ::std::sync::Arc<::x11::xlib::Xlib>,
}

impl fmt::Debug for XlibHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "XlibHandle")
    }
}

impl XlibHandle {
    #[cfg(not(feature = "runtime-linking"))]
    fn new() -> Result<Self, XlibInitError> {
        Ok(XlibHandle)
    }

    #[cfg(feature = "runtime-linking")]
    fn new() -> Result<Self, XlibInitError> {
        let functions = ::x11::xlib::Xlib::open().map_err(|e| {
            XlibInitError::LibraryLoadingError(e.detail().to_string())
        })?;

        Ok(XlibHandle {
            functions: ::std::sync::Arc::new(functions)
        })
    }

    pub fn initialize_xlib() -> Result<Self, XlibInitError> {
        let mut guard = INIT_FLAG.lock().unwrap();

        if *guard {
            Err(XlibInitError::AlreadyInitialized)
        } else {
            let xlib_handle = Self::new()?;

            let status = unsafe {
                xlib_function!(&xlib_handle, XInitThreads())
            };

            if status == 0 {
                return Err(XlibInitError::XInitThreadsError);
            }

            error::set_xlib_error_handler(&xlib_handle);

            *guard = true;

            Ok(xlib_handle)
        }
    }

    /// Create new connection to X11 server.
    pub fn create_display(&self) -> Result<Display, ()> {
        Display::new(self.clone())
    }
}

unsafe impl Send for XlibHandle {}
unsafe impl Sync for XlibHandle {}