
//! Inter-Client Communication Conventions Manual 2.0 properties
//!
//! [ICCCM 2.0 documentation](https://www.x.org/releases/X11R7.7/doc/xorg-docs/icccm/icccm.html)

use std::os::raw::{c_int, c_void, c_long};
use std::marker::PhantomData;
use std::ffi::CString;

use x11::xlib;

use core::window::input_output::TopLevelInputOutputWindow;
use core::window::{Window, WindowProperties, PropertyData, Property, ChangePropertyMode};
use core::utils::{AtomList, Atom, to_xlib_bool};
use core::XlibHandle;

impl TopLevelInputOutputWindow {
    /// Set `WM_HINTS` property.
    ///
    /// Returns error if there is no enough memory to
    /// allocate `xlib::XWMHints` structure.
    pub fn start_configuring_hints(self) -> Result<HintsConfigurator, Self> {
        HintsConfigurator::new(self)
    }

    /// Set `WM_NORMAL_HINTS` property.
    ///
    /// Returns error if there is no enough memory to
    /// allocate `xlib::XSizeHints` structure.
    pub fn start_configuring_normal_hints(self) -> Result<NormalHintsConfigurator, Self> {
        NormalHintsConfigurator::new(self)
    }

    /// Set `WM_PROTOCOLS` property.
    pub fn set_protocols(self, mut atom_list: AtomList) -> Result<Self, Self> {
        let status = unsafe {
            xlib_function!(
                self.xlib_handle(),
                XSetWMProtocols(
                    self.raw_display(),
                    self.window_id(),
                    atom_list.as_mut_ptr(),
                    atom_list.len()
                )
            )
        };

        if status == 0 {
            Err(self)
        } else {
            Ok(self)
        }
    }

    /// Set `WM_TRANSIENT_FOR` property.
    pub fn set_transient_for_hint(self, window_id: xlib::Window) -> Self {
        unsafe {
            xlib_function!(self.xlib_handle(), XSetTransientForHint(self.raw_display(), self.window_id(), window_id));
        }

        self
    }

    /// Set `WM_CLASS` property.
    ///
    /// Allowed characters [a-zA-Z0-9 _-]
    ///
    /// TODO: support all STRING type characters without control characters
    ///
    /// Note that `instance_name` and `class_name` arguments' length is limited
    /// by property data size constraints. See documentation of
    /// `WindowProperty::change_property` for more information.
    pub fn set_class(self, instance_name: String, class_name: String) -> Result<Self, SetClassPropertyError<Self>> {

        fn check_string(text: &str) -> Result<(),()> {
            for c in text.chars() {
                match c {
                     'a'...'z' | 'A'...'Z' | '0'...'9' |
                     ' ' | '-' | '_' => (),
                    _ => return Err(()),
                }
            }
            Ok(())
        }

        if check_string(&instance_name).is_err() ||
            check_string(&class_name).is_err() {
                return Err(SetClassPropertyError::UnknownCharacter(self))
        }

        // There shouldn't be any null bytes because
        // check_string does not allow them.
        let c_instance_name = CString::new(instance_name).unwrap();
        let c_class_name = CString::new(class_name).unwrap();

        let mut property_data = PropertyData::<u8>::new(
            Atom::from_raw(xlib::XA_WM_CLASS),
            Atom::from_raw(xlib::XA_STRING),
        );

        property_data.data_mut().extend_from_slice(c_instance_name.as_bytes_with_nul());
        property_data.data_mut().extend_from_slice(c_class_name.as_bytes_with_nul());

        let property = Property::Char(property_data);

        match self.change_property(property, ChangePropertyMode::Replace) {
            Ok(()) => Ok(self),
            Err(()) => Err(SetClassPropertyError::ChangePropertyError(self)),
        }
    }

    /// Set `WM_ICON_SIZE` property.
    pub fn set_icon_size(self,
        min_width: i32,
        min_height: i32,
        max_width: i32,
        max_height: i32,
        width_inc: i32,
        height_inc: i32,
    ) -> Self {
        let mut property_data = PropertyData::<u32>::new(
            Atom::from_raw(xlib::XA_WM_ICON_SIZE),
            Atom::from_raw(xlib::XA_WM_ICON_SIZE),
        );
        {
            let data = property_data.data_mut();
            data.push(min_width as u32);
            data.push(min_height as u32);
            data.push(max_width as u32);
            data.push(max_height as u32);
            data.push(width_inc as u32);
            data.push(height_inc as u32);
        }

        let property = Property::Long(property_data);

        // This should not panic because change_property function
        // returns error only if there is too many data items in property.
        self.change_property(property, ChangePropertyMode::Replace).unwrap();

        self
    }
}

#[derive(Debug)]
pub enum SetClassPropertyError<T> {
    UnknownCharacter(T),
    /// See documentation for `WindowProperty::change_property`.
    ChangePropertyError(T),
}

#[derive(Debug, Copy, Clone)]
pub enum TextProperty {
    /// `WM_NAME`
    Name,

    /// `WM_ICON_NAME`
    IconName,

    /// `WM_COMMAND`
    Command,

    /// `WM_CLIENT_MACHINE`
    ClientMachine,
}

impl From<TextProperty> for Atom {
    fn from(property: TextProperty) -> Self {
        match property {
            TextProperty::ClientMachine => Atom::from_raw(xlib::XA_WM_CLIENT_MACHINE),
            TextProperty::IconName => Atom::from_raw(xlib::XA_WM_ICON_NAME),
            TextProperty::Command => Atom::from_raw(xlib::XA_WM_COMMAND),
            TextProperty::Name => Atom::from_raw(xlib::XA_WM_NAME),
        }
    }
}

/// Allocated `xlib::XWMHints` structure.
struct Hints {
    wm_hints_ptr: *mut xlib::XWMHints,
    _marker: PhantomData<xlib::XWMHints>,
    _xlib_handle: XlibHandle,
}

impl Hints {
    /// Returns error if there is no enough memory to
    /// allocate `xlib::XWMHints` structure.
    fn new(xlib_handle: &XlibHandle) -> Result<Self, ()> {
        let wm_hints_ptr = unsafe { xlib_function!(xlib_handle, XAllocWMHints()) };

        if wm_hints_ptr.is_null() {
            Err(())
        } else {
            Ok(Self {
                wm_hints_ptr,
                _marker: PhantomData,
                _xlib_handle: xlib_handle.clone(),
            })
        }
    }

    fn as_mut_ptr(&mut self) -> *mut xlib::XWMHints {
        self.wm_hints_ptr
    }
}

impl Drop for Hints {
    fn drop(&mut self) {
        unsafe {
            xlib_function!(self._xlib_handle, XFree(self.wm_hints_ptr as *mut c_void));
        }
    }
}

bitflags! {
    struct WindowHintsFlags: c_long {
        const INPUT = xlib::InputHint;
        const STATE = xlib::StateHint;
        const ICON_PIXMAP = xlib::IconPixmapHint;
        const ICON_WINDOW = xlib::IconWindowHint;
        const ICON_POSITION = xlib::IconPositionHint;
        const ICON_MASK = xlib::IconMaskHint;
        const WINDOW_GROUP = xlib::WindowGroupHint;
        const URGENCY = xlib::XUrgencyHint;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum WindowState {
    Withdrawn = 0,
    Normal = 1,
    Iconic = 2,
}

/// Sets `TopLevelInputOutputWindow`'s `WM_HINTS` property.
pub struct HintsConfigurator {
    window: TopLevelInputOutputWindow,
    hints: Hints,
    window_hints_flags: WindowHintsFlags,
}

impl HintsConfigurator {
    /// Returns error if there is no enough memory to
    /// allocate `xlib::XWMHints` structure.
    fn new(window: TopLevelInputOutputWindow) -> Result<Self, TopLevelInputOutputWindow> {
        let hints = match Hints::new(window.xlib_handle()) {
            Ok(hints) => hints,
            Err(()) => return Err(window),
        };

        Ok(
            Self {
                window,
                hints,
                window_hints_flags: WindowHintsFlags::empty(),
            }
        )
    }

    pub fn set_input(mut self, value: bool) -> Self {
        let xlib_bool = to_xlib_bool(value);
        unsafe {
            (*self.hints.wm_hints_ptr).input = xlib_bool;
        }
        self.window_hints_flags |= WindowHintsFlags::INPUT;
        self
    }

    pub fn set_initial_state(mut self, value: WindowState) -> Self {
        unsafe {
            (*self.hints.wm_hints_ptr).initial_state = value as c_int;
        }
        self.window_hints_flags |= WindowHintsFlags::STATE;

        self
    }

    pub fn set_icon_pixmap(mut self, pixmap_id: xlib::Pixmap) -> Self {
        unsafe {
            (*self.hints.wm_hints_ptr).icon_pixmap = pixmap_id;
        }
        self.window_hints_flags |= WindowHintsFlags::ICON_PIXMAP;

        self
    }


    pub fn set_icon_window(mut self, window_id: xlib::Window) -> Self {
        unsafe {
            (*self.hints.wm_hints_ptr).icon_window = window_id;
        }
        self.window_hints_flags |= WindowHintsFlags::ICON_WINDOW;

        self
    }


    pub fn set_icon_position(mut self, x: c_int, y: c_int) -> Self {
        unsafe {
            (*self.hints.wm_hints_ptr).icon_x = x;
            (*self.hints.wm_hints_ptr).icon_y = y;
        }
        self.window_hints_flags |= WindowHintsFlags::ICON_POSITION;

        self
    }

    pub fn set_icon_mask(mut self, pixmap_id: xlib::Pixmap) -> Self {
        unsafe {
            (*self.hints.wm_hints_ptr).icon_mask = pixmap_id;
        }
        self.window_hints_flags |= WindowHintsFlags::ICON_MASK;

        self
    }

    pub fn set_window_group(mut self, window_group_id: xlib::XID) -> Self {
        unsafe {
            (*self.hints.wm_hints_ptr).window_group = window_group_id;
        }
        self.window_hints_flags |= WindowHintsFlags::WINDOW_GROUP;

        self
    }

    pub fn set_urgency(mut self, value: bool) -> Self {
        if value {
            self.window_hints_flags |= WindowHintsFlags::URGENCY;
        } else {
            self.window_hints_flags -= WindowHintsFlags::URGENCY;
        }

        self
    }

    pub fn end(mut self) -> TopLevelInputOutputWindow {
        unsafe {
             (*self.hints.wm_hints_ptr).flags = self.window_hints_flags.bits();

            xlib_function!(
                self.window.xlib_handle(),
                XSetWMHints(
                    self.window.raw_display(),
                    self.window.window_id(),
                    self.hints.as_mut_ptr()
                )
            );
        }

        self.window
    }
}

/// Allocated `xlib::XSizeHints` structure.
struct SizeHints {
    size_hints_ptr: *mut xlib::XSizeHints,
    _marker: PhantomData<xlib::XSizeHints>,
    _xlib_handle: XlibHandle,
}

impl SizeHints {
    /// Returns error if there is no enough memory to
    /// allocate `xlib::XSizeHints` structure.
    fn new(xlib_handle: &XlibHandle) -> Result<Self, ()> {
        let size_hints_ptr = unsafe { xlib_function!(xlib_handle, XAllocSizeHints()) };

        if size_hints_ptr.is_null() {
            Err(())
        } else {
            Ok(Self {
                size_hints_ptr,
                _marker: PhantomData,
                _xlib_handle: xlib_handle.clone(),
            })
        }
    }

    fn as_mut_ptr(&mut self) -> *mut xlib::XSizeHints {
        self.size_hints_ptr
    }
}

impl Drop for SizeHints {
    fn drop(&mut self) {
        unsafe {
            xlib_function!(self._xlib_handle, XFree(self.size_hints_ptr as *mut c_void));
        }
    }
}

/// Sets `TopLevelInputOutputWindow`'s `WM_NORMAL_HINTS` property.
pub struct NormalHintsConfigurator {
    window: TopLevelInputOutputWindow,
    size_hints: SizeHints,
}

impl NormalHintsConfigurator {
    /// Returns error if there is no enough memory to
    /// allocate `xlib::XSizeHints` structure.
    fn new(window: TopLevelInputOutputWindow) -> Result<Self, TopLevelInputOutputWindow> {
        let size_hints = match SizeHints::new(window.xlib_handle()) {
            Ok(hints) => hints,
            Err(()) => return Err(window),
        };

        Ok(Self { window, size_hints })
    }

    pub fn set_max_window_size(mut self, width: c_int, height: c_int) -> Self {
        unsafe {
            (*self.size_hints.as_mut_ptr()).flags |= xlib::PMaxSize;
            (*self.size_hints.as_mut_ptr()).max_width = width;
            (*self.size_hints.as_mut_ptr()).max_height = height;
        }

        self
    }

    pub fn set_min_window_size(mut self, width: c_int, height: c_int) -> Self {
        unsafe {
            (*self.size_hints.as_mut_ptr()).flags |= xlib::PMinSize;
            (*self.size_hints.as_mut_ptr()).min_width = width;
            (*self.size_hints.as_mut_ptr()).min_height = height;
        }

        self
    }

    pub fn set_resize_increments(mut self, width: c_int, height: c_int) -> Self {
        unsafe {
            (*self.size_hints.as_mut_ptr()).flags |= xlib::PResizeInc;
            (*self.size_hints.as_mut_ptr()).width_inc = width;
            (*self.size_hints.as_mut_ptr()).height_inc = height;
        }

        self
    }

    pub fn set_min_and_max_aspect_ratios(
        mut self,
        min_numerator: c_int,
        min_denominator: c_int,
        max_numerator: c_int,
        max_denominator: c_int,
    ) -> Self {
        unsafe {
            (*self.size_hints.as_mut_ptr()).flags |= xlib::PAspect;

            (*self.size_hints.as_mut_ptr()).min_aspect.x = min_numerator;
            (*self.size_hints.as_mut_ptr()).min_aspect.y = min_denominator;

            (*self.size_hints.as_mut_ptr()).max_aspect.x = max_numerator;
            (*self.size_hints.as_mut_ptr()).max_aspect.y = max_denominator;
        }

        self
    }

    pub fn set_base_size(mut self, width: c_int, height: c_int) -> Self {
        unsafe {
            (*self.size_hints.as_mut_ptr()).flags |= xlib::PBaseSize;
            (*self.size_hints.as_mut_ptr()).base_width = width;
            (*self.size_hints.as_mut_ptr()).base_height = height;
        }

        self
    }

    pub fn set_win_gravity(mut self, win_gravity: c_int) -> Self {
        unsafe {
            (*self.size_hints.as_mut_ptr()).flags |= xlib::PWinGravity;
            (*self.size_hints.as_mut_ptr()).win_gravity = win_gravity;
        }

        self
    }

    pub fn end(mut self) -> TopLevelInputOutputWindow {
        unsafe {
            xlib_function!(
                self.window.xlib_handle(),
                XSetWMNormalHints(
                    self.window.raw_display(),
                    self.window.window_id(),
                    self.size_hints.as_mut_ptr()
                )
            );
        }

        self.window
    }
}
