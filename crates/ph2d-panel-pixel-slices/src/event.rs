//! Os gestos: cada controlo vira um pedido à família. Os textos e os números escritos só seguem ao
//! confirmar (`Enter` ou sair do campo), e só se mudaram: um passo por confirmação. Arrastar a letra de
//! um número é um passo por arrasto: cada valor reescreve o mesmo passo até a mão soltar.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::card_field::CardFieldSpec;
use ph2d_editor_core::widget::{ButtonState, DropdownState, TextInputState};
use ph2d_i18n::tr;
use ph2d_pixel_slices::campos::{Campo, Vista};

use crate::canal::{self, Contexto, Pedido};
use crate::{PixelSlicesPanel, PixelSlicesPanelState, ids};

fn botao(store: &mut WidgetStore, id: NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

fn textos() -> impl Iterator<Item = NodeId> {
    [ids::SLICE_NAME, ids::USER_DATA].into_iter()
}

/// Os números à Figma: um píxel por passo e por píxel de arrasto; um tamanho não desce de 1 (G6).
fn regista_numeros(store: &mut WidgetStore) {
    for (id, c) in ids::CAMPOS {
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
        if matches!(c, Campo::W | Campo::H | Campo::CentroW | Campo::CentroH) {
            // A taxa deixa o arrasto sem tecto; a faixa dá-lhe o piso.
            store.set_number_drag_rate(id, 1.0);
            store.set_number_range(id, 1.0, f64::from(i32::MAX), 1.0);
        }
    }
}

pub(crate) fn populate(store: &mut WidgetStore) {
    for id in ids::BOTOES {
        botao(store, id);
    }
    store.set_tooltip(ids::TRANSFORM, tr("panel.pixel.slices.transform_tip"));
    for id in textos() {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
        store.mark_cancel_on_escape(id);
    }
    regista_numeros(store);
    store.register_picker_swatch(ids::COLOR);
    store.register(ids::COLOR, InteractiveState::Plain);
    store.register(
        ids::MENU,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    store.register(
        ids::SLICES_DRAG,
        InteractiveState::BlenderHit {
            parent: PixelSlicesPanel::NODE_ID,
            kind: BlenderHitKind::DragHandle,
        },
    );
}

/// As linhas da lista: o sprite decide quantas são.
pub(crate) fn regista_linhas(store: &mut WidgetStore, n: usize) {
    for i in 0..n {
        botao(store, ids::linha(i));
    }
}

/// O número de um campo no documento; `None` = misto.
const fn numero_do_campo(v: &Vista, c: Campo) -> Option<i32> {
    match c {
        Campo::X => v.bounds[0],
        Campo::Y => v.bounds[1],
        Campo::W => v.bounds[2],
        Campo::H => v.bounds[3],
        Campo::CentroX => v.centro[0],
        Campo::CentroY => v.centro[1],
        Campo::CentroW => v.centro[2],
        Campo::CentroH => v.centro[3],
        Campo::PivotX => v.ponto[0],
        Campo::PivotY => v.ponto[1],
    }
}

fn campo_de(id: NodeId) -> Option<Campo> {
    ids::CAMPOS.iter().find(|(i, _)| *i == id).map(|(_, c)| *c)
}

fn valor_de(v: &Vista, id: NodeId) -> Option<String> {
    if id == ids::SLICE_NAME {
        return Some(v.nome.clone().unwrap_or_default());
    }
    if id == ids::USER_DATA {
        return Some(v.dados.clone().unwrap_or_default());
    }
    None
}

fn escreve(store: &mut WidgetStore, id: NodeId, novo: String) {
    if let Some(InteractiveState::TextInput { text, caret, .. }) = store.get_mut(id)
        && *text != novo
    {
        *caret = novo.len();
        *text = novo;
    }
}

/// Os campos mostram o documento, salvo o que tem o cursor ou a mão; o número misto aparece vazio.
pub(crate) fn semeia(store: &mut WidgetStore, x: &Contexto) {
    let foco = store.focus_id();
    let arrastado = store.number_input_drag().map(|d| d.id);
    for (id, c) in ids::CAMPOS {
        if Some(id) == foco || Some(id) == arrastado {
            continue;
        }
        match x.vista.as_ref().and_then(|v| numero_do_campo(v, c)) {
            Some(n) => store.set_number_value(id, f64::from(n)),
            None => store.blank_number_input(id),
        }
    }
    for id in textos().filter(|id| Some(*id) != foco) {
        let novo = x
            .vista
            .as_ref()
            .and_then(|v| valor_de(v, id))
            .unwrap_or_default();
        escreve(store, id, novo);
    }
}

/// A mão soltou a letra: o próximo número é outro passo.
fn fecha_o_arrasto(st: &mut PixelSlicesPanelState, store: &WidgetStore) {
    let agora = store
        .number_input_drag()
        .is_some_and(|d| d.crossed_threshold && campo_de(d.id).is_some());
    if st.a_arrastar && !agora {
        canal::pede(Pedido::PassoFechou);
    }
    st.a_arrastar = agora;
}

/// A amostra e o seletor partilhado: o seletor aberto nela manda a cor à família; fechado, ela mostra
/// a do documento. Antes, o fim do arrasto de um número (o mesmo passo partilhado do desfazer).
pub(crate) fn cor(st: &mut PixelSlicesPanelState, store: &mut WidgetStore, x: &Contexto) {
    fecha_o_arrasto(st, store);
    let aberto = store.picker_target() == Some(ids::COLOR);
    let doc = x.vista.as_ref().and_then(|v| v.cor);
    if aberto {
        let c = store.widget_color(ids::COLOR);
        if !st.a_escolher_cor {
            st.cor_ao_abrir = c;
        }
        if c != st.cor_ao_abrir {
            st.cor_ao_abrir = None;
        }
        if let Some(nova) = c.filter(|c| st.cor_ao_abrir.is_none() && Some(*c) != doc) {
            canal::pede(Pedido::Cor(nova));
        }
    } else {
        if st.a_escolher_cor {
            canal::pede(Pedido::PassoFechou);
        }
        if let Some(c) = doc {
            store.set_widget_color(ids::COLOR, c);
        }
    }
    st.a_escolher_cor = aberto;
}

fn confirma(host: &dyn PanelHostInternal, id: NodeId) {
    let Some(v) = canal::contexto().vista else {
        return;
    };
    let Some(t) = host.store().text(id).map(str::to_owned) else {
        return;
    };
    if valor_de(&v, id).as_deref() == Some(t.as_str()) {
        return;
    }
    canal::pede(if id == ids::SLICE_NAME {
        Pedido::Nome(t)
    } else {
        Pedido::Dados(t)
    });
}

/// Um número mudou: escrito (só se difere do documento) ou sob a letra arrastada. O `Esc` a meio do
/// arrasto chega aqui depois de a mão largar, e ainda é do mesmo passo.
fn numero(st: &mut PixelSlicesPanelState, host: &dyn PanelHostInternal, id: NodeId, c: Campo) {
    let Some(n) = host.store().number_value(id).filter(|n| n.is_finite()) else {
        return;
    };
    let arrasta = st.a_arrastar
        || host
            .store()
            .number_input_drag()
            .is_some_and(|d| d.id == id && d.crossed_threshold);
    let doc = canal::contexto().vista.and_then(|v| numero_do_campo(&v, c));
    let t = format!("{:.0}", n.trunc());
    if arrasta {
        st.a_arrastar = true;
        canal::pede(Pedido::Arrasta(c, t));
    } else if doc.map(f64::from) != Some(n.trunc()) {
        canal::pede(Pedido::Campo(c, t));
    }
}

fn fecha_o_menu(store: &mut WidgetStore) {
    if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(ids::MENU) {
        *open = false;
    }
}

/// O pedido de um clique fixo.
fn pedido_de(id: NodeId, x: &Contexto) -> Option<Pedido> {
    let v = x.vista.as_ref();
    Some(match id {
        i if i == ids::ALL => Pedido::Todas,
        i if i == ids::NONE => Pedido::Nenhuma,
        i if i == ids::TRANSFORM => Pedido::Transform,
        i if i == ids::NINE => Pedido::Nove(!v?.nove),
        i if i == ids::PIVOT => Pedido::Pivot(!v?.pivot),
        i if i == ids::DELETE || i == ids::MENU_DELETE => Pedido::Apaga,
        i if i == ids::DUPLICATE || i == ids::MENU_DUPLICATE => Pedido::Duplica,
        i if i == ids::MENU_REMOVE_KEY => Pedido::RemoveMudanca,
        _ => {
            let i = (0..x.linhas.len()).find(|&i| ids::linha(i) == id)?;
            Pedido::Seleciona(x.linhas[i].id)
        }
    })
}

pub(crate) fn apply_event(
    st: &mut PixelSlicesPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // Aqui nada distingue o duplo clique: dois cliques depressa são dois gestos.
    let ev = match ev {
        WidgetEvent::DoubleClick(id) => WidgetEvent::Click(id),
        outro => outro,
    };
    if let Some(o) = user_data_event(st, host, &ev) {
        return o;
    }
    match ev {
        WidgetEvent::ValueChanged(id) => match campo_de(id) {
            Some(c) => {
                numero(st, host, id, c);
                EventOutcome::Consumed
            }
            None => EventOutcome::Ignored,
        },
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if textos().any(|t| t == id) => {
            confirma(host, id);
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if textos().any(|t| t == id) => {
            let velho = canal::contexto()
                .vista
                .as_ref()
                .and_then(|v| valor_de(v, id))
                .unwrap_or_default();
            escreve(host.store_mut(), id, velho);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::MENU_PROPERTIES => {
            fecha_o_menu(host.store_mut());
            canal::mostrar_propriedades();
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) => match pedido_de(id, &canal::contexto()) {
            Some(p) => {
                fecha_o_menu(host.store_mut());
                canal::pede(p);
                EventOutcome::Consumed
            }
            None => EventOutcome::Ignored,
        },
        _ => EventOutcome::Ignored,
    }
}

/// Os gestos do cartão User Data: abrir, `+`, `×`, o chip do tipo, e os textos ao confirmar.
fn user_data_event(
    st: &mut PixelSlicesPanelState,
    host: &mut dyn PanelHostInternal,
    ev: &WidgetEvent,
) -> Option<EventOutcome> {
    let ud = ids::user_data();
    let (id, texto) = match *ev {
        WidgetEvent::Click(id) => (id, false),
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) => (id, true),
        _ => return None,
    };
    if id == ud.toggle {
        if texto {
            return Some(EventOutcome::Consumed);
        }
        st.user_aberto = !st.user_aberto;
        return Some(EventOutcome::Consumed);
    }
    let (slice, u) = canal::contexto().user?;
    let h = ph2d_editor_core::panel::cards::user_data::hit(
        &ud,
        u.props.get("").map_or(0, std::collections::BTreeMap::len),
        id,
    )?;
    let e_texto = matches!(
        h,
        ph2d_editor_core::panel::cards::user_data::Hit::Name(_)
            | ph2d_editor_core::panel::cards::user_data::Hit::Value(_)
            | ph2d_editor_core::panel::cards::user_data::Hit::Text
    );
    if e_texto != texto {
        return None;
    }
    let v = ph2d_pixel_ponte::user::vista(&u);
    if let Some(Some(e)) =
        ph2d_editor_core::panel::cards::user_data::on_event(host.store(), &ud, &v, ev)
        && let Some(e) = ph2d_pixel_ponte::user::traduz(&u, &e)
        && ph2d_pixel_core::user::edit::apply_edit(&u, &e).is_some()
    {
        canal::pede_uma(Pedido::UserData(slice, e));
    }
    Some(EventOutcome::Consumed)
}

/// Fecha o painel que ficou sem nada a mostrar.
pub(crate) fn sem_nada(host: &mut dyn PanelHostInternal) {
    host.set_panel_visible(PixelSlicesPanel::ID, false);
}
