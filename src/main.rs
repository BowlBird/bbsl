// https://docs.rs/wayland-client/latest/wayland_client/
// https://bugaevc.gitbooks.io/writing-wayland-clients/content/black-square/basic-principles-of-wayland.html

use std::{thread::sleep, time::Duration};
use clap::Parser;

use wayland_client::{backend::{ObjectData, ObjectId}, globals::Global, protocol::{wl_registry, wl_surface::{self, WlSurface}}, Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols::ext::session_lock::v1::client::{__interfaces::ext_session_lock_manager_v1_requests, ext_session_lock_manager_v1::{self, ExtSessionLockManagerV1}, ext_session_lock_surface_v1, ext_session_lock_v1::{self, ExtSessionLockV1}};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

struct AppState {
    session_lock_manager: Option<ExtSessionLockManagerV1>,
    quit: bool,
}


impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        if let wl_registry::Event::Global { name, interface, version} = event {
            if interface == "ext_session_lock_manager_v1" {
                let session_lock_manager = registry.bind::<ExtSessionLockManagerV1, (), AppState>(name, version, qh, ());
                session_lock_manager.lock(qh, ());
                state.session_lock_manager = Some(session_lock_manager);
            }
        }
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for AppState {
    fn event(
        _state: &mut Self,
        surface: &wl_surface::WlSurface,
        event: wl_surface::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        println!("enter!");
    }
}

impl Dispatch<ext_session_lock_surface_v1::ExtSessionLockSurfaceV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        surface: &ext_session_lock_surface_v1::ExtSessionLockSurfaceV1,
        event: ext_session_lock_surface_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {}
}

impl Dispatch<ext_session_lock_v1::ExtSessionLockV1, ()> for AppState {
    fn event(
        state: &mut Self,
        lock: &ExtSessionLockV1,
        event: ext_session_lock_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        println!("{:?}", event);
        // lock.get_lock_surface(surface, output, qh, ());
        sleep(Duration::from_secs(1));
        lock.unlock_and_destroy();
        state.quit = true;
    }
}

impl Dispatch<ext_session_lock_manager_v1::ExtSessionLockManagerV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        session_lock_manager: &ExtSessionLockManagerV1,
        event: ext_session_lock_manager_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {}
}

fn main() {
    Args::parse();

    let mut state = AppState {session_lock_manager: None, quit: false};

    let connection = Connection::connect_to_env().expect("Unable to connect to Wayland environment.");
    let display = connection.display();

    let mut event_queue: EventQueue<AppState> = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    let registry = display.get_registry(&queue_handle, ());

    event_queue.roundtrip(&mut state).expect("Could not block for compositor events.");

    while !state.quit {
        let _ = event_queue.blocking_dispatch(&mut state);
    }
}
