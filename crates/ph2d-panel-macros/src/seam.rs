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
use ph2d_editor_core::macro_edits::{
    CampoDaEtapa, MacroPanelInfo, MacroRequest, OndeAplicar, PropsDaEtapa,
};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

/// Os botões, numa tabela: pintar, registar e despachar leem a MESMA.
pub(crate) const BOTOES: [ph2d_a11y::NodeId; 13] = [
    ids::MACROS_CLOSE,
    ids::MACROS_RECORD,
    ids::MACROS_STOP,
    ids::MACROS_RENAME,
    ids::MACROS_DELETE,
    ids::MACROS_HINT,
    ids::MACROS_LIB_SAVE,
    ids::MACROS_LIB_SAVE_ASSET,
    ids::MACROS_LIB_IMPORT,
    ids::MACROS_LIB_APPLY_NEW,
    ids::MACROS_LIB_APPLY_OVER,
    ids::MACROS_LIB_EXPORT,
    ids::MACROS_LIB_DELETE,
];

/// **Os verbos da biblioteca**: o botão e o pedido que ele faz, numa tabela só — a pintura e o
/// despacho leem-na (`None` = o gesto não é possível agora, e o botão pinta-se apagado).
pub(crate) fn pedido_da_biblioteca(
    id: ph2d_a11y::NodeId,
    info: &MacroPanelInfo,
    escolhida: Option<&str>,
) -> Option<MacroRequest> {
    let mostrado = || {
        info.objeto
            .as_ref()
            .filter(|_| info.pode_guardar)
            .map(|o| o.entity_bits)
    };
    let nome = || escolhida.map(str::to_owned);
    match id {
        ids::MACROS_LIB_SAVE => mostrado().map(|entity_bits| MacroRequest::Guarda { entity_bits }),
        ids::MACROS_LIB_SAVE_ASSET => {
            mostrado().map(|entity_bits| MacroRequest::GuardaComoAsset { entity_bits })
        }
        ids::MACROS_LIB_IMPORT => Some(MacroRequest::Importa),
        ids::MACROS_LIB_APPLY_NEW => nome().map(|nome| MacroRequest::Aplica {
            nome,
            onde: OndeAplicar::Novo { em: None },
        }),
        ids::MACROS_LIB_APPLY_OVER => {
            let alvo = info.por_cima_alvo.filter(|_| info.por_cima_de.is_some())?;
            nome().map(|nome| MacroRequest::Aplica {
                nome,
                onde: OndeAplicar::PorCima { alvo },
            })
        }
        ids::MACROS_LIB_EXPORT => nome().map(|nome| MacroRequest::Exporta { nome }),
        ids::MACROS_LIB_DELETE => nome().map(|nome| MacroRequest::ApagaDaBiblioteca { nome }),
        _ => None,
    }
}

/// Os ids da tabela [`pedido_da_biblioteca`].
pub(crate) const VERBOS_DA_BIBLIOTECA: [ph2d_a11y::NodeId; 7] = [
    ids::MACROS_LIB_SAVE,
    ids::MACROS_LIB_SAVE_ASSET,
    ids::MACROS_LIB_IMPORT,
    ids::MACROS_LIB_APPLY_NEW,
    ids::MACROS_LIB_APPLY_OVER,
    ids::MACROS_LIB_EXPORT,
    ids::MACROS_LIB_DELETE,
];

/// Os campos numéricos e a escala do arrasto de cada um (unidades por px — sem faixa: a família
/// recusa o que não cabe e o painel diz porquê).
pub(crate) const CAMPOS: [(ph2d_a11y::NodeId, f64); 3] = [
    (ids::MACROS_WAIT, ARRASTO_FINO),
    (ids::MACROS_SIZE, 0.5),
    (ids::MACROS_VALUE, ARRASTO_FINO),
];

/// A escala do arrasto da espera (s/px) e do valor.
const ARRASTO_FINO: f64 = 0.01; // LITERAL-PX-OK: escala no domínio do documento, não medida de design
/// O passo das setas da espera (s) e do valor.
pub(crate) const PASSO_FINO: f64 = 0.1; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

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

/// O registo POR QUADRO das linhas da biblioteca (os ids derivam do nome).
pub(crate) fn register_lib_rows(store: &mut WidgetStore, nomes: &[String]) {
    for n in nomes {
        store.register_if_absent(ids::lib_row_id(n), InteractiveState::Plain);
    }
}

/// Abre o campo de renomear com `seed` dentro e o cursor no fim.
pub(crate) fn open_rename(store: &mut WidgetStore, seed: &str) {
    abre_campo(store, ids::MACROS_RENAME_INPUT, seed);
}

/// Abre o campo de texto `id` com `seed` dentro, focado, e o Esc a cancelar.
fn abre_campo(store: &mut WidgetStore, id: ph2d_a11y::NodeId, seed: &str) {
    store.register(
        id,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: seed.to_string(),
            caret: seed.len(),
            selection_anchor: None,
        },
    );
    store.set_focus(Some(id));
    store.mark_cancel_on_escape(id);
}

/// A macro da biblioteca de uma linha, se `id` for uma delas.
fn macro_de(id: ph2d_a11y::NodeId) -> Option<String> {
    state::with_current(|i| {
        i.biblioteca
            .iter()
            .find(|m| ids::lib_row_id(&m.nome) == id)
            .map(|m| m.nome.clone())
    })
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
        // ── A biblioteca ──
        WidgetEvent::DoubleClick(id) if macro_de(id).is_some() => {
            let nome = macro_de(id).expect("acabou de ser lida");
            abre_campo(host.store_mut(), ids::MACROS_LIB_RENAME_INPUT, &nome);
            st.lib_escolhida = Some(nome);
            st.lib_renomeando = true;
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if macro_de(id).is_some() => {
            st.lib_escolhida = macro_de(id);
            st.lib_renomeando = false;
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if VERBOS_DA_BIBLIOTECA.contains(&id) => {
            let pedido =
                state::with_current(|i| pedido_da_biblioteca(id, i, st.lib_escolhida.as_deref()));
            if let Some(r) = pedido {
                if id == ids::MACROS_LIB_DELETE {
                    st.lib_escolhida = None;
                    st.lib_renomeando = false;
                }
                pede(host, r);
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Submit(id) if id == ids::MACROS_LIB_RENAME_INPUT => {
            st.lib_renomeando = false;
            let texto = match host.store().get(id) {
                Some(InteractiveState::TextInput { text, .. }) => text.trim().to_owned(),
                _ => String::new(),
            };
            if let Some(de) = st.lib_escolhida.clone()
                && !texto.is_empty()
                && texto != de
            {
                // A escolha segue o nome novo (cai se a família recusar o renomear).
                st.lib_escolhida = Some(texto.clone());
                pede(host, MacroRequest::Renomeia { de, para: texto });
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == ids::MACROS_LIB_RENAME_INPUT => {
            st.lib_renomeando = false;
            EventOutcome::Consumed
        }
        // ── As etapas ──
        WidgetEvent::DoubleClick(id) if etapa_de(id).is_some() => {
            let (e, _) = etapa_de(id).expect("acabou de ser lida");
            st.escolhida = Some(e);
            st.renomeando = true;
            st.dica = false;
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
            st.dica = false;
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
        WidgetEvent::Click(id) if id == ids::MACROS_HINT => {
            if let Some(e) = st.escolhida {
                st.renomeando = true;
                st.dica = true;
                let seed = state::etapa(e).and_then(|x| x.dica).unwrap_or_default();
                open_rename(host.store_mut(), &seed);
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::MACROS_RENAME => {
            if let Some(e) = st.escolhida {
                st.renomeando = true;
                st.dica = false;
                let seed = state::etapa(e)
                    .map(|x| crate::nomes::nome(&x))
                    .unwrap_or_default();
                open_rename(host.store_mut(), &seed);
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::MACROS_DELETE => {
            if let Some(e) = st.escolhida.take() {
                st.renomeando = false;
                pede(
                    host,
                    MacroRequest::Apaga {
                        entity_bits: e.entity_bits,
                        trilha: e.trilha,
                        k: e.k,
                    },
                );
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
                let texto = (!texto.trim().is_empty()).then_some(texto);
                let campo = if std::mem::take(&mut st.dica) {
                    CampoDaEtapa::Dica(texto)
                } else {
                    CampoDaEtapa::Nome(texto)
                };
                edita(host, e, campo);
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == ids::MACROS_RENAME_INPUT => {
            st.renomeando = false;
            st.dica = false;
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
