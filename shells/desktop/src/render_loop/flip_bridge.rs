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
) -> (bool, Option<FlipStyleSnapshot>) {
    let flip_active = tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new("flip"));

    // ── 1. Panel visibility (mirror of the Vector dock takeover). ──
    hero.panel_visibility.insert("flip", flip_active);
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
        // ── 2. Picker read-back: is the picker targeting our Stroke swatch? ──
        if hero.store.picker_target() == Some(ph2d_editor::ids::FLIP_STROKE_SWATCH)
            && let Some((value, _, _, _)) = hero
                .store
                .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
        {
            tool.set_stroke_rgba(value.rgba);
        }
        // Seed the swatch's stored colour so the picker opens on the live colour.
        hero.store
            .set_widget_color(ph2d_editor::ids::FLIP_STROKE_SWATCH, tool.stroke_rgba());
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

    (flip_active, style)
}
