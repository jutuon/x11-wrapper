use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::mem;
use std::ptr;
use std::slice;

use x11::xlib;

use super::display::{Display};
use super::XlibHandle;

pub const XLIB_NONE: xlib::XID = 0;

/// UTF-8 text
#[derive(Debug)]
pub struct Text {
    display_handle: Display,
    text_property: xlib::XTextProperty,
}

#[derive(Debug)]
pub enum TextError<T> {
    NoMemory,
    LocaleNotSupported,
    /// There was internal Null byte in the string.
    NulError,
    /// Count of unconverted characters and text with unconverted characters
    /// replaced with default characters.
    UnconvertedCharacters(c_int, T),
    ConverterNotFound,
    XlibReturnedNullPointer,
    XlibReturnedNegativeTextCount,
    UnknownError,
}

const X_NO_MEMORY: c_int = -1;
const X_LOCALE_NOT_SUPPORTED: c_int = -2;
const X_CONVERTER_NOT_FOUND: c_int = -3;

impl Text {
    /// Xutf8TextListToTextProperty
    pub fn new(display: &Display, text: String) -> Result<Self, TextError<Self>> {
        let c_string = CString::new(text).map_err(|_| TextError::NulError)?;

        let mut one_text = c_string.as_ptr() as *mut c_char;

        let mut text_property: xlib::XTextProperty = unsafe { mem::zeroed() };

        let status = unsafe {
            xlib_function!(
                display.xlib_handle(),
                Xutf8TextListToTextProperty(
                    Some(display.raw_display()),
                    &mut one_text,
                    1,
                    xlib::XUTF8StringStyle,
                    &mut text_property
                )
            )
        };

        match status {
            0 => Ok(Self {
                display_handle: display.clone(),
                text_property,
            }),
            X_NO_MEMORY => {
                // -1
                Err(TextError::NoMemory)
            }
            X_LOCALE_NOT_SUPPORTED => {
                // -2
                Err(TextError::LocaleNotSupported)
            }
            value if value < -2 => {
                // TODO: This may make a memory leak.
                Err(TextError::UnknownError)
            }
            value => {
                let text = Self {
                    display_handle: display.clone(),
                    text_property,
                };
                Err(TextError::UnconvertedCharacters(value, text))
            }
        }
    }

    pub fn raw_text_property(&mut self) -> *mut xlib::XTextProperty {
        &mut self.text_property
    }

    /// Converts CString to String with method `to_string_lossy`.
    ///
    /// Xutf8TextPropertyToTextList, XFreeStringList
    pub fn to_string_list(&mut self) -> Result<Vec<String>, TextError<Vec<String>>> {
        Self::xlib_text_property_to_string_list(
            self.text_property,
            self.display_handle.xlib_handle(),
            self.display_handle.raw_display(),
        )
    }

    /// Xutf8TextPropertyToTextList, XFreeStringList
    pub(crate) fn xlib_text_property_to_string_list(
        mut text_property: xlib::XTextProperty,
        _xlib_handle: &XlibHandle,
        raw_display: *mut xlib::Display,
    ) -> Result<Vec<String>, TextError<Vec<String>>> {
        let mut text_list: *mut *mut c_char = ptr::null_mut();

        let mut text_count = 0;

        let result = unsafe {
            xlib_function!(
                _xlib_handle,
                Xutf8TextPropertyToTextList(
                    Some(raw_display),
                    &mut text_property,
                    &mut text_list,
                    &mut text_count
                )
            )
        };

        match result {
            X_NO_MEMORY => {
                // -1
                return Err(TextError::NoMemory);
            }
            X_LOCALE_NOT_SUPPORTED => {
                // -2
                return Err(TextError::LocaleNotSupported);
            }
            X_CONVERTER_NOT_FOUND => {
                // -3
                return Err(TextError::ConverterNotFound);
            }
            value if value < -3 => {
                // TODO: possible memory leak?
                return Err(TextError::UnknownError);
            }
            _ => (),
        }

        if text_list.is_null() {
            return Err(TextError::XlibReturnedNullPointer);
        }

        if text_count < 0 {
            unsafe {
                xlib_function!(_xlib_handle, XFreeStringList(None, text_list));
            }

            return Err(TextError::XlibReturnedNegativeTextCount);
        }

        let texts: &[*mut c_char] = unsafe {
            // TODO: check that c_int fits in usize
            slice::from_raw_parts(text_list, text_count as usize)
        };

        let mut string_vec = vec![];

        for text_ptr in texts {
            let c_string = unsafe { CString::from_raw(*text_ptr) };

            string_vec.push(c_string.to_string_lossy().to_string());
        }

        let final_result = if result == 0 {
            Ok(string_vec)
        } else {
            Err(TextError::UnconvertedCharacters(result, string_vec))
        };

        unsafe {
            xlib_function!(_xlib_handle, XFreeStringList(None, text_list));
        }

        final_result
    }
}

impl Drop for Text {
    /// XFree
    fn drop(&mut self) {
        unsafe {
            xlib_function!(
                self.display_handle.xlib_handle(),
                XFree(None, self.text_property.value as *mut c_void)
            );
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
                'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => (),
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Atom {
    // Atom does not require display handle because atoms
    // exists in X server until there is no connections to X server.
    atom_id: xlib::Atom,
}

impl Atom {
    /// Returns error if there was no matching atom when `only_if_exists` is `True`.
    ///
    /// If `only_if_exists` is `False`, new atom will be created if there isn't an
    /// atom matching `atom_name`.
    ///
    /// XInternAtom
    pub fn new(
        display: &Display,
        mut atom_name: AtomName,
        only_if_exists: bool,
    ) -> Result<Atom, ()> {
        let only_if_exists = if only_if_exists {
            xlib::True
        } else {
            xlib::False
        };

        let atom_id = unsafe {
            xlib_function!(
                display.xlib_handle(),
                XInternAtom(Some(display.raw_display()), atom_name.as_ptr(), only_if_exists)
            )
        };

        if atom_id == 0 {
            Err(())
        } else {
            Ok(Atom { atom_id })
        }
    }

    /// XGetAtomName, XFree
    pub fn get_name(&self, display: &Display) -> Result<String, ()> {
        let text_ptr = unsafe {
            xlib_function!(
                display.xlib_handle(),
                XGetAtomName(Some(display.raw_display()), self.atom_id())
            )
        };

        if text_ptr.is_null() {
            Err(())
        } else {
            let name = {
                let c_str = unsafe { CStr::from_ptr(text_ptr) };
                c_str.to_string_lossy().to_string()
            };

            unsafe {
                xlib_function!(display.xlib_handle(), XFree(None, text_ptr as *mut c_void));
            }

            Ok(name)
        }
    }

    pub fn atom_id(&self) -> xlib::Atom {
        self.atom_id
    }

    /// This does not check if atom id is zero
    pub(crate) fn from_raw(atom_id: xlib::Atom) -> Atom {
        Self { atom_id }
    }
}

/// Max list length is `std::os::raw::c_int::max_value()`.
///
/// In C language, minimum requirement for int type is 16 bits, so
/// if atom count will be equal or less than `i16::max_value()` you
/// don't have to worry about panics from `add` method.
pub struct AtomList(Vec<Atom>);

impl AtomList {
    pub fn new() -> Self {
        AtomList(Vec::new())
    }

    /// Panics if list length is `std::os::raw::c_int::max_value()`.
    pub fn add(&mut self, atom: Atom) {
        if self.len() == c_int::max_value() {
            panic!("Error: AtomList is full.");
        }

        self.0.push(atom)
    }

    /// This method returns values in range [0; c_int::max_value()]
    pub fn len(&self) -> c_int {
        self.0.len() as c_int
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut xlib::Atom {
        self.0.as_mut_slice().as_mut_ptr() as *mut xlib::Atom
    }

    pub fn atoms(&self) -> &Vec<Atom> {
        &self.0
    }
}

pub(crate) fn to_xlib_bool(value: bool) -> xlib::Bool {
    if value {
        xlib::True
    } else {
        xlib::False
    }
}
