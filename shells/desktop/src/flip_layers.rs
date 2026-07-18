//! ADR-0114 W2 T2.15/T2.16 — apply the Flip Layers-panel events to the document.
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
    playhead: &ph2d_core::Playhead,
    // W8: o DOMÍNIO da seleção (do snapshot da tool) — All/Delete agem em pontos ou em
    // traços conforme o toggle do painel.
    point_domain: bool,
) -> bool {
    let Some(oid) = flip.objects().first().map(|o| o.id) else {
        return false;
    };
    match ev {
        // ── Edit Mode (W6): as ops de SELEÇÃO que não têm gesto de canvas. ──
        //
        // Elas moram aqui, no drain do documento, e não na tool: a seleção é atributo do
        // TRAÇO (`FlipStroke::selected`), não estado de estilo — a tool não tem, e não
        // deve ter, acesso ao `FlipDoc`.
        //
        // Todas operam no desenho VISÍVEL (`flip_select::visible_drawing`, que recebe o
        // doc imutável): selecionar/desmarcar/apagar nunca materializa uma chave nova.
        PanelEvent::Click(id)
            if *id == ids::FLIP_EDIT_SELECT_ALL
                || *id == ids::FLIP_EDIT_DESELECT
                || *id == ids::FLIP_EDIT_DELETE =>
        {
            let Some((_oid, _lid, did)) =
                crate::flip_select::visible_drawing(flip, playhead, *active_layer)
            else {
                return false; // camada travada / quadro sem desenho
            };
            let Some(drawing) = flip.object_mut(oid).and_then(|o| o.drawing_mut(did)) else {
                return false;
            };
            if *id == ids::FLIP_EDIT_SELECT_ALL {
                if point_domain {
                    drawing.select_all_points()
                } else {
                    let mut changed = false;
                    for s in &mut drawing.strokes {
                        changed |= !std::mem::replace(&mut s.selected, true);
                    }
                    changed
                }
            } else if *id == ids::FLIP_EDIT_DESELECT {
                drawing.clear_selection()
            } else if point_domain {
                // Delete no domínio Point: dissolve as âncoras (o traço fica).
                drawing.delete_selected_points() > 0
            } else {
                drawing.delete_selected() > 0
            }
        }
        PanelEvent::Click(id) if *id == ids::FLIP_LAYER_ADD => {
            if let Some(obj) = flip.object_mut(oid) {
                let name = format!("Layer {}", obj.layers().len() + 1);
                let new = obj.add_layer(name);
                // Camada nova ganha uma chave EM BRANCO no quadro atual (regra do
                // GP): senão ela nasce sem célula nenhuma e a tira fica muda até o
                // primeiro traço.
                let frame = obj.frame_at(playhead);
                obj.insert_frame(
                    new,
                    frame,
                    ph2d_flip::Hold::Implicit,
                    ph2d_flip::KeyKind::Keyframe,
                );
                *active_layer = Some(new);
                return true;
            }
            false
        }
        PanelEvent::Click(id) if *id == ids::FLIP_LAYER_DUPLICATE => {
            // Duplica a camada ATIVA (§4.C) — uma cópia independente, acima da original,
            // que vira a ativa (o Illustrator/PS). Sem camada ativa: no-op (o botão já
            // nasce desabilitado, mas a recusa mora aqui também, não só na pintura).
            let Some(target) = *active_layer else {
                return false;
            };
            if let Some(obj) = flip.object_mut(oid)
                && let Some(new) = obj.duplicate_layer(target)
            {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_flip::FlipDoc;

    /// A doc with one object + two layers (bottom `a`, top `b`); `b` active.
    fn doc_2layers() -> (FlipDoc, LayerId, LayerId) {
        let mut doc = FlipDoc::new();
        let oid = doc.push_object("Flip");
        let obj = doc.object_mut(oid).unwrap();
        let a = obj.add_layer("A");
        let b = obj.add_layer("B");
        (doc, a, b)
    }

    fn wid(layer: LayerId, kind: FlipLayerWidget) -> ph2d_editor::NodeId {
        ids::flip_layer_widget_id(u64::from(layer.0), kind)
    }

    #[test]
    fn add_and_delete_layer() {
        let (mut doc, _a, _b) = doc_2layers();
        let oid = doc.objects().first().unwrap().id;
        let mut active = None;
        assert!(apply_panel_event(
            &PanelEvent::Click(ids::FLIP_LAYER_ADD),
            &mut doc,
            &mut active,
            &ph2d_core::Playhead::default(),
            false,
        ));
        assert_eq!(doc.object(oid).unwrap().layers().len(), 3);
        assert!(active.is_some(), "new layer becomes active");
        // Delete the active layer.
        assert!(apply_panel_event(
            &PanelEvent::Click(ids::FLIP_LAYER_DELETE),
            &mut doc,
            &mut active,
            &ph2d_core::Playhead::default(),
            false,
        ));
        assert_eq!(doc.object(oid).unwrap().layers().len(), 2);
    }

    /// 🔴 **O botão Duplicate DUPLICA a camada ativa e a cópia vira a ativa** (§4.C) — o
    /// consumidor real (`apply_panel_event`) sobre o doc, não uma cópia da decisão.
    ///
    /// Mutação que sangra: o braço do `FLIP_LAYER_DUPLICATE` chamar `add_layer` (camada
    /// vazia) em vez de `duplicate_layer` — o count cresce igual, mas a cópia não leva a
    /// arte, e o `drawings().len()` não dobra.
    #[test]
    fn duplicate_layer_button_copies_the_active_layer() {
        use ph2d_flip::{Hold, KeyKind, Point, Rgba};
        let (mut doc, _a, b) = doc_2layers();
        let oid = doc.objects().first().unwrap().id;
        // A camada ativa `b` ganha um desenho não-vazio.
        let obj = doc.object_mut(oid).unwrap();
        let d = obj
            .insert_frame(b, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let mut s = ph2d_flip::FlipStroke::new();
        s.push_point(Point {
            pos: ph2d_core::Vec2::new(0.0, 0.0),
            width: 2.0,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
        obj.drawing_mut(d).unwrap().strokes.push(s);
        let drawings_before = obj.drawings().len();

        let mut active = Some(b);
        assert!(apply_panel_event(
            &PanelEvent::Click(ids::FLIP_LAYER_DUPLICATE),
            &mut doc,
            &mut active,
            &ph2d_core::Playhead::default(),
            false,
        ));
        let obj = doc.object(oid).unwrap();
        assert_eq!(obj.layers().len(), 3, "a cópia é uma camada nova");
        assert_ne!(active, Some(b), "a cópia vira a ativa");
        assert!(active.is_some());
        assert_eq!(
            obj.drawings().len(),
            drawings_before + 1,
            "a cópia leva a ARTE (um desenho novo), não é uma camada vazia"
        );
    }

    /// Sem camada ativa, Duplicate é no-op (a recusa mora no apply, não só na pintura).
    #[test]
    fn duplicate_layer_button_is_a_noop_without_an_active_layer() {
        let (mut doc, _a, _b) = doc_2layers();
        let oid = doc.objects().first().unwrap().id;
        let before = doc.object(oid).unwrap().layers().len();
        let mut active = None;
        assert!(!apply_panel_event(
            &PanelEvent::Click(ids::FLIP_LAYER_DUPLICATE),
            &mut doc,
            &mut active,
            &ph2d_core::Playhead::default(),
            false,
        ));
        assert_eq!(doc.object(oid).unwrap().layers().len(), before);
    }

    #[test]
    fn row_select_sets_active_layer() {
        let (mut doc, a, _b) = doc_2layers();
        let mut active = None;
        // Row-select is not a doc mutation (returns false) but sets active.
        assert!(!apply_panel_event(
            &PanelEvent::Click(wid(a, FlipLayerWidget::Row)),
            &mut doc,
            &mut active,
            &ph2d_core::Playhead::default(),
            false,
        ));
        assert_eq!(active, Some(a));
    }

    #[test]
    fn visibility_and_lock_toggle() {
        let (mut doc, a, _b) = doc_2layers();
        let oid = doc.objects().first().unwrap().id;
        let mut active = None;
        assert!(apply_panel_event(
            &PanelEvent::Click(wid(a, FlipLayerWidget::Visibility)),
            &mut doc,
            &mut active,
            &ph2d_core::Playhead::default(),
            false,
        ));
        assert!(!doc.object(oid).unwrap().layer(a).unwrap().visible);
        assert!(apply_panel_event(
            &PanelEvent::Click(wid(a, FlipLayerWidget::Lock)),
            &mut doc,
            &mut active,
            &ph2d_core::Playhead::default(),
            false,
        ));
        assert!(doc.object(oid).unwrap().layer(a).unwrap().locked);
    }

    #[test]
    fn reorder_up_swaps_layers() {
        let (mut doc, a, b) = doc_2layers(); // [a, b]
        let oid = doc.objects().first().unwrap().id;
        let mut active = None;
        // Raise the bottom layer `a` → [b, a].
        assert!(apply_panel_event(
            &PanelEvent::Click(wid(a, FlipLayerWidget::MoveUp)),
            &mut doc,
            &mut active,
            &ph2d_core::Playhead::default(),
            false,
        ));
        let ids_now: Vec<LayerId> = doc
            .object(oid)
            .unwrap()
            .layers()
            .iter()
            .map(|l| l.id)
            .collect();
        assert_eq!(ids_now, vec![b, a]);
    }

    #[test]
    fn opacity_setvalue_and_blend_selectoption() {
        let (mut doc, _a, b) = doc_2layers();
        let oid = doc.objects().first().unwrap().id;
        let mut active = None;
        // Opacity slider on the top layer.
        assert!(apply_panel_event(
            &PanelEvent::SetValue(wid(b, FlipLayerWidget::Opacity), 0.4),
            &mut doc,
            &mut active,
            &ph2d_core::Playhead::default(),
            false,
        ));
        assert!((doc.object(oid).unwrap().layer(b).unwrap().opacity - 0.4).abs() < 1e-6);
        // Blend dropdown option → SelectOption(blend_chip_id, mode). Mode 1 = Multiply.
        assert!(apply_panel_event(
            &PanelEvent::SelectOption(wid(b, FlipLayerWidget::Blend), "1".to_string()),
            &mut doc,
            &mut active,
            &ph2d_core::Playhead::default(),
            false,
        ));
        assert_eq!(
            doc.object(oid).unwrap().layer(b).unwrap().blend,
            BlendMode::Multiply,
            "blend option applied through the seam"
        );
    }
}
