//! Os gestos: cada controlo muda uma cópia das preferências do âmbito e pede-a inteira à família.
//! Não há passo de desfazer (o Aseprite também não o dá): a família só marca o documento mudado.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::card_field::CardFieldSpec;
use ph2d_editor_core::widget::{ButtonState, TextInputState};

use crate::canal::{self, Caixa, Numero};
use crate::{PixelPrefsPanel, PixelPrefsPanelState, ids};
use ph2d_pixel_render::prefs_canal::{self as fio, PrefsRequest, Scope};

fn botao(store: &mut WidgetStore, id: NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub(crate) fn populate(store: &mut WidgetStore) {
    // O cabeçalho de cada cartão dobra: é uma secção dobrável, o cânone do Pixel Tools e do Inspector.
    store.mark_collapsible_section(ids::SEC_SCOPE);
    store.mark_collapsible_section(ids::PREFS_SEC_GRID);
    store.mark_collapsible_section(ids::SEC_PIXEL_GRID);
    store.mark_collapsible_section(ids::SEC_CHECKER);
    store.mark_collapsible_section(ids::SEC_ONION);
    for id in [
        ids::SCOPE_DOC,
        ids::SCOPE_NEW,
        ids::SET_DEFAULTS,
        ids::PREFS_RESET,
        ids::ONION_BEHIND,
        ids::ONION_MERGE,
    ] {
        botao(store, id);
    }
    // O rótulo curto cabe; a frase longa vai à dica (P12, foto 07: «Auto opacity (fades w…»).
    for (k, dica) in [
        (Caixa::GridAuto, "panel.pixel_prefs.auto_tip"),
        (Caixa::PixelAuto, "panel.pixel_prefs.auto_tip"),
        (Caixa::CheckerZoom, "panel.pixel_prefs.apply_zoom_tip"),
    ] {
        store.set_tooltip(ids::caixa(k), ph2d_i18n::tr(dica));
    }
    for (id, _) in ids::CAIXAS {
        botao(store, id);
    }
    for (id, n) in ids::NUMEROS {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.mark_card_field(
            id,
            CardFieldSpec {
                step: 1.0,
                scrub: 1.0,
                default: None,
            },
        );
        let (lo, hi) = match n {
            Numero::GridX | Numero::GridY => (f64::from(i16::MIN), f64::from(i16::MAX)),
            Numero::GridOpacity | Numero::PixelOpacity => (0.0, canal::BYTE_MAX),
            Numero::OnionOpacity | Numero::OnionStep => (0.0, canal::PCT_MAX),
            Numero::OnionPrev | Numero::OnionNext => (0.0, f64::from(u16::MAX)),
            _ => (1.0, f64::from(u16::MAX)),
        };
        store.set_number_drag_rate(id, 1.0);
        store.set_number_range(id, lo, hi, 1.0);
    }
    for (id, _) in ids::CORES {
        store.register_picker_swatch(id);
        store.register(id, InteractiveState::Plain);
    }
    store.register(
        ids::PREFS_DRAG,
        InteractiveState::BlenderHit {
            parent: PixelPrefsPanel::NODE_ID,
            kind: BlenderHitKind::DragHandle,
        },
    );
}

/// Os campos e as amostras mostram o âmbito, salvo o que tem o cursor, a mão ou o seletor aberto; o
/// seletor aberto numa amostra manda a cor nova.
pub(crate) fn semeia(st: &mut PixelPrefsPanelState, store: &mut WidgetStore) {
    let Some(mut p) = fio::view().of(st.scope) else {
        return;
    };
    let foco = store.focus_id();
    let arrastado = store.number_input_drag().map(|d| d.id);
    for (id, n) in ids::NUMEROS {
        if Some(id) != foco && Some(id) != arrastado {
            store.set_number_value(id, canal::numero(&p, n) as f64);
        }
    }
    let aberto = store.picker_target();
    let antes = p;
    for (id, c) in ids::CORES {
        if aberto == Some(id) && st.a_escolher == Some(id) {
            if let Some(nova) = store.widget_color(id) {
                canal::poe_cor(&mut p, c, nova);
            }
        } else {
            store.set_widget_color(id, canal::cor(&p, c));
        }
    }
    st.a_escolher = aberto.filter(|a| ids::CORES.iter().any(|(id, _)| id == a));
    if p != antes {
        fio::request(PrefsRequest::Set(st.scope, p));
    }
}

/// O gesto `ev` sobre as preferências `p` do âmbito `scope`: o pedido que ele faz, se faz.
#[must_use]
pub fn pedido(
    ev: &WidgetEvent,
    scope: Scope,
    p: ph2d_pixel_core::PixelPrefs,
    numero: impl Fn(NodeId) -> Option<f64>,
) -> Option<PrefsRequest> {
    let mut p = p;
    match *ev {
        WidgetEvent::ValueChanged(id) => {
            let (_, n) = ids::NUMEROS.iter().find(|(i, _)| *i == id)?;
            canal::poe_numero(&mut p, *n, numero(id)?);
        }
        WidgetEvent::Click(id) if id == ids::SET_DEFAULTS => {
            return Some(PrefsRequest::SetAsDefaults);
        }
        WidgetEvent::Click(id) if id == ids::PREFS_RESET => {
            return Some(PrefsRequest::Reset(scope));
        }
        // Os pares de segmentos: `Behind`/`In Front` e `Merge`/`Red/Blue Tint` escolhem, não alternam.
        WidgetEvent::Click(id) if id == ids::ONION_BEHIND => p.onion.front = false,
        WidgetEvent::Click(id) if id == ids::ONION_MERGE => p.onion.tint = false,
        WidgetEvent::Click(id) if id == ids::caixa(canal::Caixa::OnionFront) => {
            p.onion.front = true
        }
        WidgetEvent::Click(id) if id == ids::caixa(canal::Caixa::OnionTint) => p.onion.tint = true,
        WidgetEvent::Click(id) => {
            let (_, c) = ids::CAIXAS.iter().find(|(i, _)| *i == id)?;
            canal::alterna(&mut p, *c);
        }
        _ => return None,
    }
    Some(PrefsRequest::Set(scope, p))
}

pub(crate) fn apply_event(
    st: &mut PixelPrefsPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        WidgetEvent::Click(id) if id == ids::SCOPE_DOC || id == ids::SCOPE_NEW => {
            st.scope = if id == ids::SCOPE_NEW {
                Scope::New
            } else {
                Scope::Document
            };
            return EventOutcome::Consumed;
        }
        _ => {}
    }
    let Some(p) = fio::view().of(st.scope) else {
        return EventOutcome::Ignored;
    };
    let store = host.store();
    match pedido(&ev, st.scope, p, |id| {
        store.number_value(id).filter(|n| n.is_finite())
    }) {
        Some(r) => {
            fio::request(r);
            EventOutcome::Consumed
        }
        None => EventOutcome::Ignored,
    }
}
