//! O receptor contra um KWin DE VERDADE: outro cliente arrasta dois ficheiros pela `text/uri-list`
//! e larga-os sobre a janela-alvo. Sem placa: duas superfícies `layer-shell` de cor lisa, e o
//! rato é o `org_kde_kwin_fake_input` da sessão virtual — nunca o rato do dono.
//!
//! Corre só dentro de um `kwin_wayland --virtual`: `docs/MiroClone/ferramentas/soltar_wayland/corre.sh`.
#![cfg(target_os = "linux")]
#![allow(unsafe_code)]

use std::ffi::c_void;
use std::io::Write;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::time::{Duration, Instant};

use ph2d_wayland_dnd::{DndEvent, DropReceiver, URI_LIST_MIME};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};
use smithay_client_toolkit::compositor::{CompositorHandler, CompositorState};
use smithay_client_toolkit::data_device_manager::data_device::{DataDevice, DataDeviceHandler};
use smithay_client_toolkit::data_device_manager::data_offer::{DataOfferHandler, DragOffer};
use smithay_client_toolkit::data_device_manager::data_source::{DataSourceHandler, DragSource};
use smithay_client_toolkit::data_device_manager::{DataDeviceManagerState, WritePipe};
use smithay_client_toolkit::output::{OutputHandler, OutputState};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::seat::pointer::{PointerEvent, PointerEventKind, PointerHandler};
use smithay_client_toolkit::seat::{Capability, SeatHandler, SeatState};
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::{
    Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
};
use smithay_client_toolkit::shm::slot::SlotPool;
use smithay_client_toolkit::shm::{Shm, ShmHandler};
use smithay_client_toolkit::{
    delegate_compositor, delegate_data_device, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm, registry_handlers,
};
use wayland_client::globals::{GlobalList, registry_queue_init};
use wayland_client::protocol::wl_data_device::WlDataDevice;
use wayland_client::protocol::wl_data_device_manager::DndAction;
use wayland_client::protocol::wl_data_source::WlDataSource;
use wayland_client::protocol::wl_output::{Transform, WlOutput};
use wayland_client::protocol::wl_pointer::WlPointer;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::protocol::wl_shm::Format;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols_plasma::fake_input::client::org_kde_kwin_fake_input::OrgKdeKwinFakeInput;

const BTN_LEFT: u32 = 0x110;
const SOURCE_SIDE: u32 = 64;
const WAIT: Duration = Duration::from_secs(5);
const LIST: &[u8] = b"file:///tmp/ph2d%20dnd/%C3%A1gua.png\nfile:///tmp/ph2d%20dnd/b.png\n";

fn expected() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/tmp/ph2d dnd/água.png"),
        PathBuf::from("/tmp/ph2d dnd/b.png"),
    ]
}

#[derive(Default)]
struct Seen {
    configured: Option<(u32, u32)>,
    entered: bool,
    press_serial: Option<u32>,
    sends: usize,
    dropped: bool,
    finished: bool,
    cancelled: bool,
}

struct Client {
    registry: RegistryState,
    seats: SeatState,
    outputs: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer: LayerSurface,
    manager: DataDeviceManagerState,
    device: Option<DataDevice>,
    pointer: Option<WlPointer>,
    drag: Option<DragSource>,
    list: Vec<u8>,
    seen: Seen,
}

struct Side {
    conn: Connection,
    queue: EventQueue<Client>,
    globals: GlobalList,
    state: Client,
}

impl Side {
    /// `full`: a janela-alvo, ancorada às quatro bordas; senão a origem, 64×64 no canto, por cima.
    fn connect(full: bool) -> Self {
        let conn = Connection::connect_to_env().expect("WAYLAND_DISPLAY of the virtual KWin");
        let (globals, queue) = registry_queue_init::<Client>(&conn).expect("registry");
        let qh = queue.handle();
        let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("zwlr_layer_shell_v1");
        let shm = Shm::bind(&globals, &qh).expect("wl_shm");
        let manager = DataDeviceManagerState::bind(&globals, &qh).expect("wl_data_device_manager");
        let surface = compositor.create_surface(&qh);
        let (layer_kind, anchor, size) = if full {
            (Layer::Bottom, Anchor::all(), (0, 0))
        } else {
            (
                Layer::Overlay,
                Anchor::TOP | Anchor::LEFT,
                (SOURCE_SIDE, SOURCE_SIDE),
            )
        };
        let layer =
            layer_shell.create_layer_surface(&qh, surface, layer_kind, Some("ph2d-dnd"), None);
        layer.set_anchor(anchor);
        layer.set_size(size.0, size.1);
        layer.commit();
        let pool = SlotPool::new(4096, &shm).expect("shm pool");
        let state = Client {
            registry: RegistryState::new(&globals),
            seats: SeatState::new(&globals, &qh),
            outputs: OutputState::new(&globals, &qh),
            shm,
            pool,
            layer,
            manager,
            device: None,
            pointer: None,
            drag: None,
            list: LIST.to_vec(),
            seen: Seen::default(),
        };
        let mut side = Self {
            conn,
            queue,
            globals,
            state,
        };
        side.until(
            |c| c.seen.configured.is_some(),
            "the layer surface is configured",
        );
        // O `commit` com o buffer nasce DENTRO do `configure`: sem outra volta ele não sai.
        side.pump();
        side
    }

    fn pump(&mut self) {
        self.queue.roundtrip(&mut self.state).expect("roundtrip");
    }

    fn until(&mut self, done: impl Fn(&Client) -> bool, what: &str) {
        let end = Instant::now() + WAIT;
        while !done(&self.state) {
            assert!(Instant::now() < end, "timed out waiting until {what}");
            self.pump();
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn handles(&self) -> Handles {
        let display = NonNull::new(self.conn.backend().display_ptr().cast::<c_void>()).unwrap();
        let surface =
            NonNull::new(self.state.layer.wl_surface().id().as_ptr().cast::<c_void>()).unwrap();
        Handles { display, surface }
    }
}

struct Handles {
    display: NonNull<c_void>,
    surface: NonNull<c_void>,
}

impl HasDisplayHandle for Handles {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let raw = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(self.display));
        // SAFETY: a ligação da janela-alvo vive mais que o receptor (largado antes, no teste).
        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}

impl HasWindowHandle for Handles {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let raw = RawWindowHandle::Wayland(WaylandWindowHandle::new(self.surface));
        // SAFETY: idem, a superfície da janela-alvo.
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}

struct Pointer {
    fake: OrgKdeKwinFakeInput,
}

impl Pointer {
    fn bind(source: &mut Side) -> Self {
        let qh = source.queue.handle();
        let fake: OrgKdeKwinFakeInput = source
            .globals
            .bind(&qh, 3..=4, ())
            .expect("org_kde_kwin_fake_input (KWIN_WAYLAND_NO_PERMISSION_CHECKS=1)");
        fake.authenticate(
            "ph2d-wayland-dnd test".into(),
            "drive a drag in a virtual session".into(),
        );
        // A sessão virtual não tem rato: o seat só ganha um ao primeiro movimento falso.
        fake.pointer_motion_absolute(1.0, 1.0);
        source.until(|c| c.pointer.is_some(), "the seat gains a pointer");
        Self { fake }
    }

    fn to(&self, source: &mut Side, x: f64, y: f64) {
        self.fake.pointer_motion_absolute(x, y);
        source.pump();
    }

    fn button(&self, source: &mut Side, pressed: bool) {
        self.fake.button(BTN_LEFT, u32::from(pressed));
        source.pump();
    }
}

/// Os eventos do receptor até `done`, a bombear a ORIGEM (é ela que escreve a lista no `send`).
fn collect(
    receiver: &DropReceiver,
    source: &mut Side,
    done: impl Fn(&[DndEvent]) -> bool,
    what: &str,
) -> Vec<DndEvent> {
    let end = Instant::now() + WAIT;
    let mut events = Vec::new();
    loop {
        events.extend(receiver.drain());
        if done(&events) {
            return events;
        }
        assert!(
            Instant::now() < end,
            "timed out waiting for {what}; got {events:?}"
        );
        source.pump();
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn start_drag(source: &mut Side, pointer: &Pointer) {
    source.state.seen = Seen {
        configured: source.state.seen.configured,
        ..Seen::default()
    };
    let half = f64::from(SOURCE_SIDE) / 2.0;
    let end = Instant::now() + WAIT;
    while !source.state.seen.entered {
        assert!(
            Instant::now() < end,
            "timed out waiting until the pointer enters the source"
        );
        pointer.to(source, half + 1.0, half);
        pointer.to(source, half, half);
        std::thread::sleep(Duration::from_millis(20));
    }
    pointer.button(source, true);
    source.until(
        |c| c.seen.press_serial.is_some(),
        "the source sees its own press",
    );
    let qh = source.queue.handle();
    let s = &mut source.state;
    let drag = s
        .manager
        .create_drag_and_drop_source(&qh, [URI_LIST_MIME], DndAction::Copy);
    drag.start_drag(
        s.device.as_ref().expect("data device"),
        s.layer.wl_surface(),
        None,
        s.seen.press_serial.unwrap(),
    );
    s.drag = Some(drag);
    source.pump();
}

#[test]
#[ignore = "needs a virtual KWin: docs/MiroClone/ferramentas/soltar_wayland/corre.sh"]
fn a_drag_from_another_client_lands_its_files_where_it_was_released() {
    let target = Side::connect(true);
    let (w, h) = target.state.seen.configured.unwrap();
    let mut source = Side::connect(false);
    let pointer = Pointer::bind(&mut source);
    let receiver = DropReceiver::attach(&target.handles(), || {})
        .expect("attach")
        .expect("a Wayland window");

    // 1. Passa por cima e volta para a origem: o aviso aparece com os nomes e some, sem largar.
    start_drag(&mut source, &pointer);
    pointer.to(&mut source, f64::from(w) / 3.0, f64::from(h) / 3.0);
    let ev = collect(
        &receiver,
        &mut source,
        |e| e.iter().any(|e| matches!(e, DndEvent::Hover { .. })),
        "a Hover",
    );
    let Some(DndEvent::Hover { paths, .. }) =
        ev.iter().find(|e| matches!(e, DndEvent::Hover { .. }))
    else {
        unreachable!()
    };
    assert_eq!(
        paths,
        &expected(),
        "the overlay names the files being dragged"
    );
    pointer.to(&mut source, 8.0, 8.0);
    collect(
        &receiver,
        &mut source,
        |e| e.contains(&DndEvent::Leave),
        "a Leave",
    );
    pointer.button(&mut source, false);
    source.until(
        |c| c.seen.cancelled,
        "the drag released on its own origin is cancelled",
    );

    // 2. Larga no meio da janela-alvo: os dois ficheiros, na posição onde se largou.
    start_drag(&mut source, &pointer);
    let mid = (f64::from(w) / 2.0, f64::from(h) / 2.0);
    pointer.to(&mut source, mid.0 - 40.0, mid.1 - 30.0);
    collect(
        &receiver,
        &mut source,
        |e| e.iter().any(|e| matches!(e, DndEvent::Hover { .. })),
        "a Hover",
    );
    pointer.to(&mut source, mid.0, mid.1);
    pointer.button(&mut source, false);
    let ev = collect(
        &receiver,
        &mut source,
        |e| e.iter().any(|e| matches!(e, DndEvent::Drop { .. })),
        "a Drop",
    );
    let Some(DndEvent::Drop { list, at }) = ev.iter().find(|e| matches!(e, DndEvent::Drop { .. }))
    else {
        unreachable!()
    };
    assert_eq!(list.paths, expected());
    assert_eq!(list.not_files, 0);
    assert_eq!(
        *at, mid,
        "the drop lands where the pointer was released, in surface coordinates"
    );
    source.until(
        |c| c.seen.finished,
        "the receiver finishes the offer (wl_data_offer.finish)",
    );
    assert!(source.state.seen.dropped && !source.state.seen.cancelled);
    assert_eq!(
        source.state.seen.sends, 1,
        "one read per offer: the drop reuses the list read at enter"
    );

    drop(receiver);
}

/// A ORIGEM só, para o smoke com o app inteiro: arrasta os ficheiros de `PH2D_DND_FILES` (um caminho
/// por linha) até o meio do ecrã e larga — o app maximizado está lá.
#[test]
#[ignore = "driven by docs/MiroClone/ferramentas/soltar_wayland/no_app.sh"]
fn drag_the_env_files_onto_the_screen_centre() {
    let files = std::env::var("PH2D_DND_FILES").expect("PH2D_DND_FILES");
    let mut list = Vec::new();
    for f in files.lines().filter(|l| !l.is_empty()) {
        let uri = percent_encoding::utf8_percent_encode(f, percent_encoding::NON_ALPHANUMERIC)
            .to_string()
            .replace("%2F", "/");
        list.extend_from_slice(format!("file://{uri}\r\n").as_bytes());
    }
    let mut source = Side::connect(false);
    source.state.list = list;
    let pointer = Pointer::bind(&mut source);
    start_drag(&mut source, &pointer);
    let (w, h) = (1280.0, 800.0);
    for step in 1..=10 {
        let t = f64::from(step) / 10.0;
        pointer.to(
            &mut source,
            32.0 + t * (w / 2.0 - 32.0),
            32.0 + t * (h / 2.0 - 32.0),
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    pointer.button(&mut source, false);
    source.until(
        |c| c.seen.finished || c.seen.cancelled,
        "the app takes or refuses the drop",
    );
    assert!(source.state.seen.finished, "the app finished the drop");
}

impl CompositorHandler for Client {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: i32,
    ) {
    }
    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: Transform,
    ) {
    }
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlSurface, _: u32) {}
    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: &WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: &WlOutput,
    ) {
    }
}

impl OutputHandler for Client {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.outputs
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlOutput) {}
}

impl LayerShellHandler for Client {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {}

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        layer: &LayerSurface,
        c: LayerSurfaceConfigure,
        _: u32,
    ) {
        let (w, h) = (c.new_size.0.max(1), c.new_size.1.max(1));
        let stride = w as i32 * 4;
        let (buffer, canvas) = self
            .pool
            .create_buffer(w as i32, h as i32, stride, Format::Argb8888)
            .expect("buffer");
        canvas.fill(0xff);
        buffer.attach_to(layer.wl_surface()).expect("attach");
        layer.wl_surface().damage_buffer(0, 0, w as i32, h as i32);
        layer.commit();
        self.seen.configured = Some((w, h));
    }
}

impl ShmHandler for Client {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl SeatHandler for Client {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seats
    }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}
    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        cap: Capability,
    ) {
        if self.device.is_none() {
            self.device = Some(self.manager.get_data_device(qh, &seat));
        }
        if cap == Capability::Pointer && self.pointer.is_none() {
            self.pointer = Some(self.seats.get_pointer(qh, &seat).expect("pointer"));
        }
    }
    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: WlSeat,
        _: Capability,
    ) {
    }
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}
}

impl PointerHandler for Client {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlPointer,
        events: &[PointerEvent],
    ) {
        for e in events {
            match e.kind {
                PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. } => {
                    self.seen.entered = true
                }
                PointerEventKind::Press { serial, .. } => self.seen.press_serial = Some(serial),
                _ => {}
            }
        }
    }
}

impl DataDeviceHandler for Client {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataDevice,
        _: f64,
        _: f64,
        _: &WlSurface,
    ) {
    }
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}
    fn motion(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice, _: f64, _: f64) {}
    fn selection(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}
    fn drop_performed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}
}

impl DataOfferHandler for Client {
    fn source_actions(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        _: DndAction,
    ) {
    }
    fn selected_action(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        _: DndAction,
    ) {
    }
}

impl DataSourceHandler for Client {
    fn accept_mime(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataSource,
        _: Option<String>,
    ) {
    }
    fn send_request(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataSource,
        mime: String,
        mut fd: WritePipe,
    ) {
        assert_eq!(mime, URI_LIST_MIME);
        self.seen.sends += 1;
        fd.write_all(&self.list).expect("write the uri-list");
    }
    fn cancelled(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {
        self.seen.cancelled = true;
    }
    fn dnd_dropped(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {
        self.seen.dropped = true;
    }
    fn dnd_finished(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {
        self.seen.finished = true;
    }
    fn action(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource, _: DndAction) {}
}

impl ProvidesRegistryState for Client {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers![OutputState, SeatState];
}

impl Dispatch<OrgKdeKwinFakeInput, ()> for Client {
    fn event(
        _: &mut Self,
        _: &OrgKdeKwinFakeInput,
        _: <OrgKdeKwinFakeInput as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

delegate_compositor!(Client);
delegate_output!(Client);
delegate_layer!(Client);
delegate_shm!(Client);
delegate_seat!(Client);
delegate_pointer!(Client);
delegate_data_device!(Client);
delegate_registry!(Client);
