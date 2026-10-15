//! A costura do painel MACROS — o registo dos widgets, o que cada gesto pede, e a semente dos campos
//! a partir do instantâneo.
//!
//! ⚠️ Os botões e os campos registam-se INCONDICIONALMENTE (o store é agnóstico de estado; quem
//! decide se o clique é possível é a PINTURA, sem hit-rect não há `Click`). As LINHAS registam-se por
//! quadro, por quem as pinta.

use crate::ids;
use crate::state::{self, Escolhida, MacrosPanelState};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::macro_edits::{CampoDaEtapa, MacroRequest, PropsDaEtapa};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

/// Os botões, numa tabela: pintar, registar e despachar leem a MESMA.
pub(crate) const BOTOES: [ph2d_a11y::NodeId; 4] = [
    ids::MACROS_CLOSE,
    ids::MACROS_RECORD,
    ids::MACROS_STOP,
    ids::MACROS_RENAME,
];

/// Os campos numéricos e a escala do arrasto de cada um (unidades por px — sem faixa: a família
/// recusa o que não cabe e o painel diz porquê).
pub(crate) const CAMPOS: [(ph2d_a11y::NodeId, f64); 3] = [
    (ids::MACROS_WAIT, 0.01),
    (ids::MACROS_SIZE, 0.5),
    (ids::MACROS_VALUE, 0.01),
];

pub(crate) fn populate(store: &mut WidgetStore) {
    for id in BOTOES {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for (id, ritmo) in CAMPOS {
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
        store.set_number_drag_rate(id, ritmo);
    }
    // A amostra da cor (o selector partilhado devolve a cor pela `widget_colors`).
    store.register(ids::MACROS_COLOR, InteractiveState::Plain);
}

/// O registo POR QUADRO das linhas — feito por quem as pinta (os ids derivam do endereço).
pub(crate) fn register_rows(store: &mut WidgetStore, linhas: &[(usize, usize)]) {
    for &(t, k) in linhas {
        store.register_if_absent(ids::row_id(t, k), InteractiveState::Plain);
    }
}

/// Abre o campo de renomear com `seed` dentro e o cursor no fim.
pub(crate) fn open_rename(store: &mut WidgetStore, seed: &str) {
    store.register(
        ids::MACROS_RENAME_INPUT,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: seed.to_string(),
            caret: seed.len(),
            selection_anchor: None,
        },
    );
    store.set_focus(Some(ids::MACROS_RENAME_INPUT));
    store.mark_cancel_on_escape(ids::MACROS_RENAME_INPUT);
}

/// A etapa de uma linha, se `id` for uma delas.
fn etapa_de(id: ph2d_a11y::NodeId) -> Option<(Escolhida, f64)> {
    state::with_current(|i| {
        let o = i.objeto.as_ref()?;
        o.etapas
            .iter()
            .find(|e| ids::row_id(e.trilha, e.k) == id)
            .map(|e| {
                (
                    Escolhida {
                        entity_bits: o.entity_bits,
                        trilha: e.trilha,
                        k: e.k,
                    },
                    e.t1_s,
                )
            })
    })
}

fn pede(host: &mut dyn PanelHostInternal, r: MacroRequest) {
    host.bus_mut().push(EditorAction::Macro(r));
}

fn edita(host: &mut dyn PanelHostInternal, e: Escolhida, campo: CampoDaEtapa) {
    pede(
        host,
        MacroRequest::Edita {
            entity_bits: e.entity_bits,
            trilha: e.trilha,
            k: e.k,
            campo,
        },
    );
}

pub(crate) fn apply_event(
    st: &mut MacrosPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        WidgetEvent::DoubleClick(id) if etapa_de(id).is_some() => {
            let (e, _) = etapa_de(id).expect("acabou de ser lida");
            st.escolhida = Some(e);
            st.renomeando = true;
            let seed = state::etapa(e)
                .map(|x| crate::nomes::nome(&x))
                .unwrap_or_default();
            open_rename(host.store_mut(), &seed);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if etapa_de(id).is_some() => {
            let (e, fim) = etapa_de(id).expect("acabou de ser lida");
            st.escolhida = Some(e);
            st.renomeando = false;
            pede(
                host,
                MacroRequest::VaiPara {
                    entity_bits: e.entity_bits,
                    t_s: fim,
                },
            );
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::MACROS_RECORD => {
            pede(host, MacroRequest::Grava);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::MACROS_STOP => {
            pede(host, MacroRequest::Para);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::MACROS_CLOSE => {
            host.set_panel_visible(crate::MacrosPanel::ID, false);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::MACROS_RENAME => {
            if let Some(e) = st.escolhida {
                st.renomeando = true;
                let seed = state::etapa(e)
                    .map(|x| crate::nomes::nome(&x))
                    .unwrap_or_default();
                open_rename(host.store_mut(), &seed);
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Submit(id) if id == ids::MACROS_RENAME_INPUT => {
            st.renomeando = false;
            let texto = match host.store().get(id) {
                Some(InteractiveState::TextInput { text, .. }) => text.clone(),
                _ => String::new(),
            };
            if let Some(e) = st.escolhida {
                let nome = (!texto.trim().is_empty()).then_some(texto);
                edita(host, e, CampoDaEtapa::Nome(nome));
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == ids::MACROS_RENAME_INPUT => {
            st.renomeando = false;
            EventOutcome::Consumed
        }
        WidgetEvent::ValueChanged(id) if CAMPOS.iter().any(|(c, _)| *c == id) => {
            let (Some(e), Some(v)) = (st.escolhida, host.store().number_value(id)) else {
                return EventOutcome::Consumed;
            };
            let campo = if id == ids::MACROS_WAIT {
                CampoDaEtapa::Espera(v)
            } else if id == ids::MACROS_SIZE {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "um tamanho de pincel em px"
                )]
                CampoDaEtapa::Tamanho(v as f32)
            } else {
                CampoDaEtapa::Valor(v)
            };
            edita(host, e, campo);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::MACROS_COLOR => {
            if let Some(PropsDaEtapa::Pincelada { cor, .. }) =
                st.escolhida.and_then(state::etapa).map(|e| e.props)
            {
                let rgba = [cor[0], cor[1], cor[2], 255];
                let store = host.store_mut();
                store.set_widget_color(id, rgba);
                store.set_picker_target(Some(id));
                store.set_blender_value(
                    ph2d_editor_core::ids::INSP_BLENDER_PICKER,
                    ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
                );
            }
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}

/// **A semente do quadro**, antes de pintar: os números da etapa escolhida entram no store (a
/// `set_number_value` preserva a edição em curso) e a cor faz a ida e volta pelo selector
/// partilhado — enquanto ele aponta a amostra, a cor escolhida que difere da gravada é uma edição.
pub(crate) fn semeia(st: &MacrosPanelState, host: &mut dyn PanelHostInternal) {
    let Some(e) = st.escolhida else {
        return;
    };
    let Some(v) = state::etapa(e) else {
        return;
    };
    let store = host.store_mut();
    store.set_number_value(ids::MACROS_WAIT, v.espera_s);
    match v.props {
        PropsDaEtapa::Pincelada { cor, tamanho_px } => {
            store.set_number_value(ids::MACROS_SIZE, f64::from(tamanho_px));
            let gravada = [cor[0], cor[1], cor[2], 255];
            if store.picker_target() == Some(ids::MACROS_COLOR) {
                if let Some(c) = store.widget_color(ids::MACROS_COLOR)
                    && c[..3] != gravada[..3]
                {
                    edita(host, e, CampoDaEtapa::Cor([c[0], c[1], c[2]]));
                }
            } else {
                store.set_widget_color(ids::MACROS_COLOR, gravada);
            }
        }
        PropsDaEtapa::Param { valor } => store.set_number_value(ids::MACROS_VALUE, valor),
        PropsDaEtapa::Nenhuma => {}
    }
}
