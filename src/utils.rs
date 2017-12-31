
use x11::xlib;

pub struct Colormap(xlib::XID);

impl Colormap {
    pub(crate) fn new(id: xlib::XID) -> Self {
        Colormap(id)
    }
}

