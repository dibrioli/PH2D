//! Timeline context menus — the five tables the timeline right-clicks paint.
//!
//! Split out of `menus.rs` (which crossed the file LOC cap) because they are one
//! subject: the segment presets and their easing submenu, the track row, the stack
//! lane, and the clip strip. Every one of them is a **table**, and every table has
//! the same three consumers — the overlay paints it, `pre_populate` registers it,
//! and someone downstream resolves each row. A row added to a table and forgotten
//! by the resolver is a menu item that silently does nothing; that shape is why
//! they are tables and not hand-listed consts, and a gate walks each one.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/menus_timeline.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

/// Wire encoding of the extrapolation SIDE, carried in
/// `ContextMenuKind::TimelineExtrap` from the cascade row that opened the submenu.
/// Opaque to editor-core; the panel decodes it into an `ExtrapSide`.
pub const TL_EXTRAP_SIDE_PRE: u8 = 0;

/// After the last key — see [`TL_EXTRAP_SIDE_PRE`].
pub const TL_EXTRAP_SIDE_POST: u8 = 1;
