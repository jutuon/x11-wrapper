
#[cfg(not(feature = "runtime-linking"))]
pub extern crate x11;

#[cfg(feature = "runtime-linking")]
pub extern crate x11_dl as x11;

macro_rules! xlib_function {
    ( $xlib_handle:expr, $function:tt ( $( $function_argument:expr ),*) ) => {
        {
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

#[cfg_attr(not(feature = "runtime-linking"), link(name="X11"))]
extern "C" {}

pub mod core;
pub mod protocol;
pub mod property;

pub use core::XlibHandle;
pub use core::error::check_error;