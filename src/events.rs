use std::{ffi::CStr, os::fd::AsRawFd, ptr};

use nix::libc::{close, mmap, munmap, MAP_PRIVATE, PROT_READ};
use wayland_client::{protocol::{wl_buffer::{self, WlBuffer}, wl_callback::{self, WlCallback}, wl_compositor::{self, WlCompositor}, wl_keyboard::{self, WlKeyboard}, wl_registry::{self, WlRegistry}, wl_seat::{self, WlSeat}, wl_shm::{self, WlShm}, wl_shm_pool::{self, WlShmPool}, wl_surface::{self, WlSurface}}, Connection, Dispatch};
use wayland_protocols::xdg::shell::client::{xdg_surface::{self, XdgSurface}, xdg_toplevel::{self, XdgToplevel}, xdg_wm_base::{self, XdgWmBase}};
use xkbcommon::xkb::{ffi::{XKB_KEYMAP_COMPILE_NO_FLAGS, XKB_KEYMAP_FORMAT_TEXT_V1}, Keycode, Keymap, State};

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

impl Dispatch<WlSeat, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {}
}

impl Dispatch<WlKeyboard, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlKeyboard,
        _event: wl_keyboard::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        let mut xkb_keymap = None;
        let mut xkb_state = None;
        if let wl_keyboard::Event::Keymap { format, fd, size } = _event {
            let map_shm = unsafe {
                mmap(ptr::null_mut(), size as usize, PROT_READ, MAP_PRIVATE, fd.as_raw_fd(), 0)
            };
            let map_shm_string = unsafe {
                CStr::from_ptr(map_shm as *const _)
                    .to_string_lossy().into_owned()
            };
            xkb_keymap = Keymap::new_from_string(
                &_state.xkb_context.as_ref()
                    .expect("could not connect to xkb_context"),
                    map_shm_string, XKB_KEYMAP_FORMAT_TEXT_V1, XKB_KEYMAP_COMPILE_NO_FLAGS);
            xkb_state = Some(State::new(&xkb_keymap.unwrap()));

            unsafe {munmap(map_shm, size as usize);};
            unsafe {close(fd.as_raw_fd());};
        }

        else if let wl_keyboard::Event::Key { serial, time, key, state } = _event {
            match xkb_state {
                Some(xkb_state) => {
                    let sym = xkb_state.key_get_one_sym(Keycode::new(key));
                    println!("{:?}", sym);
                }
                None => {}
            }
            
        }

        else if let wl_keyboard::Event::Modifiers { serial, mods_depressed, mods_latched, mods_locked, group } = _event {
            match xkb_state {
                Some(mut xkb_state) => {xkb_state.update_mask(
                    mods_depressed,
                    mods_latched,
                    mods_locked,
                    0,
                    0,
                    group
                );}
                None => {}
            }
            
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
                _state.compositor = Some(_proxy.bind::<WlCompositor, (), AppState>(name, version, _qhandle, ()));
            }
            else if interface == "wl_shm" {
                _state.shm = Some(_proxy.bind::<WlShm, (), AppState>(name, version, _qhandle, ()));
            }
            else if interface == "xdg_wm_base" {
                _state.base = Some(_proxy.bind::<XdgWmBase, (), AppState>(name, version, _qhandle, ()));
            }
            else if interface == "wl_seat" {
                _state.wl_seat = Some(_proxy.bind::<WlSeat, (), AppState>(name, version, _qhandle, ()));
            }
        }
    }
}