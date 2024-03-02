// https://wayland-book.com/

use client::{WaylandClientBuilder, FrameBuffer};
use pam::Client;
use rand::Rng;
use xkbcommon::xkb::Keysym;

mod client;

fn draw(frame_buffer: &FrameBuffer) {
    let color:u32 = rand::thread_rng().gen_range(0x00000000..=0x50);

    for y in 0..frame_buffer.dimension.height {
        for x in 0..frame_buffer.dimension.width {
            unsafe {
                *(frame_buffer.pool_data.offset(((y * frame_buffer.dimension.width + x) * 4) as isize) as *mut u32) = color;
            }
        }
    }
}

fn keyboard(key: Keysym) {
    let name = &key.name().unwrap()[3..];
    println!("{}", name);
}
fn main() {
    let mut client = Client::with_password("system-auth")
    .expect("Failed to init PAM client");


    WaylandClientBuilder::default()
        .drawing_callback(draw)
        .keyboard_callback(keyboard)
        .build()
        .expect("unable to build client")
        .start();

    // Preset the login & password we will use for authentication
    client.conversation_mut().set_credentials("", "");
    // Actually try to authenticate:
    client.authenticate().expect("Authentication failed!");
    // Now that we are authenticated, it's possible to open a sesssion:
    client.open_session().expect("Failed to open a session!");
}
