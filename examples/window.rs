extern crate x11_wrapper;

use x11_wrapper::XlibHandle;
use x11_wrapper::core::window::input_output::{InputOutputWindowBuilder};
use x11_wrapper::core::window::WindowProperties;
use x11_wrapper::core::event::{EventMask, SimpleEvent, EventBuffer};
use x11_wrapper::core::utils::Text;
use x11_wrapper::protocol::Protocols;
use x11_wrapper::property::ewmh::NetWmStateHandler;
use x11_wrapper::property::icccm;
use x11_wrapper::core::window::attribute::{CommonAttributes, InputOutputWindowAttributes};

fn main() {
    println!("Hello world");

    let xlib_handle = XlibHandle::initialize_xlib().unwrap();

    let mut display = xlib_handle.create_display().unwrap();

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
        .set_text_property(window_title, icccm::TextProperty::Name)
        .set_text_property(window_icon_text, icccm::TextProperty::IconName)
        .start_configuring_normal_hints()
        .unwrap()
        .set_min_window_size(640, 480)
        .end()
        .set_protocols(protocols.protocol_atom_list())
        .unwrap()
        .map_window();

    display.flush_output_buffer();

    let mut net_wm_state_handler = NetWmStateHandler::new(&display).unwrap();
    let mut event_buffer = EventBuffer::new();

    loop {
        let event = display.read_event_blocking(&mut event_buffer).into_event().into_simple_event();

        println!("{:?}", &event);

        match &event {
            // Key E
            &SimpleEvent::KeyRelease { keycode: 26 } => {
                println!("Window properties:");
                for property in window.list_properties().atoms() {
                    let property_name = property.get_name(&display).unwrap();
                    println!("{}", property_name);
                }
            }
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

        if let Some(error) = x11_wrapper::check_error(&display) {
            eprintln!("xlib error: {:?}", error);
            break;
        }
    }
}
