//! O fio que fala com o compositor: a MESMA `wl_display` do winit, fila e fio próprios (o padrão
//! do `smithay-clipboard`). O KWin entrega `enter`/`motion`/`drop` a todos os `wl_data_device` do
//! cliente no seat, e o winit 0.30 não tem nenhum — este é o único.
#![allow(unsafe_code)]

use std::collections::BTreeMap;
use std::ffi::c_void;
use std::fmt;
use std::os::fd::{AsFd, OwnedFd};
use std::sync::mpsc::{self, Receiver, Sender, TryIter};
use std::thread::JoinHandle;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use rustix::event::{PollFd, PollFlags};
use rustix::io::Errno;
use smithay_client_toolkit::data_device_manager::DataDeviceManagerState;
use smithay_client_toolkit::data_device_manager::data_device::{
    DataDevice, DataDeviceData, DataDeviceHandler,
};
use smithay_client_toolkit::data_device_manager::data_offer::{
    DataOfferData, DataOfferHandler, DragOffer,
};
use smithay_client_toolkit::globals::GlobalData;
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::seat::{Capability, SeatHandler, SeatState};
use smithay_client_toolkit::{delegate_registry, delegate_seat, registry_handlers};
use wayland_backend::client::{Backend, ObjectId, WaylandError};
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::wl_data_device::WlDataDevice;
use wayland_client::protocol::wl_data_device_manager::{DndAction, WlDataDeviceManager};
use wayland_client::protocol::wl_data_offer::WlDataOffer;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, EventQueue, Proxy, QueueHandle, delegate_dispatch};

use crate::URI_LIST_MIME;
use crate::session::{DndEvent, Effect, Session, Token};

#[derive(Debug)]
pub enum AttachError {
    Handle(raw_window_handle::HandleError),
    Io(std::io::Error),
    Wayland(String),
}

impl fmt::Display for AttachError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Handle(e) => write!(f, "window handle: {e}"),
            Self::Io(e) => write!(f, "io: {e}"),
            Self::Wayland(e) => write!(f, "wayland: {e}"),
        }
    }
}

impl std::error::Error for AttachError {}

impl From<raw_window_handle::HandleError> for AttachError {
    fn from(e: raw_window_handle::HandleError) -> Self {
        Self::Handle(e)
    }
}

impl From<std::io::Error> for AttachError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<Errno> for AttachError {
    fn from(e: Errno) -> Self {
        Self::Io(e.into())
    }
}

/// Os ficheiros largados numa janela Wayland, como [`DndEvent`]s.
///
/// ⚠️ Largue-o ANTES de o laço do winit acabar (no `exiting`): o fio usa a `wl_display` dele.
pub struct DropReceiver {
    events: Receiver<DndEvent>,
    stop: Option<OwnedFd>,
    thread: Option<JoinHandle<()>>,
}

/// A `wl_display` do winit, levada ao fio: a libwayland é segura entre fios.
struct Display(*mut c_void);
// SAFETY: a `wl_display` é thread-safe; o fio só a usa enquanto o `DropReceiver` vive.
unsafe impl Send for Display {}

impl DropReceiver {
    /// `Ok(None)`: a janela não é Wayland (no X11 o winit já entrega `DroppedFile`).
    /// `wake` é chamado do fio a cada evento (ex.: `Window::request_redraw`).
    pub fn attach(
        window: &(impl HasDisplayHandle + HasWindowHandle),
        wake: impl Fn() + Send + 'static,
    ) -> Result<Option<Self>, AttachError> {
        let (RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) = (
            window.display_handle()?.as_raw(),
            window.window_handle()?.as_raw(),
        ) else {
            return Ok(None);
        };
        // SAFETY: o ponteiro é o `wl_surface` vivo da janela que o chamador empresta.
        let surface =
            unsafe { ObjectId::from_ptr(WlSurface::interface(), w.surface.as_ptr().cast()) }
                .map_err(|e| AttachError::Wayland(format!("{e:?}")))?
                .protocol_id();
        let display = Display(d.display.as_ptr());
        let (stop_r, stop_w) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC)?;
        let (tx, events) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let thread = std::thread::Builder::new()
            .name("ph2d-wayland-dnd".into())
            .spawn(move || {
                let display = display;
                match Worker::new(display, surface, tx, Box::new(wake)) {
                    Ok((worker, queue)) => {
                        let _ = ready_tx.send(Ok(()));
                        worker.run(queue, stop_r);
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                    }
                }
            })?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Some(Self {
                events,
                stop: Some(stop_w),
                thread: Some(thread),
            })),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => {
                let _ = thread.join();
                Err(AttachError::Wayland(
                    "the receiver thread died during setup".into(),
                ))
            }
        }
    }

    /// Os eventos chegados desde a última chamada, por ordem.
    pub fn drain(&self) -> TryIter<'_, DndEvent> {
        self.events.try_iter()
    }
}

impl Drop for DropReceiver {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = rustix::io::write(&stop, &[1]);
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

struct Read {
    fd: OwnedFd,
    buf: Vec<u8>,
}

struct Worker {
    conn: Connection,
    registry: RegistryState,
    seats: SeatState,
    manager: DataDeviceManagerState,
    devices: Vec<DataDevice>,
    surface: u32,
    session: Session,
    offers: BTreeMap<Token, DragOffer>,
    reads: BTreeMap<Token, Read>,
    tx: Sender<DndEvent>,
    wake: Box<dyn Fn() + Send>,
}

impl Worker {
    fn new(
        display: Display,
        surface: u32,
        tx: Sender<DndEvent>,
        wake: Box<dyn Fn() + Send>,
    ) -> Result<(Self, EventQueue<Self>), AttachError> {
        // SAFETY: a `wl_display` do winit vive mais que este fio (ver `DropReceiver`).
        let backend = unsafe { Backend::from_foreign_display(display.0.cast()) };
        let conn = Connection::from_backend(backend);
        let (globals, queue) =
            registry_queue_init::<Self>(&conn).map_err(|e| AttachError::Wayland(e.to_string()))?;
        let qh = queue.handle();
        let manager = DataDeviceManagerState::bind(&globals, &qh)
            .map_err(|e| AttachError::Wayland(e.to_string()))?;
        let seats = SeatState::new(&globals, &qh);
        let devices = seats
            .seats()
            .map(|s| manager.get_data_device(&qh, &s))
            .collect();
        let worker = Self {
            conn,
            registry: RegistryState::new(&globals),
            seats,
            manager,
            devices,
            surface,
            session: Session::default(),
            offers: BTreeMap::new(),
            reads: BTreeMap::new(),
            tx,
            wake,
        };
        Ok((worker, queue))
    }

    fn run(mut self, mut queue: EventQueue<Self>, stop: OwnedFd) {
        self.serve(&mut queue, &stop);
        // As ofertas vivas destroem-se ANTES da fila (a libwayland avisa «proxies still attached»).
        for device in &self.devices {
            let data = device.data();
            if let Some(o) = data.selection_offer() {
                o.destroy();
            }
            if let Some(o) = data.drag_offer() {
                o.destroy();
            }
        }
        let _ = self.conn.flush();
    }

    /// Nunca `blocking_dispatch`: um `poll` sobre a ligação, o fio de parar e as leituras abertas.
    fn serve(&mut self, queue: &mut EventQueue<Self>, stop: &OwnedFd) {
        loop {
            if let Err(e) = queue.dispatch_pending(self) {
                eprintln!("ph2d-wayland-dnd: dispatch: {e}");
                return;
            }
            match self.conn.flush() {
                Ok(()) => {}
                Err(WaylandError::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    eprintln!("ph2d-wayland-dnd: flush: {e}");
                    return;
                }
            }
            let Some(guard) = queue.prepare_read() else {
                continue;
            };
            let tokens: Vec<Token> = self.reads.keys().copied().collect();
            let revents: Vec<PollFlags> = {
                let wl = guard.connection_fd();
                let mut fds = Vec::with_capacity(2 + tokens.len());
                fds.push(PollFd::new(&wl, PollFlags::IN));
                fds.push(PollFd::new(&stop, PollFlags::IN));
                for t in &tokens {
                    fds.push(PollFd::new(&self.reads[t].fd, PollFlags::IN));
                }
                match rustix::event::poll(&mut fds, None) {
                    Ok(_) => fds.iter().map(PollFd::revents).collect(),
                    Err(Errno::INTR) => continue,
                    Err(e) => {
                        eprintln!("ph2d-wayland-dnd: poll: {e}");
                        return;
                    }
                }
            };
            if !revents[1].is_empty() {
                return;
            }
            if revents[0].contains(PollFlags::IN) {
                match guard.read() {
                    Ok(_) => {}
                    Err(WaylandError::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        eprintln!("ph2d-wayland-dnd: read: {e}");
                        return;
                    }
                }
            } else if !revents[0].is_empty() {
                return;
            } else {
                drop(guard);
            }
            for (t, rev) in tokens.iter().zip(&revents[2..]) {
                if !rev.is_empty() {
                    self.pump_read(*t);
                }
            }
        }
    }

    /// Lê o que houver sem bloquear; no fim (EOF ou erro) entrega a lista à sessão.
    fn pump_read(&mut self, token: Token) {
        let Some(read) = self.reads.get_mut(&token) else {
            return;
        };
        let mut chunk = [0u8; 4096];
        let done = loop {
            match rustix::io::read(&read.fd, &mut chunk) {
                Ok(0) => break true,
                Ok(n) => read.buf.extend_from_slice(&chunk[..n]),
                Err(Errno::AGAIN) => break false,
                Err(Errno::INTR) => {}
                Err(_) => {
                    read.buf.clear();
                    break true;
                }
            }
        };
        if done {
            let read = self.reads.remove(&token).expect("read present");
            let fx = self.session.data_ready(token, &read.buf);
            self.apply(fx);
        }
    }

    fn start_read(offer: &DragOffer) -> rustix::io::Result<Read> {
        let (r, w) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC)?;
        // Só a ponta de LEITURA não bloqueia: a de escrita vai para a fonte (o flag partilha-se).
        offer.inner().receive(URI_LIST_MIME.into(), w.as_fd());
        drop(w);
        rustix::fs::fcntl_setfl(&r, rustix::fs::OFlags::NONBLOCK)?;
        Ok(Read {
            fd: r,
            buf: Vec::new(),
        })
    }

    fn apply(&mut self, fx: Vec<Effect>) {
        for e in fx {
            match e {
                Effect::Emit(ev) => {
                    if self.tx.send(ev).is_ok() {
                        (self.wake)();
                    }
                }
                Effect::Finish(t) => {
                    self.reads.remove(&t);
                    if let Some(o) = self.offers.remove(&t) {
                        o.finish();
                        o.destroy();
                    }
                }
                Effect::Cancel(t) => {
                    self.reads.remove(&t);
                    self.offers.remove(&t);
                }
            }
        }
    }
}

impl DataDeviceHandler for Worker {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        device: &WlDataDevice,
        x: f64,
        y: f64,
        surface: &WlSurface,
    ) {
        let Some(offer) = device
            .data::<DataDeviceData>()
            .and_then(DataDeviceData::drag_offer)
        else {
            return;
        };
        let ours = surface.id().protocol_id() == self.surface;
        let mimes = offer.with_mime_types(<[String]>::to_vec);
        let (token, fx) = self.session.enter(ours, &mimes, (x, y));
        self.apply(fx);
        let Some(token) = token else {
            // Sem isto o cursor do arrasto promete que a janela aceita.
            offer.accept_mime_type(offer.serial, None);
            offer.set_actions(DndAction::empty(), DndAction::empty());
            return;
        };
        // Sem `accept` + `set_actions` (v3) o KWin nunca manda o `drop`.
        offer.accept_mime_type(offer.serial, Some(URI_LIST_MIME.into()));
        offer.set_actions(DndAction::Copy, DndAction::Copy);
        match Self::start_read(&offer) {
            Ok(read) => {
                self.reads.insert(token, read);
                self.offers.insert(token, offer);
            }
            Err(e) => {
                eprintln!("ph2d-wayland-dnd: receive: {e}");
                self.offers.insert(token, offer);
                let fx = self.session.data_ready(token, b"");
                self.apply(fx);
            }
        }
    }

    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {
        let fx = self.session.leave();
        self.apply(fx);
    }

    fn motion(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice, x: f64, y: f64) {
        let fx = self.session.motion((x, y));
        self.apply(fx);
    }

    /// A área de transferência não é deste receptor (ver a Parte 3 do handoff).
    fn selection(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}

    fn drop_performed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {
        let fx = self.session.drop_performed();
        self.apply(fx);
    }
}

/// As acções vêm do `set_actions(Copy, Copy)` do `enter`; nada a decidir depois.
impl DataOfferHandler for Worker {
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

impl SeatHandler for Worker {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seats
    }

    fn new_seat(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: WlSeat) {
        self.devices.push(self.manager.get_data_device(qh, &seat));
    }

    fn new_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat, _: Capability) {}

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: WlSeat,
        _: Capability,
    ) {
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, seat: WlSeat) {
        self.devices.retain(|d| d.data().seat() != &seat);
    }
}

impl ProvidesRegistryState for Worker {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers![SeatState];
}

delegate_registry!(Worker);
delegate_seat!(Worker);
// O `delegate_data_device!` do sctk traz também a `wl_data_source`; este receptor nunca a cria.
delegate_dispatch!(Worker: [WlDataDeviceManager: GlobalData] => DataDeviceManagerState);
delegate_dispatch!(Worker: [WlDataOffer: DataOfferData] => DataDeviceManagerState);
delegate_dispatch!(Worker: [WlDataDevice: DataDeviceData] => DataDeviceManagerState);
