//! ADR-0113 W2 T2.15/T2.16 — apply the Flip Layers-panel events to the document.
//!
//! The docked panel forwards layer edits as `ToolPanelEvent`s (the tool ignores
//! them — they are DOCUMENT edits, mirror of the Vector Boolean/Arrange ops). The
//! shell drain calls [`apply_panel_event`], which decodes the (fixed or runtime
//! per-row) id against the active object's layers and mutates `FlipDoc` +
//! the active-layer pointer. Runtime ids are decoded via the shared
//! `flip_layer_widget_id` (the same twin the panel paints with).

use ph2d_editor::ids::{self, FlipLayerWidget};
use ph2d_editor::tool::PanelEvent;
use ph2d_flip::{BlendMode, FlipDoc, LayerId};

/// Decode a runtime per-row id → `(LayerId, kind)` via the active object's
/// layers (brute-force over rows × kinds — a handful of layers).
fn decode_widget(flip: &FlipDoc, id: ph2d_editor::NodeId) -> Option<(LayerId, FlipLayerWidget)> {
    let obj = flip.objects().first()?;
    for l in obj.layers() {
        for kind in FlipLayerWidget::ALL {
            if ids::flip_layer_widget_id(u64::from(l.id.0), kind) == id {
                return Some((l.id, kind));
            }
        }
    }
    None
}

/// Apply one panel event to the Flip document + the active-layer pointer.
/// No-op when `ev` doesn't address a Flip layer control. Returns `true` if it
/// mutated the document (so the caller can note a real edit happened).
pub(crate) fn apply_panel_event(
    ev: &PanelEvent,
    flip: &mut FlipDoc,
    active_layer: &mut Option<LayerId>,
) -> bool {
    let Some(oid) = flip.objects().first().map(|o| o.id) else {
        return false;
    };
    match ev {
        PanelEvent::Click(id) if *id == ids::FLIP_LAYER_ADD => {
            if let Some(obj) = flip.object_mut(oid) {
                let name = format!("Layer {}", obj.layers().len() + 1);
                let new = obj.add_layer(name);
                *active_layer = Some(new);
                return true;
            }
            false
        }
        PanelEvent::Click(id) if *id == ids::FLIP_LAYER_DELETE => {
            let Some(target) = active_layer.or_else(|| {
                flip.object(oid)
                    .and_then(|o| o.layers().last().map(|l| l.id))
            }) else {
                return false;
            };
            if let Some(obj) = flip.object_mut(oid)
                && obj.remove_layer(target)
            {
                obj.remove_unused_drawings();
                // Point the active layer at the new top (or clear if none left).
                *active_layer = obj.layers().last().map(|l| l.id);
                return true;
            }
            false
        }
        PanelEvent::Click(id) => {
            let Some((layer, kind)) = decode_widget(flip, *id) else {
                return false;
            };
            match kind {
                FlipLayerWidget::Row => {
                    *active_layer = Some(layer);
                    // Selection is not a document mutation — no undo step.
                    false
                }
                FlipLayerWidget::Visibility => {
                    if let Some(l) = flip.object_mut(oid).and_then(|o| o.layer_mut(layer)) {
                        l.visible = !l.visible;
                        return true;
                    }
                    false
                }
                FlipLayerWidget::Lock => {
                    if let Some(l) = flip.object_mut(oid).and_then(|o| o.layer_mut(layer)) {
                        l.locked = !l.locked;
                        return true;
                    }
                    false
                }
                FlipLayerWidget::MoveUp => {
                    flip.object_mut(oid).is_some_and(|o| o.raise_layer(layer))
                }
                FlipLayerWidget::MoveDown => {
                    flip.object_mut(oid).is_some_and(|o| o.lower_layer(layer))
                }
                FlipLayerWidget::Opacity | FlipLayerWidget::Blend => false,
            }
        }
        PanelEvent::SetValue(id, v) => match decode_widget(flip, *id) {
            Some((layer, FlipLayerWidget::Opacity)) => {
                if let Some(l) = flip.object_mut(oid).and_then(|o| o.layer_mut(layer)) {
                    l.opacity = (*v as f32).clamp(0.0, 1.0);
                    return true;
                }
                false
            }
            _ => false,
        },
        PanelEvent::SelectOption(id, val) => {
            // Blend chip → the id is the layer's Blend widget; `val` is the mode.
            let Some((layer, FlipLayerWidget::Blend)) = decode_widget(flip, *id) else {
                return false;
            };
            let Ok(mode) = val.parse::<u8>() else {
                return false;
            };
            if let Some(l) = flip.object_mut(oid).and_then(|o| o.layer_mut(layer)) {
                l.blend = BlendMode::from_u8(mode);
                return true;
            }
            false
        }
        PanelEvent::Toggle(..) => false,
    }
}
