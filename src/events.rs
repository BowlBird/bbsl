use wayland_client::{protocol::{wl_buffer::{self, WlBuffer}, wl_callback::{self, WlCallback}, wl_compositor::{self, WlCompositor}, wl_registry::{self, WlRegistry}, wl_shm::{self, WlShm}, wl_shm_pool::{self, WlShmPool}, wl_surface::{self, WlSurface}}, Connection, Dispatch};
use wayland_protocols::xdg::shell::client::{xdg_surface::{self, XdgSurface}, xdg_toplevel::{self, XdgToplevel}, xdg_wm_base::{self, XdgWmBase}};

use crate::{drawing::draw_frame, AppState, Rect};

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
        if let xdg_toplevel::Event::Configure { width, height, states: _ } = _event {
            _state.dimension = Some(Rect {width, height})
        }
        else if let xdg_toplevel::Event::Close = _event {
            _state.quit = true;
        }
    }
}

impl Dispatch<WlCallback, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlCallback,
        _event: wl_callback::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        /* callback_data is the current frame time, this can be used for animations. */
        if let wl_callback::Event::Done { callback_data: _ } = _event {
            
            let wl_surface = _state
                .wl_surface.as_ref()
                .expect("cannot connect to wl_surface");
            let _ = wl_surface.frame(_qhandle, ());

            let buffer = draw_frame(_state, _qhandle)
                .expect("could not draw frame");

            wl_surface.attach(Some(&buffer), 0, 0);
            wl_surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
            wl_surface.commit();
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
            // println!("{}", interface);
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