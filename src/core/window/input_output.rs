//! InputOutput windows.

use std::os::raw::{c_int, c_uint, c_void};
use std::sync::Arc;

use x11::xlib;

use super::attribute::*;
use super::Window;

use core::display::{DisplayHandle};
use core::color::{ColormapID, CreatedColormap};
use core::visual::Visual;
use core::screen::Screen;
use core::utils::{AtomList, Text};

pub struct BuildTopLevelWindow;

#[derive(Debug)]
pub struct InputOutputWindowBuilder<T> {
    display_handle: Arc<DisplayHandle>,
    attributes: WindowAttributes,
    colormap_and_visual: Option<(CreatedColormap, Visual)>,
    parent_window_id: xlib::Window,
    x: c_int,
    y: c_int,
    width: c_uint,
    height: c_uint,
    builder: T,
}

impl <T> InputOutputWindowBuilder<T> {
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
}

impl <T> GetAndSetAttributes for InputOutputWindowBuilder<T> {
    fn attributes(&self) -> &WindowAttributes {
        &self.attributes
    }
    fn attributes_mut(&mut self) -> &mut WindowAttributes {
        &mut self.attributes
    }
}

macro_rules! impl_traits {
    ( InputOutputWindowBuilder<T>, $( $trait:ty ),+) => {
        $(
            impl <T> $trait for InputOutputWindowBuilder<T> {}
        )+
    };
    ( $type:ty, $( $trait:ty ),+) => {
        $(
            impl $trait for $type {}
        )+
    };
}

impl_traits!(
    InputOutputWindowBuilder<T>,
    AttributeBackgroundPixmap,
    AttributeBackgroundPixel,
    AttributeBorderPixmap,
    AttributeBorderPixel,
    AttributeGravity,
    AttributeWindowGravity,
    AttributeBackingStore,
    AttributeBackingPlanes,
    AttributeBackingPixel,
    AttributeSaveUnder,
    AttributeEventMask,
    AttributeDoNotPropagate,
    AttributeColormap,
    AttributeCursor
);

impl InputOutputWindowBuilder<BuildTopLevelWindow> {
    /// Parent of created window will be root window of `Screen`.
    ///
    /// Returns error if `Screen` does not support `Visual` or `Screen`'s root window
    /// is not found.
    pub fn new(
        screen: &Screen,
        window_visual: WindowVisual,
    ) -> Result<Self, ()> {
        let parent_window_id = screen.root_window_id().ok_or(())?;

        let mut builder = Self {
            attributes: WindowAttributes::default(),
            display_handle: screen.display_handle().clone(),
            colormap_and_visual: None,
            parent_window_id,
            x: 0,
            y: 0,
            width: 640,
            height: 480,
            builder: BuildTopLevelWindow,
        };

        if let WindowVisual::Visual(visual) = window_visual {
            let created_colormap = CreatedColormap::create(screen.display_handle().clone(), screen, &visual)?;

            builder.set_colormap(Colormap::Colormap(created_colormap.id()));
            builder.colormap_and_visual = Some((created_colormap, visual));
        }

        Ok(builder)
    }

    pub fn build_input_output_window(mut self) -> Result<TopLevelInputOutputWindow, ()> {
        let (visual, depth, colormap) =
            if let Some((colormap, visual)) = self.colormap_and_visual {
                (
                    visual.raw_visual(),
                    visual.depth(),
                    Some(colormap),
                )
            } else {
                (
                    xlib::CopyFromParent as *mut xlib::Visual,
                    xlib::CopyFromParent,
                    None,
                )
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
                self.attributes.selected_attributes().bits(),
                self.attributes.xlib_attributes_mut_ptr(),
            )
        };

        if window_id == 0 {
            Err(())
        } else {
            Ok(TopLevelInputOutputWindow {
                display_handle: self.display_handle,
                colormap,
                window_id,
                attributes: self.attributes,
            })
        }
    }
}

#[derive(Debug)]
pub struct TopLevelInputOutputWindow {
    display_handle: Arc<DisplayHandle>,
    colormap: Option<CreatedColormap>,
    window_id: xlib::Window,
    attributes: WindowAttributes,
}

impl TopLevelInputOutputWindow {
    pub fn map_window(&mut self) {
        // TODO: check errors

        unsafe {
            xlib::XMapWindow(self.display_handle.raw_display(), self.window_id);
        }
    }

    pub fn unmap_window(&mut self) {
        unsafe {
            xlib::XUnmapWindow(self.display_handle.raw_display(), self.window_id);
        }
    }


    pub fn set_stack_mode(&mut self, stack_mode: StackMode) {
        self.set_sibling_and_stack_mode::<TopLevelInputOutputWindow>(None, stack_mode);
    }

    /// If sibling is `None`, sibling configuration option is not set.
    /// If sibling is `Some(sibling)`, the window in sibling argument must
    /// really be a sibling window or BadMatch error is generated.
    pub fn set_sibling_and_stack_mode<W: Window>(
        &mut self,
        sibling: Option<&W>,
        stack_mode: StackMode,
    ) {
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
                &mut changes,
            );
        }
    }


    pub fn iconify(&mut self, screen: &Screen) -> Result<(), ()> {
        unsafe {
            let status = xlib::XIconifyWindow(
                self.display_handle.raw_display(),
                self.window_id,
                screen.screen_number(),
            );

            if status == 0 {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    pub fn withdraw(&mut self, screen: &Screen) -> Result<(), ()> {
        unsafe {
            let status = xlib::XWithdrawWindow(
                self.display_handle.raw_display(),
                self.window_id,
                screen.screen_number(),
            );

            if status == 0 {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    pub fn set_stack_mode_top_level_window(
        &mut self,
        screen: &Screen,
        stack_mode: StackMode,
    ) -> Result<(), ()> {
        self.set_sibling_and_stack_mode_top_level_window::<TopLevelInputOutputWindow>(
            screen,
            None,
            stack_mode,
        )
    }

    /// If sibling is `None`, sibling configuration option is not set.
    /// If sibling is `Some(sibling)`, the window in sibling argument must
    /// really be a sibling window or BadMatch error is generated.
    pub fn set_sibling_and_stack_mode_top_level_window<W: Window>(
        &mut self,
        screen: &Screen,
        sibling: Option<&W>,
        stack_mode: StackMode,
    ) -> Result<(), ()> {
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
                &mut changes,
            );

            if status == 0 {
                Err(())
            } else {
                Ok(())
            }
        }
    }
}

impl Drop for TopLevelInputOutputWindow {
    fn drop(&mut self) {
        unsafe {
            // TODO: check errors
            xlib::XDestroyWindow(self.display_handle.raw_display(), self.window_id);
        }
    }
}

impl GetAndSetAttributes for TopLevelInputOutputWindow {
    fn attributes(&self) -> &WindowAttributes {
        &self.attributes
    }
    fn attributes_mut(&mut self) -> &mut WindowAttributes {
        &mut self.attributes
    }
}

impl_traits!(
    TopLevelInputOutputWindow,
    AttributeBackgroundPixmap,
    AttributeBackgroundPixel,
    AttributeBorderPixmap,
    AttributeBorderPixel,
    AttributeGravity,
    AttributeWindowGravity,
    AttributeBackingStore,
    AttributeBackingPlanes,
    AttributeBackingPixel,
    AttributeSaveUnder,
    AttributeEventMask,
    AttributeDoNotPropagate,
    AttributeColormap,
    AttributeCursor
);

impl Window for TopLevelInputOutputWindow {
    fn raw_display(&self) -> *mut xlib::Display {
        self.display_handle.raw_display()
    }

    fn window_id(&self) -> xlib::Window {
        self.window_id
    }
}

#[repr(i16)]
pub enum StackMode {
    Above = xlib::Above as i16,
    Below = xlib::Below as i16,
    TopIf = xlib::TopIf as i16,
    BottomIf = xlib::BottomIf as i16,
    Opposite = xlib::Opposite as i16,
}

pub enum WindowVisual {
    Visual(Visual),
    CopyFromParent,
}