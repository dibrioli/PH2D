//! Os gates do TUTORIAL do ciclo 20 (`13_o_acabamento`) — irmão dos da cena `=135`: lá *o que a
//! cena faz*, aqui *o que o PDF afirma dela* (cada nome entre `<code>` e cada figura).

use super::*;
use crate::motion_state::MotionState;

const TUTORIAL: &str =
    include_str!("../../../docs/Motion Nodes/tutoriais/src/13_o_acabamento.html");

/// O que vem entre `abre` e `fecha`, por ordem.
fn entre<'a>(texto: &'a str, abre: &str, fecha: &str) -> Vec<&'a str> {
    texto
        .split(abre)
        .skip(1)
        .filter_map(|resto| resto.split(fecha).next())
        .collect()
}

/// Todo nome que o ecrã MOSTRA nesta cena: os títulos dos cartões, as linhas e as secções do
/// `Filter` e do `Lens`, os tipos do `Effect` e as opções do `Apply To`.
fn o_que_o_ecra_mostra() -> Vec<String> {
    let mut m = MotionState::new();
    monta(&mut m.doc, &m.registry).expect("a cena monta");
    let snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    let mut nomes: Vec<String> = snap.nodes.iter().map(|v| v.display_name.clone()).collect();
    for tipo in [ph2d_node_fx_filter::TYPE_NAME, ph2d_node_fx_lens::TYPE_NAME] {
        let id = ph2d_nodegraph::node::NodeTypeId::of(tipo);
        for h in m.registry.param_ui(id).unwrap_or(&[]) {
            nomes.push(ph2d_i18n::tr(h.label).to_string());
        }
    }
    for chave in ph2d_node_fx_filter::KIND_LABELS
        .iter()
        .chain(&ph2d_node_fx_filter::SCOPE_LABELS)
        .chain(&[
            "node.group.vignette",
            "node.group.aberration",
            "node.group.grade",
            "node.group.grain",
        ])
    {
        nomes.push(ph2d_i18n::tr(chave).to_string());
    }
    nomes
}

/// ⭐⭐⭐ **CADA NOME QUE O TUTORIAL MANDA PROCURAR ESTÁ NO ECRÃ** (doc 103 §1): um passo que diz
/// «troque `Apply To`» afirma que a linha existe — com o nome EXACTO que o cartão pinta.
#[test]
fn every_name_the_finishing_tutorial_puts_in_code_is_on_screen() {
    let ecra = o_que_o_ecra_mostra();
    // Os dois que não são nomes de ecrã: o caminho que o artista ESCREVE e a cena do rodapé.
    let escritos = [LUT_QUENTE, "PH2D_GPU_COOK_DEMO=135"];
    let codigos = entre(TUTORIAL, "<code>", "</code>");
    assert!(codigos.len() > 30, "o tutorial nomeia {} coisas", codigos.len());
    for c in codigos {
        assert!(
            ecra.iter().any(|n| n == c) || escritos.contains(&c),
            "o tutorial manda procurar `{c}` e nenhum cartão da cena o mostra"
        );
    }
    // E o inverso, para o que o roteiro ensina: o tutorial nomeia cada um.
    for nome in [
        "Apply To",
        "Each Shape",
        "Whole Layer",
        "Effect",
        "Bevel",
        "Inner Shadow",
        "Drop Shadow",
        "Vignette",
        "Aberration",
        "LUT File",
        "Grain",
    ] {
        assert!(
            TUTORIAL.contains(&format!("<code>{nome}</code>")),
            "o roteiro ensina `{nome}` e o tutorial nunca o nomeia"
        );
    }
}

/// Cada figura que o tutorial mostra existe em `fig/` (uma foto em falta sai como um quadrado
/// vazio no PDF, sem erro).
#[test]
fn every_figure_of_the_finishing_tutorial_exists() {
    let fig = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Motion Nodes/tutoriais/src/"
    );
    let imagens = entre(TUTORIAL, "<img src=\"", "\"");
    assert!(!imagens.is_empty());
    for img in imagens {
        assert!(
            std::path::Path::new(fig).join(img).is_file(),
            "a figura `{img}` não existe"
        );
    }
}

/// O passo (1) só se VÊ se os discos se tocam: o diâmetro passa o vão da grelha.
#[test]
fn the_discs_touch_so_whole_layer_differs_from_each_shape() {
    assert!(2.0 * ESTRELA > super::super::ritmo_demo::VAO);
}
