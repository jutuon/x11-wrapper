
//! Event handling.

use std::mem;
use std::os::raw::{c_int, c_long, c_uint};

use x11::xlib;

pub struct EventBuffer {
    event: xlib::XEvent,
}

impl EventBuffer {
    pub(crate) fn zeroed() -> Self {
        Self {
            event: unsafe {
                mem::zeroed()
            }
        }
    }

    pub(crate) fn event_mut_ptr(&mut self) -> *mut xlib::XEvent {
        &mut self.event
    }

    pub fn raw_event(&self) -> &xlib::XEvent {
        &self.event
    }

    pub fn event<'a>(&'a self) -> Event<'a> {
        unsafe {
            match self.event.type_ {
                xlib::MotionNotify => Event::MotionNotify(&self.event.motion),

                xlib::ButtonPress => Event::ButtonPress(&self.event.button),
                xlib::ButtonRelease => Event::ButtonRelease(&self.event.button),
                xlib::ColormapNotify => Event::ColormapNotify(&self.event.colormap),
                xlib::EnterNotify => Event::EnterNotify(&self.event.crossing),
                xlib::LeaveNotify => Event::LeaveNotify(&self.event.crossing),
                xlib::Expose => Event::Expose(&self.event.expose),

                xlib::GraphicsExpose => Event::GraphicsExpose(&self.event.graphics_expose),
                xlib::NoExpose => Event::NoExpose(&self.event.no_expose),

                xlib::FocusIn => Event::FocusIn(&self.event.focus_change),
                xlib::FocusOut => Event::FocusOut(&self.event.focus_change),

                xlib::KeymapNotify => Event::KeymapNotify(&self.event.keymap),
                xlib::KeyPress => Event::KeyPress(&self.event.key),
                xlib::KeyRelease => Event::KeyRelease(&self.event.key),
                // MotionNotify
                xlib::PropertyNotify => Event::PropertyNotify(&self.event.property),
                xlib::ResizeRequest => Event::ResizeRequest(&self.event.resize_request),

                xlib::CirculateNotify => Event::CirculateNotify(&self.event.circulate),
                xlib::ConfigureNotify => Event::ConfigureNotify(&self.event.configure),
                xlib::DestroyNotify => Event::DestroyNotify(&self.event.destroy_window),
                xlib::GravityNotify => Event::GravityNotify(&self.event.gravity),
                xlib::MapNotify => Event::MapNotify(&self.event.map),
                xlib::ReparentNotify => Event::ReparentNotify(&self.event.reparent),
                xlib::UnmapNotify => Event::UnmapNotify(&self.event.unmap),

                // CirculateNotify
                // ConfigureNotify
                xlib::CreateNotify => Event::CreateNotify(&self.event.create_window),
                // DestroyNotify
                // GravityNotify
                // MapNotify
                // ReparentNotify
                // UnmapNotify

                xlib::CirculateRequest => Event::CirculateRequest(&self.event.circulate_request),
                xlib::ConfigureRequest => Event::ConfigureRequest(&self.event.configure_request),
                xlib::MapRequest => Event::MapRequest(&self.event.map_request),

                xlib::ClientMessage => Event::ClientMessage(&self.event.client_message),
                xlib::MappingNotify => Event::MappingNotify(&self.event.mapping),
                xlib::SelectionClear => Event::SelectionClear(&self.event.selection_clear),
                xlib::SelectionNotify => Event::SelectionNotify(&self.event.selection),
                xlib::SelectionRequest => Event::SelectionRequest(&self.event.selection_request),
                xlib::VisibilityNotify => Event::VisibilityNotify(&self.event.visibility),

                event_type => Event::UnknownEvent(event_type),
            }
        }
    }

}


/// Events like in Xlib manual section "Event Processing Overview".
#[derive(Debug)]
pub enum Event<'a> {
    MotionNotify(&'a xlib::XPointerMovedEvent),

    ButtonPress(&'a xlib::XButtonPressedEvent),
    ButtonRelease(&'a xlib::XButtonReleasedEvent),
    ColormapNotify(&'a xlib::XColormapEvent),
    EnterNotify(&'a xlib::XEnterWindowEvent),
    LeaveNotify(&'a xlib::XLeaveWindowEvent),
    Expose(&'a xlib::XExposeEvent),

    GraphicsExpose(&'a xlib::XGraphicsExposeEvent),
    NoExpose(&'a xlib::XNoExposeEvent),

    FocusIn(&'a xlib::XFocusInEvent),
    FocusOut(&'a xlib::XFocusOutEvent),

    KeymapNotify(&'a xlib::XKeymapEvent),
    KeyPress(&'a xlib::XKeyPressedEvent),
    KeyRelease(&'a xlib::XKeyReleasedEvent),
    // MotionNotify(&'a xlib::XPointerMovedEvent),
    PropertyNotify(&'a xlib::XPropertyEvent),
    ResizeRequest(&'a xlib::XResizeRequestEvent),

    CirculateNotify(&'a xlib::XCirculateEvent),
    ConfigureNotify(&'a xlib::XConfigureEvent),
    DestroyNotify(&'a xlib::XDestroyWindowEvent),
    GravityNotify(&'a xlib::XGravityEvent),
    MapNotify(&'a xlib::XMapEvent),
    ReparentNotify(&'a xlib::XReparentEvent),
    UnmapNotify(&'a xlib::XUnmapEvent),

    // CirculateNotify(&'a xlib::XCirculateEvent),
    // ConfigureNotify(&'a xlib::XConfigureEvent),
    CreateNotify(&'a xlib::XCreateWindowEvent),
    // DestroyNotify(&'a xlib::XDestroyWindowEvent),
    // GravityNotify(&'a xlib::XGravityEvent),
    // MapNotify(&'a xlib::XMapEvent),
    // ReparentNotify(&'a xlib::XReparentEvent),
    // UnmapNotify(&'a xlib::XUnmapEvent),

    CirculateRequest(&'a xlib::XCirculateRequestEvent),
    ConfigureRequest(&'a xlib::XConfigureRequestEvent),
    MapRequest(&'a xlib::XMapRequestEvent),

    ClientMessage(&'a xlib::XClientMessageEvent),
    MappingNotify(&'a xlib::XMappingEvent),
    SelectionClear(&'a xlib::XSelectionClearEvent),
    SelectionNotify(&'a xlib::XSelectionEvent),
    SelectionRequest(&'a xlib::XSelectionRequestEvent),
    VisibilityNotify(&'a xlib::XVisibilityEvent),

    UnknownEvent(c_int),
}

impl <'a> Event<'a> {
    pub fn into_simple_event(self) -> SimpleEvent<'a> {
        match self {
            Event::MotionNotify(e) => SimpleEvent::MotionNotify { x: e.x, y: e.y },
            Event::ButtonPress(e) => SimpleEvent::ButtonPress { button: e.button },
            Event::ButtonRelease(e) => SimpleEvent::ButtonRelease { button: e.button },
            Event::KeyPress(e) => SimpleEvent::KeyPress { keycode: e.keycode },
            Event::KeyRelease(e) => SimpleEvent::KeyRelease { keycode: e.keycode },
            Event::EnterNotify(_) => SimpleEvent::EnterNotify,
            Event::LeaveNotify(_) => SimpleEvent::LeaveNotify,
            Event::FocusIn(_) => SimpleEvent::FocusIn,
            Event::FocusOut(_) => SimpleEvent::FocusOut,
            Event::MapNotify(_) => SimpleEvent::MapNotify,
            Event::UnmapNotify(_) => SimpleEvent::UnmapNotify,
            Event::ConfigureNotify(e) => SimpleEvent::ConfigureNotify { x: e.x, y: e.y, width: e.width, height: e.height },
            Event::ClientMessage(e) => SimpleEvent::ClientMessage(e),
            e => SimpleEvent::UnknownEvent(e),
        }
    }
}

#[derive(Debug)]
pub enum SimpleEvent<'a> {
    MotionNotify { x: c_int, y: c_int },
    ButtonPress { button: c_uint },
    ButtonRelease { button: c_uint },
    KeyPress { keycode: c_uint },
    KeyRelease { keycode: c_uint },
    EnterNotify,
    LeaveNotify,
    FocusIn,
    FocusOut,
    DestroyNotify,
    MapNotify,
    UnmapNotify,
    ConfigureNotify { x: c_int, y: c_int, width: c_int, height: c_int },
    ClientMessage(&'a xlib::XClientMessageEvent),
    UnknownEvent(Event<'a>),
}

bitflags! {
    pub struct EventMask: c_long {
        const KEY_PRESS = xlib::KeyPressMask;
        const KEY_RELEASE = xlib::KeyReleaseMask;
        const BUTTON_PRESS = xlib::ButtonPressMask;
        const BUTTON_RELEASE = xlib::ButtonReleaseMask;
        const ENTER_WINDOW = xlib::EnterWindowMask;
        const LEAVE_WINDOW = xlib::LeaveWindowMask;
        const POINTER_MOTION = xlib::PointerMotionMask;
        const POINTER_MOTION_HINT = xlib::PointerMotionHintMask;
        const BUTTON_1_MOTION = xlib::Button1MotionMask;
        const BUTTON_2_MOTION = xlib::Button2MotionMask;
        const BUTTON_3_MOTION = xlib::Button3MotionMask;
        const BUTTON_4_MOTION = xlib::Button4MotionMask;
        const BUTTON_5_MOTION = xlib::Button5MotionMask;
        const BUTTON_MOTION = xlib::ButtonMotionMask;
        const KEYMAP_STATE = xlib::KeymapStateMask;
        const EXPOSURE_MASK = xlib::ExposureMask;
        const VISIBILITY_CHANGE = xlib::VisibilityChangeMask;
        const STRUCTURE_NOTIFY = xlib::StructureNotifyMask;
        const RESIZE_REDIRECT = xlib::ResizeRedirectMask;
        const SUBSTRUCTURE_NOTIFY = xlib::SubstructureNotifyMask;
        const SUBSTRUCTURE_REDIRECT = xlib::SubstructureRedirectMask;
        const FOCUS_CHANGE = xlib::FocusChangeMask;
        const PROPERTY_CHANGE = xlib::PropertyChangeMask;
        const COLORMAP_CHANGE = xlib::ColormapChangeMask;
        const OWNER_GRAB_BUTTON = xlib::OwnerGrabButtonMask;
    }
}

pub trait EventCreator {
    fn raw_event_mut(&mut self) -> &mut xlib::XEvent;
}


/// Zeroed memory `xlib::XEvent`.
pub struct AnyEventCreator {
    raw_event: xlib::XEvent,
}


impl AnyEventCreator {
    /// All fields of `xlib::XEvent` will be zero.
    pub fn new() -> Self {
        let raw_event = unsafe {
            mem::zeroed()
        };

        Self {
            raw_event
        }
    }
}

impl EventCreator for AnyEventCreator {
    fn raw_event_mut(&mut self) -> &mut xlib::XEvent {
        &mut self.raw_event
    }
}

/// Zeroed memory XClientMessageEvent.
pub struct ClientMessageEventCreator(AnyEventCreator);


impl ClientMessageEventCreator {
    /// Sets events type to `xlib::ClientMessage`.
    pub fn new() -> Self {
        let mut event = AnyEventCreator::new();

        event.raw_event_mut().type_ = xlib::ClientMessage;

        ClientMessageEventCreator(event)
    }

    pub fn client_message_mut(&mut self) -> &mut xlib::XClientMessageEvent {
        unsafe {
            &mut self.raw_event_mut().client_message
        }
    }
}

impl EventCreator for ClientMessageEventCreator {
    fn raw_event_mut(&mut self) -> &mut xlib::XEvent {
        self.0.raw_event_mut()
    }
}

/// See documentation of `Display::send_event`.
pub(crate) fn send_event<T: EventCreator>(raw_display: *mut xlib::Display, window_id: xlib::Window, propagate: bool, event_mask: EventMask, event_creator: &mut T) -> Result<(),()> {
    let propagate = if propagate {
        xlib::True
    } else {
        xlib::False
    };

    let event = event_creator.raw_event_mut();

    unsafe {
        event.any.display = raw_display;
    }

    let status = unsafe {
        xlib::XSendEvent(raw_display, window_id, propagate, event_mask.bits(), event)
    };

    if status == 0 {
        Err(())
    } else {
        Ok(())
    }
}