//! O fio que fala com o compositor: a MESMA `wl_display` do winit, fila e fio próprios (o padrão do
//! `ph2d-wayland-dnd`). Liga o `zwp_tablet_manager_v2` CAPADO na versão 1 (ADR-0189: ligar a 2 do
//! KWin com código que não a conhece mata a ligação inteira) e traduz cada evento do
//! `zwp_tablet_tool_v2` 1:1 num [`ProtocolEvent`]; toda a lei vive na [`Sessao`].
#![allow(unsafe_code)]

use std::collections::BTreeMap;
use std::ffi::c_void;
use std::fmt;
use std::fs::File;
use std::io::Write as _;
use std::os::fd::OwnedFd;
use std::sync::mpsc::{self, Receiver, Sender, TryIter};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;

use cursor_icon::CursorIcon;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use rustix::event::{PollFd, PollFlags};
use rustix::io::Errno;
use wayland_backend::client::{Backend, ObjectId, WaylandError};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum, event_created_child,
};
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::WpCursorShapeDeviceV1;
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1;
use wayland_protocols::wp::tablet::zv2::client::{
    zwp_tablet_manager_v2::ZwpTabletManagerV2,
    zwp_tablet_pad_dial_v2::ZwpTabletPadDialV2,
    zwp_tablet_pad_group_v2::{self, ZwpTabletPadGroupV2},
    zwp_tablet_pad_ring_v2::ZwpTabletPadRingV2,
    zwp_tablet_pad_strip_v2::ZwpTabletPadStripV2,
    zwp_tablet_pad_v2::{self, ZwpTabletPadV2},
    zwp_tablet_seat_v2::{self, ZwpTabletSeatV2},
    zwp_tablet_tool_v2::{self, ZwpTabletToolV2},
    zwp_tablet_v2::{self, ZwpTabletV2},
};

use crate::forma::forma_de;
use crate::sessao::{PenEvent, ProtocolEvent, Sessao, ToolKind};

#[path = "wayland_despacho.rs"]
mod despacho;

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

/// A caneta de uma janela Wayland, como [`PenEvent`]s.
///
/// ⚠️ Largue-o ANTES de o laço do winit acabar (no `exiting`): o fio usa a `wl_display` dele.
pub struct PenReceiver {
    events: Receiver<PenEvent>,
    /// Escrever acorda o fio (cursor novo); fechar pára-o.
    ctl: Option<OwnedFd>,
    cursor: Arc<Mutex<CursorIcon>>,
    thread: Option<JoinHandle<()>>,
}

/// A `wl_display` do winit, levada ao fio: a libwayland é segura entre fios.
struct Display(*mut c_void);
// SAFETY: a `wl_display` é thread-safe; o fio só a usa enquanto o `PenReceiver` vive.
unsafe impl Send for Display {}

impl PenReceiver {
    /// `Ok(None)`: a janela não é Wayland. `Err`: o compositor não tem `zwp_tablet_manager_v2`
    /// (ou a ligação falhou). `wake` é chamado do fio a cada lote de eventos
    /// (ex.: `Window::request_redraw`).
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
        let flags = rustix::pipe::PipeFlags::CLOEXEC | rustix::pipe::PipeFlags::NONBLOCK;
        let (ctl_r, ctl_w) = rustix::pipe::pipe_with(flags)?;
        let (tx, events) = mpsc::channel();
        let cursor = Arc::new(Mutex::new(CursorIcon::Default));
        let cursor_fio = Arc::clone(&cursor);
        let log = abre_registo();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let thread = std::thread::Builder::new()
            .name("ph2d-pen".into())
            .spawn(move || {
                let display = display;
                match Worker::new(display, surface, tx, Box::new(wake), cursor_fio, log) {
                    Ok((worker, queue)) => {
                        let _ = ready_tx.send(Ok(()));
                        worker.run(queue, ctl_r);
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                    }
                }
            })?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Some(Self {
                events,
                ctl: Some(ctl_w),
                cursor,
                thread: Some(thread),
            })),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => {
                let _ = thread.join();
                Err(AttachError::Wayland(
                    "the pen thread died during setup".into(),
                ))
            }
        }
    }

    /// Os eventos chegados desde a última chamada, por ordem.
    pub fn drain(&self) -> TryIter<'_, PenEvent> {
        self.events.try_iter()
    }

    /// O cursor da caneta sobre a janela (o do rato é do winit). Barato se não mudou.
    pub fn set_cursor(&self, icon: CursorIcon) {
        let mut atual = self.cursor.lock().unwrap_or_else(PoisonError::into_inner);
        if *atual == icon {
            return;
        }
        *atual = icon;
        drop(atual);
        if let Some(ctl) = &self.ctl {
            // Cheio (`EAGAIN`) = o fio já tem um aviso por ler: basta.
            let _ = rustix::io::write(ctl, b"c");
        }
    }
}

impl Drop for PenReceiver {
    fn drop(&mut self) {
        drop(self.ctl.take());
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// `PH2D_PEN_LOG=<ficheiro>`: uma linha por [`PenEvent`] (o smoke compara-a com o libinput).
fn abre_registo() -> Option<File> {
    let path = std::env::var_os("PH2D_PEN_LOG")?;
    match File::options().create(true).append(true).open(&path) {
        Ok(f) => Some(f),
        Err(e) => {
            eprintln!("ph2d-pen: PH2D_PEN_LOG {}: {e}", path.to_string_lossy());
            None
        }
    }
}

struct Ferramenta {
    proxy: ZwpTabletToolV2,
    kind: ToolKind,
    forma: Option<WpCursorShapeDeviceV1>,
    /// O `serial` do último `proximity_in` na NOSSA superfície, enquanto dura.
    entrou: Option<u32>,
}

struct Assento {
    nome: u32,
    seat: WlSeat,
    tablet_seat: ZwpTabletSeatV2,
}

struct Worker {
    conn: Connection,
    surface: u32,
    manager: ZwpTabletManagerV2,
    formas: Option<WpCursorShapeManagerV1>,
    assentos: Vec<Assento>,
    tools: BTreeMap<u32, Ferramenta>,
    tablets: Vec<ZwpTabletV2>,
    pads: Vec<ZwpTabletPadV2>,
    /// Grupos, anéis, faixas e discos dos pads: só se destroem no fim.
    pad_parts: Vec<Box<dyn FnOnce()>>,
    sessao: Sessao,
    cursor: Arc<Mutex<CursorIcon>>,
    buf: Vec<PenEvent>,
    tx: Sender<PenEvent>,
    wake: Box<dyn Fn() + Send>,
    log: Option<File>,
}

impl Worker {
    fn new(
        display: Display,
        surface: u32,
        tx: Sender<PenEvent>,
        wake: Box<dyn Fn() + Send>,
        cursor: Arc<Mutex<CursorIcon>>,
        log: Option<File>,
    ) -> Result<(Self, EventQueue<Self>), AttachError> {
        // SAFETY: a `wl_display` do winit vive mais que este fio (ver `PenReceiver`).
        let backend = unsafe { Backend::from_foreign_display(display.0.cast()) };
        let conn = Connection::from_backend(backend);
        let (globals, queue) =
            registry_queue_init::<Self>(&conn).map_err(|e| AttachError::Wayland(e.to_string()))?;
        let qh = queue.handle();
        // ⛔ `1..=1`: ver o cabeçalho.
        let manager = globals
            .bind::<ZwpTabletManagerV2, _, _>(&qh, 1..=1, ())
            .map_err(|e| AttachError::Wayland(format!("zwp_tablet_manager_v2: {e}")))?;
        let formas = globals
            .bind::<WpCursorShapeManagerV1, _, _>(&qh, 1..=1, ())
            .ok();
        let mut worker = Self {
            conn,
            surface,
            manager,
            formas,
            assentos: Vec::new(),
            tools: BTreeMap::new(),
            tablets: Vec::new(),
            pads: Vec::new(),
            pad_parts: Vec::new(),
            sessao: Sessao::default(),
            cursor,
            buf: Vec::new(),
            tx,
            wake,
            log,
        };
        let seats: Vec<(u32, u32)> = globals.contents().with_list(|l| {
            l.iter()
                .filter(|g| g.interface == WlSeat::interface().name)
                .map(|g| (g.name, g.version))
                .collect()
        });
        for (nome, versao) in seats {
            worker.liga_assento(globals.registry(), nome, versao, &qh);
        }
        Ok((worker, queue))
    }

    fn liga_assento(&mut self, reg: &WlRegistry, nome: u32, versao: u32, qh: &QueueHandle<Self>) {
        // v5 basta (o `release`); nada do seat é lido aqui.
        let seat = reg.bind::<WlSeat, _, _>(nome, versao.min(5), qh, ());
        let tablet_seat = self.manager.get_tablet_seat(&seat, qh, ());
        self.assentos.push(Assento {
            nome,
            seat,
            tablet_seat,
        });
    }

    fn run(mut self, mut queue: EventQueue<Self>, ctl: OwnedFd) {
        self.serve(&mut queue, &ctl);
        // Os proxies destroem-se ANTES da fila (a libwayland avisa «proxies still attached»).
        for t in std::mem::take(&mut self.tools).into_values() {
            if let Some(f) = t.forma {
                f.destroy();
            }
            t.proxy.destroy();
        }
        for p in self.pad_parts.drain(..) {
            p();
        }
        for p in self.pads.drain(..) {
            p.destroy();
        }
        for t in self.tablets.drain(..) {
            t.destroy();
        }
        for a in self.assentos.drain(..) {
            a.tablet_seat.destroy();
            if a.seat.version() >= 5 {
                a.seat.release();
            }
        }
        if let Some(f) = self.formas.take() {
            f.destroy();
        }
        self.manager.destroy();
        let _ = self.conn.flush();
    }

    /// Nunca `blocking_dispatch`: um `poll` sobre a ligação e o fio de controlo.
    fn serve(&mut self, queue: &mut EventQueue<Self>, ctl: &OwnedFd) {
        loop {
            if let Err(e) = queue.dispatch_pending(self) {
                eprintln!("ph2d-pen: dispatch: {e}");
                return;
            }
            self.entrega();
            match self.conn.flush() {
                Ok(()) => {}
                Err(WaylandError::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    eprintln!("ph2d-pen: flush: {e}");
                    return;
                }
            }
            let Some(guard) = queue.prepare_read() else {
                continue;
            };
            let (wl, controlo) = {
                let fd = guard.connection_fd();
                let mut fds = [
                    PollFd::new(&fd, PollFlags::IN),
                    PollFd::new(ctl, PollFlags::IN),
                ];
                match rustix::event::poll(&mut fds, None) {
                    Ok(_) => (fds[0].revents(), fds[1].revents()),
                    Err(Errno::INTR) => continue,
                    Err(e) => {
                        eprintln!("ph2d-pen: poll: {e}");
                        return;
                    }
                }
            };
            if wl.contains(PollFlags::IN) {
                match guard.read() {
                    Ok(_) => {}
                    Err(WaylandError::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        eprintln!("ph2d-pen: read: {e}");
                        return;
                    }
                }
            } else if !wl.is_empty() {
                return;
            } else {
                drop(guard);
            }
            if !controlo.is_empty() {
                if !Self::le_controlo(ctl) {
                    return;
                }
                self.aplica_cursor_a_todas(&queue.handle());
            }
        }
    }

    /// Esvazia o tubo; `false` = fechado (o `PenReceiver` foi largado).
    fn le_controlo(ctl: &OwnedFd) -> bool {
        let mut buf = [0u8; 64];
        loop {
            match rustix::io::read(ctl, &mut buf) {
                Ok(0) => return false,
                Ok(_) | Err(Errno::INTR) => {}
                Err(Errno::AGAIN) => return true,
                Err(_) => return false,
            }
        }
    }

    fn entrega(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        let mut algum = false;
        for ev in self.buf.drain(..) {
            if let Some(f) = self.log.as_mut() {
                let _ = writeln!(f, "{ev}");
            }
            algum |= self.tx.send(ev).is_ok();
        }
        if algum {
            (self.wake)();
        }
    }

    fn aplica_cursor_a_todas(&mut self, qh: &QueueHandle<Self>) {
        let ids: Vec<u32> = self.tools.keys().copied().collect();
        for id in ids {
            self.aplica_cursor(id, qh);
        }
    }

    /// Sem `wp_cursor_shape_manager_v1` não se mexe no cursor: o compositor mostra o dele.
    fn aplica_cursor(&mut self, id: u32, qh: &QueueHandle<Self>) {
        let icon = *self.cursor.lock().unwrap_or_else(PoisonError::into_inner);
        let (Some(formas), Some(t)) = (&self.formas, self.tools.get_mut(&id)) else {
            return;
        };
        let Some(serial) = t.entrou else {
            return;
        };
        let dev = t
            .forma
            .get_or_insert_with(|| formas.get_tablet_tool_v2(&t.proxy, qh, ()));
        dev.set_shape(serial, forma_de(icon));
    }

    fn feed(&mut self, id: u32, ev: ProtocolEvent) {
        self.sessao.feed(u64::from(id), ev, &mut self.buf);
    }
}
