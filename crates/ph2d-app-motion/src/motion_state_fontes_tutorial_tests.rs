//! Os gates do TUTORIAL do ciclo 8 (`08_de_onde_vem_as_coisas`) — irmão dos da cena `=119` pelo
//! teto de 700 LOC, e o corte é por RESPONSABILIDADE: lá *o que a cena faz*, aqui *o que o PDF
//! afirma dela* (os cartões, as linhas e as figuras que o texto nomeia).

use crate::motion_state::MotionState;

/// A FONTE do tutorial deste ciclo — lida, para que os nomes do gate e os do texto não possam
/// divergir em silêncio.
const TUTORIAL: &str =
    include_str!("../../../docs/Motion Nodes/tutoriais/src/08_de_onde_vem_as_coisas.html");

/// ⭐⭐⭐ **CADA PASSO DO TUTORIAL É POSSÍVEL NO APP** (ciclo 8, passo 7 — doc 103 §1).
///
/// ⛔⛔ **Um passo que manda clicar numa linha AFIRMA que ela está no cartão** — e o modo de falha
/// é o pior possível: o dono procura, não encontra, e conclui que o programa está partido.
#[test]
fn every_row_the_source_tutorial_names_is_on_the_card() {
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("119", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    let pedidos: &[(&str, &[&str])] = &[
        ("Text: a palavra", &["Text", "Tracking"]),
        ("Shape: a forma", &["Shape"]),
        ("Table: o grafico", &["Table File"]),
        ("Drive: o tamanho no ecra", &["Scale"]),
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
}

/// ⭐⭐ **AS FIGURAS QUE O TEXTO NOMEIA EXISTEM** — e o gerador delas escreve todas as que ele cita.
///
/// ⛔ Uma `<img>` partida num PDF é um rectângulo vazio com uma legenda por baixo: o leitor lê a
/// legenda e acredita nela. *A legenda é a afirmação; a figura é a prova, e sem ela sobra só a
/// afirmação.*
#[test]
fn every_figure_the_source_tutorial_shows_exists() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    let mut n = 0;
    for pedaco in TUTORIAL.split("src=\"../fig/").skip(1) {
        let nome = pedaco.split('"').next().expect("o nome da figura");
        assert!(
            dir.join(nome).is_file(),
            "o tutorial mostra `{nome}` e a figura nao existe — corra `write_the_source_figures`"
        );
        n += 1;
    }
    assert!(n >= 6, "o tutorial tem seis figuras, achei {n}");
}

/// ⭐⭐ **O TUTORIAL NOMEIA A CENA CERTA** — e o roteador tem esse nível.
///
/// ⛔ Um tutorial que manda correr `=118` sobre a cena do ciclo 8 abre a cena de OUTRO ciclo, e o
/// dono passa o smoke inteiro a procurar panos que não existem.
#[test]
fn the_tutorial_opens_the_scene_this_cycle_built() {
    assert!(
        TUTORIAL.contains("PH2D_GPU_COOK_DEMO=119"),
        "o tutorial tem de abrir a cena `=119`"
    );
    assert!(
        super::super::demo_router::MAX_DEMO_LEVEL >= 119,
        "o roteador tem de conhecer o nivel 119"
    );
}
