//! InputOutput windows.

use std::os::raw::{c_int, c_uint};

use x11::xlib;

use super::attribute::*;
use super::{Selection, Window, WindowProperties};

use core::display::X11Display;
use core::color::{ColormapID, CreatedColormap};
use core::visual::Visual;
use core::screen::Screen;
use core::XlibHandle;

pub struct BuildTopLevelWindow;

#[derive(Debug)]
pub struct InputOutputWindowBuilder<T> {
    display_handle: X11Display,
    attributes: WindowAttributes,
    colormap_and_visual: Option<(CreatedColormap, Visual)>,
    parent_window_id: xlib::Window,
    x: c_int,
    y: c_int,
    width: c_uint,
    height: c_uint,
    builder: T,
}

impl<T> InputOutputWindowBuilder<T> {
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

impl<T> GetAndSetAttributes for InputOutputWindowBuilder<T> {
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
    InputOutputWindowAttributes,
    CommonAttributes
);

impl InputOutputWindowBuilder<BuildTopLevelWindow> {
    /// Parent of created window will be root window of `Screen`.
    ///
    /// Returns error if `Screen` does not support `Visual` or `Screen`'s root window
    /// is not found.
    ///
    /// XCreateColormap - BadAlloc, BadMatch, BadValue, BadWindow
    pub fn new<T: Into<WindowVisual>>(screen: &Screen, window_visual: T) -> Result<Self, ()> {
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

        if let WindowVisual::Visual(visual) = window_visual.into() {
            let created_colormap =
                CreatedColormap::create(screen.display_handle().clone(), screen, &visual)?;

            builder = builder.set_colormap(Colormap::Colormap(created_colormap.id()));
            builder.colormap_and_visual = Some((created_colormap, visual));
        }

        Ok(builder)
    }

    /// XCreateWindow
    pub fn build_input_output_window(mut self) -> Result<TopLevelInputOutputWindow, ()> {
        let (visual, depth, colormap) = if let Some((colormap, visual)) = self.colormap_and_visual {
            (visual.raw_visual(), visual.depth(), Some(colormap))
        } else {
            (
                xlib::CopyFromParent as *mut xlib::Visual,
                xlib::CopyFromParent,
                None,
            )
        };

        let window_id = unsafe {
            xlib_function!(
                self.display_handle.xlib_handle(),
                XCreateWindow(
                    Some(self.display_handle.raw_display()),
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
                    self.attributes.xlib_attributes_mut_ptr()
                )
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
    display_handle: X11Display,
    colormap: Option<CreatedColormap>,
    window_id: xlib::Window,
    attributes: WindowAttributes,
}

impl TopLevelInputOutputWindow {
    /// XMapWindow
    pub fn map_window(self) -> Self {
        // TODO: check errors

        unsafe {
            xlib_function!(
                self.xlib_handle(),
                XMapWindow(Some(self.display_handle.raw_display()), self.window_id)
            );
        }

        self
    }

    /// XUnmapWindow
    pub fn unmap_window(self) -> Self {
        unsafe {
            xlib_function!(
                self.xlib_handle(),
                XUnmapWindow(Some(self.display_handle.raw_display()), self.window_id)
            );
        }

        self
    }

    /// XIconifyWindow
    pub fn iconify(&mut self, screen: &Screen) -> Result<(), ()> {
        unsafe {
            let status = xlib_function!(
                self.xlib_handle(),
                XIconifyWindow(
                    Some(self.display_handle.raw_display()),
                    self.window_id,
                    screen.screen_number()
                )
            );

            if status == 0 {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    /// XWithdrawWindow
    pub fn withdraw(&mut self, screen: &Screen) -> Result<(), ()> {
        unsafe {
            let status = xlib_function!(
                self.xlib_handle(),
                XWithdrawWindow(
                    Some(self.display_handle.raw_display()),
                    self.window_id,
                    screen.screen_number()
                )
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
    /// XDestroyWindow - BadWindow
    fn drop(&mut self) {
        unsafe {
            // TODO: check errors
            xlib_function!(
                self.xlib_handle(),
                XDestroyWindow(Some(self.display_handle.raw_display()), self.window_id)
            );
        }
    }
}

impl WindowProperties for TopLevelInputOutputWindow {}
impl Selection for TopLevelInputOutputWindow {}

/*
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
    InputOutputWindowAttributes,
    CommonAttributes
);
*/

impl Window for TopLevelInputOutputWindow {
    fn xlib_handle(&self) -> &XlibHandle {
        self.display_handle.xlib_handle()
    }

    fn raw_display(&self) -> *mut xlib::Display {
        self.display_handle.raw_display()
    }

    fn window_id(&self) -> xlib::Window {
        self.window_id
    }
}

pub enum WindowVisual {
    Visual(Visual),
    CopyFromParent,
}

impl From<Visual> for WindowVisual {
    fn from(visual: Visual) -> Self {
        WindowVisual::Visual(visual)
    }
}
