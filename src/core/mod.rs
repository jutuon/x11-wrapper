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

use self::display::X11Display;

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

/// Stores Xlib functions when crate feature
/// `runtime-linking` is enabled.
#[cfg(not(feature = "runtime-linking"))]
#[derive(Clone)]
pub struct XlibHandle;

#[cfg(feature = "runtime-linking")]
#[derive(Clone)]
pub struct XlibHandle {
    #[cfg(feature = "multithreading")]
    pub(crate) functions: ::std::sync::Arc<::x11::xlib::Xlib>,
    #[cfg(not(feature = "multithreading"))]
    pub(crate) functions: ::std::rc::Rc<::x11::xlib::Xlib>,
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
        let functions = ::x11::xlib::Xlib::open()
            .map_err(|e| XlibInitError::LibraryLoadingError(e.detail().to_string()))?;

        #[cfg(feature = "multithreading")]
        let functions = std::sync::Arc::new(functions);

        #[cfg(not(feature = "multithreading"))]
        let functions = std::rc::Rc::new(functions);

        Ok(XlibHandle {
            functions
        })
    }

    /// Initialize Xlib. This function will return error if
    /// `XlibHandle` is already created.
    ///
    /// XSetErrorHandler
    ///
    /// If Cargo feature `multithreading` is enabled function
    /// XInitThreads is also called.
    pub fn initialize_xlib() -> Result<Self, XlibInitError> {
        let mut guard = INIT_FLAG.lock().unwrap();

        if *guard {
            Err(XlibInitError::AlreadyInitialized)
        } else {
            let xlib_handle = Self::new()?;

            #[cfg(feature = "multithreading")]
            {
                let status = unsafe { xlib_function!(&xlib_handle, XInitThreads(None)) };

                if status == 0 {
                    return Err(XlibInitError::XInitThreadsError);
                }
            }

            error::set_xlib_error_handler(&xlib_handle);

            *guard = true;

            Ok(xlib_handle)
        }
    }

    /// Create new connection to X11 server.
    ///
    /// XOpenDisplay
    pub fn create_display(&self) -> Result<X11Display, ()> {
        X11Display::new(self.clone())
    }
}

#[cfg(feature = "multithreading")]
unsafe impl Send for XlibHandle {}
#[cfg(feature = "multithreading")]
unsafe impl Sync for XlibHandle {}
