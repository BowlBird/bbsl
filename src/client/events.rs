use std::{collections::VecDeque, ffi::CStr, os::fd::AsRawFd, ptr, time::Instant};

use nix::libc::{close, mmap, munmap, MAP_PRIVATE, PROT_READ};
use wayland_client::{protocol::{wl_buffer::{self, WlBuffer}, wl_callback::{self, WlCallback}, wl_compositor::{self, WlCompositor}, wl_keyboard::{self, WlKeyboard}, wl_registry::{self, WlRegistry}, wl_seat::{self, WlSeat}, wl_shm::{self, WlShm}, wl_shm_pool::{self, WlShmPool}, wl_surface::{self, WlSurface}}, Connection, Dispatch, QueueHandle, WEnum};
use wayland_protocols::xdg::shell::client::{xdg_surface::{self, XdgSurface}, xdg_toplevel::{self, XdgToplevel}, xdg_wm_base::{self, XdgWmBase}};
use xkbcommon::xkb::{ffi::{XKB_KEYMAP_COMPILE_NO_FLAGS, XKB_KEYMAP_FORMAT_TEXT_V1}, Keycode, Keymap, State};

use crate::{client::drawing::{attach_to_surface, generate_frame_buffer, FrameBuffer}};

use super::{drawing::Release, AppState, HeldKey, KeyRepeatInfo, Rect};

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
            // _proxy.destroy();
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

            let wl_surface = _state.wl_surface.as_ref().expect("unable to connect to surface");

            render_from_frame_queue(
                &mut _state.frame_buffers,
                _state.drawing_callback,
                wl_surface,
                &_state.dimension,
                _state.shm.as_ref().expect("could not connect to shm"),
                _qhandle
            )

            // let buffer = draw_frame(&_state, _qhandle)
            //     .expect("unable to generate buffer");

            // wl_surface.attach(Some(&buffer), 0, 0);
            // wl_surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
            // wl_surface.commit();
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
            let width = if width == 0 {1} else {width};
            let height = if height == 0 {1} else {height};
            _state.dimension = Some(Rect {width, height});
            _state.frame_buffers.iter().for_each(|buffer| {
                buffer.release();
            });
            _state.frame_buffers.clear();
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

            render_from_frame_queue(
                &mut _state.frame_buffers,
                _state.drawing_callback,
                wl_surface,
                &_state.dimension,
                _state.shm.as_ref().expect("could not connect to shm"),
                _qhandle
            )
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
        if let wl_keyboard::Event::Keymap { format, fd, size } = _event {
            let map_shm = unsafe {
                mmap(ptr::null_mut(), size as usize, PROT_READ, MAP_PRIVATE, fd.as_raw_fd(), 0)
            };
            let map_shm_string = unsafe {
                CStr::from_ptr(map_shm as *const _)
                    .to_string_lossy().into_owned()
            };
            _state.xkb_keymap = Keymap::new_from_string(
                &_state.xkb_context.as_ref()
                    .expect("could not connect to xkb_context"),
                    map_shm_string, XKB_KEYMAP_FORMAT_TEXT_V1, XKB_KEYMAP_COMPILE_NO_FLAGS
            );
            _state.xkb_state = Some(State::new(&_state.xkb_keymap.as_ref().unwrap()));
            unsafe {munmap(map_shm, size as usize);};
            unsafe {close(fd.as_raw_fd());};
        }

        else if let wl_keyboard::Event::Key { serial, time, key, state } = _event {
            match state {
                WEnum::Value(state) => {

                    let keysym = _state.xkb_state.as_ref()
                        .expect("could not connect to xkb_state")
                        .key_get_one_sym(Keycode::new((key + 8) as u32));

                    if let wl_keyboard::KeyState::Pressed = state {
                        (_state.keyboard_callback)(keysym);
                        _state.held_key = Some(HeldKey { keysym, instant: Instant::now(), repeat_count: 0 });
                    }
                    else if let wl_keyboard::KeyState::Released = state {
                        match &_state.held_key {
                            Some(held_key) => {
                                if held_key.keysym == keysym {
                                    _state.held_key = None;
                                }
                            }
                            None => {}
                        }
                    }
                },
                WEnum::Unknown(_) => {}
            }
            
        }
        else if let wl_keyboard::Event::Modifiers { serial, mods_depressed, mods_latched, mods_locked, group } = _event {
            _state.held_key = None;
            match &_state.xkb_state {
                Some(_) => {_state.xkb_state.as_mut().unwrap().update_mask(
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
        else if let wl_keyboard::Event::RepeatInfo { rate, delay } = _event {
            _state.key_repeat_info = Some(KeyRepeatInfo { rate, delay});
        }
        /* enumerate keys that were already pressed while entering */
        else if let wl_keyboard::Event::Enter { serial, surface, keys } = _event {
            keys.iter().for_each(|key| {
                (_state.keyboard_callback)(
                    _state.xkb_state.as_ref()
                        .expect("could not connect to xkb_state")
                        .key_get_one_sym(Keycode::new((key + 8) as u32))
                );
            })
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

fn render_from_frame_queue(
    frame_buffers: &mut VecDeque<FrameBuffer>,
    drawing_callback: fn(&FrameBuffer), 
    wl_surface: &WlSurface, 
    dimension: &Option<Rect>,
    wl_shm: &WlShm,
    qh: &QueueHandle<AppState>
) {
    if frame_buffers.len() == 2 {
        drawing_callback(&frame_buffers[0]);
        attach_to_surface(Some(&frame_buffers[0]), wl_surface);
        frame_buffers.swap(0, 1);
    }
    else {
        match &dimension {
            Some(dimension) => {
                let frame_buffer = generate_frame_buffer(&dimension, wl_shm, qh)
                    .expect("could not generate frame_buffer!");
                drawing_callback(&frame_buffer);
                attach_to_surface(Some(&frame_buffer), wl_surface);
                frame_buffers.push_back(frame_buffer);
            }
            None => attach_to_surface(None, wl_surface)
        }
    }
}