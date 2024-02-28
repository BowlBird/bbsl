// https://wayland-book.com/

mod events;
mod drawing;

use std::str::FromStr;
use wayland_client::{protocol::{wl_compositor::WlCompositor, wl_keyboard::WlKeyboard, wl_seat::WlSeat, wl_shm::WlShm, wl_surface::WlSurface}, Connection, EventQueue};
use wayland_protocols::xdg::shell::client::{xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel, xdg_wm_base::XdgWmBase};
use pam::Client;
use xkbcommon::xkb::{ffi::XKB_CONTEXT_NO_FLAGS, Context};

struct Rect {
    width: i32,
    height: i32
}

struct AppState {
    compositor: Option<WlCompositor>,
    shm: Option<WlShm>,
    base: Option<XdgWmBase>,
    wl_surface: Option<WlSurface>,
    xdg_surface: Option<XdgSurface>,
    xdg_toplevel: Option<XdgToplevel>,
    wl_seat: Option<WlSeat>,
    wl_keyboard: Option<WlKeyboard>,
    xkb_context: Option<Context>,
    dimension: Option<Rect>,
    quit: bool
}


fn main() {

    let mut state = AppState {
        compositor: None,
        shm: None,
        base: None,
        wl_surface: None,
        xdg_surface: None,
        xdg_toplevel: None,
        wl_seat: None,
        wl_keyboard: None,
        xkb_context: None, 
        dimension: None,
        quit: false,
    };

    let connection = Connection::connect_to_env()
        .expect("could not connect to env");
    state.xkb_context = Some(Context::new(XKB_CONTEXT_NO_FLAGS));

    let display = connection.display();

    let mut event_queue: EventQueue<AppState> = connection.new_event_queue();
    let qh = event_queue.handle();
    display.get_registry(&qh, ());
    let _ = event_queue.roundtrip(&mut state);

    state.wl_keyboard = Some(state
        .wl_seat.as_ref()
        .expect("could not connect to wl_seat")
        .get_keyboard(&qh, ()));

    state.wl_surface = Some(state
        .compositor.as_ref()
        .expect("could not connect to compositor")
        .create_surface(&qh, ()));
    
    let wl_surface = state
        .wl_surface.as_ref()
        .expect("could not connect to wl_surface");
    
    state.xdg_surface = Some(state
        .base.as_ref()
        .expect("could not connect to xdg_wm_base")
        .get_xdg_surface(wl_surface, &qh, ()));

    let xdg_surface = state
        .xdg_surface.as_ref()
        .expect("could not connect to xdg_surface");

    state.xdg_toplevel = Some(xdg_surface.get_toplevel(&qh, ()));

    let _xdg_toplevel = state
        .xdg_toplevel.as_ref()
        .expect("could not connect to xdg_toplevel");

    _xdg_toplevel.set_title(String::from_str("bbsl").expect("could not create string"));
    
    wl_surface.commit();    
    let _ = wl_surface.frame(&qh, ());

    _xdg_toplevel.set_fullscreen(None);

    let mut client = Client::with_password("system-auth")
    .expect("Failed to init PAM client");

    // Preset the login & password we will use for authentication
    client.conversation_mut().set_credentials("carson", "8dyUfd9W9JbeRKXqaz");
    // Actually try to authenticate:
    client.authenticate().expect("Authentication failed!");
    // Now that we are authenticated, it's possible to open a sesssion:
    client.open_session().expect("Failed to open a session!");


    while !state.quit {

        let _ = event_queue.blocking_dispatch(&mut state);
    }
}
