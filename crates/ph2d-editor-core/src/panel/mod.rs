//! Panel system — trait-driven panel host infrastructure
//! (ADR-0029).
//!
//! Replaces the pre-ADR `ph2d-editor::panel_registry` fn-pointer
//! style with typed `Panel<State>` trait + `ErasedPanel` wrapper +
//! `PanelHost` trait tier. Goal: panel crates depend ONLY on
//! `ph2d-editor-core`, not on the orchestrator god-crate.
//!
//! ## Modules
//!
//! - [`host`] — `PanelHost` (public stable) + `PanelHostInternal`
//!   (`#[doc(hidden)]` unstable, full surface).
//! - [`panel_trait`] — the `Panel` trait each panel crate implements.
//! - [`paint_ctx`] — `PaintCtx` handed to each `Panel::paint`.
//! - [`manifest`] — `PanelManifest` + `PanelManifest::for_panel::<P>()`
//!   constructor.
//! - [`erased`] — `ErasedPanel` runtime wrapper owning `Box<P::State>`.
//! - [`registry`] — process-wide `PANEL_REGISTRY` installed at boot.
//! - [`event_outcome`] — `EventOutcome::{Consumed, Ignored, Observed}`.

pub mod erased;
pub mod event_outcome;
pub mod host;
pub mod manifest;
pub mod paint_ctx;
pub mod panel_trait;
pub mod registry;
/// ⭐⭐⭐ **O vocabulário de LINHAS de um painel de propriedades** — uma lei, N hospedeiros.
pub mod rows;
pub mod seam_macro;

pub use erased::ErasedPanel;
pub use event_outcome::EventOutcome;
pub use host::{PanelHost, PanelHostInternal};
pub use manifest::{ErasedApplyEventFn, ErasedPaintFn, PanelManifest, PopulateFn};
pub use paint_ctx::PaintCtx;
pub use panel_trait::Panel;
/// ⭐⭐ **A CHAVE de um texto, re-exportada aqui porque o contrato do painel a usa** — o
/// [`Panel::TITLE`] é um `TextKey`, e um painel não devia precisar de declarar uma dependência nova
/// só para escrever o nome dele. ⚠️ Duas bancadas (`widget-gallery`, `widget-lab`) não dependem da
/// `ph2d-i18n` e é por elas que isto existe: *o vocabulário de um trait vem da crate do trait*.
pub use ph2d_i18n::TextKey;
pub use registry::{
    PANEL_REGISTRY, PanelRegistry, install_panel_registry, with_registry, with_registry_opt,
    with_registry_ref,
};
pub use rows::{RowCtx, label_col_w};
pub use seam_macro::seam_reset_button;
