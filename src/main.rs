use wayland_client::{protocol::wl_registry::{self, WlRegistry}, Connection, Dispatch, EventQueue, QueueHandle};

struct AppState;

impl Dispatch<WlRegistry, ()> for AppState {
    fn event(
            _state: &mut Self,
            _proxy: &WlRegistry,
            _event: <WlRegistry as wayland_client::Proxy>::Event,
            _data: &(),
            _conn: &Connection,
            _qhandle: &wayland_client::QueueHandle<Self>,
        ) {
        if let wl_registry::Event::Global { name, interface, version } = _event {
            println!("{}", interface);
        }
    }
}


fn main() {
    let connection = Connection::connect_to_env()
        .expect("could not connect to env");

    let display = connection.display();
    let event_queue: EventQueue<AppState> = connection.new_event_queue();
    let qh = event_queue.
    display.get_registry(qh, ())

    let _ = event_queue.roundtrip(&mut AppState);
}
