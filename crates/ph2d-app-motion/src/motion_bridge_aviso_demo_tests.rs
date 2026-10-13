//! Os gates da cena `=141` (doc 132 §5, P5): a cena monta pelo roteador, o gesto do roteiro dá
//! ESSE aviso (com a cura) e não liga o fio; o gesto automático espera e não se repete com o
//! aviso no ecrã.

use super::*;
use crate::motion_state::MotionState;
use crate::motion_state::aviso_demo::*;
use ph2d_editor_core::{ToastQueue, ToastSeverity};
use ph2d_motion_doc::MotionDoc;

fn cena() -> (MotionState, Cena) {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let c = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = vec![c.saida];
    (m, c)
}

fn aplica(m: &mut MotionState, toasts: &mut ToastQueue) {
    apply_graph_intents(
        m,
        &mut ph2d_core::Playhead::default(),
        toasts,
        &mut ph2d_editor_core::screens::layout::CenterSplit::None,
    );
}

fn conduzido(m: &MotionState, n: ph2d_nodegraph::graph::NodeId) -> bool {
    m.doc
        .graph
        .param_sources(n)
        .is_some_and(|s| s.contains_key(PARAM))
}

/// ⭐ **A cena monta pelo ROTEADOR** (o mesmo caminho do `PH2D_GPU_COOK_DEMO=141`), com uma saída.
#[test]
fn a_cena_141_monta_pelo_roteador() {
    let m = MotionState::new();
    let mut doc = MotionDoc::default();
    let (sinks, _) = crate::motion_demo_legend::monta("141", &mut doc, &m.registry);
    assert_eq!(sinks.len(), 1, "a cena 141 desenha UMA saída");
    let tipos: Vec<&str> = doc
        .graph
        .nodes()
        .iter()
        .map(|n| n.type_name.as_str())
        .collect();
    for t in [
        "source.shape",
        "motion.integrate",
        "force.vortex",
        "motion.oscillator",
        "motion.transform",
        "value.lfo",
    ] {
        assert!(tipos.contains(&t), "a cena não tem `{t}`: {tipos:?}");
    }
}

/// ⭐⭐ **O gesto do roteiro dá ESSE aviso** — o da corrente por-elemento, com a cura — e o fio
/// não fica. O controlo: o mesmo gesto a partir do `LFO` (um valor) liga e não avisa.
#[test]
fn o_gesto_do_roteiro_dispara_o_aviso_longo_e_nao_liga() {
    let (mut m, c) = cena();
    let mut toasts = ToastQueue::default();
    ph2d_panel_motion_graph::push_intent(o_gesto(&m.doc.graph).expect("o gesto existe"));
    aplica(&mut m, &mut toasts);
    let avisos: Vec<_> = toasts.iter().collect();
    assert_eq!(avisos.len(), 1, "um aviso: {:?}", avisos);
    assert_eq!(avisos[0].severity, ToastSeverity::Warning);
    let msg = &avisos[0].message;
    assert!(
        msg.starts_with(&cabeca_do_aviso()),
        "o aviso não é o da corrente por-elemento: {msg:?}"
    );
    assert!(
        msg.len() > cabeca_do_aviso().len(),
        "o aviso perdeu a CURA (o nó a inserir): {msg:?}"
    );
    // A cura nomeia o nó que LÊ os valores, pelo nome que o menu mostra (doc 132 §5: a ordem do
    // registo nomeava o `audio.bands`, com o nome cru do tipo).
    let no_do_menu = ph2d_i18n::tr(
        m.registry
            .manifests()
            .find(|t| t.name == ph2d_node_value_attribute::MANIFEST.name)
            .and_then(|t| m.registry.ui_manifest(t.id))
            .expect("o Attribute tem nome no menu")
            .display_key,
    );
    assert!(
        msg.contains(no_do_menu) && !msg.contains("audio.") && !msg.contains("value."),
        "a cura não nomeia o nó do menu que lê a corrente: {msg:?}"
    );
    eprintln!("cena 141: o aviso = {msg:?} ({} caracteres)", msg.len());
    assert!(!conduzido(&m, c.transform), "o fio recusado ficou ligado");

    // Controlo: o LFO conduz.
    let mut calado = ToastQueue::default();
    ph2d_panel_motion_graph::push_intent(ph2d_panel_motion_graph::GraphIntent::DriveParam {
        from_node: c.lfo.0,
        from_port: 0,
        to_node: c.transform.0,
        param: PARAM,
    });
    aplica(&mut m, &mut calado);
    assert!(
        calado.is_empty(),
        "o LFO avisou: {:?}",
        calado.iter().collect::<Vec<_>>()
    );
    assert!(
        conduzido(&m, c.transform),
        "controlo: o fio do LFO não ligou"
    );
    let _ = c.oscilador;
}

/// O gesto automático espera a cena assentar e não se repete com o aviso no ecrã.
#[test]
fn o_gesto_automatico_espera_e_nao_empilha() {
    let (m, _) = cena();
    let mut toasts = ToastQueue::default();
    assert!(gesto_se_mudo(&m.doc.graph, &toasts, ESPERA_QUADROS - 1).is_none());
    assert!(gesto_se_mudo(&m.doc.graph, &toasts, ESPERA_QUADROS).is_some());
    toasts.push(ph2d_editor_core::Toast::warning(format!(
        "{} — x",
        cabeca_do_aviso()
    )));
    assert!(
        gesto_se_mudo(&m.doc.graph, &toasts, ESPERA_QUADROS + 9).is_none(),
        "repetiu com o aviso ainda no ecrã"
    );
}
