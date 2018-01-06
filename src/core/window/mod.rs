//! Different X11 Windows

pub mod input;
pub mod input_output;
pub mod attribute;

use std::os::raw::{c_uint, c_int, c_long, c_void, c_ulong};
use std::mem;
use std::slice;
use std::ptr;

use x11::xlib;

use self::input_output::TopLevelInputOutputWindow;
use core::screen::Screen;
use core::utils::{Atom, XLIB_NONE, AtomList};

/// A non root window
pub trait Window {
    fn raw_display(&self) -> *mut xlib::Display;
    fn window_id(&self) -> xlib::Window;
}

pub struct ReconfigureWindow<W: Window> {
    window: W,
    window_changes: xlib::XWindowChanges,
    value_mask: WindowChangesMask,
}

impl <W: Window> ReconfigureWindow<W> {
    pub fn new(window: W) -> Self {
        Self {
            window,
            window_changes: unsafe {
                mem::zeroed()
            },
            value_mask: WindowChangesMask::empty(),
        }
    }

    pub fn set_x(mut self, x: c_int) -> Self {
        self.window_changes.x = x;
        self.value_mask |= WindowChangesMask::X;
        self
    }

    pub fn set_y(mut self, y: c_int) -> Self {
        self.window_changes.y = y;
        self.value_mask |= WindowChangesMask::Y;
        self
    }

    /// Panics if width is zero.
    pub fn set_width(mut self, width: c_int) -> Self {
        if width == 0 { panic!("width is zero") }
        self.window_changes.width = width;
        self.value_mask |= WindowChangesMask::WIDTH;
        self
    }

    /// Panics if height is zero.
    pub fn set_height(mut self, height: c_int) -> Self {
        if height == 0 { panic!("height is zero") }

        self.window_changes.height = height;
        self.value_mask |= WindowChangesMask::HEIGHT;
        self
    }

    pub fn set_stack_mode(mut self, mode: StackMode) -> Self {
        self.window_changes.stack_mode = mode as c_int;
        self.value_mask |= WindowChangesMask::STACK_MODE;
        self
    }
}

impl ReconfigureWindow<TopLevelInputOutputWindow> {
    pub fn set_border_width(mut self, border_width: c_int) -> Self {
        self.window_changes.border_width = border_width;
        self.value_mask |= WindowChangesMask::BORDER_WIDTH;
        self
    }

    /// Sibling must really be sibling of window which will be reconfigured.
    pub fn set_sibling_and_stack_mode<S: Window>(mut self, sibling: &S, mode: StackMode) -> Self {
        self.window_changes.sibling = sibling.window_id();
        self.value_mask |= WindowChangesMask::SIBLING;

        self.set_stack_mode(mode)
    }

    pub fn configure(mut self, screen: &Screen) -> Result<TopLevelInputOutputWindow, TopLevelInputOutputWindow> {
        let status = unsafe {
            xlib::XReconfigureWMWindow(
                self.window.raw_display(),
                self.window.window_id(),
                screen.screen_number(),
                self.value_mask.bits(),
                &mut self.window_changes
            )
        };

        if status == 0 {
            Err(self.window)
        } else {
            Ok(self.window)
        }
    }
}

bitflags! {
    struct WindowChangesMask: c_uint {
        const X = xlib::CWX as c_uint;
        const Y = xlib::CWY as c_uint;
        const WIDTH = xlib::CWWidth as c_uint;
        const HEIGHT = xlib::CWHeight as c_uint;
        const BORDER_WIDTH = xlib::CWBorderWidth as c_uint;
        const SIBLING = xlib::CWSibling as c_uint;
        const STACK_MODE = xlib::CWStackMode as c_uint;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(i16)]
pub enum StackMode {
    Above = xlib::Above as i16,
    Below = xlib::Below as i16,
    TopIf = xlib::TopIf as i16,
    BottomIf = xlib::BottomIf as i16,
    Opposite = xlib::Opposite as i16,
}

#[derive(Debug)]
pub struct PropertyData<T> {
    property_type: Atom,
    data: Vec<T>,
}

impl PropertyData<u8> {
    pub fn from_data(data: &[u8], property_type: Atom) -> Self {
        Self {
            property_type,
            data: data.to_vec(),
        }
    }

    pub fn new(property_type: Atom) -> Self {
        Self {
            property_type,
            data: vec![],
        }
    }
}

impl PropertyData<u16> {
    pub fn from_data(data: &[u16], property_type: Atom) -> Self {
        Self {
            property_type,
            data: data.to_vec(),
        }
    }

    pub fn new(property_type: Atom) -> Self {
        Self {
            property_type,
            data: vec![],
        }
    }
}

impl PropertyData<u32> {
    pub fn from_data(data: &[u32], property_type: Atom) -> Self {
        Self {
            property_type,
            data: data.to_vec(),
        }
    }

    pub fn new(property_type: Atom) -> Self {
        Self {
            property_type,
            data: vec![],
        }
    }
}

impl <T> PropertyData<T> {
    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    pub fn property_type(&self) -> Atom {
        self.property_type
    }
}

#[derive(Debug)]
pub enum Property {
    Char(PropertyData<u8>),
    Short(PropertyData<u16>),
    Long(PropertyData<u32>),
}

pub trait WindowProperties: Window {
    /// Returns property's all data.
    ///
    /// ### Arguments
    /// `is_deleted` - If true, the specified property will be deleted if there is no errors.
    fn get_property(
        &self,
        property_name: Atom,
        property_type: PropertyType,
        is_deleted: bool,
    ) -> Result<Property, PropertyError> {
        let mut actual_type_return = 0;
        let mut actual_format_return = 0;
        let mut nitems_return = 0;
        let mut bytes_after_return: c_ulong = 0;
        let mut prop_return = ptr::null_mut();

        let result = unsafe {
            xlib::XGetWindowProperty(
                self.raw_display(),
                self.window_id(),
                property_name.atom_id(),
                0, // data offset

                // We want all data so lets use value (c_long::max_value() / 4)
                // as argument because xlib uses long_length argument like this
                // L = MINIMUM(T, 4 * long_length)
                (c_long::max_value() / 4),

                to_xlib_bool(is_deleted),
                property_type.to_xlib_property_function_parameter(),
                &mut actual_type_return,
                &mut actual_format_return,
                &mut nitems_return,
                &mut bytes_after_return,
                &mut prop_return
            )
        };

        if result != xlib::Success as c_int {
            return Err(PropertyError::FunctionFailed)
        }

        if prop_return.is_null() {
            return Err(PropertyError::PropertyDataHandleNull)
        }

        // TODO: check that c_ulong can fit in usize
        // TODO: Check that actual_type_return is valid atom or trust Xlib?

        if actual_type_return == XLIB_NONE {
            // property does not exist

            // free the xlib one extra byte
            unsafe {
                xlib::XFree(prop_return as *mut c_void);
            }

            return Err(PropertyError::DoesNotExist);
        }

        let result = match property_type {
            PropertyType::Atom(atom) if atom.atom_id() != actual_type_return => {
                // wrong type

                let data: &[u8] = unsafe {
                    slice::from_raw_parts(prop_return, bytes_after_return as usize)
                };

                let property_data = PropertyData::<u8>::from_data(data, Atom::from_raw(actual_type_return));

                let data_format = match actual_format_return {
                    8 => PropertyDataFormat::Char,
                    16 => PropertyDataFormat::Short,
                    32 => PropertyDataFormat::Long,
                    format => {
                        return Err(PropertyError::UnknownDataFormat(format))
                    }
                };

                Err(PropertyError::WrongType(property_data, data_format))
            },
            PropertyType::Atom(_) | PropertyType::AnyPropertyType => {
                // successful property request

                let property_type_atom = Atom::from_raw(actual_type_return);
                let property_data = match actual_format_return {
                    8 => {
                        let data: &[u8] = unsafe {
                            slice::from_raw_parts(prop_return, nitems_return as usize)
                        };

                        Property::Char(PropertyData::<u8>::from_data(data, property_type_atom))
                    }
                    16 => {
                        let data: &[u16] = unsafe {
                            slice::from_raw_parts(prop_return as *const u16, nitems_return as usize)
                        };

                        Property::Short(PropertyData::<u16>::from_data(data, property_type_atom))
                    }
                    32 => {
                        let data: &[u32] = unsafe {
                            slice::from_raw_parts(prop_return as *const u32, nitems_return as usize)
                        };

                        Property::Long(PropertyData::<u32>::from_data(data, property_type_atom))
                    }
                    format => {
                        return Err(PropertyError::UnknownDataFormat(format));
                    }
                };

                Ok(property_data)
            }
        };

        unsafe {
            xlib::XFree(prop_return as *mut c_void);
        }

        result
    }

    fn list_properties(&self) -> AtomList {
        let mut atom_list = AtomList::new();

        let mut num_prop = 0;

        let xlib_atom_list: *mut xlib::Atom = unsafe {
            xlib::XListProperties(self.raw_display(), self.window_id(), &mut num_prop)
        };

        if xlib_atom_list.is_null() {
            return atom_list;
        }

        if num_prop < 0 {
            eprintln!("x11_wrapper warning: property count is negative, returning empty AtomList");
            return atom_list;
        }

        // TODO: Check that c_int fits in usize.
        // TODO: Check that atom in atom_slice is valid atom or trust Xlib?

        let atom_slice: &[xlib::Atom] = unsafe {
            slice::from_raw_parts(xlib_atom_list, num_prop as usize)
        };

        for atom in atom_slice {
            atom_list.add(Atom::from_raw(*atom));
        }

        drop(atom_slice);

        unsafe {
            xlib::XFree(xlib_atom_list as *mut c_void);
        }

        atom_list
    }

    fn delete_property(&self, property_name: Atom) {
        unsafe {
            xlib::XDeleteProperty(self.raw_display(), self.window_id(), property_name.atom_id());
        }
    }
/*
    fn change_property(
        &self,
        property_name: Atom,
        property_type: Atom,
        property_data_format: PropertyDataFormat,
        mode: ChangePropertyMode,
    ) {

    }

    */
}

#[derive(Debug)]
pub enum PropertyError {
    DoesNotExist,
    /// Property's real type did not match, but here is property's data in bytes.
    WrongType(PropertyData<u8>, PropertyDataFormat),
    /// Xlib function call failed.
    FunctionFailed,
    /// Xlib did not allocate data for property.
    PropertyDataHandleNull,
    UnknownDataFormat(c_int),
}


#[derive(Debug, Clone, Copy)]
pub enum PropertyDataFormat {
    /// 8 bits
    Char,
    /// 16 bits
    Short,
    /// 32 bits
    Long,
}

#[derive(Debug, Clone, Copy)]
pub enum ChangePropertyMode {
    Replace,
    Prepend,
    Append,
}

impl ChangePropertyMode {
    fn to_xlib_function_parameter(self) -> c_int {
        match self {
            ChangePropertyMode::Replace => xlib::PropModeReplace,
            ChangePropertyMode::Prepend => xlib::PropModePrepend,
            ChangePropertyMode::Append => xlib::PropModeAppend,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PropertyType {
    Atom(Atom),
    AnyPropertyType,
}

impl PropertyType {
    fn to_xlib_property_function_parameter(self) -> xlib::Atom {
        match self {
            PropertyType::Atom(atom) => atom.atom_id(),
            PropertyType::AnyPropertyType => xlib::AnyPropertyType as xlib::Atom,
        }
    }
}

pub(crate) fn to_xlib_bool(value: bool) -> xlib::Bool {
    if value {
        xlib::True
    } else {
        xlib::False
    }
}
