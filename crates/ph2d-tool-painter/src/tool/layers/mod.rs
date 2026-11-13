//! W3 layer host — layers / selection / masks / adjustments / groups, split out
//! of the former `tool.rs` god-object (pure mechanical move). This directory was
//! itself decomposed by responsibility (ADR-0099 host slim) from the former
//! single `tool/layers.rs`; every method keeps its name + signature + visibility
//! (they are `impl PainterTool` blocks spread across sibling files in this crate).
//!
//! - [`model`]    — read-only model accessors (the panel's projection).
//! - [`preview`]  — GPU preview plumbing + content-version bumps + trivial-stack.
//! - [`cache`]    — composite-cache invalidation chokepoint.
//! - [`undo`]     — structural snapshot / restore / commit.
//! - [`mutate`]   — per-property mutators + structural edits (add/delete/group/…)
//!   + selection ops.
//! - [`adjustments`] — adjustment-layer param mutators (sliders / toggles /
//!   curves / gradient / channel-mixer / selective-color).

mod adjustments;
#[cfg(test)]
mod aplica_mascara_tests;
mod cache;
#[cfg(test)]
pub(crate) use cache::TROCA_RECOMPOE_SEMPRE;
#[cfg(test)]
pub(crate) use mutate::TELA_NOVA_PELO_MMAP;
#[cfg(test)]
mod copias_de_plano_tests;
mod estrutura;
#[cfg(test)]
mod estrutura_vidro_tests;
mod model;
mod mutate;
mod passo;
#[cfg(test)]
mod passo_tests;
mod preview;
#[cfg(test)]
mod sonda_4k_tests;
#[cfg(test)]
mod sonda_n7_app_tests;
mod texture;
#[cfg(test)]
mod troca_figura_tests;
mod undo;

// Preserve the historic public path `tool::layers::MAX_CURVE_POINTS_PER_CHANNEL`
// (used by `add_curve_point` in `adjustments`; not re-exported from `lib.rs`, so
// the re-export is path-only — silence the unused-import lint it would trip).
#[allow(unused_imports)]
pub use adjustments::MAX_CURVE_POINTS_PER_CHANNEL;
