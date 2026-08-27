//! As provas da máscara de sujidade do lado do NÓ — o que se mede sem uma GPU e sem uma cena.

use super::*;
use ph2d_nodegraph::graph::Graph;

fn with_glow() -> (Graph, ph2d_nodegraph::graph::NodeId) {
    let mut g = Graph::new();
    let n = g.add_node(crate::TYPE_NAME);
    (g, n)
}

#[test]
fn a_graph_with_no_name_authored_has_no_dirt() {
    let (g, _) = with_glow();
    assert_eq!(source(&g), None);
    // E um grafo SEM nó nenhum também — o caminho que a shell corre em toda cena
    // que não tem glow.
    assert_eq!(source(&Graph::new()), None);
}

#[test]
fn an_empty_or_blank_name_reads_as_no_dirt() {
    // ⚠️ Esta é a metade que separa *apaguei a escolha* de *escolhi uma coisa sem nome*: as
    // duas são "sem máscara", e uma delas chega pelo campo de texto do painel.
    for blank in ["", "   ", "\t", "\n  "] {
        let (mut g, n) = with_glow();
        g.set_text_param(n, DIRT_KEY, blank.to_string());
        assert_eq!(source(&g), None, "{blank:?} tinha de ler como ausente");
    }
}

#[test]
fn an_authored_name_is_read_back_trimmed() {
    let (mut g, n) = with_glow();
    g.set_text_param(n, DIRT_KEY, "  Lens Dirt \n".to_string());
    assert_eq!(source(&g).as_deref(), Some("Lens Dirt"));
}

/// **A identidade: o knob nasce em `0` e o nó não muda o stream.**
///
/// ⚠️ O `fx.glow` é um `passthrough`, então o param novo não pode ter tocado no cook. O que
/// este gate afirma é o outro lado — que o DEFAULT do manifesto é o neutro, e que ele é lido
/// pelo `from_graph` sem uma segunda cópia hard-coded.
#[test]
fn the_dirt_intensity_is_born_neutral_and_read_from_the_manifest() {
    let (g, _) = with_glow();
    let glow = crate::from_graph(&g).expect("ha' um fx.glow");
    assert_eq!(glow.dirt_intensity, 0.0);
    let declared = crate::MANIFEST
        .params
        .iter()
        .find(|p| p.name == DIRT_INTENSITY)
        .expect("o param esta' declarado");
    assert_eq!(declared.default, 0.0);
    // E um override chega ao leitor.
    let (mut g, n) = with_glow();
    g.set_param(n, DIRT_INTENSITY, 2.5);
    assert_eq!(crate::from_graph(&g).unwrap().dirt_intensity, 2.5);
}

/// **O knob é gateado pela PRESENÇA do nome, e o gate aponta para os params que existem.**
///
/// ⚠️ Um `ParamGateText` cujo `param` ou `when_text` estivesse escrito errado esconderia (ou
/// mostraria) a linha errada sem uma linha vermelha em lado nenhum — o painel simplesmente não
/// encontraria o alvo. Este gate liga as três pontas: o `ParamSpec`, a chave de texto e o hint.
#[test]
fn the_gate_names_a_param_that_exists_and_a_text_key_the_panel_paints() {
    let g = GATES.first().expect("ha' um gate");
    assert_eq!(g.param, DIRT_INTENSITY);
    assert_eq!(g.when_text, DIRT_KEY);
    assert!(g.when_present, "a intensidade aparece COM imagem, nao sem");
    assert!(
        crate::MANIFEST.params.iter().any(|p| p.name == g.param),
        "o gate esconde um param que nao esta' declarado"
    );
    let hints = crate::PARAM_HINTS;
    assert!(
        hints.iter().any(|h| h.param == DIRT_KEY),
        "o text param nao tem linha no painel — o artista nao teria como escolher a imagem"
    );
    assert!(
        hints.iter().any(|h| h.param == DIRT_INTENSITY),
        "o knob nao tem linha no painel"
    );
}

/// O teto digitável existe e é maior que o curso do slider — senão o `ParamHardMax` não
/// compra nada.
#[test]
fn the_typed_ceiling_is_wider_than_the_hand() {
    let hint = crate::PARAM_HINTS
        .iter()
        .find(|h| h.param == DIRT_INTENSITY)
        .expect("hint");
    let hard = crate::PARAM_HARD_MAX
        .iter()
        .find(|m| m.param == DIRT_INTENSITY)
        .expect("hard max");
    assert_eq!(hard.max, HARD_MAX);
    assert!(
        hard.max > hint.max,
        "um teto digitavel que nao passa do slider nao alcanca nada: {} vs {}",
        hard.max,
        hint.max
    );
}
