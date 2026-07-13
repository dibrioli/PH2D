//! ADR-0114 W2 — Flip tool ⟷ shell bridge.
//!
//! Per-frame jobs (mirror of the `vector_bridge`):
//! 1. **Panel visibility** — show the docked `ph2d-panel-flip` iff the `flip`
//!    tool is active; hide the real Inspector (edge-triggered) so they don't
//!    both claim the dock slot.
//! 2. **Picker read-back** — while the shared OKLCH picker targets the Flip
//!    Stroke swatch, feed the live picked colour into the tool.
//! 3. **Style/mode publish** — cache `flip_active` + the brush/mode snapshot for
//!    the `input_dispatch` (which decides capture + bakes the stroke WITHOUT a
//!    downcast — the concrete-tool downcast lives HERE, allowlisted).
//! 4. **Layers publish** — build the panel's layer snapshot from the active
//!    `FlipObject` (the panel needn't depend on the doc model).
//!
//! Runs in the same phase as `vector_bridge` (after the ActivateTool drain,
//! before paint): a freshly-activated tool is seen this frame.

use ph2d_editor::{HeroScreen, ToolId, ToolRegistry};
use ph2d_flip::{FlipDoc, LayerId};
use ph2d_tool_flip::{FlipStyleSnapshot, FlipTool};

/// Build the panel's layer snapshot from the active object (the first object,
/// matching `flip_draw::bake_stroke`). `active` falls back to the TOP layer so
/// the panel highlights a sensible default + the stroke has a target.
#[cfg(feature = "panel-flip")]
fn layers_snapshot(flip: &FlipDoc, active: Option<LayerId>) -> ph2d_panel_flip::FlipLayersSnapshot {
    use ph2d_panel_flip::{FlipLayerRow, FlipLayersSnapshot};
    let Some(obj) = flip.objects().first() else {
        return FlipLayersSnapshot::default();
    };
    let rows = obj
        .layers()
        .iter()
        .map(|l| FlipLayerRow {
            id: u64::from(l.id.0),
            name: l.name.clone(),
            blend: l.blend.to_u8(),
            opacity: l.opacity,
            visible: l.visible,
            locked: l.locked,
        })
        .collect();
    let active = active
        .filter(|id| obj.layer(*id).is_some())
        .or_else(|| obj.layers().last().map(|l| l.id));
    FlipLayersSnapshot {
        rows,
        active: active.map(|id| u64::from(id.0)),
    }
}

/// Build the frame-strip snapshot (W3): the cells of the ACTIVE layer + the
/// transport + the authoring flags. The panel paints exactly this and knows
/// nothing about frames — the model stays in one place.
#[cfg(feature = "panel-flip-frames")]
fn strip_snapshot(
    flip: &FlipDoc,
    active_layer: Option<LayerId>,
    playhead: &ph2d_core::Playhead,
    strip: &crate::flip_strip::FlipStrip,
) -> ph2d_panel_flip_frames::FlipStripSnapshot {
    use ph2d_panel_flip_frames::{FlipCell, FlipStripSnapshot};
    let Some(obj) = flip.objects().first() else {
        return FlipStripSnapshot::default();
    };
    let layer = active_layer
        .and_then(|id| obj.layer(id))
        .or_else(|| obj.layers().last());
    let Some(layer) = layer else {
        return FlipStripSnapshot {
            fps: obj.fps,
            playing: playhead.is_playing(),
            ..Default::default()
        };
    };
    // O quadro-FONTE: sob um Loop, a célula destacada tem de ser a que está NA TELA
    // (a do vão), não a do quadro cru — que na 2ª volta já saiu da tira.
    let raw = obj.frame_at(playhead);
    let frame = layer.source_frame(raw);
    let cells = layer
        .cells()
        .into_iter()
        .map(|(key, drawing, exposure)| FlipCell {
            key,
            exposure,
            breakdown: layer
                .frames()
                .get(&key)
                .is_some_and(|f| f.kind == ph2d_flip::KeyKind::Breakdown),
            instanced: obj.drawing(drawing).is_some_and(|d| d.is_instanced()),
            selected: strip.selected_keys().contains(&key),
        })
        .collect();
    FlipStripSnapshot {
        has_layer: true,
        layer_name: layer.name.clone(),
        cells,
        current_frame: frame,
        // A chave ATIVA (a que está na tela) — no meio de um hold é a que começou
        // antes, não o quadro corrente. É ela que a tira destaca.
        current_key: layer
            .active_key(frame)
            .filter(|k| layer.frames().get(k).is_some_and(|f| f.drawing.is_some())),
        playing: playhead.is_playing(),
        fps: obj.fps,
        ghost: obj.onion.enabled,
        ghost_before: obj.onion.frames_before,
        ghost_after: obj.onion.frames_after,
        autokey: strip.autokey,
        additive: strip.additive,
        falloff: strip.falloff,
        tween_count: strip.tween_count,
        cycle: cycle_wire(layer.cycle.post),
    }
}

/// `CycleMode` → o discriminante que o chip do painel mostra (a ordem do enum).
#[cfg(feature = "panel-flip-frames")]
fn cycle_wire(post: ph2d_flip::CycleMode) -> u8 {
    match post {
        ph2d_flip::CycleMode::None => 0,
        ph2d_flip::CycleMode::Hold => 1,
        ph2d_flip::CycleMode::Loop => 2,
        ph2d_flip::CycleMode::PingPong => 3,
    }
}

/// `(flip_active, style)` — `flip_active` = the Flip tool is ACTIVE; `style` =
/// the brush/mode snapshot (present whenever the tool is registered). Also
/// drives panel visibility + publishes the style/layers snapshots the panel
/// paints. `flip` = the live document; `active_layer` = the shell's active layer
/// (drawn-onto + highlighted).
pub(crate) fn publish(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    flip: &FlipDoc,
    active_layer: Option<LayerId>,
    playhead: &ph2d_core::Playhead,
    strip: &crate::flip_strip::FlipStrip,
) -> (bool, Option<FlipStyleSnapshot>) {
    let flip_active = tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new("flip"));

    // ── 1. Panel visibility (mirror of the Vector dock takeover). ──
    hero.panel_visibility.insert("flip", flip_active);
    // A tira (faixa inferior) acompanha a tool — é chrome de autoria.
    hero.panel_visibility.insert("flip_frames", flip_active);
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(flip_active, Ordering::Relaxed);
        if was != flip_active {
            hero.panel_visibility.insert("inspector", !flip_active);
        }
    }

    // The tool persists in the registry whether active or not, so its style
    // survives tool switches (mirror of the vector/painter bridges).
    let style = if let Some(tool) = tools
        .tool_by_id_mut(&ToolId::new("flip"))
        .and_then(|t| t.as_any_mut().downcast_mut::<FlipTool>())
    {
        // ── 2. Picker read-back: qual swatch o picker está editando? A do TRAÇO ou a
        //    do BALDE — são cores distintas (colorir usa outra paleta que desenhar).
        let picked = hero
            .store
            .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
            .map(|(value, _, _, _)| value.rgba);
        match (hero.store.picker_target(), picked) {
            (Some(id), Some(rgba)) if id == ph2d_editor::ids::FLIP_STROKE_SWATCH => {
                tool.set_stroke_rgba(rgba);
            }
            (Some(id), Some(rgba)) if id == ph2d_editor::ids::FLIP_FILL_SWATCH => {
                tool.set_fill_rgba(rgba);
            }
            _ => {}
        }
        // Seed the swatches' stored colour so the picker opens on the live colour.
        hero.store
            .set_widget_color(ph2d_editor::ids::FLIP_STROKE_SWATCH, tool.stroke_rgba());
        hero.store
            .set_widget_color(ph2d_editor::ids::FLIP_FILL_SWATCH, tool.fill_rgba());
        Some(tool.ui_snapshot())
    } else {
        None
    };

    // ── 3+4. Publish the style + layers snapshots the panel paints. ──
    #[cfg(feature = "panel-flip")]
    {
        ph2d_panel_flip::set_current_flip_style(if flip_active { style } else { None });
        ph2d_panel_flip::set_current_flip_layers(if flip_active {
            layers_snapshot(flip, active_layer)
        } else {
            ph2d_panel_flip::FlipLayersSnapshot::default()
        });
    }

    // ── 5. Publish the frame-strip snapshot (W3). ──
    #[cfg(feature = "panel-flip-frames")]
    ph2d_panel_flip_frames::set_current_flip_strip(if flip_active {
        strip_snapshot(flip, active_layer, playhead, strip)
    } else {
        ph2d_panel_flip_frames::FlipStripSnapshot::default()
    });

    (flip_active, style)
}
