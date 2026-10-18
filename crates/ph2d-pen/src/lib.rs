//! **A caneta de verdade** — pressão, inclinação, ponta, borracha e botões de uma mesa
//! digitalizadora, o que o winit 0.30 não entrega no Wayland (ADR-0189).
//!
//! Três camadas:
//! - [`sessao`] — a máquina pura: eventos do protocolo → [`PenEvent`]s, UM por `frame`, sem
//!   compositor (testável em qualquer sistema; X11 e Windows entram depois pela mesma porta);
//! - [`ponteiro`] — a caneta como o rato a veria: `PenEvent` → os `PointerEvent`s da shell;
//! - `wayland` (Linux) — o fio que liga o `zwp_tablet_v2` na ligação do winit, filtra a superfície
//!   da janela e põe o cursor da ferramenta.

pub mod ponteiro;
pub mod sessao;

#[cfg(target_os = "linux")]
mod forma;
#[cfg(target_os = "linux")]
mod wayland;

pub use sessao::{
    PenButton, PenEvent, PenPhase, PenSample, ProtocolEvent, Sessao, ToolKey, ToolKind,
};
#[cfg(target_os = "linux")]
pub use wayland::{AttachError, PenReceiver};
