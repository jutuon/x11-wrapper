
//! Window attributes

use std::os::raw::{c_int, c_ulong, c_long};

use x11::xlib;

use core::event::EventMask;
use core::utils::{XLIB_NONE};

#[derive(Debug)]
pub struct WindowAttributes {
    attributes: xlib::XSetWindowAttributes,
    selected_attributes: AttributeMask,
}

impl WindowAttributes {
    pub(crate) fn selected_attributes(&self) -> AttributeMask {
        self.selected_attributes
    }

    pub(crate) fn xlib_attributes_mut_ptr(&mut self) -> *mut xlib::XSetWindowAttributes {
        &mut self.attributes
    }
}

impl Default for WindowAttributes {

    /// Xlib defaults
    ///
    /// ```rust
    /// // default attributes
    /// let attributes = xlib::XSetWindowAttributes {
    ///     background_pixmap: XLIB_NONE,
    ///     background_pixel: 0, // default undefined
    ///     border_pixmap: xlib::CopyFromParent as xlib::Pixmap,
    ///     border_pixel: 0, // default undefined
    ///     bit_gravity: xlib::ForgetGravity,
    ///     win_gravity: xlib::NorthWestGravity,
    ///     backing_store: xlib::NotUseful,
    ///     backing_planes: c_ulong::max_value(),
    ///     backing_pixel: 0,
    ///     save_under: xlib::False,
    ///     event_mask: 0,
    ///     do_not_propagate_mask: 0,
    ///     override_redirect: xlib::False,
    ///     colormap: xlib::CopyFromParent as xlib::Colormap,
    ///     cursor: XLIB_NONE,
    /// };
    /// ```
    fn default() -> Self {
        Self {
            attributes: xlib::XSetWindowAttributes {
                 background_pixmap: XLIB_NONE,
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
                 cursor: XLIB_NONE,
             },
            selected_attributes: AttributeMask::empty(),
        }
    }
}


bitflags! {
    pub(crate) struct AttributeMask: c_ulong {
        const BACK_PIXMAP = xlib::CWBackPixmap;
        const BACK_PIXEL = xlib::CWBackPixel;
        const BORDER_PIXMAP = xlib::CWBorderPixmap;
        const BORDER_PIXEL = xlib::CWBorderPixel;
        const BIT_GRAVITY = xlib::CWBitGravity;
        const WIN_GRAVITY = xlib::CWWinGravity;
        const BACKING_STORE = xlib::CWBackingStore;
        const BACKING_PLANES = xlib::CWBackingPlanes;
        const BACKING_PIXEL = xlib::CWBackingPixel;
        const OVERRIDE_REDIRECT = xlib::CWOverrideRedirect;
        const SAVE_UNDER = xlib::CWSaveUnder;
        const EVENT_MASK = xlib::CWEventMask;
        const DONT_PROPAGATE = xlib::CWDontPropagate;
        const COLORMAP = xlib::CWColormap;
        const CURSOR = xlib::CWCursor;
    }
}

trait AttributeConversions: Sized {
    type Xlib;
    fn to_xlib_attribute(&self) -> Self::Xlib;
    fn from_xlib_attribute(Self::Xlib) -> Self;
}

macro_rules! impl_conversion {
    ( $( $x:ty ),* ) => {
        $(
            impl AttributeConversions for $x {
                type Xlib = Self;

                fn to_xlib_attribute(&self) -> Self {
                    *self
                }

                fn from_xlib_attribute(value: Self::Xlib) -> Self {
                    value
                }
            }
        )*
    };
}

impl_conversion!(c_ulong, c_int);


macro_rules! attribute_functions {
    ( $attribute_field: tt: $attribute_type: ty, $setter_name: tt, $attribute_mask: expr) => {
        fn $attribute_field(&self) -> $attribute_type {
            AttributeConversions::from_xlib_attribute(self.attributes().attributes.$attribute_field)
        }

        fn $setter_name(&mut self, value: $attribute_type ) {
            self.attributes_mut().attributes.$attribute_field = value.to_xlib_attribute();
            self.attributes_mut().selected_attributes |= $attribute_mask;
        }
    };
}

pub trait GetAndSetAttributes {
    fn attributes(&self) -> &WindowAttributes;
    fn attributes_mut(&mut self) -> &mut WindowAttributes;
}

#[derive(Debug, Copy, Clone)]
pub enum BackgroundPixmap {
    Background(xlib::Pixmap),
    ParentRelative,
    None,
}

impl AttributeConversions for BackgroundPixmap {
    type Xlib = xlib::Pixmap;

    fn to_xlib_attribute(&self) -> xlib::Pixmap {
        match *self {
            BackgroundPixmap::Background(id) => id,
            BackgroundPixmap::ParentRelative => xlib::ParentRelative as xlib::Pixmap,
            BackgroundPixmap::None => XLIB_NONE,
        }
    }

    fn from_xlib_attribute(id: xlib::Pixmap) -> Self {
        match id {
            XLIB_NONE => BackgroundPixmap::None,
            id if id == xlib::ParentRelative as xlib::Pixmap => BackgroundPixmap::ParentRelative,
            id => BackgroundPixmap::Background(id),
        }
    }
}



#[derive(Debug, Copy, Clone)]
pub enum BorderPixmap {
    Border(xlib::Pixmap),
    CopyFromParent,
}


impl AttributeConversions for BorderPixmap {
    type Xlib = xlib::Pixmap;

    fn to_xlib_attribute(&self) -> xlib::Pixmap {
        match *self {
            BorderPixmap::Border(id) => id,
            BorderPixmap::CopyFromParent => xlib::CopyFromParent as xlib::Pixmap,
        }
    }

    fn from_xlib_attribute(id: xlib::Pixmap) -> Self {
        match id {
            id if id == xlib::CopyFromParent as xlib::Pixmap => BorderPixmap::CopyFromParent,
            id => BorderPixmap::Border(id),
        }
    }
}



#[derive(Debug, Copy, Clone)]
pub enum Gravity {
    Forget,
    Static,
}


impl Default for Gravity {
    /// `Gravity::Forget`
    fn default() -> Self {
        Gravity::Forget
    }
}

impl AttributeConversions for Gravity  {
    type Xlib = c_int;

    fn to_xlib_attribute(&self) -> Self::Xlib {
        match *self {
            Gravity::Forget => xlib::ForgetGravity,
            Gravity::Static => xlib::StaticGravity,
        }
    }

    fn from_xlib_attribute(value: Self::Xlib) -> Self {
        match value {
            xlib::ForgetGravity => Gravity::Forget,
            xlib::StaticGravity => Gravity::Static,
            value => {
                eprintln!("x11_wrapper warning: unknown gravity value {}, using default value", value);
                Gravity::default()
            }
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub enum WindowGravity {
    NorthWest,
    North,
    NorthEast,
    West,
    Center,
    East,
    SouthWest,
    South,
    SouthEast,
    Unmap,
}

impl Default for WindowGravity {
    /// `WindowGravity::NorthWest`
    fn default() -> Self {
        WindowGravity::NorthWest
    }
}

impl AttributeConversions for WindowGravity {
    type Xlib = c_int;

    fn to_xlib_attribute(&self) -> Self::Xlib {
        match *self {
            WindowGravity::NorthWest => xlib::NorthWestGravity,
            WindowGravity::North => xlib::NorthGravity,
            WindowGravity::NorthEast => xlib::NorthEastGravity,
            WindowGravity::West => xlib::WestGravity,
            WindowGravity::Center => xlib::CenterGravity,
            WindowGravity::East => xlib::EastGravity,
            WindowGravity::SouthWest => xlib::SouthWestGravity,
            WindowGravity::South => xlib::SouthGravity,
            WindowGravity::SouthEast => xlib::SouthEastGravity,
            WindowGravity::Unmap => xlib::UnmapGravity,
        }
    }

    fn from_xlib_attribute(id: Self::Xlib) -> Self {
        match id {
            xlib::NorthWestGravity => WindowGravity::NorthWest,
            xlib::NorthGravity => WindowGravity::North,
            xlib::NorthEastGravity => WindowGravity::NorthEast,
            xlib::WestGravity => WindowGravity::West,
            xlib::CenterGravity => WindowGravity::Center,
            xlib::EastGravity => WindowGravity::East,
            xlib::SouthWestGravity => WindowGravity::SouthWest,
            xlib::SouthGravity => WindowGravity::South,
            xlib::SouthEastGravity => WindowGravity::SouthEast,
            xlib::UnmapGravity => WindowGravity::Unmap,
            value => {
                eprintln!("x11_wrapper warning: unknown window gravity value {}, using default value", value);
                WindowGravity::default()
            }
        }
    }
}



#[derive(Debug, Clone, Copy)]
pub enum BackingStore {
    NotUseful,
    WhenMapped,
    Always,
}

impl AttributeConversions for BackingStore {
    type Xlib = c_int;

    fn to_xlib_attribute(&self) -> Self::Xlib {
        match *self {
            BackingStore::NotUseful => xlib::NotUseful,
            BackingStore::WhenMapped => xlib::WhenMapped,
            BackingStore::Always => xlib::Always,
        }
    }

    fn from_xlib_attribute(value: Self::Xlib) -> Self {
        match value {
            xlib::NotUseful => BackingStore::NotUseful,
            xlib::WhenMapped => BackingStore::WhenMapped,
            xlib::Always => BackingStore::Always,
            value => {
                eprintln!("x11_wrapper warning: unknown backing store value {}, using default value", value);
                BackingStore::default()
            }
        }
    }
}

impl Default for BackingStore {
    fn default() -> Self {
        BackingStore::NotUseful
    }
}



#[derive(Debug, Copy, Clone)]
pub struct SaveUnder(pub bool);

impl AttributeConversions for SaveUnder {
    type Xlib = c_int;

    fn to_xlib_attribute(&self) -> Self::Xlib {
        if self.0 {
            xlib::True
        } else {
            xlib::False
        }
    }

    fn from_xlib_attribute(value: Self::Xlib) -> Self {
        match value {
            xlib::True => SaveUnder(true),
            xlib::False => SaveUnder(false),
            value => {
                eprintln!("x11_wrapper warning: unknown save under value {}, using default value", value);
                SaveUnder::default()
            }
        }
    }
}

impl Default for SaveUnder {
    fn default() -> Self {
        SaveUnder(false)
    }
}

impl AttributeConversions for EventMask {
    type Xlib = c_long;

    fn to_xlib_attribute(&self) -> Self::Xlib {
        self.bits()
    }

    fn from_xlib_attribute(value: Self::Xlib) -> Self {
        match EventMask::from_bits(value) {
            Some(events) => events,
            None => {
                eprintln!("x11_wrapper warning: unknown bits in event mask {:#b}", value);
                EventMask::from_bits_truncate(value)
            }
        }
    }
}

bitflags!(
    pub struct DoNotPropagateMask: c_long {
        const KEY_PRESS = xlib::KeyPressMask;
        const KEY_RELEASE = xlib::KeyReleaseMask;
        const BUTTON_PRESS = xlib::ButtonPressMask;
        const BUTTON_RELEASE = xlib::ButtonReleaseMask;
        const POINTER_MOTION = xlib::PointerMotionMask;
        const BUTTON_1_MOTION = xlib::Button1MotionMask;
        const BUTTON_2_MOTION = xlib::Button2MotionMask;
        const BUTTON_3_MOTION = xlib::Button3MotionMask;
        const BUTTON_4_MOTION = xlib::Button4MotionMask;
        const BUTTON_5_MOTION = xlib::Button5MotionMask;
        const BUTTON_MOTION = xlib::ButtonMotionMask;
    }
);


impl AttributeConversions for DoNotPropagateMask {
    type Xlib = c_long;

    fn to_xlib_attribute(&self) -> Self::Xlib {
        self.bits()
    }

    fn from_xlib_attribute(value: Self::Xlib) -> Self {
        match DoNotPropagateMask::from_bits(value) {
            Some(events) => events,
            None => {
                eprintln!("x11_wrapper warning: unknown bits in 'do not propagate' mask {:#b}", value);
                DoNotPropagateMask::from_bits_truncate(value)
            }
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub struct OverrideRedirect(pub bool);

impl AttributeConversions for OverrideRedirect {
    type Xlib = c_int;

    fn to_xlib_attribute(&self) -> Self::Xlib {
        if self.0 {
            xlib::True
        } else {
            xlib::False
        }
    }

    fn from_xlib_attribute(value: Self::Xlib) -> Self {
        match value {
            xlib::True => OverrideRedirect(true),
            xlib::False => OverrideRedirect(false),
            value => {
                eprintln!("x11_wrapper warning: unknown override redirect value {}, using default value", value);
                OverrideRedirect::default()
            }
        }
    }
}

impl Default for OverrideRedirect {
    fn default() -> Self {
        OverrideRedirect(false)
    }
}


#[derive(Debug, Clone, Copy)]
pub enum Colormap {
    Colormap(xlib::Colormap),
    CopyFromParent,
}

impl AttributeConversions for Colormap {
    type Xlib = xlib::Colormap;

    fn to_xlib_attribute(&self) -> Self::Xlib {
        match *self {
            Colormap::Colormap(id) => id,
            Colormap::CopyFromParent => xlib::CopyFromParent as xlib::Colormap,
        }
    }

    fn from_xlib_attribute(id: Self::Xlib) -> Self {
        if id == xlib::CopyFromParent as xlib::Colormap {
            Colormap::CopyFromParent
        } else {
            Colormap::Colormap(id)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Cursor {
    Cursor(xlib::Cursor),
    None,
}

impl AttributeConversions for Cursor {
    type Xlib = xlib::Cursor;

    fn to_xlib_attribute(&self) -> Self::Xlib {
        match *self {
            Cursor::Cursor(id) => id,
            Cursor::None => XLIB_NONE as Self::Xlib,
        }
    }

    fn from_xlib_attribute(id: Self::Xlib) -> Self {
        if id == XLIB_NONE as Self::Xlib {
            Cursor::None
        } else {
            Cursor::Cursor(id)
        }
    }
}


/// This attribute is separate trait because for top level
/// windows this is always false.
pub trait AttributeOverrideRedirect: GetAndSetAttributes {
    attribute_functions!(
        override_redirect: OverrideRedirect,
        set_override_redirect,
        AttributeMask::OVERRIDE_REDIRECT
    );
}

pub trait CommonAttributes: GetAndSetAttributes {
    attribute_functions!(
        win_gravity: WindowGravity,
        set_win_gravity,
        AttributeMask::WIN_GRAVITY
    );

    attribute_functions!(
        event_mask: EventMask,
        set_event_mask,
        AttributeMask::EVENT_MASK
    );

    attribute_functions!(
        do_not_propagate_mask: DoNotPropagateMask,
        set_do_not_propagate,
        AttributeMask::DONT_PROPAGATE
    );

    attribute_functions!(
        cursor: Cursor,
        set_cursor,
        AttributeMask::CURSOR
    );
}

pub trait InputOutputWindowAttributes: GetAndSetAttributes {
    attribute_functions!(
        background_pixmap: BackgroundPixmap,
        set_background_pixmap,
        AttributeMask::BACK_PIXMAP
    );

    attribute_functions!(
        background_pixel: c_ulong,
        set_background_pixel,
        AttributeMask::BACK_PIXEL
    );

    attribute_functions!(
        border_pixmap: BorderPixmap,
        set_border_pixmap,
        AttributeMask::BORDER_PIXMAP
    );

    attribute_functions!(
        border_pixel: c_ulong,
        set_border_pixel,
        AttributeMask::BORDER_PIXEL
    );

    attribute_functions!(
        bit_gravity: Gravity,
        set_bit_gravity,
        AttributeMask::BIT_GRAVITY
    );

    attribute_functions!(
        backing_store: BackingStore,
        set_backing_store,
        AttributeMask::BACKING_STORE
    );

    attribute_functions!(
        backing_planes: c_ulong,
        set_backing_planes,
        AttributeMask::BACKING_PLANES
    );

    attribute_functions!(
        backing_pixel: c_ulong,
        set_backing_pixel,
        AttributeMask::BACKING_PIXEL
    );

    attribute_functions!(
        save_under: SaveUnder,
        set_save_under,
        AttributeMask::SAVE_UNDER
    );

    attribute_functions!(
        colormap: Colormap,
        set_colormap,
        AttributeMask::COLORMAP
    );
}


/*
template


impl AttributeConversions for  {
    type Xlib = ;

    fn to_xlib_attribute(&self) -> Self::Xlib {

    }

    fn from_xlib_attribute(id: Self::Xlib) -> Self {

    }
}

attribute_trait!(

);

*/
