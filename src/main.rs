// https://wayland-book.com/
use std::{os::{fd::BorrowedFd, raw::c_void}, ptr, str::FromStr};

use nix::libc::{close, ftruncate, mmap, shm_open, MAP_FAILED, MAP_SHARED, O_CREAT, O_EXCL, O_RDWR, PROT_READ, PROT_WRITE};
use rand::{distributions::Alphanumeric, Rng};
use wayland_client::{protocol::{wl_buffer::{self, WlBuffer}, wl_compositor::{self, WlCompositor}, wl_registry::{self, WlRegistry}, wl_shm::{self, WlShm}, wl_shm_pool::{self, WlShmPool}, wl_surface::{self, WlSurface}}, Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols::xdg::shell::client::{xdg_surface::{self, XdgSurface}, xdg_toplevel::{self, XdgToplevel}, xdg_wm_base::{self, XdgWmBase}};

struct AppState {
    compositor: Option<WlCompositor>,
    shm: Option<WlShm>,
    base: Option<XdgWmBase>,
    wl_surface: Option<WlSurface>
}

impl Dispatch<WlCompositor, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlCompositor,
        _event: wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {}
}

impl Dispatch<WlSurface, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        _event: wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {}
}

impl Dispatch<WlShm, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {}
}

impl Dispatch<WlShmPool, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {}
}

impl Dispatch<WlBuffer, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_buffer::Event::Release = _event {
            _proxy.destroy();
        }
    }
}

impl Dispatch<XdgWmBase, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &XdgWmBase,
        _event: xdg_wm_base::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = _event {
            _proxy.pong(serial);
        }
    }
}

impl Dispatch<XdgSurface, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &XdgSurface,
        _event: xdg_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = _event {
            _proxy.ack_configure(serial);

            let buffer = draw_frame(&_state, _qhandle)
                .expect("unable to generate buffer");

            let wl_surface = _state.wl_surface.as_ref().expect("unable to connect to surface");
            wl_surface.attach(Some(&buffer), 0, 0);
            wl_surface.commit();
        }
    }
}

impl Dispatch<XdgToplevel, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &XdgToplevel,
        _event: xdg_toplevel::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Configure { width, height, states } = _event {
        }
    }
}

impl Dispatch<WlRegistry, ()> for AppState {
    fn event(
            _state: &mut Self,
            _proxy: &WlRegistry,
            _event: wl_registry::Event,
            _data: &(),
            _conn: &Connection,
            _qhandle: &wayland_client::QueueHandle<Self>,
        ) {
        if let wl_registry::Event::Global { name, interface, version } = _event {
            if interface == "wl_compositor" {
                _state.compositor = Some(_proxy.bind::<WlCompositor, (), AppState>(name, version, _qhandle, ()))
            }
            else if interface == "wl_shm" {
                _state.shm = Some(_proxy.bind::<WlShm, (), AppState>(name, version, _qhandle, ()))
            }
            else if interface == "xdg_wm_base" {
                _state.base = Some(_proxy.bind::<XdgWmBase, (), AppState>(name, version, _qhandle, ()));
            }
        }
    }
}

fn draw_frame(state: &AppState, qh: &QueueHandle<AppState>) -> Result<WlBuffer, ()> {

    let width = 640;
    let height = 480;
    let stride = width * 4;
    let shm_pool_size = height * stride;

    let fd = unsafe {
        let random: String = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();
        let name = format!("/buffer-{}\0", random);
        shm_open(name.as_ptr() as *const i8, O_RDWR | O_CREAT | O_EXCL, 0600)
        // syscall(SYS_memfd_create, "buffer", 0) as RawFd
    };
    
    if fd == -1 {
        return Err(())
    }

    unsafe {
        let result = ftruncate(fd, shm_pool_size as i64);
        if result == -1 {
            return Err(())
        }
    }


    let _pool_data = unsafe {
        mmap(ptr::null_mut() as *mut c_void, shm_pool_size as usize, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0)
    };

    if _pool_data == MAP_FAILED {
        return Err(());
    }

    let pool = state.shm.as_ref().unwrap().create_pool(unsafe {BorrowedFd::borrow_raw(fd) }, shm_pool_size as i32, &qh, ());

    let index = 0;
    let offset = height * stride * index;

    let buffer = pool.create_buffer(
        offset as i32, 
        width as i32, 
        height as i32, 
        stride as i32, 
        wl_shm::Format::Xrgb8888, 
        qh, 
        ()
    );
    pool.destroy();
    unsafe {close(fd)};

    return Ok(buffer);
}


fn main() {

    let mut state = AppState {
        compositor: None,
        shm: None,
        base: None,
        wl_surface: None,
    };

    let connection = Connection::connect_to_env()
        .expect("could not connect to env");

    let display = connection.display();

    let mut event_queue: EventQueue<AppState> = connection.new_event_queue();
    let qh = event_queue.handle();
    display.get_registry(&qh, ());
    let _ = event_queue.roundtrip(&mut state);

    state.wl_surface = Some(state
        .compositor.as_ref()
        .expect("could not connect to compositor")
        .create_surface(&qh, ()));
    
    let wl_surface = state
        .wl_surface.as_ref()
        .expect("could not connect to wl_surface");

    let xdg_surface = state
        .base.as_ref()
        .expect("could not connect to xdg_wm_base")
        .get_xdg_surface(wl_surface, &qh, ());

    let _xdg_toplevel = xdg_surface.get_toplevel(&qh, ());
    _xdg_toplevel.set_title(String::from_str("bbsl").expect("could not create string"));
    
    wl_surface.commit();

    loop {
        let _ = event_queue.blocking_dispatch(&mut state);
    }
}
