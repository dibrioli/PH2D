//! Brush **Wet Paint** section NodeIds (ADR-0134 — the `ph2d-wet-paint` fluid engine as a paint
//! mode). Sibling of `painter_watercolor.rs`: fixed-id, tool-global widgets forwarding over the
//! frozen `PanelEvent` channel to `PainterTool` setters. Today the section carries only the
//! **Enable** checkbox — the ARMED state that makes the Brush paint WET and survives tool
//! round-trips (eraser / selection / smear and back), exactly like the Watercolor and Impasto
//! enables (Enio 2026-07-21: *"se saio do brush para a borracha ou para a seleção, ao voltar não
//! estou mais no modo wet"*), plus the W3 curated knob rows (painted only while armed).

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

/// The Wet Paint **Click** ids — ONE membership check for the panel's click forward in its
/// `event.rs` (the allowlist without which a painted, registered checkbox is dead under the
/// mouse) and for the populate loop.
pub const PAINTER_WETPAINT_CLICKS: [NodeId; 2] = [PAINTER_WETPAINT_ENABLE, PAINTER_WETPAINT_RESET];

// ── The W3 curated knobs (SPEC §16 — the ~40-knob tuning table curated to the seven an artist
//    reaches for; the rest stay named engine constants). Each is a `card_row` number chip whose
//    committed/scrubbed value forwards as `SetValue` via `PAINTER_WETPAINT_FIELDS` +
//    `number_field::is_param_field`, exactly like the Watercolor fields. ──

/// **Water** — the brush's water load per dab (engine `sliders.water`, `0..1`, boot `1.0`).
pub const PAINTER_WETPAINT_WATER: NodeId = hash_node_id("painter_brush.wetpaint_water");
/// **Pigment** — pigment per dab (SPEC §10 gain; knob `pigmentPerDab`, default `600`).
pub const PAINTER_WETPAINT_PIGMENT: NodeId = hash_node_id("painter_brush.wetpaint_pigment");
/// **Pickup** — how much settled paint the tip lifts back (dirty brush; knob `pickup`, `0.005`).
pub const PAINTER_WETPAINT_PICKUP: NodeId = hash_node_id("painter_brush.wetpaint_pickup");
/// **Dry Speed** — the evaporation multiplier (knob `evaporation`, default `1`).
pub const PAINTER_WETPAINT_DRY_SPEED: NodeId = hash_node_id("painter_brush.wetpaint_dry_speed");
/// **Edge Darkening** — the watercolor signature rim (knob `edgeDarkening`, default `50`).
pub const PAINTER_WETPAINT_EDGE: NodeId = hash_node_id("painter_brush.wetpaint_edge");
/// **Gravity** — the run/drip pull (knob `gravity`, default `0.005`; direction stays straight down).
pub const PAINTER_WETPAINT_GRAVITY: NodeId = hash_node_id("painter_brush.wetpaint_gravity");
/// **Erase Strength** — the wet eraser's lift per pass (engine `sliders.erase`, boot `0.4`).
pub const PAINTER_WETPAINT_ERASE: NodeId = hash_node_id("painter_brush.wetpaint_erase");

/// The Wet Paint **number-field** ids — ONE membership list for the panel's `SetValue` forward
/// (`number_field::is_param_field`), the populate loop, and the tool's routing match.
pub const PAINTER_WETPAINT_FIELDS: [NodeId; 7] = [
    PAINTER_WETPAINT_WATER,
    PAINTER_WETPAINT_PIGMENT,
    PAINTER_WETPAINT_PICKUP,
    PAINTER_WETPAINT_DRY_SPEED,
    PAINTER_WETPAINT_EDGE,
    PAINTER_WETPAINT_GRAVITY,
    PAINTER_WETPAINT_ERASE,
];
