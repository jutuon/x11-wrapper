//! Extended Window Manager Hints 1.3 properties.
//!
//! [EWMH 1.3 documentation](https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html)

use std::os::raw::c_long;

use core::utils::{Atom, AtomName};
use core::event::ClientMessageEventCreator;
use core::display::X11Display;
use core::window::input_output::TopLevelInputOutputWindow;
use core::window::Window;

/// Handler for `_NET_WM_STATE`.
pub struct NetWmStateHandler {
    event: ClientMessageEventCreator,
    fullscreen: Atom,
    net_wm_state: Atom,
}

impl NetWmStateHandler {
    /// Returns error if querying atom_name fails.
    ///
    /// XInternAtom
    pub fn new(display: &X11Display) -> Result<Self, ()> {
        let fullscreen_name = AtomName::new("_NET_WM_STATE_FULLSCREEN".to_string())
            .map_err(|_| ())
            .unwrap();
        let fullscreen = Atom::new(display, fullscreen_name, false)?;

        let net_wm_state_name = AtomName::new("_NET_WM_STATE".to_string())
            .map_err(|_| ())
            .unwrap();
        let net_wm_state = Atom::new(display, net_wm_state_name, false)?;

        Ok(Self {
            fullscreen,
            event: ClientMessageEventCreator::new(),
            net_wm_state,
        })
    }

    /// `_NET_WM_STATE_FULLSCREEN`
    pub fn fullscreen_atom(&self) -> Atom {
        self.fullscreen
    }

    /// Prepare client message for toggling fullscreen property
    /// of `window`.
    pub fn toggle_fullscreen(
        &mut self,
        window: &TopLevelInputOutputWindow,
    ) -> &mut ClientMessageEventCreator {
        let fullscreen_atom = self.fullscreen_atom().atom_id() as c_long;

        {
            let event = self.event.client_message_mut();
            event.message_type = self.net_wm_state.atom_id();
            event.window = window.window_id();
            event.format = 32;

            let data = event.data.as_longs_mut();
            data[0] = 2; // toggle property
            data[1] = fullscreen_atom;
            data[2] = 0; // no second property
            data[3] = 2; // direct user action
            data[4] = 0;
        }

        &mut self.event
    }
}
