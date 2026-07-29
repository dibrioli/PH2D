//! **Expression modal** NodeIds (plano 10 W1) — the three-column card that
//! replaces the inline formula field: a searchable gallery of recipes, the sheet
//! of rows the artist tunes, and the formula bar that shows what the sheet
//! produces.
//!
//! Two id families, and the split is not cosmetic:
//!
//! * The **gallery** derives from the recipe's stable id (`shake`, `limit`, …).
//!   One card per recipe, forever; the same string the catalog stores.
//! * The **sheet** derives from the row INDEX, not the recipe. ⚠️ A stack may hold
//!   the same recipe twice — two `Shake` rows at different speeds is an ordinary
//!   thing to build — so keying a row's widgets by recipe would give the two rows
//!   the SAME slider, and dragging one would move both.
//!
//! Uniqueness across both families and against the static chrome ids is gated
//! where the real recipe ids are visible (`ph2d-panel-timeline`), the same shape
//! `wet_tuning_ids_dont_collide` uses.

use super::{NodeId, hash_node_id};

/// The card's title band — a `TimelineSurface` whose gestures MOVE the card.
/// Stops short of the close X so the two hit rects never share a pixel.
pub const EXPR_MODAL_HANDLE: NodeId = hash_node_id("expr_modal.handle");
/// Close X. Discards the sheet — the same as Cancel, because a card you can
/// dismiss two ways must dismiss the same way twice.
pub const EXPR_MODAL_CLOSE: NodeId = hash_node_id("expr_modal.close");
/// The gallery's search field.
pub const EXPR_MODAL_SEARCH: NodeId = hash_node_id("expr_modal.search");
/// Commit: the sheet's formula becomes the property's expression.
pub const EXPR_MODAL_APPLY: NodeId = hash_node_id("expr_modal.apply");
/// Discard. ⚠️ Gated: opening and cancelling must leave the document byte-identical.
pub const EXPR_MODAL_CANCEL: NodeId = hash_node_id("expr_modal.cancel");

/// A gallery card, keyed by the recipe's stable id.
#[must_use]
pub fn expr_gallery_id(recipe: &str) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("expr_modal.card.{recipe}"))
}

/// A refusal card in the gallery — the routing answer for something the catalog
/// does NOT build (typing `loop` finds where the loop lives).
#[must_use]
pub fn expr_refusal_id(key: &str) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("expr_modal.refusal.{key}"))
}

/// A sheet row's knob widget — keyed by (row, knob) POSITION, never by recipe.
#[must_use]
pub fn expr_knob_id(row: usize, knob: usize) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("expr_modal.knob.{row}.{knob}"))
}

/// A sheet row's numeric CHIP (type the number instead of dragging it).
#[must_use]
pub fn expr_chip_id(row: usize, knob: usize) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("expr_modal.chip.{row}.{knob}"))
}

/// A sheet row's bypass eye.
#[must_use]
pub fn expr_bypass_id(row: usize) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("expr_modal.bypass.{row}"))
}

/// A sheet row's remove button.
#[must_use]
pub fn expr_remove_id(row: usize) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("expr_modal.remove.{row}"))
}

/// A sheet row's **combine mode** chip (`+` / `x` / `=`) — how the row lands on the
/// value the rows above it produced.
///
/// Offered only for a SOURCE row; a modifier folds the value itself and has nothing to
/// choose, so the id is simply never registered for one.
#[must_use]
pub fn expr_combine_id(row: usize) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("expr_modal.combine.{row}"))
}
