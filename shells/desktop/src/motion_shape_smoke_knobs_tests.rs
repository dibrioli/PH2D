//! Os gates da cena `=3` — os knobs de forma.
//!
//! ⚠️ **A cena promete que as sete são DIFERENTES**, e é isso que se prova. Uma fileira em que
//! duas formas coincidem é pior que uma cena vazia: ela diz *"o knob funciona"* sobre um
//! número que o cozimento ignorou.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// **A CENA MONTA E COZINHA SEIS INSTÂNCIAS** — uma por forma, cada uma com o seu
/// `geometry_id`.
///
/// ⚠️ **O `geometry_id` é content-addressed**, então duas formas com o MESMO descritor
/// partilhariam um handle. Exigir seis handles DISTINTOS é, por construção, exigir seis
/// descritores distintos — o oráculo mais barato de *"as seis são diferentes"* que existe,
/// e ele não precisa da geometria nem do shell.
#[test]
fn the_row_builds_seven_shapes_with_seven_distinct_descriptors() {
    let reg = registry();
    let mut g = Graph::new();
    let out = build_knob_row(&mut g);
    g.validate(&reg).expect("bem-tipado");

    let keys: std::collections::BTreeSet<String> = g
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.shape")
        .map(|n| {
            let over = g.node_param_overrides(n.id);
            ph2d_node_motion_shape::shape_key(|name| {
                over.and_then(|m| m.get(name).copied())
                    .unwrap_or_else(|| crate::render_loop::motion_shape_gen::manifest_default(name))
            })
        })
        .collect();
    assert_eq!(
        keys.len(),
        7,
        "sete formas, sete descritores — duas iguais partilhariam o geometry_id e a fileira \
         mostraria a mesma coisa duas vezes"
    );

    // ⚠️ **O cook NÃO é o oráculo aqui, e a 1ª versão deste gate caiu por isso:** o
    // `source.shape` lê um EXTERNAL que o SHELL publica (`motion_shape_gen::publish`), então
    // fora do app cada nó devolve um stream vazio — o gate media o shell ausente, não a cena.
    // O que se prova aqui é a FIAÇÃO: as seis formas chegam ao mesmo sink.
    let mut cook = Cook::new();
    cook.cook(&g, &reg, out, 0.0)
        .expect("a cena coze sem panicar");
    assert_eq!(
        g.nodes()
            .iter()
            .filter(|n| n.type_name == "source.shape")
            .count(),
        7,
        "sete formas na fileira"
    );
    assert!(
        g.nodes()
            .iter()
            .any(|n| n.id == out && n.type_name == "motion.output"),
        "e todas terminam no sink"
    );
}

/// **CADA FORMA MOSTRA SÓ OS KNOBS DA SUA ESPÉCIE** — a segunda metade da mensagem do
/// smoke, e a que o artista verifica clicando.
///
/// A rosquinha tem `inner`; a caixa não. A caixa tem `smoothing`; a rosquinha não. Se a
/// tabela de visibilidade deixasse de separar as duas, a mensagem estaria a mandar o Enio
/// procurar um slider que aparece em todo lado.
#[test]
fn the_circle_family_and_the_box_show_different_knobs() {
    let reg = registry();
    let hints = reg
        .param_gates(ph2d_node_motion_shape::MANIFEST.id)
        .expect("a forma declara gates");
    let shows = |param: &str, kind: f32| {
        hints
            .iter()
            .find(|g| g.param == param)
            .is_none_or(|g| g.values.contains(&(kind as i32)))
    };
    assert!(shows("inner", CIRCLE), "a rosquinha tem miolo");
    assert!(!shows("inner", SQUARE), "a caixa nao tem miolo");
    assert!(shows("smoothing", SQUARE), "a caixa tem suavizacao");
    assert!(
        !shows("smoothing", CIRCLE),
        "o circulo nao tem canto a suavizar"
    );
    assert!(shows("sweep", PIE), "a pizza autora a fatia");
    assert!(
        shows("corner_tr", RECTANGLE),
        "a caixa retangular tem os quatro cantos"
    );
}

/// **TODO KNOB QUE A CENA AUTORA TEM DE APARECER NO PAINEL DAQUELA FORMA.**
///
/// ⚠️ **É a forma mais afiada do defeito que o Enio reportou** (*"não há opções de corner na
/// UI"*): a cena escreve `corner = 0,18` no anel cortado, a geometria RESPONDE — as quatro
/// quinas arredondam —, e o painel **esconde** o slider, porque a tabela de visibilidade é
/// keyed no `kind` e o `kind` daquela forma é `Circle`. Um círculo INTEIRO não tem quina; um
/// círculo com `sweep < 360` tem quatro. A tabela olhava só metade do descritor.
///
/// ⚠️ **E o gate irmão (`no_kind_hides_a_live_knob_or_shows_a_dead_one`) passava**, porque a
/// base dele é o descritor NEUTRO — um círculo inteiro. *A fixture não continha o fenômeno.*
/// Este gate não tem como não conter: ele lê os knobs que a CENA de facto escreveu.
#[test]
fn every_knob_the_scene_authors_is_visible_on_that_shape() {
    let reg = registry();
    let mut g = Graph::new();
    build_knob_row(&mut g);
    let gates = reg
        .param_gates(ph2d_node_motion_shape::MANIFEST.id)
        .expect("a forma declara gates");

    for node in g.nodes().iter().filter(|n| n.type_name == "source.shape") {
        let Some(over) = g.node_param_overrides(node.id) else {
            continue;
        };
        let kind = over.get("kind").copied().unwrap_or(0.0) as i32;
        for (name, value) in over {
            // Só os knobs que a cena moveu do neutro: escrever o default não é autorar.
            if *value == crate::render_loop::motion_shape_gen::manifest_default(name) {
                continue;
            }
            let shown = gates
                .iter()
                .find(|gt| gt.param == name)
                .is_none_or(|gt| gt.values.contains(&kind));
            assert!(
                shown,
                "a cena escreve `{name} = {value}` numa forma de kind {kind}, e o painel \
                 ESCONDE esse slider — o artista vê o efeito e não acha o controle"
            );
        }
    }
}
