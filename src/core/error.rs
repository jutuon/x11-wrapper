
//! Xlib error handling

use std::io;
use std::io::Write;
use std::os::raw::{c_int, c_ulong, c_uchar, c_char};
use std::sync::Mutex;
use std::process;
use std::mem;

use super::display::Display;

use x11::xlib;

lazy_static! {
    static ref ERROR_BUFFER: Mutex<Option<ErrorEvent>> = Mutex::new(None);
}

#[derive(Debug, Clone, Copy)]
pub enum ProtocolError {
    BadAccess,
    BadAlloc,
    BadAtom,
    BadColor,
    BadCursor,
    BadDrawable,
    BadFont,
    BadGC,
    BadIDChoice,
    BadImplementation,
    BadLength,
    BadMatch,
    BadName,
    BadPixmap,
    BadRequest,
    BadValue,
    BadWindow,
    UnknownError(c_uchar),
}

impl ProtocolError {
    fn from_xlib_error_code(code: c_uchar) -> ProtocolError {
        match code {
            xlib::BadAccess => ProtocolError::BadAccess,
            xlib::BadAlloc => ProtocolError::BadAlloc,
            xlib::BadAtom => ProtocolError::BadAtom,
            xlib::BadColor => ProtocolError::BadColor,
            xlib::BadCursor => ProtocolError::BadCursor,
            xlib::BadDrawable => ProtocolError::BadDrawable,
            xlib::BadFont => ProtocolError::BadFont,
            xlib::BadGC => ProtocolError::BadGC,
            xlib::BadIDChoice => ProtocolError::BadIDChoice,
            xlib::BadImplementation => ProtocolError::BadImplementation,
            xlib::BadLength => ProtocolError::BadLength,
            xlib::BadMatch => ProtocolError::BadMatch,
            xlib::BadName => ProtocolError::BadName,
            xlib::BadPixmap => ProtocolError::BadPixmap,
            xlib::BadRequest => ProtocolError::BadRequest,
            xlib::BadValue => ProtocolError::BadValue,
            xlib::BadWindow => ProtocolError::BadWindow,
            code => ProtocolError::UnknownError(code),
        }
    }

    pub fn to_xlib_error_code(&self) -> c_uchar {
        match *self {
            ProtocolError::BadAccess => xlib::BadAccess,
            ProtocolError::BadAlloc => xlib::BadAlloc,
            ProtocolError::BadAtom => xlib::BadAtom,
            ProtocolError::BadColor => xlib::BadColor,
            ProtocolError::BadCursor => xlib::BadCursor,
            ProtocolError::BadDrawable => xlib::BadDrawable,
            ProtocolError::BadFont => xlib::BadFont,
            ProtocolError::BadGC => xlib::BadGC,
            ProtocolError::BadIDChoice => xlib::BadIDChoice,
            ProtocolError::BadImplementation => xlib::BadImplementation,
            ProtocolError::BadLength => xlib::BadLength,
            ProtocolError::BadMatch => xlib::BadMatch,
            ProtocolError::BadName => xlib::BadName,
            ProtocolError::BadPixmap => xlib::BadPixmap,
            ProtocolError::BadRequest => xlib::BadRequest,
            ProtocolError::BadValue => xlib::BadValue,
            ProtocolError::BadWindow => xlib::BadWindow,
            ProtocolError::UnknownError(code) => code,
        }
    }
}

#[derive(Debug)]
pub struct ErrorEvent {
    pub resource_id: xlib::XID,
    pub serial: c_ulong,
    pub error: ProtocolError,
    pub request_code: c_uchar,
    pub minor_code: c_uchar,
}

#[derive(Debug)]
pub struct ErrorEventAndText {
    pub error: ErrorEvent,
    pub error_text: String,
}

#[inline(never)]
// Note that panics in this function will make undefined behavior, because
// Xlib will call this function.
// eprintln! macro may panic so write! macro is used instead.
extern "C" fn protocol_error_handler(
    _raw_display: *mut xlib::Display,
    event: *mut xlib::XErrorEvent,
) -> c_int {
    let mut buffer = match ERROR_BUFFER.lock() {
        Ok(mutex_guard) => mutex_guard,
        Err(error) => {
            // Abort the program because there shouldn't be any panics
            // happening when accessing error buffer mutex.
            let mut stderr = io::stderr();
            let _ = write!(stderr, "x11_wrapper bug: error buffer mutex error {}", error);

            process::abort();
        }
    };

    let error = unsafe {
        ErrorEvent {
            resource_id: (*event).resourceid,
            serial: (*event).serial,
            error: ProtocolError::from_xlib_error_code((*event).error_code),
            request_code: (*event).request_code,
            minor_code: (*event).minor_code,
        }
    };

    let mut stderr = io::stderr();
    let _ = write!(stderr, "x11_wrapper: {:?}", error);

    if buffer.is_none() {
        *buffer = Some(error);
    }

    0
}



pub(crate) fn set_xlib_error_handler() {
    unsafe {
        xlib::XSetErrorHandler(Some(protocol_error_handler));
    }
}

/// Locks error buffer mutex and returns error buffers
/// current value. Sets error buffer's value to `None`.
///
/// There is only space for one error in the buffer. If there is
/// already an error in the buffer and Xlib calls error handler
/// function, the function will simply discard the new error.
pub fn check_error(display: &Display) -> Option<ErrorEventAndText> {
    let mut buffer = ERROR_BUFFER.lock().unwrap();
    buffer.take().map(|error_event| {
        if mem::size_of::<c_char>() != 8 {
            eprintln!("x11_wrapper warning: c_char is not eight bytes");

            ErrorEventAndText {
                error: error_event,
                error_text: String::new(),
            }
        } else if mem::size_of::<c_uchar>() != 8 {
            eprintln!("x11_wrapper warning: c_uchar is not eight bytes");

            ErrorEventAndText {
                error: error_event,
                error_text: String::new(),
            }
        } else {
            const TEXT_BUFFER_SIZE: usize = 256;

            let mut text_buffer: [c_uchar; TEXT_BUFFER_SIZE] = [0; TEXT_BUFFER_SIZE];

            unsafe {
                xlib::XGetErrorText(
                    display.raw_display(),
                    error_event.error.to_xlib_error_code() as c_int,
                    text_buffer.as_mut_ptr() as *mut c_char,
                    TEXT_BUFFER_SIZE as c_int,
                );
            }

            // TODO: Check that last byte of the buffer is zero?

            let mut zero_byte_index = 0;

            for (i, data) in text_buffer.iter().enumerate() {
                if *data == 0 {
                    zero_byte_index = i;
                }
            };

            let (text, _) = text_buffer.split_at(zero_byte_index);

            ErrorEventAndText {
                error: error_event,
                error_text: String::from_utf8_lossy(text).into_owned(),
            }
        }
    })
}

pub enum QueryError {
    UnknownEnumValue,
}

pub type QueryResult<T> = Result<T, QueryError>;
