//! Os gates do TUTORIAL do ciclo 7 (`07_a_cor_e_o_rasto`) — irmão dos da cena `=118` pelo teto de
//! 700 LOC, e o corte é por RESPONSABILIDADE: lá *o que a cena faz*, aqui *o que o PDF afirma
//! dela* (os cartões, as linhas, as opções e as figuras que o texto nomeia).

use super::*;
use crate::motion_state::MotionState;

/// A FONTE do tutorial deste ciclo — lida para que os nomes do gate e os do texto não possam
/// divergir em silêncio.
const TUTORIAL: &str =
    include_str!("../../../docs/Motion Nodes/tutoriais/src/07_a_cor_e_o_rasto.html");

/// ⭐⭐⭐ **CADA PASSO DO TUTORIAL É POSSÍVEL NO APP** (ciclo 7, passo 7 — doc 103 §1).
///
/// ⛔⛔ **Um passo que manda arrastar uma linha AFIRMA que ela está no cartão.** O irmão
/// (`every_card_an_announcement_...`) defende o anúncio do terminal; este defende o PDF.
#[test]
fn every_row_the_appearance_tutorial_names_is_on_the_card() {
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("118", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    let pedidos: &[(&str, &[&str])] = &[
        ("Tint", &["Color", "Mode"]),
        ("Color Ramp", &["Gradient"]),
        (
            "Trail",
            &["Length", "Tail Alpha", "Tail Size", "Tail Hue Shift"],
        ),
        (NOME_ORDEM, &["Delay By", "Lag"]),
        (NOME_CAMPO, &["Delay By"]),
        ("Falloff", &["Invert", "Shape"]),
    ];
    for (titulo, linhas) in pedidos {
        let v = snap
            .nodes
            .iter()
            .find(|v| v.display_name == *titulo)
            .unwrap_or_else(|| {
                let havia: Vec<&str> = snap.nodes.iter().map(|v| v.display_name.as_str()).collect();
                panic!("o tutorial nomeia o cartao `{titulo}` e a cena tem {havia:?}")
            });
        let rows: Vec<&str> = v.params.iter().map(|c| c.hint.label).collect();
        for l in *linhas {
            assert!(
                rows.contains(l),
                "o tutorial manda mexer em `{l}` no cartao `{titulo}`, e o cartao mostra {rows:?}"
            );
            assert!(
                TUTORIAL.contains(&format!("<code>{l}</code>")),
                "o gate defende a linha `{l}` e o tutorial nunca a nomeia"
            );
        }
        assert!(
            TUTORIAL.contains(titulo),
            "o gate defende o cartao `{titulo}` e o tutorial nunca o nomeia"
        );
    }
    // ⚠️ E as OPÇÕES que o texto manda escolher existem nos menus que ele nomeia.
    let reg = &m.registry;
    let opcoes = |tipo: &str, p: &str| -> Vec<&'static str> {
        reg.param_ui(ph2d_nodegraph::node::NodeTypeId::of(tipo))
            .and_then(|h| h.iter().find(|h| h.param == p))
            .and_then(|h| match h.widget {
                // ⚠️ O array carrega CHAVES desde a 5.ª fatia do HR-15; o tutorial nomeia a
                // PALAVRA que o artista lê no menu, logo a comparação passa pela tabela.
                ph2d_node_registry::ParamWidget::Enum { labels } => {
                    Some(labels.iter().map(|l| ph2d_i18n::tr(l)).collect())
                }
                _ => None,
            })
            .unwrap_or_default()
    };
    for (tipo, p, opcao) in [
        (
            "motion.slit_scan",
            ph2d_node_motion_slit_scan::RAMP,
            "Field",
        ),
        (
            "motion.slit_scan",
            ph2d_node_motion_slit_scan::RAMP,
            "Order",
        ),
        ("motion.falloff", "shape", "Circle"),
        ("motion.tint", "mode", "Gradient"),
    ] {
        assert!(
            opcoes(tipo, p).contains(&opcao),
            "o tutorial manda escolher `{opcao}` no `{tipo}.{p}`, e o menu tem {:?}",
            opcoes(tipo, p)
        );
        assert!(
            TUTORIAL.contains(&format!("<code>{opcao}</code>")),
            "o gate defende a opcao `{opcao}` e o tutorial nunca a nomeia"
        );
    }
    // ⚠️ **O `End` do `Tint` só aparece DEPOIS do `Mode = Gradient`** — é a ordem do passo 12, e o
    // gate pergunta ao cartão nesse estado (a 1.ª redacção perguntava nos defaults e reprovou
    // sobre um texto certo).
    let tint = m
        .doc
        .graph
        .nodes()
        .iter()
        .find(|i| i.type_name == "motion.tint")
        .map(|i| i.id)
        .expect("o Tint");
    m.doc.graph.set_param(tint, "mode", 1.0);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    let rows: Vec<&str> = snap
        .nodes
        .iter()
        .find(|v| v.display_name == "Tint")
        .map(|v| v.params.iter().map(|c| c.hint.label).collect())
        .unwrap_or_default();
    assert!(
        rows.contains(&"End") && TUTORIAL.contains("<code>End</code>"),
        "em Gradient o Tint mostra a linha `End` que o passo 12 nomeia: {rows:?}"
    );
    assert!(
        !TUTORIAL.contains("--release"),
        "o tutorial manda correr com `--release`"
    );
    assert!(
        TUTORIAL.contains("PH2D_GPU_COOK_DEMO=118"),
        "o tutorial tem de abrir a `=118`"
    );
    assert!(
        !TUTORIAL.contains("__"),
        "o tutorial tem um marcador por preencher"
    );
}

/// ⚠️ **AS FIGURAS que o tutorial mostra existem, e são SEIS DIFERENTES.**
#[test]
fn the_six_figures_the_appearance_tutorial_shows_exist_and_differ() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    let nomes = [
        "cor_uma",
        "cor_por_peca",
        "rasto_sem",
        "rasto_com",
        "tempo_ordem",
        "tempo_lugar",
    ];
    let mut corpos: Vec<String> = Vec::new();
    for n in nomes {
        assert!(
            TUTORIAL.contains(&format!("{n}.svg")),
            "o gate defende a figura `{n}` e o tutorial nao a mostra"
        );
        let s = std::fs::read_to_string(dir.join(format!("{n}.svg"))).unwrap_or_else(|e| {
            panic!(
                "a figura `{n}` nao esta' no disco ({e}) -- corra o `write_the_appearance_figures`"
            )
        });
        assert!(s.len() > 500, "a figura `{n}` esta' vazia");
        corpos.push(s);
    }
    for i in 0..corpos.len() {
        for j in (i + 1)..corpos.len() {
            assert_ne!(
                corpos[i], corpos[j],
                "as figuras `{}` e `{}` sao IDENTICAS",
                nomes[i], nomes[j]
            );
        }
    }
}
