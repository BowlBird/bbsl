// https://wayland-book.com/

use std::{io::{stdout, Write}, sync::Mutex, thread};

use client::{AppState, FrameBuffer, WaylandClient, WaylandClientBuilder};
use pam_client::{conv_mock::Conversation, Context, Flag};
use rand::Rng;
use users::get_current_username;
use xkbcommon::xkb::{self, Keysym};

mod client;

static OUTPUT: Mutex<String> = Mutex::new(String::new());

fn draw(_state: &AppState, frame_buffer: &FrameBuffer) {
    let color:u32 = rand::thread_rng().gen_range(0x00000000..=0x50);

    for y in 0..frame_buffer.dimension.height {
        for x in 0..frame_buffer.dimension.width {
            unsafe {
                *(frame_buffer.pool_data.offset(((y * frame_buffer.dimension.width + x) * 4) as isize) as *mut u32) = color;
            }
        }
    }
}

fn authenticate(state: &mut AppState, password: &str) {
    let user = get_current_username()
    .expect("could not get current user!");
    let user = user.to_str().unwrap();

    let mut context = Context::new(
        "bbsl",  // Service name
        None,
        Conversation::with_credentials(user, password)
     ).expect("Failed to initialize PAM context");
     
    // Authenticate the user
    match context.authenticate(Flag::NONE) {
        Ok(_) => {
            state.quit = true;
        },
        Err(err) => {
            println!("HERE: {:?} {:?}", context.conversation().log, err);
        }
    }
}

fn keyboard(state: &mut AppState, key: Keysym) {
    let key_name = xkb::keysym_to_utf8(key);
    let key_name = key_name.trim_matches(char::from(0));
    let mut output = OUTPUT.lock().unwrap();

    if key.name().unwrap() == "XK_BackSpace" {
        output.pop();
    }
    else if key.name().unwrap() == "XK_Return" {
        authenticate(state, output.as_str());
    }
    else {
        output.push_str(key_name);
    }
}

fn main() {
    let mut client = WaylandClientBuilder::default()
        .drawing_callback(draw)
        .keyboard_callback(keyboard)
        .build()
        .expect("unable to build client");
    client.start();
}
