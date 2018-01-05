
//! Inter-Client Communication Conventions Manual 2.0 properties

use std::os::raw::{c_int, c_void};
use std::marker::PhantomData;

use x11::xlib;

use core::window::input_output::TopLevelInputOutputWindow;
use core::window::Window;
use core::utils::{Text, AtomList};

impl TopLevelInputOutputWindow {
    /// Set `WM_NAME` property.
    pub fn set_window_name(self, mut text: Text) -> Self {
        unsafe {
            xlib::XSetWMName(
                self.raw_display(),
                self.window_id(),
                text.raw_text_property(),
            );
        }
        self
    }

    /// Set `WM_ICON_NAME` property.
    pub fn set_window_icon_name(self, mut text: Text) -> Self {
        unsafe {
            xlib::XSetWMIconName(
                self.raw_display(),
                self.window_id(),
                text.raw_text_property(),
            );
        }

        self
    }

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
            xlib::XSetWMProtocols(
                self.raw_display(),
                self.window_id(),
                atom_list.as_mut_ptr(),
                atom_list.len() as c_int,
            )
        };

        if status == 0 {
            Err(self)
        } else {
            Ok(self)
        }
    }
}


/// Allocated `xlib::XWMHints` structure.
struct Hints {
    wm_hints_ptr: *mut xlib::XWMHints,
    _marker: PhantomData<xlib::XWMHints>,
}

impl Hints {
    /// Returns error if there is no enough memory to
    /// allocate `xlib::XWMHints` structure.
    fn new() -> Result<Self, ()> {
        let wm_hints_ptr = unsafe { xlib::XAllocWMHints() };

        if wm_hints_ptr.is_null() {
            Err(())
        } else {
            Ok(Self {
                wm_hints_ptr,
                _marker: PhantomData,
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
            xlib::XFree(self.wm_hints_ptr as *mut c_void);
        }
    }
}

/// Sets `TopLevelInputOutputWindow`'s `WM_HINTS` property.
pub struct HintsConfigurator {
    window: TopLevelInputOutputWindow,
    hints: Hints,
}

impl HintsConfigurator {
    /// Returns error if there is no enough memory to
    /// allocate `xlib::XWMHints` structure.
    fn new(window: TopLevelInputOutputWindow) -> Result<Self, TopLevelInputOutputWindow> {
        let hints = match Hints::new() {
            Ok(hints) => hints,
            Err(()) => return Err(window),
        };

        Ok(Self { window, hints })
    }

    pub fn end(mut self) -> TopLevelInputOutputWindow {
        unsafe {
            xlib::XSetWMHints(
                self.window.raw_display(),
                self.window.window_id(),
                self.hints.as_mut_ptr(),
            );
        }

        self.window
    }
}

/// Allocated `xlib::XSizeHints` structure.
struct SizeHints {
    size_hints_ptr: *mut xlib::XSizeHints,
    _marker: PhantomData<xlib::XSizeHints>,
}

impl SizeHints {
    /// Returns error if there is no enough memory to
    /// allocate `xlib::XSizeHints` structure.
    fn new() -> Result<Self, ()> {
        let size_hints_ptr = unsafe { xlib::XAllocSizeHints() };

        if size_hints_ptr.is_null() {
            Err(())
        } else {
            Ok(Self {
                size_hints_ptr,
                _marker: PhantomData,
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
            xlib::XFree(self.size_hints_ptr as *mut c_void);
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
        let size_hints = match SizeHints::new() {
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

    pub fn end(mut self) -> TopLevelInputOutputWindow {
        unsafe {
            xlib::XSetWMNormalHints(
                self.window.raw_display(),
                self.window.window_id(),
                self.size_hints.as_mut_ptr(),
            );
        }

        self.window
    }
}
