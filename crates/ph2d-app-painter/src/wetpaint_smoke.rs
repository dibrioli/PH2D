//! `PH2D_WETPAINT_SMOKE` — a ready-to-paint **Wet Paint** canvas (ADR-0134, W1).
//!
//! Spawns a white 1024² paint canvas. The painter opens in **Digital** — the app's initial default
//! (Enio, 2026-07-22: *"o modo que aparece ao abrir o painter deve ser o digital"*) — so **pick Wet
//! Paint from the Paint Mode dropdown** at the head of the Brush panel, then drag: the paint goes down
//! as FLUID — pigment suspended in water that keeps moving after pen-up (levels, bleeds, dries). A new
//! feature ships with the example that demonstrates it ([[feedback_ready_to_smoke_example]]).
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
//!   PH2D_WETPAINT_SMOKE=1 cargo run --release -p ph2d-host-desktop
//! ```
//!
//! What W1 shows (and what it does not):
//! - Drag: the stroke deposits pigment + water through the engine's own SPEC
//!   §9 mapping, driven by the painter's REAL stroke engine — pressure,
//!   spacing, stabilizer, Symmetry and Tiling all feed the fluid for free.
//! - Release and WAIT: the 40 Hz sim keeps running — the wash levels, edges
//!   darken as it dries. The session spans strokes (wet-on-wet works).
//! - Undo / switching tools ends the session cleanly (the look stays; the
//!   water stops). That is the bake — there is no separate commit.
//! - NOT yet here: the panel (knobs are engine defaults — W3), Paper presets,
//!   Shape/Grain in the wet deposit, per-dab Randomize colour (W2 seams).
//!
//! ⚠️ **The mode is NOT armed in code any more** (Enio, 2026-07-22: *"quando abro o painter o Wet
//! paint ainda é o que aparece primeiro mas deveria ser o digital"*). It used to be, because until W3
//! nothing else selected Wet Paint; now the **Paint Mode** dropdown does, a real control gate-covered
//! in `seam_paint_media.rs`. Arming under the table made the painter open on a medium the app default
//! says it should not — the exact scar the impasto smoke has been warning about — and it skipped the
//! very seam the smoke exists to prove. So the smoke gives you the canvas and gets out of the way: the
//! dropdown is the door.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// **O maior nível a que este roteador de facto responde** — CONTADO no [`nivel`], nunca escrito de memória
/// (CLAUDE.md §5.0): qualquer valor abre a tela de 1024² do `1`; o `2` é a cena do RELEVO DOS FIOS (W3b,
/// prompt 50): o Painter abre sozinho em Wet Paint com o Oil Bristle a 103 px e dois traços inclinados já
/// pintados — o de HOJE (o relevo sem escada) em cima e o de ONTEM (sem relevo) em baixo.
pub const NIVEIS: u32 = 2;

/// Whether the smoke is armed. Cheap enough to call per frame.
pub fn enabled() -> bool {
    std::env::var_os("PH2D_WETPAINT_SMOKE").is_some()
}

/// A cena pedida: `2` = o relevo dos fios; tudo o resto = a tela em branco do `1`.
fn nivel() -> u32 {
    match std::env::var("PH2D_WETPAINT_SMOKE").as_deref() {
        Ok("2") => 2,
        _ => 1,
    }
}

/// Sentar a selecção na tela e focá-la; no nível `2` o Painter abre SOZINHO (o modo Paint da imagem, como a
/// cena do Composite). Mora aqui, não na shell: a shell só chama (`the_shell_only_shrinks`).
pub fn arm_hero(hero: &mut ph2d_editor_core::HeroScreen, bits: u64) {
    if nivel() == 2 {
        crate::composite_smoke::arm_hero(hero, bits);
        return;
    }
    hero.gizmo.replace_selection(Some(bits));
    hero.bus
        .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
            kind: ph2d_editor_core::ViewFocusKind::Selected,
        });
}

/// Spawn the blank paint canvas when `PH2D_WETPAINT_SMOKE=1`. Returns the
/// entity bits so the caller can seat the selection on it.
pub fn spawn_if_enabled(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<u64> {
    if !enabled() {
        return None;
    }
    match ph2d_image_import::spawn_blank_canvas(
        sim,
        renderer,
        asset_db,
        cell_idx,
        1024,
        2, // opaque white — a watercolor ground
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    ) {
        Ok((label, bits)) => {
            println!(
                "PH2D_WETPAINT_SMOKE: canvas '{label}' ready — pick the Painter tool. It opens in \
                 DIGITAL; choose Wet Paint from the Paint Mode dropdown, then drag and watch the \
                 water move."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_WETPAINT_SMOKE: could not spawn the canvas: {e}");
            None
        }
    }
}

/// Pre-configure the brush COLOUR the first time the Painter binds a document under the smoke.
/// Idempotent (one-shot), so the artist's own edits are never overwritten.
///
/// ⚠️ **It no longer selects the medium** — the painter opens in Digital and the artist picks Wet Paint
/// from the dropdown (Enio, 2026-07-22). Only the colour is set, and that reaches the wet slot for free:
/// the paint colour is synced across every mode's `BrushSpec` on purpose (Brush = Fill = picker, one
/// colour). The SIZE is per-slot, so it is left at the wet default — which the smoke should SHOW rather
/// than hide, by its own rule ("if a default is bad, the smoke says so").
pub fn arm_brush_once(painter: &mut ph2d_tool_painter::PainterTool) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ARMED: AtomicBool = AtomicBool::new(false);
    if !enabled() || ARMED.swap(true, Ordering::Relaxed) {
        return;
    }
    painter.set_brush_color_srgb8([30, 90, 200]); // a wet ultramarine (synced to the wet slot)
    if nivel() == 2 {
        pinta_o_relevo(painter);
        return;
    }
    println!(
        "PH2D_WETPAINT_SMOKE: canvas ready in DIGITAL — pick Wet Paint from the Paint Mode dropdown, \
         then drag."
    );
}

/// O Oil Bristle de fábrica, e o de ONTEM (o mesmo sem o relevo de cada fio, `bristleHeight`).
#[cfg(feature = "panel-painter-layers")]
fn oil_bristle() -> Option<(ph2d_brush_presets::Preset, ph2d_brush_presets::Preset)> {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let hoje = ph2d_brush_presets::Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == "Oil Bristle")?
        .preset;
    let mut ontem = hoje.clone();
    ontem.parametros.engine.remove("bristleHeight");
    Some((hoje, ontem))
}

/// **A cena do relevo** (`=2`): Wet Paint pelo seletor do painel, o Oil Bristle a 103 px, e dois traços a ~8°
/// (o ângulo da foto do dono de 18/10) pela MESMA porta do rato — o de hoje em cima, o de ontem em baixo; o
/// pincel fica o de hoje, para o artista riscar o dele ao lado.
#[cfg(feature = "panel-painter-layers")]
fn pinta_o_relevo(painter: &mut ph2d_tool_painter::PainterTool) {
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};
    let Some((hoje, ontem)) = oil_bristle() else {
        eprintln!("PH2D_WETPAINT_SMOKE=2: o Oil Bristle de fábrica não foi encontrado");
        return;
    };
    painter.handle_panel_event(PanelEvent::SelectOption(
        ph2d_tool_painter::ids::PAINTER_BRUSH_MEDIA,
        ph2d_tool_painter::PaintMedia::WetPaint.to_u8().to_string(),
    ));
    let traco = |painter: &mut ph2d_tool_painter::PainterTool, y0: f32| {
        let cp = |x: f32, phase| CanvasPointer {
            pos: [x, y0 - 0.14 * (x - 120.0)],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        painter.on_canvas_pointer(cp(120.0, PointerPhase::Down));
        for k in 1..=260 {
            painter.on_canvas_pointer(cp(120.0 + 3.0 * k as f32, PointerPhase::Move));
            painter.on_tick(8.0);
        }
        painter.on_canvas_pointer(cp(900.0, PointerPhase::Up));
    };
    for (p, y0) in [(&ontem, 820.0), (&hoje, 420.0)] {
        let _ = painter.wet_preset_aplicar(p);
        painter.set_brush_size_px(51.5);
        traco(painter, y0);
    }
    println!(
        "PH2D_WETPAINT_SMOKE=2: O ROTEIRO\n\
  1) A tela abre com dois tracos azuis inclinados, do pincel Oil Bristle a 103 px: o de CIMA e o de \
     HOJE (os fios com relevo, sem escada); o de BAIXO e o de ONTEM (sem relevo).\n\
  2) Aproxime o zoom no de cima: os fios claros e escuros correm lisos ao longo do traco. Deu errado se: \
     aparecem degraus ou blocos pretos e brancos (o 'pixelado' de 18/10).\n\
  3) Risque o SEU traco grande e inclinado no espaco livre, com o pincel que ja esta na mao (o Oil \
     Bristle de hoje). Deu errado se: ele sai diferente do traco de cima, ou serrilhado.\n\
  4) Ctrl+Z tira o seu traco de uma vez."
    );
}

/// Sem a biblioteca de presets (a feature `panel-painter-layers`) não há Oil Bristle de fábrica a aplicar.
#[cfg(not(feature = "panel-painter-layers"))]
fn pinta_o_relevo(_: &mut ph2d_tool_painter::PainterTool) {
    eprintln!(
        "PH2D_WETPAINT_SMOKE=2: precisa da feature `panel-painter-layers` (os presets de fábrica)"
    );
}
