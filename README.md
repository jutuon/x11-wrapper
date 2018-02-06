# x11-wrapper

Safe Rust wrapper for X11 window creation using Xlib.

Consider using [XCB](https://github.com/rtbo/rust-xcb) instead of
this library if you need error handling or multithreading support.

If you need to do some graphics rendering with OpenGL to X11 window, you have to use Xlib
as OpenGL graphics drivers don't support XCB. Xlib supports mixing Xlib and XCB function calls, so it should
be possible to create OpenGL context using Xlib display and use XCB for events.
However there might be two problems: Graphics driver may call Xlib functions
or modify Xlib data structures causing multithreading issues and Xlib may still
terminate your program when connection X11 server is lost.

## Status

Basic window features work. See example code `examples/window.rs`.

You can run the example with command `cargo run --release --example window`.

## Features

- [ ] Change resolution
- [ ] Clipboard
- [ ] Drag and drop
- [x] Events
- [x] Fullscreen toggle
- [ ] Text input
- [x] Window creation
- [ ] Window icon
- [x] Window title

## License

This project is licensed under terms of

* Apache 2.0 license or
* MIT license

at your opinion.

## Contributions

Contributions will be licensed as stated in License section
of this file unless otherwise specified.
