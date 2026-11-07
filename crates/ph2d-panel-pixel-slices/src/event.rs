//! Os gestos: cada controlo vira um pedido à família. Os números e os textos só seguem ao confirmar
//! (`Enter` ou sair do campo), e só se mudaram: um passo por confirmação.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::{ButtonState, DropdownState, TextInputState};
use ph2d_i18n::tr;
use ph2d_pixel_slices::campos::{Campo, Vista};

use crate::canal::{self, Contexto, Pedido};
use crate::cards::valor;
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
    ids::CAMPOS
        .into_iter()
        .map(|(id, _)| id)
        .chain([ids::SLICE_NAME, ids::USER_DATA])
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

/// O que um campo mostra agora.
fn valor_do_campo(v: &Vista, c: Campo) -> String {
    valor(match c {
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
    })
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
    campo_de(id).map(|c| valor_do_campo(v, c))
}

fn escreve(store: &mut WidgetStore, id: NodeId, novo: String) {
    if let Some(InteractiveState::TextInput { text, caret, .. }) = store.get_mut(id)
        && *text != novo
    {
        *caret = novo.len();
        *text = novo;
    }
}

/// Os campos mostram o documento, salvo o que tem o cursor.
pub(crate) fn semeia(store: &mut WidgetStore, x: &Contexto) {
    let foco = store.focus_id();
    for id in textos().filter(|id| Some(*id) != foco) {
        let novo = x
            .vista
            .as_ref()
            .and_then(|v| valor_de(v, id))
            .unwrap_or_default();
        escreve(store, id, novo);
    }
}

/// A amostra e o seletor partilhado: o seletor aberto nela manda a cor à família; fechado, ela mostra
/// a do documento.
pub(crate) fn cor(st: &mut PixelSlicesPanelState, store: &mut WidgetStore, x: &Contexto) {
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
            canal::pede(Pedido::CorFechou);
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
    } else if id == ids::USER_DATA {
        Pedido::Dados(t)
    } else if let Some(c) = campo_de(id) {
        Pedido::Campo(c, t)
    } else {
        return;
    });
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
    _st: &mut PixelSlicesPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // Aqui nada distingue o duplo clique: dois cliques depressa são dois gestos.
    let ev = match ev {
        WidgetEvent::DoubleClick(id) => WidgetEvent::Click(id),
        outro => outro,
    };
    match ev {
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

/// Fecha o painel que ficou sem nada a mostrar.
pub(crate) fn sem_nada(host: &mut dyn PanelHostInternal) {
    host.set_panel_visible(PixelSlicesPanel::ID, false);
}
