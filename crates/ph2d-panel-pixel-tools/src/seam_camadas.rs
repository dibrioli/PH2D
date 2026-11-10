//! A costura das CAMADAS no painel PIXEL — o registo por quadro das partes das linhas, o que a
//! camada Normal Map recusa, o renomear e o pedido de cada clique numa linha (filho de
//! [`crate::seam`], que re-exporta as entradas; ficheiro irmão pelo tecto de 600 LOC, P14).

use crate::ids;
use crate::state::with_current;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::pixel_panel::PixelRequest;
use ph2d_editor_core::widget::{ButtonState, CheckboxState, CheckboxValue, TextInputState};
use ph2d_i18n::tr;

/// O registo POR QUADRO das partes das linhas de camada (os ids derivam do id da camada), com a
/// dica de cada ícone.
pub(crate) fn register_layers(
    store: &mut WidgetStore,
    rows: &[ph2d_editor_core::pixel_panel::PixelLayerRow],
) {
    let dicas = [
        (ids::LayerWidget::Eye, "panel.pixel.layer.eye"),
        (ids::LayerWidget::Lock, "panel.pixel.layer.lock"),
        (ids::LayerWidget::Solo, "panel.pixel.layer.solo"),
        (ids::LayerWidget::Collapse, "panel.pixel.layer.collapse"),
        (ids::LayerWidget::Normal, "panel.pixel.layer.normal_tip"),
        (ids::LayerWidget::Effects, "panel.pixel.layer.effects_tip"),
    ];
    for id in [
        ids::LAYER_FX_OUTLINE,
        ids::LAYER_FX_INSIDE,
        ids::LAYER_FX_SQUARE,
    ] {
        let (state, value) = (CheckboxState::Normal, CheckboxValue::Unchecked);
        store.register_if_absent(id, InteractiveState::Checkbox { state, value });
    }
    for (id, _) in ids::LAYER_FX_BUTTONS {
        let state = ButtonState::Normal;
        store.register_if_absent(id, InteractiveState::Button { state });
    }
    store.set_tooltip(ids::LAYER_FX_OUTLINE, tr("pixel.layer.effects.outline_tip"));
    let apply = ids::LAYER_FX_BUTTONS[1].0;
    store.set_tooltip(apply, tr("pixel.layer.effects.apply.tooltip"));
    for r in rows {
        for w in ids::LayerWidget::ALL {
            let id = ids::layer_widget_id(r.id, w);
            let state = ButtonState::Normal;
            store.register_if_absent(id, InteractiveState::Button { state });
            if let Some((_, k)) = dicas.iter().find(|(x, _)| *x == w) {
                store.set_tooltip(id, tr(k));
            }
        }
    }
}

/// P14c-pixel 1b, por quadro: o que a camada Normal Map recusa fica DESLIGADO e a dica diz porquê —
/// o Merge Down entre normal e cor, e a caixa Normal Map fora de um sprite RGB (a cel guarda o modo
/// do sprite). Desligado não aceita o clique (`is_focusable`); a dica chega ao ponteiro pelo rect.
pub(crate) fn desliga_o_que_a_normal_recusa(
    store: &mut WidgetStore,
    i: &ph2d_editor_core::pixel_panel::PixelPanelInfo,
) {
    use ph2d_editor_core::pixel_panel::{NewSpriteMode, PixelLayerAction};
    let merge = ids::LAYER_ACTIONS
        .iter()
        .find(|(_, a, _)| *a == PixelLayerAction::MergeDown)
        .map(|(id, ..)| *id);
    if let Some(id) = merge
        && let Some(InteractiveState::Button { state }) = store.get_mut(id)
    {
        if i.merge_down_mixes_normal {
            *state = ButtonState::Disabled;
        } else if *state == ButtonState::Disabled {
            *state = ButtonState::Normal;
        }
        let dica = if i.merge_down_mixes_normal {
            tr("panel.pixel.layer.merge_normal_tip")
        } else {
            ""
        };
        store.set_tooltip(id, dica);
    }
    let rgb = i.mode == NewSpriteMode::Rgba;
    if let Some(InteractiveState::Checkbox { state, .. }) = store.get_mut(ids::LAYER_NORMAL) {
        if !rgb {
            *state = CheckboxState::Disabled;
        } else if *state == CheckboxState::Disabled {
            *state = CheckboxState::Normal;
        }
    }
    let dica = if rgb {
        tr("panel.pixel.layer.normal_tip")
    } else {
        tr("panel.pixel.layer.normal_needs_rgb")
    };
    store.set_tooltip(ids::LAYER_NORMAL, dica);
}

/// Semeia e foca o campo do renomear UMA vez quando ele abre (semear a cada quadro apagaria o que
/// o artista escreve); se a camada sumiu, o renomear fecha.
pub(crate) fn seed_rename(st: &mut crate::PixelToolsPanelState, store: &mut WidgetStore) {
    let Some((id, false)) = st.rename else {
        return;
    };
    let Some(name) = with_current(|i| i.layers.iter().find(|r| r.id == id).map(|r| r.name.clone()))
    else {
        st.rename = None;
        return;
    };
    let caret = name.len();
    store.register(
        ids::LAYER_RENAME,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: name,
            caret,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(ids::LAYER_RENAME));
    store.mark_cancel_on_escape(ids::LAYER_RENAME);
    st.rename = Some((id, true));
}

/// Layer Properties ▸ Normal Map: alterna a da camada activa.
pub(crate) fn pedido_normal() -> Option<PixelRequest> {
    with_current(|i| {
        i.active_layer
            .filter(|_| i.editing)
            .map(PixelRequest::LayerNormal)
    })
}

/// O pedido de um clique numa parte de linha de camada.
pub(crate) fn pedido_camada(id: NodeId) -> Option<PixelRequest> {
    use ids::LayerWidget as W;
    with_current(|info| {
        if !info.editing {
            return None;
        }
        if let Some(e) = efeito_de(info, id) {
            return Some(e);
        }
        if let Some((_, a, _)) = ids::LAYER_ACTIONS.iter().find(|(b, ..)| *b == id) {
            return Some(PixelRequest::LayerAction(*a));
        }
        info.layers.iter().find_map(|r| {
            W::ALL.iter().find_map(|&w| {
                (ids::layer_widget_id(r.id, w) == id).then_some(match w {
                    W::Row => PixelRequest::LayerSelect(r.id),
                    W::Eye => PixelRequest::LayerVisible(r.id),
                    W::Lock => PixelRequest::LayerLocked(r.id),
                    W::Solo => PixelRequest::LayerSolo(r.id),
                    W::Collapse => PixelRequest::LayerCollapsed(r.id),
                    W::Continuous => PixelRequest::LayerContinuous(r.id),
                    W::Normal => PixelRequest::LayerNormal(r.id),
                    W::Effects => PixelRequest::LayerSelect(r.id),
                })
            })
        })
    })
}

/// P13: o pedido de uma caixa ou botão de Layer Properties ▸ Effects (o valor vem do MODELO).
pub(crate) fn pedido_efeito(id: NodeId) -> Option<PixelRequest> {
    with_current(|i| efeito_de(i, id).filter(|_| i.editing))
}

fn efeito_de(
    i: &ph2d_editor_core::pixel_panel::PixelPanelInfo,
    id: NodeId,
) -> Option<PixelRequest> {
    use ph2d_editor_core::pixel_panel::PixelLayerEffect as E;
    let layer = i.active_layer?;
    let o = i.layers.iter().find(|r| r.id == layer)?.outline;
    let b = ids::LAYER_FX_BUTTONS;
    let e = match (id, o) {
        (x, None) if x == ids::LAYER_FX_OUTLINE => E::On(i.fg),
        (x, Some(_)) if x == ids::LAYER_FX_OUTLINE || x == b[2].0 => E::Off,
        (x, Some(o)) if x == ids::LAYER_FX_INSIDE => E::Inside(!o.inside),
        (x, Some(o)) if x == ids::LAYER_FX_SQUARE => E::Square(!o.square),
        (x, Some(_)) if x == b[0].0 => E::Color(i.fg),
        (x, Some(_)) if x == b[1].0 => E::Apply,
        _ => return None,
    };
    Some(PixelRequest::LayerEffect(layer, e))
}

/// O renomear de uma camada: duplo clique na linha abre o campo; Enter/perder o foco grava num
/// passo; Esc desiste. `None` = o evento não é deste fluxo.
pub(crate) fn renomear(
    st: &mut crate::PixelToolsPanelState,
    host: &mut dyn PanelHostInternal,
    ev: &WidgetEvent,
) -> Option<EventOutcome> {
    match *ev {
        WidgetEvent::DoubleClick(id) => {
            let alvo = with_current(|i| {
                i.layers
                    .iter()
                    .find(|r| ids::layer_widget_id(r.id, ids::LayerWidget::Row) == id)
                    .map(|r| r.id)
            });
            Some(match alvo {
                Some(layer) => {
                    st.rename = Some((layer, false));
                    EventOutcome::Consumed
                }
                None => EventOutcome::Ignored,
            })
        }
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == ids::LAYER_RENAME => {
            let Some((layer, _)) = st.rename.take() else {
                return Some(EventOutcome::Consumed);
            };
            let name = match host.store().get(ids::LAYER_RENAME) {
                Some(InteractiveState::TextInput { text, .. }) => text.trim().to_owned(),
                _ => String::new(),
            };
            if host.store().focus_id() == Some(ids::LAYER_RENAME) {
                host.store_mut().set_focus(None);
            }
            let pedido = Some(PixelRequest::LayerRename { id: layer, name });
            Some(crate::seam::empurra(host, pedido))
        }
        WidgetEvent::Cancel(id) if id == ids::LAYER_RENAME => {
            st.rename = None;
            if host.store().focus_id() == Some(ids::LAYER_RENAME) {
                host.store_mut().set_focus(None);
            }
            Some(EventOutcome::Consumed)
        }
        _ => None,
    }
}
