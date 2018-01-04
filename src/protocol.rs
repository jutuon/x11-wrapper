//! Handle optional protocols.

use std::os::raw::c_long;

use core::utils::{Atom, AtomList, AtomName};
use core::display::Display;

use x11::xlib;

/// Create handlers for protocols which will
/// be enabled with window property `WM_PROTOCOLS`.
pub struct Protocols {
    delete_window: Option<Atom>,
}

impl Protocols {
    pub fn new() -> Self {
        Self {
            delete_window: None,
        }
    }

    /// Returns error if `Atom` creation failed.
    pub fn enable_delete_window(
        &mut self,
        display: &Display,
    ) -> Result<ProtocolHandlerDeleteWindow, ()> {
        let name = AtomName::new("WM_DELETE_WINDOW".to_string()).map_err(|_| ())?;
        let atom = Atom::new(display, name, false)?;

        self.delete_window = Some(atom);

        Ok(ProtocolHandlerDeleteWindow {
            protocol_name: atom,
        })
    }

    /// Value for `WM_PROTOCOLS` property.
    pub fn protocol_atom_list(self) -> AtomList {
        let mut atom_list = AtomList::new();

        if let Some(atom) = self.delete_window {
            atom_list.add(atom)
        }

        atom_list
    }
}

/// Handler for protocol `WM_DELETE_WINDOW`.
pub struct ProtocolHandlerDeleteWindow {
    protocol_name: Atom,
}

impl ProtocolHandlerDeleteWindow {
    /// Return true if event matches the protocol.
    pub fn check_event(&self, event: &xlib::XClientMessageEvent) -> bool {
        event.format == 32 && event.data.as_longs()[0] == self.protocol_name.atom_id() as c_long
    }
}
