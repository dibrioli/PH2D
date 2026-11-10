//! **A aba Input Map** — o documento ([`doc::InputMapDoc`]), a disposição ([`layout`]) e a pintura das três vistas
//! (plano `docs/InputMap/01_plano.md` §3). Um modelo, três vistas: a aba pinta de um [`snapshot::Snapshot`] que a
//! família publica no `frame` e empurra [`edit::InputMapEdit`] para `pending`; nunca guarda cópia editável do mapa.

pub mod census;
pub mod doc;
pub mod edit;
pub mod ids;
pub mod interact;
pub mod layout;
pub mod paint;
pub mod palette;
pub mod raw_keys;
mod smoke_api;
pub mod snapshot;
pub mod view_keyboard;
pub mod view_list;
pub mod view_try;

pub use doc::{InputMapDoc, View};
pub use edit::InputMapEdit;
pub use snapshot::Snapshot;

/// A aba de família activa é uma `T`?
pub fn active_is<T: 'static>(hero: &mut ph2d_editor_core::HeroScreen) -> bool {
    hero.documents
        .active_surface_mut()
        .is_some_and(|(f, _)| f.as_any().is::<T>())
}
