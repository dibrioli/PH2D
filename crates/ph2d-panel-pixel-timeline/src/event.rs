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
use ph2d_editor_core::interaction::{
    InteractiveState, PixelTimelineHitKind as H, WidgetEvent, WidgetStore,
};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::pixel_layer_row::PixelLayerWidget as W;
use ph2d_editor_core::pixel_panel::{
    PixelDirection, PixelPanelInfo, PixelRange, PixelRangeKind, PixelRequest,
};
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
    botao(store, ids::ONION_SETTINGS);
    store.set_tooltip(ids::TOGGLES[0].1, tr("panel.pixel.timeline.onion_tip"));
    for (_, id, _, dica) in ids::ACTIONS {
        botao(store, id);
        store.set_tooltip(id, tr(dica));
    }
    botao(store, ids::TAG_DELETE);
    botao(store, ids::CONSTANT_RATE);
    store.set_tooltip(
        ids::CONSTANT_RATE,
        tr("panel.pixel.timeline.constant_rate_tip"),
    );
    for id in ids::RANGE_EDGES {
        superficie(store, id, H::RangeOutline);
    }
    for (right, id) in [false, true].into_iter().zip(ids::ONION_EDGES) {
        superficie(store, id, H::OnionEdge { right });
    }
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
    store.register(
        ids::PLAY_SPEED,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(2),
        },
    );
    for i in 0..ph2d_editor_core::pixel_panel::PixelSpeed::ALL.len() {
        botao(store, ids::speed_option_id(i));
    }
    for (id, _, dica) in ids::PLAY_TOGGLES {
        botao(store, id);
        store.set_tooltip(id, tr(dica));
    }
    store.set_tooltip(ids::PLAY_SPEED, tr("panel.pixel.timeline.speed"));
}

fn superficie(store: &mut WidgetStore, id: NodeId, kind: H) {
    let parent = ph2d_editor_core::ids::PIXEL_TIMELINE_PANEL;
    store.register_if_absent(id, InteractiveState::PixelTimelineSurface { parent, kind });
}

/// O registo POR QUADRO dos ids que derivam da linha do tempo (cabeçalhos, cels e pontas de tag
/// são superfícies de arrasto: [`crate::gestos`]).
pub(crate) fn register_frame(store: &mut WidgetStore, info: &PixelPanelInfo) {
    for r in &info.layers {
        for w in W::TIMELINE {
            let id = ids::layer_widget_id(r.id, w);
            let state = ButtonState::Normal;
            store.register_if_absent(id, InteractiveState::Button { state });
            if let Some(k) = w.tip_key() {
                store.set_tooltip(id, tr(k));
            }
        }
        for f in &info.frames {
            let kind = H::Cel {
                layer: r.id,
                frame: f.id,
            };
            superficie(store, ids::cell_id(r.id, f.id), kind);
        }
    }
    for f in &info.frames {
        superficie(store, ids::header_id(f.id), H::Header { frame: f.id });
    }
    for i in 0..info.tags.len() {
        store.register_if_absent(ids::tag_id(i), InteractiveState::Plain);
        let index = u16::try_from(i).unwrap_or(u16::MAX);
        for right in [false, true] {
            superficie(
                store,
                ids::tag_edge_id(i, right),
                H::TagEdge { index, right },
            );
        }
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
        if let Some(i) = (0..info.tags.len()).find(|&i| ids::tag_id(i) == id) {
            let pos = usize::try_from(info.tags[i].from).ok()?;
            return info
                .frames
                .get(pos)
                .map(|f| PixelRequest::FrameSelect(f.id));
        }
        for r in &info.layers {
            for w in W::TIMELINE {
                if ids::layer_widget_id(r.id, w) == id {
                    return Some(match w {
                        W::Row => PixelRequest::LayerSelect(r.id),
                        W::Eye => PixelRequest::LayerVisible(r.id),
                        W::Lock => PixelRequest::LayerLocked(r.id),
                        W::Solo => PixelRequest::LayerSolo(r.id),
                        W::Collapse => PixelRequest::LayerCollapsed(r.id),
                        W::Continuous => PixelRequest::LayerContinuous(r.id),
                        W::Normal => PixelRequest::LayerNormal(r.id),
                    });
                }
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

fn numero(host: &dyn PanelHostInternal, id: NodeId, constante: bool) -> Option<PixelRequest> {
    let v = inteiro(host, id)?;
    with_current(|info| {
        if !info.editing {
            return None;
        }
        match id {
            ids::FRAME_DURATION if constante => {
                u32::try_from(v).ok().map(PixelRequest::ConstantFrameRate)
            }
            ids::FRAME_DURATION => u32::try_from(v).ok().map(PixelRequest::FrameDuration),
            // From/To do diálogo são base 1; a família prende-os (to < from → to = from).
            ids::TAG_FROM | ids::TAG_TO => {
                let index = tag_activa(info)?;
                let t = info.tags.get(index)?;
                let v = u32::try_from(v - 1).ok()?;
                let (from, to) = if id == ids::TAG_FROM {
                    (v, t.to)
                } else {
                    (t.from, v)
                };
                Some(PixelRequest::TagRange { index, from, to })
            }
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
    if let Some(o) = crate::seam_user::evento(host, &ev) {
        return o;
    }
    if ev == WidgetEvent::Click(ids::ONION_SETTINGS) {
        ph2d_pixel_render::prefs_canal::pede_abrir_onion();
        return EventOutcome::Consumed;
    }
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
            let pedido = numero(host, id, st.constante.is_some());
            empurra(host, pedido)
        }
        WidgetEvent::Click(id) if id == ids::CONSTANT_RATE => {
            st.constante = with_current(|i| i.editing.then(|| (i.active_frame, i.range.clone())));
            EventOutcome::Consumed
        }
        // O nome da camada: activa, e a fila inteira passa a ser o intervalo.
        WidgetEvent::Click(id) if fila_de(id).is_some() => {
            let Some(layer) = fila_de(id) else {
                return EventOutcome::Ignored;
            };
            let range = PixelRange {
                kind: PixelRangeKind::Layers,
                layers: vec![layer],
                frames: Vec::new(),
            };
            let _ = empurra(host, Some(PixelRequest::LayerSelect(layer)));
            empurra(host, Some(PixelRequest::Range(range)))
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
        WidgetEvent::Click(id) if id == ids::PLAY_SPEED => EventOutcome::Consumed,
        WidgetEvent::Click(id) if do_tocar(id) => {
            use ph2d_editor_core::pixel_panel::PixelSpeed;
            if let Some(i) = (0..PixelSpeed::ALL.len()).find(|&i| ids::speed_option_id(i) == id)
                && let Some(InteractiveState::Dropdown {
                    open,
                    selected_index,
                    ..
                }) = host.store_mut().get_mut(ids::PLAY_SPEED)
            {
                *open = false;
                *selected_index = Some(i);
            }
            let pedido =
                with_current(|info| info.editing.then(|| tocar_com(id, info.playback)).flatten());
            empurra(host, pedido)
        }
        WidgetEvent::Click(id) => empurra(host, pedido(id)),
        _ => EventOutcome::Ignored,
    }
}

/// A camada cujo NOME é `id` (com sprite em edição).
fn fila_de(id: NodeId) -> Option<u32> {
    with_current(|i| {
        i.layers
            .iter()
            .find(|r| ids::layer_widget_id(r.id, W::Row) == id)
            .map(|r| r.id)
            .filter(|_| i.editing)
    })
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

/// `id` é uma opção da velocidade ou um interruptor do *Playback*.
fn do_tocar(id: NodeId) -> bool {
    let opcoes = ph2d_editor_core::pixel_panel::PixelSpeed::ALL.len();
    (0..opcoes).any(|i| ids::speed_option_id(i) == id)
        || ids::PLAY_TOGGLES.iter().any(|(t, ..)| *t == id)
}

/// As opções do tocar com o controlo `id` aplicado sobre `p`.
fn tocar_com(
    id: NodeId,
    mut p: ph2d_editor_core::pixel_panel::PixelPlayback,
) -> Option<PixelRequest> {
    use ph2d_editor_core::pixel_panel::PixelSpeed;
    if let Some(i) = (0..PixelSpeed::ALL.len()).find(|&i| ids::speed_option_id(i) == id) {
        p.speed = PixelSpeed::ALL[i];
    } else if id == ids::PLAY_ONCE {
        p.once = !p.once;
    } else if id == ids::PLAY_ALL {
        p.all = !p.all;
    } else if id == ids::PLAY_SUBTAGS {
        p.subtags = !p.subtags;
    } else if id == ids::PLAY_REWIND {
        p.rewind = !p.rewind;
    } else {
        return None;
    }
    Some(PixelRequest::Playback(p))
}
