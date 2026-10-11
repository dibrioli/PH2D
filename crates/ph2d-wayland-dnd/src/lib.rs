//! **Soltar ficheiros na janela, no Wayland** — o que o winit 0.30 não entrega.
//!
//! O winit 0.30 não tem `wl_data_device`: no Wayland, um ficheiro largado do Dolphin nunca vira
//! `DroppedFile`. Esta crate é um receptor de DnD ao lado dele, na MESMA ligação ao compositor
//! (o padrão do `smithay-clipboard`), com fila e fio próprios. Decisão e fontes:
//! `docs/MiroClone/handoffs/HANDOFF_CONTINUACAO_W14_SOLTAR_NO_WAYLAND_2026-10-11.md` Parte 2.
//!
//! Três camadas, as duas primeiras sem compositor (testáveis em qualquer sistema):
//! - [`uri_list`] — a `text/uri-list` que o gestor de ficheiros manda;
//! - [`session`] — o gesto entrar → mover → largar → acabar, e o que entregar à app;
//! - `receiver` (Linux) — o fio que fala com o compositor e empurra [`DndEvent`]s.

pub mod session;
pub mod uri_list;

#[cfg(target_os = "linux")]
mod receiver;

#[cfg(target_os = "linux")]
pub use receiver::{AttachError, DropReceiver};
pub use session::DndEvent;
pub use uri_list::UriList;

/// O único tipo que pedimos ao arrasto: ficheiros como `file://`.
pub const URI_LIST_MIME: &str = "text/uri-list";
