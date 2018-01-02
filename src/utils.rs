use std::ffi::{ CString, CStr };
use std::os::raw::{ c_char, c_void, c_int };
use std::mem;

use x11::xlib;

use display::Display;

/// UTF-8 text
#[derive(Debug)]
pub struct Text {
    text_property: xlib::XTextProperty,
}

#[derive(Debug)]
pub enum TextError {
    NoMemory,
    LocaleNotSupported,
    /// There was internal Null byte in the string.
    NulError,
    /// Count of unconverted characters and Text with unconverted characters
    /// replaced with default characters.
    UnconvertedCharacters(c_int, Text),
    UnknownError,
}

impl Text {
    pub fn new(display: &Display, text: String) -> Result<Self, TextError> {
        let c_string = CString::new(text).map_err(|_| TextError::NulError)?;

        let mut one_text = c_string.as_ptr() as *mut c_char;

        let mut text_property: xlib::XTextProperty = unsafe {
            mem::zeroed()
        };

        let status = unsafe {
            xlib::Xutf8TextListToTextProperty(
                display.raw_display(),
                &mut one_text,
                1,
                xlib::XUTF8StringStyle,
                &mut text_property,
            )
        };

        match status {
            0 => {
                Ok(Self {
                    text_property
                })
            },
            -1 => { // XNoMemory
                Err(TextError::NoMemory)
            },
            -2 => { // XLocaleNotSupported
                Err(TextError::LocaleNotSupported)
            },
            value if value < -2 => {
                // TODO: This may make a memory leak.
                Err(TextError::UnknownError)
            },
            value => {
                let text = Self {
                    text_property
                };
                Err(TextError::UnconvertedCharacters(value, text))
            }
        }
    }

    pub fn raw_text_property(&mut self) -> *mut xlib::XTextProperty {
        &mut self.text_property
    }
}

impl Drop for Text {
    fn drop(&mut self) {
        unsafe {
            xlib::XFree(self.text_property.value as *mut c_void);
        }
    }
}

pub enum AtomNameError {
    UnknownCharacter,
    /// There was internal Null byte in the string.
    NulError,
}

/// Characters [a-zA-Z0-9_] with ASCII encoding.
///
/// TODO: Is Host Portable Character Encoding about the same as ASCII?
/// TODO: Support all characters from Host Portable Character Encoding
pub struct AtomName(CString);

impl AtomName {
    /// Returns error if there were unexpected characters in the string.
    pub fn new(string: String) -> Result<Self, AtomNameError> {

        for c in string.chars() {
             match c {
                'a'...'z' | 'A'...'Z' | '0'...'9'| '_' => (),
                _ => return Err(AtomNameError::UnknownCharacter),
            }
        }

        match CString::new(string) {
            Ok(c_string) => Ok(AtomName(c_string)),
            Err(_) => Err(AtomNameError::NulError),
        }
    }

    fn as_ptr(&mut self) -> *const c_char {
        self.0.as_ptr()
    }
}

pub struct Atom {
    // Atom does not require display handle because atoms
    // exists in X server until there is no connections to X server.
    atom_id: xlib::Atom,
}

impl Atom {
    /// Returns error if there was no matching atom.
    pub fn get(display: &Display, mut atom_name: AtomName) -> Result<Atom, ()> {
        let atom_id = unsafe {
            xlib::XInternAtom(display.raw_display(), atom_name.as_ptr(), xlib::True)
        };

        if atom_id == 0 {
            Err(())
        } else {
            Ok(Atom {
                atom_id
            })
        }
    }

    pub fn get_name(&self, display: &Display) -> Result<String, ()> {
        let text_ptr = unsafe {
            xlib::XGetAtomName(display.raw_display(), self.atom_id())
        };

        if text_ptr.is_null() {
            Err(())
        } else {
            let name = {
                let c_str = unsafe {
                    CStr::from_ptr(text_ptr)
                };
                c_str.to_string_lossy().to_string()
            };

            unsafe {
                xlib::XFree(text_ptr as *mut c_void);
            }

            Ok(name)
        }
    }

    pub fn atom_id(&self) -> xlib::Atom {
        self.atom_id
    }
}