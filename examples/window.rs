extern crate x11_wrapper;

use std::thread;
use std::time::Duration;

fn main() {
    println!("Hello world");

    let display = x11_wrapper::display::Display::new().unwrap();

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

    window.map_window();

    display.flush_output_buffer();

    thread::sleep(Duration::from_millis(12000));
}