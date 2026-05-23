//! Image-edit drains — one sub-file per tool.
//!
//! Each drain is a self-contained state-machine collapse: read the
//! sprite's source RGBA via [`crate::hero_intents::texture_edit::read_sprite_source`],
//! run the tool's algorithm, commit via
//! [`crate::hero_intents::texture_edit::commit_edited_texture`], capture
//! a single-level `ImageEditSnapshot` for Cmd+Z. Each function returns
//! `true` iff a toast was pushed (caller marks `title_dirty`).
//!
//! Sub-files (one per Image Tools chrome pill):
//! - [`trim_transparency`] — crop transparent border.
//! - [`make_square`] — pad to square.
//! - [`rasterize`] — bake Transform (scale + rotation) into pixels.
//! - [`padding`] — signed per-edge resize (expand / crop).
//! - [`color_equalization`] — CLAHE + brightness/contrast/saturation/auto-WB.
//! - [`equalize_sizes`] — cross-sprite size normalization +
//!   arrange-on-grid layout.
//! - [`upscale`] — Lanczos3 / Nearest / xBR upscale.
//! - [`bgremoval`] — Bg Removal + Separate Islands spawn.
//! - [`undo`] — restore the most recent snapshot.
//!
//! Decomposed from a single `image_edit.rs` (1224 LOC) into this
//! directory module to stay under HR-18's 600-LOC file cap after the
//! Image Tools fan-out (slots 1-4 + Separate Islands) doubled the
//! drain hub's size.

mod bgremoval;
mod color_equalization;
mod equalize_sizes;
mod make_square;
mod padding;
mod rasterize;
mod trim_transparency;
mod undo;
mod upscale;

pub(crate) use bgremoval::drain_bgremoval;
pub(crate) use color_equalization::drain_color_equalization;
pub(crate) use equalize_sizes::drain_equalize_sizes;
pub(crate) use make_square::drain_make_square;
pub(crate) use padding::drain_padding;
pub(crate) use rasterize::drain_rasterize;
pub(crate) use trim_transparency::drain_trim_transparency;
pub(crate) use undo::drain_undo_image_edit;
pub(crate) use upscale::drain_upscale;
