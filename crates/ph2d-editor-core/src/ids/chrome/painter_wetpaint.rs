//! Brush **Wet Paint** section NodeIds (ADR-0134 — the `ph2d-wet-paint` fluid engine as a paint
//! mode). Sibling of `painter_watercolor.rs`: fixed-id, tool-global widgets forwarding over the
//! frozen `PanelEvent` channel to `PainterTool` setters. Today the section carries only the
//! **Enable** checkbox — the ARMED state that makes the Brush paint WET and survives tool
//! round-trips (eraser / selection / smear and back), exactly like the Watercolor and Impasto
//! enables (Enio 2026-07-21: *"se saio do brush para a borracha ou para a seleção, ao voltar não
//! estou mais no modo wet"*). The W3 knob curation lands its ~6 sliders in this section.

use super::{NodeId, hash_node_id};

/// Collapsible **Wet Paint** section header (ALL-CAPS label + collapse chevron + assignable colour
/// dot). `mark_collapsible_section`-registered in `crate::populate`.
pub const PAINTER_WETPAINT_SECTION: NodeId = hash_node_id("painter_brush.wetpaint_section");
/// The Wet Paint header's colour dot — a picker swatch (`register_picker_swatch`).
pub const PAINTER_WETPAINT_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.wetpaint_section_color");
/// Wet Paint section **reset** icon button. `Click` → `reset_brush_wetpaint` — restores the
/// section's defaults INCLUDING the enable (the Watercolor reset's exact semantics: disarming
/// bakes the live water, since ending the session IS the bake).
pub const PAINTER_WETPAINT_RESET: NodeId = hash_node_id("painter_brush.wetpaint_reset");

/// **Enable** master toggle — the ARMED state. `Click` → `toggle_wetpaint_armed`. Off (default)
/// leaves every stroke byte-identical to a plain brush; on, the Brush IS the fluid, and the arm
/// persists across tool switches so "brush" always returns to Wet Paint until unchecked.
pub const PAINTER_WETPAINT_ENABLE: NodeId = hash_node_id("painter_brush.wetpaint_enable");
