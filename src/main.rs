use std::{thread::sleep, time::Duration};

use clap::Parser;
// https://docs.rs/wayland-client/latest/wayland_client/
use wayland_client::{backend::{ObjectData, ObjectId}, protocol::{wl_registry, wl_surface::{self, WlSurface}}, Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols::ext::session_lock::v1::client::{__interfaces::ext_session_lock_manager_v1_requests, ext_session_lock_manager_v1::{self, ExtSessionLockManagerV1}, ext_session_lock_surface_v1, ext_session_lock_v1::{self, ExtSessionLockV1}};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

struct AppState;

impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        _state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        if let wl_registry::Event::Global { name, interface, version} = event {
            println!("[{}] {} (v{})", name, interface, version);
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
    ) {}
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
        _state: &mut Self,
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

    let connection = Connection::connect_to_env().expect("Unable to connect to Wayland environment.");
    let display = connection.display();
    let mut event_queue: EventQueue<AppState> = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    let _registry = display.get_registry(&queue_handle, ());


    let session_lock = _registry.bind::<ExtSessionLockManagerV1, (), AppState>(38, 1, &queue_handle, ());
    session_lock.lock(&queue_handle, ());

    // let surface = _registry.bind::<WlSurface, (), AppState>(50, 1, &queue_handle, ());
    // surface.

    // println!("Advertised globals:");
    event_queue.roundtrip(&mut AppState).expect("Could not block for compositor events.");
}
