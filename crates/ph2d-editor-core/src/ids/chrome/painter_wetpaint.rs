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

// ⚠️ **There is no `*_ENABLE` id here any more** (2026-07-22): the three media checkboxes were
// replaced by the single `PAINTER_BRUSH_MEDIA` dropdown (`ids/chrome/painter.rs`), so this section
// is painted only while its medium is the selected one. Keeping the checkbox id "for the API" would
// leave an id that is registered and routed but never painted — the same rot the Paper slot's
// Rake/Random ids became before they were removed.

// ── Doc 22 — the wet TOOLS (the model's 7-button radio; Erase is the OTHER
//    VIEW of the rail's eraser chip, the impasto tool-list precedent). ──

/// The 7 wet tool buttons, in the model's order: Paint · Erase · Smear ·
/// Blend · Wet · Dry · Blow. Clicking USES the tool (Erase → the eraser
/// wire; everything else → the brush wire + `WetPaintState.tool`).
pub const PAINTER_WETPAINT_TOOL_IDS: [NodeId; 7] = [
    hash_node_id("painter_brush.wetpaint_tool_paint"),
    hash_node_id("painter_brush.wetpaint_tool_erase"),
    hash_node_id("painter_brush.wetpaint_tool_smear"),
    hash_node_id("painter_brush.wetpaint_tool_blend"),
    hash_node_id("painter_brush.wetpaint_tool_wet"),
    hash_node_id("painter_brush.wetpaint_tool_dry"),
    hash_node_id("painter_brush.wetpaint_tool_blow"),
];

/// The TILT dial (the model's polar pad, 8 rings x 12 spokes): a 2D drag
/// surface registered as a curve-point widget; the panel converts the drag to
/// (ring, spoke) and forwards them as `SetValue`s on the two ids below.
pub const PAINTER_WETPAINT_TILT_PAD: NodeId = hash_node_id("painter_brush.wetpaint_tilt_pad");
/// Tilt **on/off** toggle (header button) — flips without losing the dial.
pub const PAINTER_WETPAINT_TILT_TOGGLE: NodeId = hash_node_id("painter_brush.wetpaint_tilt_toggle");
/// Tilt ring (0..8) — `SetValue` carrier, never painted/registered itself.
pub const PAINTER_WETPAINT_TILT_RING: NodeId = hash_node_id("painter_brush.wetpaint_tilt_ring");
/// Tilt spoke (0..11, 30 deg steps) — `SetValue` carrier like the ring.
pub const PAINTER_WETPAINT_TILT_SPOKE: NodeId = hash_node_id("painter_brush.wetpaint_tilt_spoke");

/// **Wet canvas** — one-shot: raise the sheet's WETNESS everywhere (via max);
/// the next stroke bleeds anywhere. Creates the session if none is live.
pub const PAINTER_WETPAINT_WETCANVAS: NodeId = hash_node_id("painter_brush.wetpaint_wetcanvas");
/// **Dry canvas** — one-shot: settle all suspended pigment, zero water,
/// velocity and wetness. Instant, no drying rims.
pub const PAINTER_WETPAINT_DRYCANVAS: NodeId = hash_node_id("painter_brush.wetpaint_drycanvas");
/// **Fast dry** — one-shot: accelerated evaporation+settle passes until the
/// fluid is gone; the edge rims still darken.
pub const PAINTER_WETPAINT_FASTDRY: NodeId = hash_node_id("painter_brush.wetpaint_fastdry");
/// **Show wet** — TOGGLE: display-only damp overlay (never baked).
pub const PAINTER_WETPAINT_SHOWWET: NodeId = hash_node_id("painter_brush.wetpaint_showwet");

/// **Paper** checkbox — the tooth becomes visually part of the painting
/// (granulation + emboss printed into the pigment colours; render-only).
pub const PAINTER_WETPAINT_PAPER_VISUAL: NodeId =
    hash_node_id("painter_brush.wetpaint_paper_visual");
/// **Tuning** checkbox — shows/hides the side panel with the full knob table.
pub const PAINTER_WETPAINT_TUNING: NodeId = hash_node_id("painter_brush.wetpaint_tuning");

/// The Wet Paint **Click** ids — ONE membership check for the panel's click forward in its
/// `event.rs` (the allowlist without which a painted, registered checkbox is dead under the
/// mouse) and for the populate loop.
pub const PAINTER_WETPAINT_CLICKS: [NodeId; 15] = [
    PAINTER_WETPAINT_RESET,
    PAINTER_WETPAINT_TOOL_IDS[0],
    PAINTER_WETPAINT_TOOL_IDS[1],
    PAINTER_WETPAINT_TOOL_IDS[2],
    PAINTER_WETPAINT_TOOL_IDS[3],
    PAINTER_WETPAINT_TOOL_IDS[4],
    PAINTER_WETPAINT_TOOL_IDS[5],
    PAINTER_WETPAINT_TOOL_IDS[6],
    PAINTER_WETPAINT_TILT_TOGGLE,
    PAINTER_WETPAINT_WETCANVAS,
    PAINTER_WETPAINT_DRYCANVAS,
    PAINTER_WETPAINT_FASTDRY,
    PAINTER_WETPAINT_SHOWWET,
    PAINTER_WETPAINT_PAPER_VISUAL,
    PAINTER_WETPAINT_TUNING,
];

// ── The W3 curated knobs (SPEC §16 — the ~40-knob tuning table curated to the seven an artist
//    reaches for; the rest stay named engine constants). Each is a `card_row` number chip whose
//    committed/scrubbed value forwards as `SetValue` via `PAINTER_WETPAINT_FIELDS` +
//    `number_field::is_param_field`, exactly like the Watercolor fields. ──

/// **Grid Size (px)** — quantos pixels de canvas medem uma célula de fluido
/// (1..=30, default 1). É o PRIMEIRO widget da seção, acima do rádio de tools
/// (Enio 2026-07-29): a resolução da grade decide o custo do solver, que é
/// linear nas células, e por isso decide a taxa VISUAL da água — a 4096² a
/// razão 1 são 16,7 M células e ~19 Hz, a razão 2 são 4,2 M e o nominal.
///
/// ⚠️ Trocar o valor **ENCERRA a sessão de água viva** (a grade tem dimensão);
/// encerrar é o bake, então nada é perdido — ver `wetpaint::grid_map`.
pub const PAINTER_WETPAINT_GRID: NodeId = hash_node_id("painter_brush.wetpaint_grid");

/// **Flow Grid (×)** — quantas células de FLUIDO medem uma célula de FLUXO
/// (1..=16, default 1). A segunda metade da multi-resolução (plano 30): a
/// velocidade e a pressão ficam grossas enquanto o pigmento, a água e a
/// wetness ficam na resolução que o `Grid Size` acima decidiu.
///
/// ⚠️ **Dois números, e não um.** Eles respondem perguntas diferentes — *quão
/// fino é o pigmento?* e *quão grosso é o fluxo?* — e colapsá-los seria a
/// falha de duas-portas ao contrário: um controle governando dois fatos
/// independentes. O readout derivado abaixo dos dois é o que os torna
/// legíveis, porque **um limite que não se vê é um limite que o artista
/// descobre por acidente**.
///
/// ⚠️ Trocar o valor **ENCERRA a sessão de água viva**, exatamente como o
/// `Grid Size` — encerrar é o bake.
pub const PAINTER_WETPAINT_FLOW: NodeId = hash_node_id("painter_brush.wetpaint_flow");

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
pub const PAINTER_WETPAINT_FIELDS: [NodeId; 9] = [
    PAINTER_WETPAINT_GRID,
    PAINTER_WETPAINT_FLOW,
    PAINTER_WETPAINT_WATER,
    PAINTER_WETPAINT_PIGMENT,
    PAINTER_WETPAINT_PICKUP,
    PAINTER_WETPAINT_DRY_SPEED,
    PAINTER_WETPAINT_EDGE,
    PAINTER_WETPAINT_GRAVITY,
    PAINTER_WETPAINT_ERASE,
];
