mod events;
mod drawing;

pub use drawing::FrameBuffer;

use std::{collections::{HashMap, VecDeque}, hash::Hash, os::raw::c_void, str::FromStr, thread::sleep, time::{Duration, Instant}};

use derive_builder::Builder;
use wayland_client::{protocol::{wl_compositor::WlCompositor, wl_keyboard::WlKeyboard, wl_seat::WlSeat, wl_shm::WlShm, wl_surface::WlSurface}, Connection, EventQueue};
use wayland_protocols::xdg::shell::client::{xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel, xdg_wm_base::XdgWmBase};
use xkbcommon::xkb::{ffi::XKB_CONTEXT_NO_FLAGS, Context, Keymap, Keysym, State};

#[derive(Clone)]
pub struct Rect {
    pub width: i32,
    pub height: i32
}

#[derive(Clone)]
pub struct KeyRepeatInfo {
    delay: i32,
    rate: i32,
}

#[derive(Clone)]
pub struct HeldKey {
    keysym: Keysym,
    instant: Instant,
    repeat_count: i32,
}

#[derive(Clone)]
pub struct AppState {
    compositor: Option<WlCompositor>,
    shm: Option<WlShm>,
    base: Option<XdgWmBase>,
    wl_surface: Option<WlSurface>,
    xdg_surface: Option<XdgSurface>,
    xdg_toplevel: Option<XdgToplevel>,
    wl_seat: Option<WlSeat>,
    wl_keyboard: Option<WlKeyboard>,
    xkb_context: Option<Context>,
    xkb_keymap: Option<Keymap>,
    xkb_state: Option<State>,
    dimension: Option<Rect>,
    frame_buffers: VecDeque<FrameBuffer>,
    drawing_callback: fn(&AppState, &FrameBuffer),
    keyboard_callback: fn(&mut AppState, Keysym),
    key_repeat_info: Option<KeyRepeatInfo>,
    held_key: Option<HeldKey>,
    pub quit: bool
}

#[derive(Builder)]
#[builder(pattern = "immutable")]
pub struct WaylandClient {
    drawing_callback: fn(&AppState, &FrameBuffer),
    keyboard_callback: fn(&mut AppState, Keysym),
    #[builder(setter(skip))]
    state: Option<AppState>,
}

fn key_to_repeat<'a>(held_key: &'a Option<HeldKey>, key_repeat_info: &'a Option<KeyRepeatInfo>) -> Option<&'a HeldKey> {
    match key_repeat_info {
        Some(key_repeat_info) => {
            let now = Instant::now();

            if let Some(held_key) = held_key {
                if 0 < now.duration_since(held_key.instant).as_millis() as i128 - (key_repeat_info.delay + key_repeat_info.rate * held_key.repeat_count) as i128 {
                    return Some(held_key)
                }
            }
            None
        }
        None => {
            None
        }
    }
}

fn key_repeat_update<'a>(state: &mut AppState){
    let repeat_key = key_to_repeat(&state.held_key, &state.key_repeat_info);

    if let Some(repeat_key) = repeat_key {
        (state.keyboard_callback)(state, repeat_key.keysym);
        
        match &mut state.held_key {
            Some(held_key) => {
                held_key.repeat_count += 1;
            }
            None => {}
        }
    }
}

impl WaylandClient {
    pub fn start(&mut self) {

        self.state = Some(AppState {
            compositor: None,
            shm: None,
            base: None,
            wl_surface: None,
            xdg_surface: None,
            xdg_toplevel: None,
            wl_seat: None,
            wl_keyboard: None,
            xkb_context: None, 
            xkb_keymap: None,
            xkb_state: None,
            dimension: None,
            frame_buffers: VecDeque::new(),
            drawing_callback: self.drawing_callback,
            keyboard_callback: self.keyboard_callback,
            key_repeat_info: None,
            held_key: None,
            quit: false,
        });
        let mut state = self.state.as_mut().unwrap();
    
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
        
        while !&state.quit {
            let _ = event_queue.blocking_dispatch(&mut state);
            key_repeat_update(state);
        }
    }
}

