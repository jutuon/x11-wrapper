//! This library is a safe wrapper over Xlib and related X11 protocol extension libraries.
//!
//! # Read this before using this library.
//!
//! Xlib will **terminate** your program without asking if it encounters a system call error.
//! Losing the connection to X11 server will most likely cause system call error
//! and your program just terminates without
//! running any destructors.
//!
//! Xlib function calls may generate errors that may occur some time after that specific
//! function call. This happens because Xlib does buffering for X11 requests. With function `check_errors`
//! you can check if an Xlib error is occurred. However this wrapper library prevents some
//! errors related to wrong resource IDs, as resource ID is stored as private object attribute, but
//! if some other X11 client will destroy that
//! resource you will still get an error.
//!
//! TODO: Check which functions may make errors and document those errors.
//!
//! With running program with environment variable `_Xdebug` or calling a specific function, you can set
//! Xlib to operate synchronously, so every Xlib function will wait an response from X11 server. This allows
//! to check errors with function `check_errors` after every function call which may make an error. However this
//! is not possible with multiple connections to X11 server in separate threads.
//!
//! # Window creation
//! TODO: simple example
//!
//! # Loading libraries at runtime
//! Crate feature `runtime-linking` enables loading of Xlib and other libraries
//! at runtime. Loaded libraries however won't close properly and leave a
//! memory leak which is [x11_dl issue](https://github.com/Daggerbot/x11-rs/issues/67), but
//! that will not be a major problem as this wrapper library allows to load the libraries only once.

#[cfg(not(feature = "runtime-linking"))]
pub extern crate x11;

#[cfg(feature = "runtime-linking")]
pub extern crate x11_dl as x11;

macro_rules! xlib_function {
    ( $xlib_handle:expr, $function:tt ( Some($raw_display:expr) ) ) => {
        xlib_function!($xlib_handle, $function(Some($raw_display),))
    };
    ( $xlib_handle:expr, $function:tt ( None ) ) => {
        xlib_function!($xlib_handle, $function(None,))
    };
    ( $xlib_handle:expr, $function:tt ( Some($raw_display:expr), $( $function_argument:expr ),*) ) => {
        {
            let _: *mut x11::xlib::Display = $raw_display;
            let _: &::core::XlibHandle = $xlib_handle;

            #[cfg(feature = "multithreading")]
            xlib_function!($xlib_handle, XLockDisplay(None, $raw_display));

            #[cfg(not(feature = "runtime-linking"))]
            let result = {
                (::x11::xlib::$function)($raw_display, $( $function_argument ,)* )
            };

            #[cfg(feature = "runtime-linking")]
            let result = {
                ($xlib_handle.functions.$function)($raw_display, $( $function_argument ,)* )
            };

            #[cfg(feature = "multithreading")]
            xlib_function!($xlib_handle, XUnlockDisplay(None, $raw_display));

            result
        }
    };
    ( $xlib_handle:expr, $function:tt ( None, $( $function_argument:expr ),*) ) => {
        {
            let _: &::core::XlibHandle = $xlib_handle;

            #[cfg(not(feature = "runtime-linking"))]
            {
                (::x11::xlib::$function)( $( $function_argument ,)* )
            }

            #[cfg(feature = "runtime-linking")]
            {
                ($xlib_handle.functions.$function)( $( $function_argument ,)* )
            }
        }
    };
}

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

#[cfg_attr(not(feature = "runtime-linking"), link(name = "X11"))]
extern "C" {}

pub mod core;
pub mod protocol;
pub mod property;

pub use core::XlibHandle;
pub use core::error::check_error;
