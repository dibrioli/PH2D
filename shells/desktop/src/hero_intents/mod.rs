//! Inspector / hierarchy / image-action intent drains.
//!
//! PR 9a of `docs/Migracao/2026-05-convention-by-discovery.md`:
//! `render_frame()` in `main.rs` used to inline 20 `hero.pending_*`
//! drains spanning ~1300 LOC. Each drain is a self-contained
//! state-machine collapse: takes a one-shot intent flag from the
//! hero, applies the corresponding mutation to the ECS / asset DB /
//! renderer, pushes a toast on observable change.
//!
//! Drains are extracted as free functions receiving the exact
//! refs they need (rather than `&mut self`). This keeps the
//! field-level split borrow that `render_frame()`'s destructure of
//! `AppGfx` already provides — passing `&mut self` would conflict
//! with the live destructure.
//!
//! Convention: each fn returns `bool` indicating whether to set
//! `self.title_dirty = true`. The caller in `render_frame()` ORs the
//! flag into a local accumulator so a single field write happens
//! after the destructure ends.
//!
//! Wave 3.1 stage A — file split into per-domain siblings to honor
//! HR-18 (≤ 600 LOC per file under `shells/<plat>/src/`). Public
//! surface unchanged; `hero_intents::drain_X` paths still resolve
//! via the re-exports below.

mod hierarchy;
mod image_edit;
pub(crate) mod texture_edit;
mod view;

pub(crate) use hierarchy::drain_reparent;
pub(crate) use image_edit::{
    drain_bgremoval, drain_color_equalization, drain_equalize_sizes, drain_make_square,
    drain_padding, drain_rasterize, drain_trim_transparency, drain_undo_image_edit,
};
pub(crate) use view::drain_view_focus;
