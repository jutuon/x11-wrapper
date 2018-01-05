extern crate x11_wrapper;

use x11_wrapper::core::window::input_output::{InputOutputWindowBuilder};
use x11_wrapper::core::display::Display;
use x11_wrapper::core::event::{EventMask, SimpleEvent};
use x11_wrapper::core::utils::Text;
use x11_wrapper::protocol::Protocols;
use x11_wrapper::property::ewmh::NetWmStateHandler;
use x11_wrapper::core::window::attribute::{CommonAttributes, InputOutputWindowAttributes};

fn main() {
    println!("Hello world");

    let mut display = Display::new().unwrap();

    println!("display string: {:?}", display.display_string());
    println!("protocol version: {}", display.protocol_version());
    println!("protocol revision: {}", display.protocol_revision());
    println!("server vendor: {:?}", display.server_vendor());
    println!("vendor release: {}", display.vendor_release());


    let window_title = Text::new(&display, "Hello world".to_string()).unwrap();
    let window_icon_text = Text::new(&display, "Window".to_string()).unwrap();

    let event_mask = EventMask::KEY_PRESS | EventMask::KEY_RELEASE | EventMask::BUTTON_PRESS
        | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION
        | EventMask::ENTER_WINDOW | EventMask::LEAVE_WINDOW
        | EventMask::FOCUS_CHANGE | EventMask::STRUCTURE_NOTIFY;

    let default_screen = display.default_screen();
    let default_visual = default_screen.default_visual().unwrap();

    let mut protocols = Protocols::new();
    let delete_window_handler = protocols.enable_delete_window(&display).unwrap();

    let mut window = InputOutputWindowBuilder::new(&default_screen, default_visual)
        .unwrap()
        .set_event_mask(event_mask)
        .set_background_pixel(0x000000)
        .build_input_output_window()
        .unwrap()
        .set_window_name(window_title)
        .set_window_icon_name(window_icon_text)
        .start_configuring_normal_hints()
        .unwrap()
        .set_min_window_size(640, 480)
        .end()
        .set_protocols(protocols.protocol_atom_list())
        .unwrap()
        .map_window();

    display.flush_output_buffer();

    let mut net_wm_state_handler = NetWmStateHandler::new(&display).unwrap();

    loop {
        let event = display.read_event_blocking().event().into_simple_event();

        println!("{:?}", &event);

        match &event {
            // Key W
            &SimpleEvent::KeyRelease { keycode: 25 } => {
                let event = net_wm_state_handler.toggle_fullscreen(&window);
                default_screen.send_ewmh_client_message_event(event).unwrap();
            }
            // Key Q
            &SimpleEvent::KeyRelease { keycode: 24 } => {
                window.iconify(&default_screen).unwrap();
            }
            &SimpleEvent::ClientMessage(e) => {
                if delete_window_handler.check_event(e) {
                    break;
                }
            }
            _ => (),
        }
    }
}
