//! A costura da linha do tempo — o registo dos widgets e o pedido que cada gesto faz.
//!
//! ⚠️ Os botões e campos registam-se INCONDICIONALMENTE em [`populate`]; os cabeçalhos, as células,
//! as tags e as linhas de camada (os ids derivam da linha do tempo) registam-se por quadro, por
//! quem os pinta ([`register_frame`]).

use crate::PixelTimelinePanelState;
use crate::ids;
use crate::state::with_current;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::pixel_layer_row::PixelLayerWidget as W;
use ph2d_editor_core::pixel_panel::{PixelDirection, PixelPanelInfo, PixelRequest};
use ph2d_editor_core::widget::{ButtonState, DropdownState, TextInputState};
use ph2d_i18n::tr;

fn botao(store: &mut WidgetStore, id: NodeId) {
    let state = ButtonState::Normal;
    store.register(id, InteractiveState::Button { state });
}

pub(crate) fn populate(store: &mut WidgetStore) {
    botao(store, ids::CLOSE);
    for (_, id, _, dica) in ids::TRANSPORT {
        botao(store, id);
        store.set_tooltip(id, tr(dica));
    }
    for (_, id, _) in ids::TOGGLES {
        botao(store, id);
    }
    for (_, id, _, dica) in ids::ACTIONS {
        botao(store, id);
        store.set_tooltip(id, tr(dica));
    }
    botao(store, ids::TAG_DELETE);
    for (id, (min, max, step)) in ids::NUMBERS {
        let value = min.max(0.0);
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: value.to_string(),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, min, max, step);
    }
    store.register(
        ids::TAG_DIRECTION,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(0),
        },
    );
    for i in 0..PixelDirection::ALL.len() {
        botao(store, ids::direction_option_id(i));
    }
}

/// O registo POR QUADRO dos ids que derivam da linha do tempo.
pub(crate) fn register_frame(store: &mut WidgetStore, info: &PixelPanelInfo) {
    for r in &info.layers {
        for w in W::ALL {
            let id = ids::layer_widget_id(r.id, w);
            let state = ButtonState::Normal;
            store.register_if_absent(id, InteractiveState::Button { state });
            if let Some(k) = w.tip_key() {
                store.set_tooltip(id, tr(k));
            }
        }
        for f in &info.frames {
            store.register_if_absent(ids::cell_id(r.id, f.id), InteractiveState::Plain);
        }
    }
    for f in &info.frames {
        store.register_if_absent(ids::header_id(f.id), InteractiveState::Plain);
    }
    for i in 0..info.tags.len() {
        store.register_if_absent(ids::tag_id(i), InteractiveState::Plain);
    }
}

/// Semeia e foca um campo de renomear UMA vez quando ele abre; se o alvo sumiu, fecha.
pub(crate) fn seed_renames(st: &mut PixelTimelinePanelState, store: &mut WidgetStore) {
    if let Some((layer, false)) = st.rename_layer {
        let nome = with_current(|i| {
            i.layers
                .iter()
                .find(|r| r.id == layer)
                .map(|r| r.name.clone())
        });
        st.rename_layer = nome.map(|n| {
            semeia(store, ids::RENAME_LAYER, n);
            (layer, true)
        });
    }
    if let Some((tag, false)) = st.rename_tag {
        let nome = with_current(|i| i.tags.get(tag).map(|t| t.name.clone()));
        st.rename_tag = nome.map(|n| {
            semeia(store, ids::RENAME_TAG, n);
            (tag, true)
        });
    }
}

fn semeia(store: &mut WidgetStore, id: NodeId, text: String) {
    let caret = text.len();
    store.register(
        id,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text,
            caret,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(id));
    store.mark_cancel_on_escape(id);
}

/// A tag que cobre o frame activo (as propriedades mostram-na; a primeira, como no Aseprite).
pub(crate) fn tag_activa(info: &PixelPanelInfo) -> Option<usize> {
    let pos = activo(info)?;
    let pos = u32::try_from(pos).ok()?;
    info.tags.iter().position(|t| t.from <= pos && pos <= t.to)
}

/// A posição do frame activo.
pub(crate) fn activo(info: &PixelPanelInfo) -> Option<usize> {
    let a = info.active_frame?;
    info.frames.iter().position(|f| f.id == a)
}

/// O pedido de um clique, se o id é deste painel e há sprite em edição.
fn pedido(id: NodeId) -> Option<PixelRequest> {
    with_current(|info| {
        if !info.editing {
            return None;
        }
        let acoes = ids::TRANSPORT.iter().map(|(a, b, ..)| (*a, *b));
        let acoes = acoes
            .chain(ids::TOGGLES.iter().map(|(a, b, _)| (*a, *b)))
            .chain(ids::ACTIONS.iter().map(|(a, b, ..)| (*a, *b)));
        for (a, b) in acoes {
            if b == id {
                return Some(PixelRequest::FrameAction(a));
            }
        }
        if id == ids::TAG_DELETE {
            return tag_activa(info).map(PixelRequest::TagDelete);
        }
        if let Some(f) = info.frames.iter().find(|f| ids::header_id(f.id) == id) {
            return Some(PixelRequest::FrameSelect(f.id));
        }
        if let Some(i) = (0..info.tags.len()).find(|&i| ids::tag_id(i) == id) {
            let pos = usize::try_from(info.tags[i].from).ok()?;
            return info
                .frames
                .get(pos)
                .map(|f| PixelRequest::FrameSelect(f.id));
        }
        for r in &info.layers {
            for w in W::ALL {
                if ids::layer_widget_id(r.id, w) == id {
                    return Some(match w {
                        W::Row => PixelRequest::LayerSelect(r.id),
                        W::Eye => PixelRequest::LayerVisible(r.id),
                        W::Lock => PixelRequest::LayerLocked(r.id),
                        W::Solo => PixelRequest::LayerSolo(r.id),
                        W::Collapse => PixelRequest::LayerCollapsed(r.id),
                    });
                }
            }
            if let Some(f) = info.frames.iter().find(|f| ids::cell_id(r.id, f.id) == id) {
                return Some(PixelRequest::CelSelect {
                    layer: r.id,
                    frame: f.id,
                });
            }
        }
        None
    })
}

/// O valor inteiro de um campo, preso à faixa dele.
fn inteiro(host: &dyn PanelHostInternal, id: NodeId) -> Option<i64> {
    let v = host.store().number_value(id).filter(|v| v.is_finite())?;
    let (min, max) = ids::NUMBERS
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, (a, b, _))| (*a, *b))?;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "presa à faixa do campo antes da conversão"
    )]
    Some(ph2d_editor_core::math::safe_clamp_f64(v.round(), min, max) as i64)
}

fn numero(host: &dyn PanelHostInternal, id: NodeId) -> Option<PixelRequest> {
    let v = inteiro(host, id)?;
    with_current(|info| {
        if !info.editing {
            return None;
        }
        match id {
            ids::FRAME_DURATION => u32::try_from(v).ok().map(PixelRequest::FrameDuration),
            ids::CEL_OPACITY => u8::try_from(v).ok().map(PixelRequest::CelOpacity),
            ids::CEL_Z => i16::try_from(v).ok().map(PixelRequest::CelZ),
            ids::TAG_REPEAT => {
                let index = tag_activa(info)?;
                let repeat = u16::try_from(v).ok()?;
                Some(PixelRequest::TagRepeat { index, repeat })
            }
            _ => None,
        }
    })
}

/// Fecha um campo de renomear e devolve o texto dele (sem espaços nas pontas).
fn fecha_campo(host: &mut dyn PanelHostInternal, id: NodeId) -> String {
    let name = match host.store().get(id) {
        Some(InteractiveState::TextInput { text, .. }) => text.trim().to_owned(),
        _ => String::new(),
    };
    if host.store().focus_id() == Some(id) {
        host.store_mut().set_focus(None);
    }
    name
}

pub(crate) fn apply_event(
    st: &mut PixelTimelinePanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        WidgetEvent::DoubleClick(id) => {
            let (camada, tag) = with_current(|i| {
                let c = i
                    .layers
                    .iter()
                    .find(|r| ids::layer_widget_id(r.id, W::Row) == id)
                    .map(|r| r.id);
                let t = (0..i.tags.len()).find(|&k| ids::tag_id(k) == id);
                (c.filter(|_| i.editing), t.filter(|_| i.editing))
            });
            if let Some(c) = camada {
                st.rename_layer = Some((c, false));
                return EventOutcome::Consumed;
            }
            if let Some(t) = tag {
                st.rename_tag = Some((t, false));
                return EventOutcome::Consumed;
            }
            EventOutcome::Ignored
        }
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == ids::RENAME_LAYER => {
            let Some((layer, _)) = st.rename_layer.take() else {
                return EventOutcome::Consumed;
            };
            let name = fecha_campo(host, id);
            empurra(host, Some(PixelRequest::LayerRename { id: layer, name }))
        }
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == ids::RENAME_TAG => {
            let Some((index, _)) = st.rename_tag.take() else {
                return EventOutcome::Consumed;
            };
            let name = fecha_campo(host, id);
            let pedido = (!name.is_empty()).then_some(PixelRequest::TagRename { index, name });
            let _ = empurra(host, pedido);
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == ids::RENAME_LAYER || id == ids::RENAME_TAG => {
            if id == ids::RENAME_LAYER {
                st.rename_layer = None;
            } else {
                st.rename_tag = None;
            }
            let _ = fecha_campo(host, id);
            EventOutcome::Consumed
        }
        WidgetEvent::ValueChanged(id) if ids::NUMBERS.iter().any(|(n, _)| *n == id) => {
            let pedido = numero(host, id);
            empurra(host, pedido)
        }
        WidgetEvent::Click(id) if id == ids::CLOSE => {
            host.set_panel_visible(crate::PixelTimelinePanel::ID, false);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::TAG_DIRECTION => EventOutcome::Consumed,
        WidgetEvent::Click(id)
            if (0..PixelDirection::ALL.len()).any(|i| ids::direction_option_id(i) == id) =>
        {
            let i = (0..PixelDirection::ALL.len())
                .find(|&i| ids::direction_option_id(i) == id)
                .unwrap_or(0);
            if let Some(InteractiveState::Dropdown {
                open,
                selected_index,
                ..
            }) = host.store_mut().get_mut(ids::TAG_DIRECTION)
            {
                *open = false;
                *selected_index = Some(i);
            }
            let direction = PixelDirection::ALL[i];
            let pedido = with_current(|info| {
                let index = tag_activa(info).filter(|_| info.editing)?;
                Some(PixelRequest::TagDirection { index, direction })
            });
            empurra(host, pedido)
        }
        WidgetEvent::Click(id) => empurra(host, pedido(id)),
        _ => EventOutcome::Ignored,
    }
}

fn empurra(host: &mut dyn PanelHostInternal, pedido: Option<PixelRequest>) -> EventOutcome {
    match pedido {
        Some(r) => {
            host.bus_mut().push(EditorAction::Pixel(r));
            EventOutcome::Consumed
        }
        None => EventOutcome::Ignored,
    }
}
