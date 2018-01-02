
use std::os::raw::{c_int, c_ulong, c_uint, c_long};
use std::sync::Arc;

use x11::xlib;

use display::DisplayHandle;
use color::{CreatedColormap, ColormapID};
use visual::Visual;
use event::EventMask;
use screen::Screen;
use utils::Text;

const ERROR_TOP_LEVEL_WINDOW: &'static str = "window is not top level window";

pub struct WindowBuilder {
    display_handle: Arc<DisplayHandle>,
    attributes: xlib::XSetWindowAttributes,
    colormap_and_visual: Option<(CreatedColormap, Visual)>,
    parent_window_id: xlib::Window,
    x: c_int,
    y: c_int,
    width: c_uint,
    height: c_uint,
    top_level_window: bool,
}

impl WindowBuilder {
    /// Returns error if parent window id is zero.
    pub(crate) fn new(display_handle: Arc<DisplayHandle>, parent_window_id: xlib::Window, top_level_window: bool) -> Result<Self, ()> {

        if parent_window_id == 0 {
            return Err(());
        }

        // default attributes
        let attributes = xlib::XSetWindowAttributes {
            background_pixmap: 0,
            background_pixel: 0, // default undefined
            border_pixmap: xlib::CopyFromParent as xlib::Pixmap,
            border_pixel: 0, // default undefined
            bit_gravity: xlib::ForgetGravity,
            win_gravity: xlib::NorthWestGravity,
            backing_store: xlib::NotUseful,
            backing_planes: c_ulong::max_value(),
            backing_pixel: 0,
            save_under: xlib::False,
            event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: xlib::False,
            colormap: xlib::CopyFromParent as xlib::Colormap,
            cursor: 0,
        };

        Ok(Self {
            attributes,
            display_handle,
            colormap_and_visual: None,
            parent_window_id,
            x: 0,
            y: 0,
            width: 640,
            height: 480,
            top_level_window,
        })
    }

    pub(crate) fn set_colormap_and_visual(mut self, colormap: CreatedColormap, visual: Visual) -> Self {
        self.attributes.colormap = colormap.id();
        self.colormap_and_visual = Some((colormap, visual));
        self
    }

    /// Default values: x = 0, y = 0
    pub fn set_x_y(mut self, x: c_int, y: c_int) -> Self {
        self.x = x;
        self.y = y;

        self
    }

    /// Default values: width = 640, height = 480
    ///
    /// Panics if width or height is zero.
    pub fn set_width_height(mut self, width: c_uint, height: c_uint) -> Self {
        if width == 0 {
            panic!("WindowBuilder width is zero");
        }

        if height == 0 {
            panic!("WindowBuilder height is zero");
        }

        self.width = width;
        self.height = height;

        self
    }


    pub fn build_input_output_window(mut self) -> Result<InputOutputWindow, ()> {
        let (valuemask, visual, depth, colormap) = if let Some((colormap, visual)) = self.colormap_and_visual {
            (xlib::CWColormap, visual.raw_visual(), visual.depth(), Some(colormap))
        } else {
            (0, xlib::CopyFromParent as *mut xlib::Visual, xlib::CopyFromParent, None)
        };

        let window_id = unsafe {
            xlib::XCreateWindow(
                self.display_handle.raw_display(),
                self.parent_window_id,
                self.x,
                self.y,
                self.width,
                self.height,
                0,
                depth,
                xlib::InputOutput as c_uint,
                visual,
                valuemask,
                &mut self.attributes,
            )
        };

        if window_id == 0 {
            Err(())
        } else {
            Ok(InputOutputWindow {
                display_handle: self.display_handle,
                colormap,
                window_id,
                top_level_window: self.top_level_window,
            })
        }
    }
}

pub struct InputOutputWindow {
    display_handle: Arc<DisplayHandle>,
    colormap: Option<CreatedColormap>,
    window_id: xlib::Window,
    top_level_window: bool,
}

impl InputOutputWindow {
    pub fn map_window(&mut self) {
        // TODO: check errors

        unsafe {
            xlib::XMapWindow(self.display_handle.raw_display(), self.window_id);
        }
    }

    pub fn select_input(&mut self, event_mask: EventMask) {
        // TODO: check errors

        unsafe {
            xlib::XSelectInput(self.display_handle.raw_display(), self.window_id, event_mask.bits());
        }
    }

    pub fn map_raised(&mut self) {
        // TODO: can generate BadWindow errors
        unsafe {
            xlib::XMapRaised(self.display_handle.raw_display(), self.window_id);
        }
    }

    pub fn set_stack_mode(&mut self, stack_mode: StackMode) {
        self.set_sibling_and_stack_mode::<InputOutputWindow>(None, stack_mode);
    }

    /// If sibling is `None`, sibling configuration option is not set.
    /// If sibling is `Some(sibling)`, the window in sibling argument must
    /// really be a sibling window or BadMatch error is generated.
    pub fn set_sibling_and_stack_mode<W: WindowID>(&mut self, sibling: Option<&W>, stack_mode: StackMode) {
        let (sibling, configure_mask) = if let Some(w) = sibling {
            (w.window_id(), xlib::CWSibling | xlib::CWStackMode)
        } else {
            (0, xlib::CWStackMode)
        };

        let mut changes = xlib::XWindowChanges {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            border_width: 0,
            sibling,
            stack_mode: stack_mode as c_int,
        };

        unsafe {
            xlib::XConfigureWindow(
                self.display_handle.raw_display(),
                self.window_id(),
                configure_mask as c_uint,
                &mut changes
            );
        }
    }


    pub fn lower(&mut self) {
        unsafe {
            xlib::XLowerWindow(self.display_handle.raw_display(), self.window_id);
        }
    }

    pub fn raise(&mut self) {
        unsafe {
            xlib::XRaiseWindow(self.display_handle.raw_display(), self.window_id);
        }
    }

    pub fn circulate_subwindows_up(&mut self) {
        unsafe {
            xlib::XCirculateSubwindowsUp(self.display_handle.raw_display(), self.window_id);
        }
    }

    pub fn circulate_subwindows_down(&mut self) {
        unsafe {
            xlib::XCirculateSubwindowsDown(self.display_handle.raw_display(), self.window_id);
        }
    }

    /// Panics if window is not top level window.
    pub fn iconify(&mut self, screen: &Screen) -> Result<(), ()> {
        if self.top_level_window {
            unsafe {
                let status = xlib::XIconifyWindow(self.display_handle.raw_display(), self.window_id, screen.screen_number());

                if status == 0 {
                    Err(())
                } else {
                    Ok(())
                }
            }
        } else {
            panic!(ERROR_TOP_LEVEL_WINDOW)
        }
    }

    /// Panics if window is not top level window.
    pub fn withdraw(&mut self, screen: &Screen) -> Result<(), ()> {
        if self.top_level_window {
            unsafe {
                let status = xlib::XWithdrawWindow(self.display_handle.raw_display(), self.window_id, screen.screen_number());

                if status == 0 {
                    Err(())
                } else {
                    Ok(())
                }
            }
        } else {
            panic!(ERROR_TOP_LEVEL_WINDOW)
        }
    }

    /// Panics if window is not top level window.
    pub fn set_stack_mode_top_level_window(&mut self, screen: &Screen, stack_mode: StackMode) -> Result<(), ()> {
        self.set_sibling_and_stack_mode_top_level_window::<InputOutputWindow>(screen, None, stack_mode)
    }

    /// Panics if window is not top level window.
    ///
    /// If sibling is `None`, sibling configuration option is not set.
    /// If sibling is `Some(sibling)`, the window in sibling argument must
    /// really be a sibling window or BadMatch error is generated.
    pub fn set_sibling_and_stack_mode_top_level_window<W: WindowID>(&mut self, screen: &Screen, sibling: Option<&W>, stack_mode: StackMode) -> Result<(), ()>{
        if !self.top_level_window {
            panic!(ERROR_TOP_LEVEL_WINDOW)
        }

        let (sibling, configure_mask) = if let Some(w) = sibling {
            (w.window_id(), xlib::CWSibling | xlib::CWStackMode)
        } else {
            (0, xlib::CWStackMode)
        };

        let mut changes = xlib::XWindowChanges {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            border_width: 0,
            sibling,
            stack_mode: stack_mode as c_int,
        };

        unsafe {
            let status = xlib::XReconfigureWMWindow(
                self.display_handle.raw_display(),
                self.window_id(),
                screen.screen_number(),
                configure_mask as c_uint,
                &mut changes
            );

            if status == 0 {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    pub fn set_window_name(&mut self, mut text: Text) {
        unsafe {
            xlib::XSetWMName(self.display_handle.raw_display(), self.window_id(), text.raw_text_property());
        }
    }
}

impl Drop for InputOutputWindow {
    fn drop(&mut self) {
        unsafe {
            // TODO: check errors
            xlib::XDestroyWindow(self.display_handle.raw_display(), self.window_id);
        }
    }
}

impl WindowID for InputOutputWindow {
    fn window_id(&self) -> xlib::Window {
        self.window_id
    }
}

pub trait WindowID {
    fn window_id(&self) -> xlib::Window;
}

#[repr(i16)]
pub enum StackMode {
    Above = xlib::Above as i16,
    Below = xlib::Below as i16,
    TopIf = xlib::TopIf as i16,
    BottomIf = xlib::BottomIf as i16,
    Opposite = xlib::Opposite as i16,
}