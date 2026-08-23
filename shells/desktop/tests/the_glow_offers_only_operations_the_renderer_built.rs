//! **O DROPDOWN DO HALO E O ARRAY DE PIPELINES SÃO A MESMA LISTA** (doc 89, folha 11).
//!
//! Aqui há duas peças que não podem ver-se uma à outra: o `fx.glow` é uma **folha** e não
//! depende do `ph2d-render`; o `ph2d-render` não sabe que existe um nó. A shell é o único sítio
//! que enxerga as duas — a mesma geometria do gate irmão
//! `the_sink_blend_is_one_answer_for_both_routes`, e pela mesma razão.
//!
//! ⚠️ **O modo de falha que ele proíbe é MUDO.** Um rótulo a mais na lista do nó seria
//! escolhível no painel e, ao chegar ao passe, cairia no grampo do `operation_tag` — ou seja,
//! desenharia `Add`. O artista escolheria `Multiply` e receberia o aditivo, com os dois lados
//! verdes e nada a dizer.

/// **UM RÓTULO, UM PIPELINE.**
#[test]
fn the_glow_operations_are_the_pipelines_the_renderer_built() {
    assert_eq!(
        ph2d_node_fx_glow::OPERATION_LABELS.len(),
        ph2d_render::motion_fx::COMPOSITE_OPERATIONS,
        "o dropdown do halo e o array de pipelines tem de ser a mesma lista"
    );
}

/// **E O DROPDOWN NÃO OFERECE O QUE A NAVALHA DO §0 RECUSOU.**
///
/// ⚠️ Isto não é decoração: o `Multiply` é o terceiro modo do AE e a célula da conferência
/// pedia os três. Ele ficou de fora **por mecanismo** — o halo compõe-se sobre uma cena sem
/// profundidade, então um modo que escureça pinta por cima do que está à frente. Um dia alguém
/// vai querer acrescentá-lo por simetria com a referência; que essa pessoa leia primeiro.
#[test]
fn no_darkening_operation_is_offered_over_a_depthless_composite() {
    for label in ph2d_node_fx_glow::OPERATION_LABELS {
        assert!(
            !label.eq_ignore_ascii_case("multiply"),
            "`Multiply` escurece, e o halo compoe-se sem z: ver o doc-comment de `OPERATION`"
        );
    }
    // E o controle positivo: a lista não está vazia nem é uma lista de nomes inventados.
    assert!(
        ph2d_node_fx_glow::OPERATION_LABELS.contains(&"Add"),
        "o modo de sempre tem de continuar a ser oferecido"
    );
}

/// **A FONTE DO BRIGHT-PASS TEM DUAS, E O DEFAULT É A DE SEMPRE.**
///
/// ⚠️ O default de um param apendado **reduz** ao mundo de antes; se o `Alpha` nascesse
/// escolhido, toda cena opaca passaria a brilhar no dia da integração.
#[test]
fn the_bright_pass_source_defaults_to_the_one_that_always_shipped() {
    assert_eq!(
        ph2d_node_fx_glow::SOURCE_LABELS[0],
        "Luminance",
        "a tag 0 e' a fonte de sempre"
    );
    let neutral = ph2d_node_fx_glow::from_graph(&graph_with_a_glow());
    let g = neutral.expect("o grafo tem um fx.glow");
    assert_eq!(g.source, 0.0, "e um no' recem-criado nasce nela");
    assert_eq!(g.operation, 0.0, "idem a operacao");
}

/// Um grafo com um `fx.glow` sem override nenhum — o nó como ele nasce da palette.
fn graph_with_a_glow() -> ph2d_nodegraph::graph::Graph {
    let mut g = ph2d_nodegraph::graph::Graph::new();
    g.add_node("fx.glow");
    g
}
