//! Os gates da cena `=141` (doc 132 §5, P5): a cena monta pelo roteador; o gesto do roteiro (o fio
//! do `Oscillator` solto no CORPO do `Transform`, `DropRefused`) dá ESSE aviso, com a cura, e não
//! liga nem abre nada; o `DriveParam` sintético diz a MESMA frase; o gesto automático dispara UMA
//! vez (smoke do dono, 13/10: *«o aviso não some só e o modal de nós se abre»*).

use super::*;
use crate::motion_state::MotionState;
use crate::motion_state::aviso_demo::*;
use ph2d_editor_core::{ToastQueue, ToastSeverity};
use ph2d_motion_doc::MotionDoc;
use std::time::Duration;

/// O param escalar que o `DriveParam` sintético e o controlo do `LFO` conduzem.
const PARAM: &str = "scale";

/// O começo do aviso da corrente, sem a cura.
fn cabeca_do_aviso() -> String {
    ph2d_i18n::tr_with(
        "app.motion.motion_bridge_subgraph.can_t_drive_that_output_is_a_per_element_stream",
        &[("cura", &"")],
    )
}

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

/// ⭐⭐ **O gesto do roteiro dá ESSE aviso** — o da corrente por-elemento, com a cura — e nada
/// liga nem abre (a paleta é o `open_library`). O `DriveParam` sintético diz a MESMA frase (uma
/// porta só). O controlo: o `LFO` (um valor) liga e não avisa.
#[test]
fn o_gesto_do_roteiro_dispara_o_aviso_longo_e_nao_liga() {
    let (mut m, c) = cena();
    let mut toasts = ToastQueue::default();
    let gesto = o_gesto(&m.doc.graph).expect("o gesto existe");
    assert!(
        matches!(
            gesto,
            ph2d_panel_motion_graph::GraphIntent::DropRefused { .. }
        ),
        "o gesto é o do painel ao soltar no corpo: {gesto:?}"
    );
    let antes = m.doc.graph.edges().len();
    ph2d_panel_motion_graph::push_intent(gesto);
    aplica(&mut m, &mut toasts);
    assert!(
        m.open_library.is_none(),
        "a largada no corpo abriu a paleta"
    );
    assert_eq!(
        m.doc.graph.edges().len(),
        antes,
        "a largada recusada ligou um fio"
    );
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

    // Uma porta só para a frase: o `DriveParam` da mesma corrente diz o MESMO.
    let mut sintetico = ToastQueue::default();
    ph2d_panel_motion_graph::push_intent(ph2d_panel_motion_graph::GraphIntent::DriveParam {
        from_node: c.oscilador.0,
        from_port: 0,
        to_node: c.transform.0,
        param: PARAM,
    });
    aplica(&mut m, &mut sintetico);
    assert_eq!(
        sintetico
            .iter()
            .map(|t| t.message.clone())
            .collect::<Vec<_>>(),
        vec![msg.clone()],
        "o DriveParam e a largada no corpo dizem frases diferentes"
    );

    // O passo do controlo, pelo NOME que o menu do corpo do `Transform` mostra ao fio do `LFO`.
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    fold::fold(&m, &mut snap);
    let rotulos: Vec<String> = ph2d_panel_motion_graph::card_hidden_ports(c.transform.0)
        .inputs
        .into_iter()
        .map(|p| p.label)
        .collect();
    assert!(
        rotulos.iter().any(|r| r == "Transform: Scale"),
        "o roteiro manda escolher `Transform: Scale`: {rotulos:?}"
    );

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
}

/// ⭐ **Fora da corrente, a recusa diz o NOME do cartão** — uma fonte que JÁ é um valor não tem cura
/// a nomear (o painel só a recusa num corpo sem fileira para ela); a frase nomeia o cartão como ele
/// se mostra.
#[test]
fn a_largada_recusada_de_um_valor_nomeia_o_cartao() {
    let (mut m, c) = cena();
    let mut toasts = ToastQueue::default();
    ph2d_panel_motion_graph::push_intent(ph2d_panel_motion_graph::GraphIntent::DropRefused {
        from_node: c.lfo.0,
        from_port: 0,
        to_node: c.saida.0,
    });
    aplica(&mut m, &mut toasts);
    let msgs: Vec<String> = toasts.iter().map(|t| t.message.clone()).collect();
    assert_eq!(msgs, vec!["Nothing on Output takes this wire".to_string()]);
}

/// ⛔ **O gesto automático dispara UMA vez** (smoke do dono, 13/10: ele repetia-se sempre que o
/// aviso saía do ecrã, e o aviso nunca sumia). Antes da espera, nada; na espera, o gesto; depois,
/// nunca mais — com ou sem aviso no ecrã.
#[test]
fn o_gesto_automatico_dispara_uma_vez_so() {
    let (m, _) = cena();
    let espera = Duration::from_secs(12);
    let mut auto = GestoAuto::novo(espera);
    let g = &m.doc.graph;
    assert!(
        auto.passo(g, Duration::from_secs(11)).is_none(),
        "antes da espera"
    );
    assert_eq!(
        auto.passo(g, espera),
        o_gesto(g),
        "na espera, o gesto do roteiro"
    );
    for s in [13, 20, 60] {
        assert!(
            auto.passo(g, Duration::from_secs(s)).is_none(),
            "repetiu aos {s} s: o aviso nunca sumiria"
        );
    }
    // O instante vem do ambiente, em SEGUNDOS, e só na `=141`.
    assert_eq!(
        espera_do_ambiente(Some("1"), Some("141")),
        Some(Duration::from_secs(1))
    );
    assert_eq!(
        espera_do_ambiente(Some("15"), Some("141")),
        Some(Duration::from_secs(15))
    );
    assert_eq!(espera_do_ambiente(Some("15"), Some("140")), None);
    assert_eq!(espera_do_ambiente(Some("x"), Some("141")), None);
    assert_eq!(espera_do_ambiente(None, Some("141")), None);
}
