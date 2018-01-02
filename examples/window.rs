extern crate x11_wrapper;

use std::thread;
use std::time::Duration;

use x11_wrapper::display::Display;
use x11_wrapper::event::EventMask;

fn main() {
    println!("Hello world");

    let mut display = Display::new().unwrap();

    println!("display string: {:?}", display.display_string());
    println!("protocol version: {}", display.protocol_version());
    println!("protocol revision: {}", display.protocol_revision());
    println!("server vendor: {:?}", display.server_vendor());
    println!("vendor release: {}", display.vendor_release());

    let default_screen = display.default_screen();

    let default_visual = default_screen.default_visual().unwrap();

    let mut window = default_screen.create_window_builder(default_visual)
        .unwrap()
        .build_input_output_window()
        .unwrap();

    let event_mask =  EventMask::KEY_PRESS
                    | EventMask::KEY_RELEASE
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::ENTER_WINDOW
                    | EventMask::LEAVE_WINDOW
                    | EventMask::FOCUS_CHANGE
                    | EventMask::STRUCTURE_NOTIFY;

    window.select_input(event_mask);
    window.map_window();

    display.flush_output_buffer();

    loop {
        let event = display.read_event_blocking();

        println!("{:?}", event.event().into_simple_event());
    }
}