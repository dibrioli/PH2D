//! O nó «Send signal» (GEF 08 MI-8) pelo editor REAL: a paleta que o artista abre e os
//! cliques no painel de params, pela fila de intenções (nunca o setter directo).

use super::lsystem_selector_tests::dispatch;
use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use crate::{MotionParamIntent, ParamRow};
use ph2d_editor_core::ProjectSettings;
use ph2d_node_send_signal as send;

fn mostra(m: &MotionState) -> Vec<String> {
    let snap = build_params_snapshot(m, ProjectSettings::default()).expect("send resolvível");
    snap.rows
        .iter()
        .flat_map(|r| r.params().into_iter().map(str::to_string))
        .collect()
}

#[test]
fn send_signal_esta_na_paleta_em_output_e_os_params_editam_por_clique() {
    let mut m = MotionState::new();
    let modelo = super::library::build_palette_model(&m.registry, &[]);
    let id = ph2d_tool_registry::hash_node_id(send::MANIFEST.name);
    let grupo = modelo
        .groups
        .iter()
        .find(|g| g.subs.iter().flat_map(|s| &s.items).any(|i| i.id == id))
        .expect("Send signal está na paleta");
    assert_eq!(grupo.title, "Output", "ao lado do «Signal», quem fala para fora");
    let item = grupo.subs.iter().flat_map(|s| &s.items).find(|i| i.id == id);
    assert_eq!(item.map(|i| i.label.as_str()), Some("Send signal"));

    let n = m.doc.graph.add_node(send::MANIFEST.name);
    ph2d_panel_motion_graph::set_graph_selection(vec![n.0]);
    let snap = build_params_snapshot(&m, ProjectSettings::default()).expect("resolvível");
    let quando = snap.rows.iter().find_map(|r| match r {
        ParamRow::Enum(e) if e.name == send::WHEN => Some(e),
        _ => None,
    });
    let palavras: Vec<&str> = quando
        .expect("When é um seletor nomeado")
        .labels
        .iter()
        .map(|l| ph2d_i18n::tr(l))
        .collect();
    assert_eq!(palavras, ["Pulse", "Rises above", "Falls below"]);
    let p = mostra(&m);
    assert!(p.iter().any(|x| x == send::NAME_KEY), "{p:?}");
    assert!(!p.iter().any(|x| x == send::LEVEL), "Pulse esconde o Level: {p:?}");

    for (when, level) in [(send::WHEN_RISES, true), (send::WHEN_FALLS, true), (0, false)] {
        dispatch(
            &mut m,
            MotionParamIntent::SetParam {
                node: n.0,
                param: send::WHEN,
                value: f64::from(when),
            },
        );
        let p = mostra(&m);
        assert_eq!(p.iter().any(|x| x == send::LEVEL), level, "when={when}: {p:?}");
    }
    dispatch(
        &mut m,
        MotionParamIntent::SetTextParam {
            node: n.0,
            param: send::NAME_KEY,
            value: "door_open".to_string(),
        },
    );
    let texto = m
        .doc
        .graph
        .node_text_param_overrides(n)
        .and_then(|t| t.get(send::NAME_KEY))
        .cloned();
    assert_eq!(texto.as_deref(), Some("door_open"));
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}
