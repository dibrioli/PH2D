//! ⭐⭐ **O QUE UM GESTO DO PAINEL DE CAMADAS PEDE** — o [`PanelEvent`] lido UMA vez como um pedido
//! tipado ([`LayerEdit`]), que a pilha do documento aplica ([`PainterTool::apply_layer_edit`]). O
//! formato do fio — os ids por camada e as cargas `"camada:canal:…"` — tem uma leitura só.

use ph2d_a11y::NodeId;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_effects::adjustments::AdjustmentKind;

use super::*;
use crate::ids::{self, PainterLayerWidget};

/// Um pedido do painel de camadas. Os `*Active` agem sobre a camada activa de quem o lê.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LayerEdit {
    AddRaster,
    AddGroup,
    DuplicateActive,
    DeleteActive,
    MaskActive,
    ToggleClipActive,
    ToggleAlphaLockActive,
    ToggleReferenceActive,
    AddAdjustment(AdjustmentKind),
    /// Clique numa linha — o `NodeId` é a chave dos modificadores (Cmd/Shift) que o painel guardou.
    Row(RtLayerId, NodeId),
    Visibility(RtLayerId),
    MoveUp(RtLayerId),
    MoveDown(RtLayerId),
    ImpastoLevel(RtLayerId),
    MaskInvert(RtLayerId),
    MaskApply(RtLayerId),
    MaskView(RtLayerId),
    Opacity(RtLayerId, f32),
    ImpastoDepth(RtLayerId, f32),
    Blend(RtLayerId, BlendMode),
    Param(RtLayerId, ParamEdit),
}

/// Um pedido sobre os PARÂMETROS de um ajuste — metadado puro.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ParamEdit {
    Slider(usize, f32),
    Toggle(usize),
    Segment(usize),
    CurvePoint {
        channel: u8,
        index: usize,
        x: f32,
        y: f32,
    },
    CurveAdd(u8),
    CurveRemove(u8, usize),
    Mixer {
        output: usize,
        slot: usize,
        value: f32,
    },
    GradientOffset {
        stop: usize,
        offset: f32,
    },
    GradientAdd,
    GradientRemove(usize),
    GradientColor {
        stop: usize,
        slot: usize,
        value: f32,
    },
    SelectiveColor {
        bucket: usize,
        slot: usize,
        value: f32,
    },
}

/// ⭐⭐ **A leitura única do fio.** `find` devolve `(camada, widget)` de um id por camada — sobre a pilha
/// de quem lê. `None` = o evento não é do painel de camadas.
pub(crate) fn decode(
    ev: &PanelEvent,
    find: impl Fn(NodeId) -> Option<(RtLayerId, PainterLayerWidget)>,
) -> Option<LayerEdit> {
    use LayerEdit as E;
    use PainterLayerWidget as W;
    match ev {
        PanelEvent::Click(id) => {
            let id = *id;
            let fixo = [
                (ids::PAINTER_LAYERS_ADD, E::AddRaster),
                (ids::PAINTER_LAYERS_DUPLICATE, E::DuplicateActive),
                (ids::PAINTER_LAYERS_DELETE, E::DeleteActive),
                (ids::PAINTER_LAYERS_GROUP, E::AddGroup),
                (ids::PAINTER_LAYERS_MASK, E::MaskActive),
                (ids::PAINTER_LAYERS_CLIP, E::ToggleClipActive),
                (ids::PAINTER_LAYERS_ALPHA_LOCK, E::ToggleAlphaLockActive),
                (ids::PAINTER_LAYERS_REFERENCE, E::ToggleReferenceActive),
            ];
            if let Some((_, e)) = fixo.into_iter().find(|(k, _)| *k == id) {
                return Some(e);
            }
            let (l, w) = find(id)?;
            Some(match w {
                W::Row => E::Row(l, id),
                W::Visibility => E::Visibility(l),
                W::MoveUp => E::MoveUp(l),
                W::MoveDown => E::MoveDown(l),
                W::ImpastoLevel => E::ImpastoLevel(l),
                W::MaskInvert => E::MaskInvert(l),
                W::MaskApply => E::MaskApply(l),
                W::MaskView => E::MaskView(l),
                W::AdjToggle0 => E::Param(l, ParamEdit::Toggle(0)),
                W::AdjToggle1 => E::Param(l, ParamEdit::Toggle(1)),
                W::AdjSegment0 => E::Param(l, ParamEdit::Segment(0)),
                W::AdjSegment1 => E::Param(l, ParamEdit::Segment(1)),
                W::AdjSegment2 => E::Param(l, ParamEdit::Segment(2)),
                _ => return None,
            })
        }
        PanelEvent::SetValue(id, v) => {
            let (l, w) = find(*id)?;
            let v = *v as f32;
            let slot = match w {
                W::Opacity => return Some(E::Opacity(l, v)),
                W::ImpastoDepth => return Some(E::ImpastoDepth(l, v)),
                W::AdjParam0 => 0,
                W::AdjParam1 => 1,
                W::AdjParam2 => 2,
                W::AdjParam3 => 3,
                W::AdjParam4 => 4,
                W::AdjParam5 => 5,
                W::AdjParam6 => 6,
                W::AdjParam7 => 7,
                _ => return None,
            };
            Some(E::Param(l, ParamEdit::Slider(slot, v)))
        }
        PanelEvent::SelectOption(id, value) => decode_option(*id, value, find),
        PanelEvent::Toggle(..) => None,
    }
}

fn decode_option(
    id: NodeId,
    value: &str,
    find: impl Fn(NodeId) -> Option<(RtLayerId, PainterLayerWidget)>,
) -> Option<LayerEdit> {
    use LayerEdit as E;
    let mut it = value.split(':');
    let camada = |s: Option<&str>| s?.parse::<u64>().ok().map(RtLayerId);
    if id == ids::PAINTER_LAYERS_ADD_ADJUSTMENT {
        let k = *AdjustmentKind::ALL.get(value.parse::<usize>().ok()?)?;
        return Some(E::AddAdjustment(k));
    }
    let p = if id == ids::PAINTER_CURVE_EDIT {
        let l = camada(it.next())?;
        let (c, i, x, y) = (it.next()?, it.next()?, it.next()?, it.next()?);
        let edit = ParamEdit::CurvePoint {
            channel: c.parse().ok()?,
            index: i.parse().ok()?,
            x: x.parse().ok()?,
            y: y.parse().ok()?,
        };
        (l, edit)
    } else if id == ids::PAINTER_MIXER_EDIT {
        let l = camada(it.next())?;
        let (o, s, v) = (it.next()?, it.next()?, it.next()?);
        let edit = ParamEdit::Mixer {
            output: o.parse().ok()?,
            slot: s.parse().ok()?,
            value: v.parse().ok()?,
        };
        (l, edit)
    } else if id == ids::PAINTER_GRADIENT_EDIT {
        let l = camada(it.next())?;
        let (i, o) = (it.next()?, it.next()?);
        let edit = ParamEdit::GradientOffset {
            stop: i.parse().ok()?,
            offset: o.parse().ok()?,
        };
        (l, edit)
    } else if id == ids::PAINTER_GRADIENT_ADD {
        (RtLayerId(value.parse().ok()?), ParamEdit::GradientAdd)
    } else if id == ids::PAINTER_GRADIENT_REMOVE {
        let l = camada(it.next())?;
        (l, ParamEdit::GradientRemove(it.next()?.parse().ok()?))
    } else if id == ids::PAINTER_GRADIENT_COLOR {
        let l = camada(it.next())?;
        let (st, s, v) = (it.next()?, it.next()?, it.next()?);
        let edit = ParamEdit::GradientColor {
            stop: st.parse().ok()?,
            slot: s.parse().ok()?,
            value: v.parse().ok()?,
        };
        (l, edit)
    } else if id == ids::PAINTER_SELCOLOR_EDIT {
        let l = camada(it.next())?;
        let (b, s, v) = (it.next()?, it.next()?, it.next()?);
        let edit = ParamEdit::SelectiveColor {
            bucket: b.parse().ok()?,
            slot: s.parse().ok()?,
            value: v.parse().ok()?,
        };
        (l, edit)
    } else if id == ids::PAINTER_CURVE_ADD {
        let l = camada(it.next())?;
        (l, ParamEdit::CurveAdd(it.next()?.parse().ok()?))
    } else if id == ids::PAINTER_CURVE_REMOVE {
        let l = camada(it.next())?;
        let (c, i) = (it.next()?, it.next()?);
        (l, ParamEdit::CurveRemove(c.parse().ok()?, i.parse().ok()?))
    } else {
        let (l, w) = find(id)?;
        if w != PainterLayerWidget::Blend {
            return None;
        }
        return Some(E::Blend(l, BlendMode::from_u8(value.parse().ok()?)));
    };
    Some(E::Param(p.0, p.1))
}

/// `(camada, widget)` de um id por camada, sobre a pilha `stack`.
pub(crate) fn layer_widget_in(
    stack: &LayerStack,
    id: NodeId,
) -> Option<(RtLayerId, PainterLayerWidget)> {
    use crate::ids::painter_layer_widget_id;
    stack.all_ids().find_map(|layer| {
        PainterLayerWidget::ALL
            .into_iter()
            .find(|&kind| painter_layer_widget_id(layer.0, kind) == id)
            .map(|kind| (layer, kind))
    })
}

impl PainterTool {
    /// ⭐ **Um pedido do painel sobre a pilha do DOCUMENTO 2D** — cada braço é o que o
    /// `handle_panel_event` fazia antes de a leitura do fio ganhar uma porta só.
    pub(crate) fn apply_layer_edit(&mut self, e: LayerEdit) {
        use LayerEdit as E;
        let active = self.layers.active();
        match e {
            E::AddRaster => {
                let name = format!("Layer {}", self.layers.len() + 1);
                self.add_raster_layer(name);
            }
            E::DuplicateActive => {
                if let Some(a) = active {
                    self.duplicate_layer(a);
                }
            }
            E::DeleteActive => {
                if let Some(a) = active {
                    self.delete_layer(a);
                }
            }
            E::AddGroup => {
                self.group_selected();
            }
            E::MaskActive => {
                self.add_mask_to_active();
            }
            E::ToggleClipActive => {
                if let Some(a) = active {
                    let now = self.layers.get(a).is_some_and(|l| l.clipping);
                    self.set_layer_clipping(a, !now);
                }
            }
            E::ToggleAlphaLockActive => {
                if let Some(a) = active {
                    let now = self.layers.get(a).is_some_and(|l| l.alpha_locked);
                    self.set_layer_alpha_locked(a, !now);
                }
            }
            E::ToggleReferenceActive => {
                if let Some(a) = active {
                    let now = self.layers.get(a).is_some_and(|l| l.is_reference);
                    self.set_layer_reference(a, !now);
                }
            }
            E::AddAdjustment(kind) => {
                self.add_adjustment_layer(kind);
            }
            // Multi-select (panel stashed Cmd/Shift): Shift = range, Cmd/Ctrl = additive, plain = single.
            E::Row(layer, row) => {
                let (cmd, shift) = take_pending_select_mods(row);
                if shift {
                    self.select_range(layer);
                } else if cmd {
                    self.select_additive(layer);
                } else {
                    self.select_single(layer);
                }
            }
            E::Visibility(layer) => {
                let now = self.layers.get(layer).map(|l| l.visible).unwrap_or(true);
                self.set_layer_visible(layer, !now);
            }
            E::MoveUp(layer) => self.move_layer_up(layer),
            E::MoveDown(layer) => self.move_layer_down(layer),
            E::ImpastoLevel(layer) => self.toggle_layer_impasto_composite(layer),
            E::MaskInvert(layer) => self.toggle_mask_inverted(layer),
            E::MaskApply(layer) => {
                self.apply_mask(layer);
            }
            E::MaskView(layer) => self.toggle_mask_view_grayscale(layer),
            E::Opacity(layer, v) => self.set_layer_opacity(layer, v),
            E::ImpastoDepth(layer, v) => self.set_layer_impasto_depth_norm(layer, v),
            E::Blend(layer, mode) => self.set_layer_blend_mode(layer, mode),
            E::Param(layer, p) => self.apply_param_edit_2d(layer, p),
        }
    }

    /// Os métodos públicos de sempre — cada um com a sua via rápida de cache.
    fn apply_param_edit_2d(&mut self, layer: RtLayerId, p: ParamEdit) {
        match p {
            ParamEdit::Slider(slot, v) => self.set_adjustment_param(layer, slot, v),
            ParamEdit::Toggle(slot) => self.flip_adjustment_toggle(layer, slot),
            ParamEdit::Segment(o) => self.set_adjustment_segment(layer, o),
            ParamEdit::CurvePoint {
                channel,
                index,
                x,
                y,
            } => self.set_curve_point(layer, channel, index, x, y),
            ParamEdit::CurveAdd(ch) => {
                self.add_curve_point(layer, ch);
            }
            ParamEdit::CurveRemove(ch, i) => self.remove_curve_point(layer, ch, i),
            ParamEdit::Mixer {
                output,
                slot,
                value,
            } => self.set_channel_mixer_weight(layer, output, slot, value),
            ParamEdit::GradientOffset { stop, offset } => {
                self.set_gradient_stop_offset(layer, stop, offset);
            }
            ParamEdit::GradientAdd => {
                self.add_gradient_stop(layer);
            }
            ParamEdit::GradientRemove(stop) => self.remove_gradient_stop(layer, stop),
            ParamEdit::GradientColor { stop, slot, value } => {
                self.set_gradient_stop_color(layer, stop, slot, value);
            }
            ParamEdit::SelectiveColor {
                bucket,
                slot,
                value,
            } => self.set_selective_color_value(layer, bucket, slot, value),
        }
    }
}
