//! Brush **Wet Paint** section NodeIds (ADR-0134 — the `ph2d-wet-paint` fluid engine as a paint
//! mode). Sibling of `painter_watercolor.rs`: fixed-id, tool-global widgets forwarding over the
//! frozen `PanelEvent` channel to `PainterTool` setters. Today the section carries only the
//! **Enable** checkbox — the ARMED state that makes the Brush paint WET and survives tool
//! round-trips (eraser / selection / smear and back), exactly like the Watercolor and Impasto
//! enables (Enio 2026-07-21: *"se saio do brush para a borracha ou para a seleção, ao voltar não
//! estou mais no modo wet"*), plus the W3 curated knob rows (painted only while armed).
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/painter_wetpaint.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// Collapsible **Wet Paint** section header (ALL-CAPS label + collapse chevron + assignable colour
/// dot). `mark_collapsible_section`-registered in `crate::populate`.
pub const PAINTER_WETPAINT_SECTION: NodeId = hash_node_id("painter_brush.wetpaint_section");

/// Wet Paint section **reset** icon button. `Click` → `reset_brush_wetpaint` — restores the
/// section's defaults INCLUDING the enable (the Watercolor reset's exact semantics: disarming
/// bakes the live water, since ending the session IS the bake).
pub const PAINTER_WETPAINT_RESET: NodeId = hash_node_id("painter_brush.wetpaint_reset");

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

/// **Smooth Edges** (dono 09/10; slider `0..1` desde 10/10) — a silhueta da mancha com meio-tom, pela lei
/// do Watercolor (render-only; `0` de fábrica, `1` = o checkbox ligado de antes).
pub const PAINTER_WETPAINT_SMOOTH_EDGES: NodeId =
    hash_node_id("painter_brush.wetpaint_smooth_edges");

/// **Real Oil** checkbox (Óleo Real, doc 49): a óptica do relevo sobre a mancha (`PainterTool::set_wet_oleo_real`);
/// desligado de fábrica. Ligado, as cinco linhas de [`PAINTER_WETPAINT_OLEO_FIELDS`] aparecem por baixo.
pub const PAINTER_WETPAINT_OLEO_REAL: NodeId = hash_node_id("painter_brush.wetpaint_oleo_real");
/// **Depth** (`OleoReal::profundidade`, `0..3`, `1` = a escala medida em W0).
pub const PAINTER_WETPAINT_OLEO_PROFUNDIDADE: NodeId =
    hash_node_id("painter_brush.wetpaint_oleo_profundidade");
/// **Shadow** (`OleoReal::sombra`, `0..1`).
pub const PAINTER_WETPAINT_OLEO_SOMBRA: NodeId = hash_node_id("painter_brush.wetpaint_oleo_sombra");
/// **Gloss** (`OleoReal::brilho`, `0..1`).
pub const PAINTER_WETPAINT_OLEO_BRILHO: NodeId = hash_node_id("painter_brush.wetpaint_oleo_brilho");
/// **Light height** em GRAUS (`OleoReal::altitude`, em radianos na ferramenta).
pub const PAINTER_WETPAINT_OLEO_ALTITUDE: NodeId =
    hash_node_id("painter_brush.wetpaint_oleo_altitude");
/// **Light angle** em GRAUS (`OleoReal::azimute`, em radianos na ferramenta; `225°` = topo-esquerda).
pub const PAINTER_WETPAINT_OLEO_AZIMUTE: NodeId =
    hash_node_id("painter_brush.wetpaint_oleo_azimute");

/// As linhas numéricas do **Real Oil** — fora de [`PAINTER_WETPAINT_FIELDS`] porque só se pintam com o
/// Óleo Real ligado (aquela lista é «pintada sempre que armado», com gate). Uma lista para o `SetValue`
/// do painel, o populate e o encaminhamento da ferramenta.
pub const PAINTER_WETPAINT_OLEO_FIELDS: [NodeId; 5] = [
    PAINTER_WETPAINT_OLEO_PROFUNDIDADE,
    PAINTER_WETPAINT_OLEO_SOMBRA,
    PAINTER_WETPAINT_OLEO_BRILHO,
    PAINTER_WETPAINT_OLEO_ALTITUDE,
    PAINTER_WETPAINT_OLEO_AZIMUTE,
];

/// As faixas `(min, max)` das linhas de [`PAINTER_WETPAINT_OLEO_FIELDS`], pela MESMA ordem, nas unidades do
/// painel (os ângulos em GRAUS) — uma lei para o painel que as pinta e a ferramenta que as prende.
/// ⚠️ **Light height `5..90°`:** o Rebelle não desce de 42° (medido, W0); abaixo é extensão NOSSA e o piso
/// de 5° é escolha de desenho (a sombra rasante ainda se lê e a marcha dela continua finita). Light
/// direction dá a volta (`360° = 0°`).
pub const PAINTER_WETPAINT_OLEO_FAIXAS: [(f64, f64); 5] = [
    (0.0, 3.0),
    (0.0, 1.0),
    (0.0, 1.0),
    (5.0, 90.0),
    (0.0, 360.0),
];

/// **Environment** chip do **Real Oil** (W1c): o céu que a tinta reflecte (`OleoReal::ambiente`; wire = o
/// índice em `Ambiente::TODOS`). Pintado abaixo de [`PAINTER_WETPAINT_OLEO_FIELDS`], só com a caixa ligada.
pub const PAINTER_WETPAINT_OLEO_AMBIENTE: NodeId =
    hash_node_id("painter_brush.wetpaint_oleo_ambiente");
/// Número de ambientes (`Ambiente::TODOS`).
pub const PAINTER_WETPAINT_OLEO_AMBIENTE_COUNT: u8 = 4;
const _: () = assert!(
    ph2d_wet_paint::oleo_real::Ambiente::TODOS.len()
        == PAINTER_WETPAINT_OLEO_AMBIENTE_COUNT as usize
);

/// Opção `i` do popover do **Environment**.
#[must_use]
pub fn painter_wetpaint_oleo_ambiente_option_id(i: u8) -> NodeId {
    hash_node_id_runtime(&format!("painter_brush.wetpaint_oleo_ambiente_opt.{i}"))
}

/// **Live Water** checkbox (W1g, dono 10/10): a água corre COM o traço aberto (`Sim::sim_while_painting`).
/// Desligado de fábrica: a 4096² o pior quadro a pintar sobe de 2–4 ms a 14–45 ms (05 §4b, W1f).
pub const PAINTER_WETPAINT_LIVE_WATER: NodeId = hash_node_id("painter_brush.wetpaint_live_water");

/// **Heavy Edge** (W1g, `0..1`, `0` de fábrica): com a tela inclinada o pigmento desce mais que a água e
/// pesa na orla de baixo (`Sim::sedimenta`). Sem Tilt não faz nada.
pub const PAINTER_WETPAINT_HEAVY_EDGE: NodeId = hash_node_id("painter_brush.wetpaint_heavy_edge");

/// **Tuning** checkbox — shows/hides the side panel with the full knob table.
pub const PAINTER_WETPAINT_TUNING: NodeId = hash_node_id("painter_brush.wetpaint_tuning");

/// The Wet Paint **Click** ids — ONE membership check for the panel's click forward in its
/// `event.rs` (the allowlist without which a painted, registered checkbox is dead under the
/// mouse) and for the populate loop.
pub const PAINTER_WETPAINT_CLICKS: [NodeId; 18] = [
    PAINTER_WETPAINT_RESET,
    PAINTER_WETPAINT_TILT_TOGGLE,
    PAINTER_WETPAINT_WETCANVAS,
    PAINTER_WETPAINT_DRYCANVAS,
    PAINTER_WETPAINT_FASTDRY,
    PAINTER_WETPAINT_SHOWWET,
    PAINTER_WETPAINT_PAPER_VISUAL,
    PAINTER_WETPAINT_LIVE_WATER,
    PAINTER_WETPAINT_OLEO_REAL,
    PAINTER_WETPAINT_TUNING,
    super::PAINTER_WETPAINT_PRESET_BUTTONS[0],
    super::PAINTER_WETPAINT_PRESET_BUTTONS[1],
    super::PAINTER_WETPAINT_PRESET_BUTTONS[2],
    super::PAINTER_WETPAINT_PRESET_BUTTONS[3],
    super::PAINTER_WETPAINT_PRESET_BUTTONS[4],
    super::PAINTER_WETPAINT_PRESET_BUTTONS[5],
    super::PAINTER_WETPAINT_PRESET_BUTTONS[6],
    super::PAINTER_WETPAINT_PRESET_BUTTONS[7],
];

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

/// **Leveling** — how much the water levels out, deep toward shallow (knob `leveling`; saiu do Tuning a 13/10).
pub const PAINTER_WETPAINT_LEVELING: NodeId = hash_node_id("painter_brush.wetpaint_leveling");

/// **Level Clamp** — the cap on the leveling push (knob `levelClamp`; saiu do Tuning a 13/10).
pub const PAINTER_WETPAINT_LEVEL_CLAMP: NodeId = hash_node_id("painter_brush.wetpaint_level_clamp");

/// **Capillary** — the paper tooth's pull on the water (knob `capillary`; saiu do Tuning a 13/10).
pub const PAINTER_WETPAINT_CAPILLARY: NodeId = hash_node_id("painter_brush.wetpaint_capillary");

/// **Brake** — the absorbency brake that stops water on dry paper (knob `brake`; saiu do Tuning a 13/10).
pub const PAINTER_WETPAINT_BRAKE: NodeId = hash_node_id("painter_brush.wetpaint_brake");

/// **Brush dust** — o PÓ do pincel, o grão do aerógrafo (knob `dust`; saiu do Tuning a 19/10).
pub const PAINTER_WETPAINT_DUST: NodeId = hash_node_id("painter_brush.wetpaint_dust");

/// **Backruns** — as couves-flor da água que volta (knob `extBackrun`; saiu do Tuning, dono 25/10).
pub const PAINTER_WETPAINT_BACKRUN: NodeId = hash_node_id("painter_brush.wetpaint_backrun");

/// **Granulation** — o pigmento que assenta no grão do papel (knob `extGranulation`; saiu do Tuning, dono 25/10).
pub const PAINTER_WETPAINT_GRANULATION: NodeId = hash_node_id("painter_brush.wetpaint_granulation");

/// **Pigment diffusion** — o pigmento que se espalha na água (knob `extDiffusion`; saiu do Tuning, dono 25/10).
pub const PAINTER_WETPAINT_DIFFUSION: NodeId = hash_node_id("painter_brush.wetpaint_diffusion");

/// **Wet bloom** — a tinta que abre na água já pousada (knob `wetOpen`; saiu do Tuning, dono 25/10).
pub const PAINTER_WETPAINT_WET_BLOOM: NodeId = hash_node_id("painter_brush.wetpaint_wet_bloom");

/// **Tip pickup** — o reservatório da ponta que leva a cor de baixo (knob `tipReservoir`; saiu do Tuning, dono 25/10).
pub const PAINTER_WETPAINT_TIP_PICKUP: NodeId = hash_node_id("painter_brush.wetpaint_tip_pickup");

/// **Erase Strength** — the wet eraser's lift per pass (engine `sliders.erase`, boot `0.4`).
pub const PAINTER_WETPAINT_ERASE: NodeId = hash_node_id("painter_brush.wetpaint_erase");

// ── A linha da FERRAMENTA da água na mão (dono 25/10; o painel do Rebelle, `smoke_2026-10-24/rebelle_ferramentas`):
//    o knob que o leitor dela lê, só com ela na mão ─────────────────────────────────────────────────────────────
/// **Smear › Strength** (knob `smear`).
pub const PAINTER_WETPAINT_SMEAR_STRENGTH: NodeId =
    hash_node_id("painter_brush.wetpaint_smear_strength");
/// **Smear › Water** (knob `smearWater`).
pub const PAINTER_WETPAINT_SMEAR_WATER: NodeId = hash_node_id("painter_brush.wetpaint_smear_water");
/// **Blend › Strength** (knob `blendForce`).
pub const PAINTER_WETPAINT_BLEND_STRENGTH: NodeId =
    hash_node_id("painter_brush.wetpaint_blend_strength");
/// **Dry › Absorbency** (knob `dryer`).
pub const PAINTER_WETPAINT_DRY_ABSORBENCY: NodeId =
    hash_node_id("painter_brush.wetpaint_dry_absorbency");
/// **Blow › Strength** (knob `blow`).
pub const PAINTER_WETPAINT_BLOW_STRENGTH: NodeId =
    hash_node_id("painter_brush.wetpaint_blow_strength");

/// The Wet Paint **number-field** ids — ONE membership list for the panel's `SetValue` forward
/// (`number_field::is_param_field`), the populate loop, and the tool's routing match.
pub const PAINTER_WETPAINT_FIELDS: [NodeId; 28] = [
    PAINTER_WETPAINT_GRID,
    PAINTER_WETPAINT_FLOW,
    PAINTER_WETPAINT_WATER,
    PAINTER_WETPAINT_PIGMENT,
    PAINTER_WETPAINT_PICKUP,
    PAINTER_WETPAINT_DRY_SPEED,
    PAINTER_WETPAINT_EDGE,
    PAINTER_WETPAINT_GRAVITY,
    PAINTER_WETPAINT_LEVELING,
    PAINTER_WETPAINT_LEVEL_CLAMP,
    PAINTER_WETPAINT_CAPILLARY,
    PAINTER_WETPAINT_BRAKE,
    PAINTER_WETPAINT_DUST,
    PAINTER_WETPAINT_BACKRUN,
    PAINTER_WETPAINT_GRANULATION,
    PAINTER_WETPAINT_DIFFUSION,
    PAINTER_WETPAINT_WET_BLOOM,
    PAINTER_WETPAINT_TIP_PICKUP,
    PAINTER_WETPAINT_ERASE,
    PAINTER_WETPAINT_SMEAR_STRENGTH,
    PAINTER_WETPAINT_SMEAR_WATER,
    PAINTER_WETPAINT_BLEND_STRENGTH,
    PAINTER_WETPAINT_DRY_ABSORBENCY,
    PAINTER_WETPAINT_BLOW_STRENGTH,
    super::PAINTER_WETPAINT_VOL_SIZE,
    super::PAINTER_WETPAINT_VOL_OPACITY,
    PAINTER_WETPAINT_HEAVY_EDGE,
    PAINTER_WETPAINT_SMOOTH_EDGES,
];
