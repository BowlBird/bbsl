// https://docs.rs/wayland-client/latest/wayland_client/
// https://bugaevc.gitbooks.io/writing-wayland-clients/content/black-square/basic-principles-of-wayland.html

use std::{ffi::CStr, fs::File, os::{fd::{self, AsFd, BorrowedFd}, raw::c_int}, str::FromStr, thread::sleep, time::Duration};
use clap::Parser;
use nix::sys::memfd::{memfd_create, MemFdCreateFlag};

use wayland_client::{backend::{ObjectData, ObjectId}, globals::Global, protocol::{wl_buffer, wl_compositor::{self, WlCompositor}, wl_output::{self, WlOutput}, wl_registry, wl_shell::{self, WlShell}, wl_shell_surface, wl_shm::{self, WlShm}, wl_shm_pool, wl_surface::{self, WlSurface}}, Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols::ext::session_lock::v1::client::{__interfaces::ext_session_lock_manager_v1_requests, ext_session_lock_manager_v1::{self, ExtSessionLockManagerV1}, ext_session_lock_surface_v1, ext_session_lock_v1::{self, ExtSessionLockV1}};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1::{self, Layer, ZwlrLayerShellV1}, zwlr_layer_surface_v1::{self, Anchor}};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

struct AppState {
    session_lock_manager: Option<ExtSessionLockManagerV1>,
    compositor: Option<WlCompositor>,
    output: Option<WlOutput>,
    shm: Option<WlShm>,
    shell: Option<ZwlrLayerShellV1>,
    surface: Option<WlSurface>,
    quit: bool,
}


impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        conn: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        if let wl_registry::Event::Global { name, interface, version} = event {
            
            println!("{}", interface);
            if interface == "wl_compositor" {
                let compositor = registry.bind::<WlCompositor, (), AppState>(name, version, qh, ());
                let surface = compositor.create_surface(qh, ());
                state.compositor = Some(compositor);
                state.surface = Some(surface);
                
            }
            else if interface == "ext_session_lock_manager_v1" {
                let session_lock_manager = registry.bind::<ExtSessionLockManagerV1, (), AppState>(name, version, qh, ());
                session_lock_manager.lock(qh, ());
                state.session_lock_manager = Some(session_lock_manager);
            }
            else if interface == "wl_output" {
                let output = registry.bind::<WlOutput, (), AppState>(name, version, qh, ());
                state.output = Some(output);
            }
            else if interface == "wl_shm" {
                let shm = registry.bind::<WlShm, (), AppState>(name, version, qh, ());
                state.shm = Some(shm);
            }
            else if interface == "zwlr_layer_shell_v1" {
                let shell = registry.bind::<ZwlrLayerShellV1, (), AppState>(name, version, qh, ());
                state.shell = Some(shell);
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
        if let wl_surface::Event::Enter { output } = event {
            println!("surface entered");
        }
        else if let wl_surface::Event::Leave { output } = event {
            println!("surface left");
        }
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        surface: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        event: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {}
}

impl Dispatch<wl_shm::WlShm, ()> for AppState {
    fn event(
        _state: &mut Self,
        surface: &wl_shm::WlShm,
        event: wl_shm::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        if let wl_shm::Event::Format { format } = event {
            println!("format event wl_shm {:?}", format);
        }
    }
}

impl Dispatch<wl_shell::WlShell, ()> for AppState {
    fn event(
        _state: &mut Self,
        surface: &wl_shell::WlShell,
        event: wl_shell::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {}
}

impl Dispatch<wl_output::WlOutput, ()> for AppState {
    fn event(
        _state: &mut Self,
        surface: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {}
}

impl Dispatch<wl_compositor::WlCompositor, ()> for AppState {
    fn event(
        _state: &mut Self,
        surface: &wl_compositor::WlCompositor,
        event: wl_compositor::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {}
}

impl Dispatch<wl_shell_surface::WlShellSurface, ()> for AppState {
    fn event(
        _state: &mut Self,
        surface: &wl_shell_surface::WlShellSurface,
        event: wl_shell_surface::Event,
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
        state: &mut Self,
        lock: &ExtSessionLockV1,
        event: ext_session_lock_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        if let ext_session_lock_v1::Event::Locked = event {
            lock.unlock_and_destroy();
            sleep(Duration::from_millis(1000));
            state.quit = true;
        }
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

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        layer: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure { serial, width, height } = event {
            layer.ack_configure(serial);
            _state.surface.as_ref().expect("could not access the surface").commit();
            
        }
        if let zwlr_layer_surface_v1::Event::Closed = event {
            println!("Closed Layer!");
        }
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for AppState {
    fn event(
        _state: &mut Self,
        layer: &wl_shm_pool::WlShmPool,
        event: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {}
}

impl Dispatch<wl_buffer::WlBuffer, ()> for AppState {
    fn event(
        _state: &mut Self,
        layer: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {}
}

fn main() {
    Args::parse();

    let mut state = AppState {
        session_lock_manager: None,
        compositor: None, 
        output: None, 
        shm: None,
        shell: None,
        surface: None,
        quit: false
    };

    let connection = Connection::connect_to_env().expect("Unable to connect to Wayland environment.");
    let display = connection.display();

    let mut event_queue: EventQueue<AppState> = connection.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());

    event_queue.roundtrip(&mut state).expect("Could not block for compositor events.");

    let surface = state
        .surface.as_ref().expect("could not reach surface");

    let _zwlr_surface = state
        .shell.as_ref().expect("Couldn't connect to wlr_surface")
        .get_layer_surface(&surface, state.output.as_ref(), Layer::Top, String::from_str("bbsl").unwrap(), &qh, ());

    // _zwlr_surface.set_exclusive_zone(10);
    // _zwlr_surface.set_size(0, 0);
    // _zwlr_surface.set_anchor(Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right);
    // _zwlr_surface.set_layer(Layer::Top);
    // _zwlr_surface.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive);
    // _zwlr_surface.set_margin(0, 0, 0, 0);
    // surface.commit();

    let width = 200;
    let height = 200;
    let stride = width * 4;
    let size = stride * height;

    let shm = state.shm.as_ref().expect("couldn't get shared memory pool");
    let fd = memfd_create(CStr::from_bytes_until_nul(b"buffer\0").unwrap(), MemFdCreateFlag::empty()).expect("Couldn't create buffer");
    let buffer = shm.create_pool(fd.as_fd(), size, &qh, ()).create_buffer(0, width, height, stride, wl_shm::Format::Xbgr8888, &qh, ());

    
    surface.attach(Some(&buffer), 0, 0);
    surface.commit();

    // event_queue.roundtrip(&mut state).expect("Could not block for compositor events.");

    while !state.quit {
        let _ = event_queue.blocking_dispatch(&mut state);
    }

}
