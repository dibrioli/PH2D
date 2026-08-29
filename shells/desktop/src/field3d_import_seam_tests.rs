//! ⭐ **A COSTURA da porta de entrada** — do clique no botão até a escultura ser um nó da peça.
//!
//! ⚠️ **É a metade que compila sempre e funciona só às vezes.** O `field3d_import` tem os gates da
//! aritmética dele (o tamanho de convivência), e a `ph2d-field-mesh` tem os do campo. O que nenhum
//! dos dois alcança é o **caminho**: o botão chega ao canal? o canal chega ao mundo? o nó que nasce
//! é mesmo uma escultura, e o avaliador resolve o nome? Cada uma dessas emendas passa despercebida
//! com a suíte verde, e é a causa nº 1 da `DIRETIVA_IMPLEMENTACAO`.
//!
//! Módulo-filho: `use super::*` traz as fixtures do pai.

use super::*;

/// Uma escultura registada sob um nome.
///
/// ⚠️ **O raio é 2,5 e não 0,4, e a razão é um controlo que disparou**: com 0,4 a extensão dá 0,8, o
/// enquadramento por omissão também, e a escala de convivência sai **exatamente 1** — o gate passaria
/// com o código de escala apagado. Uma peça que já cabe no quadro não prova que alguém a fez caber.
fn register_a_sculpture(key: &str) -> f32 {
    let mesh = ph2d_mesh::shapes::uv_sphere(16, 32, 2.5);
    let b = mesh.bounds();
    let extent = (b.max[0] - b.min[0])
        .max(b.max[1] - b.min[1])
        .max(b.max[2] - b.min[2]);
    let field = ph2d_field_mesh::SampledField::from_mesh(&mesh, 32).expect("esfera");
    crate::field3d_smoke::register_sampled(key, std::sync::Arc::new(field));
    extent
}

/// ⭐ **O botão da escultura NÃO cria uma primitiva — ele pede um arquivo.**
///
/// ⚠️ O slot dele vive na mesma lista das quatro formas, e é isso que o faz aparecer sem uma linha
/// de painel. O risco é o simétrico: um braço que não o distinga cairia no `shape_at`, que devolve
/// `None` ali, e o clique **não faria absolutamente nada** — o modo de falha mais caro de
/// diagnosticar, porque não deixa rasto nenhum.
#[test]
fn the_sculpt_button_asks_for_a_file_instead_of_making_a_shape() {
    let _ = ph2d_panel_model3d::drain_intents();
    let _ = crate::field3d_smoke::take_import_request();
    let mut sim = a_world();
    crate::field3d_scene::sync_scene(&mut sim, Some(&scene(2)), 0.0);
    let root = the_root(&mut sim);
    let before = ph2d_field_ecs::walk(sim.world(), root).len();

    // ⚠️ **O gesto entra pela porta de PRODUÇÃO** (W100): a paleta escolhe e deixa a posição na
    // caixa de correio, que a ponte com a cena drena. ⛔ Encenar um `add_leaf` à mão mediria o ECS,
    // não a costura.
    //
    // ⚠️ **E a posição é procurada pela CHAVE, nunca por uma constante** — a lição está no doc deste
    // arquivo, mais abaixo: *um teste que lê a constante que testa não testa a constante*, e uma
    // prova de mutação passou verde por causa disso.
    let slot = crate::field3d_shapes::slot_of("panel.model3d.add.sculpt").expect("a escultura");
    crate::field3d_smoke::ask_shape(slot);
    crate::field3d_scene::sync_scene(&mut sim, None, 0.0);

    assert!(
        crate::field3d_smoke::take_import_request(),
        "o clique tem de pedir um arquivo — sem isto o botão é inerte e não deixa rasto"
    );
    let root = the_root(&mut sim);
    assert_eq!(
        ph2d_field_ecs::walk(sim.world(), root).len(),
        before,
        "e não pode criar nada por si: a peça só nasce depois de o arquivo ser lido"
    );
}

/// ⭐ **O slot da escultura APONTA para a escultura** — e o gate não pode perguntá-lo à constante.
///
/// ⚠️ **Uma prova de mutação passou VERDE e é por isso que este gate existe.** Trocar
/// `SCULPT_SLOT = SHAPES.len() - 1` por um literal `3` não punha nada a vermelho: o gate irmão
/// **empurra o intent com a própria constante**, então mutá-la muda a produção e a entrada do teste
/// ao mesmo tempo. Um teste que lê a constante que testa não testa a constante.
///
/// O que este mede é a **relação**: o slot tem de cair na chave da escultura, e o `shape_at` tem de
/// devolver `None` ali e `Some` em todos os outros. É isso que quebra no dia em que alguém
/// acrescentar uma primitiva no meio da lista.
#[test]
fn the_sculpt_slot_points_at_the_sculpt_button() {
    use crate::field3d_shapes::{Make, SHAPES, shape_at, slot_of};
    let slot = slot_of("panel.model3d.add.sculpt").expect("a escultura tem linha no catálogo");
    assert!(
        matches!(SHAPES[slot].make, Make::Sculpt),
        "a chave da escultura caiu numa linha que constrói outra coisa"
    );
    assert!(
        shape_at(slot, 0.5).is_none(),
        "uma escultura não é uma primitiva — o `shape_at` tem de recusar"
    );
    // ⭐ **E o CONTROLE, que é o que faz a afirmação de cima valer:** as formas de fórmula
    // constroem. Sem ele, um `shape_at` que devolvesse `None` sempre passaria — e o sintoma seria
    // *"nenhum botão faz nada"*.
    let caixa = slot_of("panel.model3d.add.box").expect("a caixa tem linha");
    assert!(
        shape_at(caixa, 0.5).is_some(),
        "a caixa tem de dar uma forma — senão o botão é inerte"
    );
}

/// ⭐ **A escultura carregada vira um NÓ, e o avaliador resolve-o** — a costura inteira.
///
/// ⚠️ **O gate mede as três emendas, e cada uma falha em silêncio sozinha**: o nome chegar ao mundo
/// (senão nada nasce), o nó ser mesmo uma escultura (um `Leaf` no lugar cozinharia uma primitiva
/// vazia), e o **registo** resolver o nome (senão o nó existe, a Hierarquia mostra-o, e a peça é
/// invisível — que é o pior dos três, porque parece um problema de câmera).
#[test]
fn a_loaded_sculpture_becomes_a_node_the_evaluator_resolves() {
    let _ = ph2d_panel_model3d::drain_intents();
    let key = "/tmp/uma-escultura.obj";
    register_a_sculpture(key);

    let mut sim = a_world();
    crate::field3d_scene::sync_scene(&mut sim, Some(&scene(2)), 0.0);
    let root = the_root(&mut sim);
    let before = ph2d_field_ecs::walk(sim.world(), root).len();

    crate::field3d_smoke::ask_spawn_sculpt(key.to_string());
    let doc = crate::field3d_scene::sync_scene(&mut sim, None, 0.0).expect("a peça cozinha");

    let root = the_root(&mut sim);
    assert_eq!(
        ph2d_field_ecs::walk(sim.world(), root).len(),
        before + 1,
        "a escultura tem de virar UM nó"
    );
    let sampled: Vec<&str> = doc
        .nodes()
        .iter()
        .filter_map(|n| match &n.kind {
            ph2d_field::NodeKind::Sampled { key } => Some(key.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        sampled,
        vec![key],
        "o nó cozido tem de ser uma escultura, e com o nome que o arquivo deu"
    );

    let reg = crate::field3d_smoke::sampled_registry();
    let h = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
    assert_eq!(
        h.sampled_count(),
        1,
        "e o registo tem de resolver o nome — senão a peça existe na Hierarquia e some da tela"
    );
}

/// ⭐ **A escultura nasce no tamanho do enquadramento, e o número vai para a POSE.**
///
/// ⚠️ **A geometria NÃO é reescrita**, e a diferença é mensurável: o campo foi construído nas
/// unidades do autor — é isso que faz a célula da grade ser a resolução real do arquivo — e o que
/// muda é a escala do nó, que um clique desfaz. Um import que reescrevesse a malha faria o artista
/// perder os números do arquivo sem ter pedido.
#[test]
fn the_imported_sculpture_arrives_at_the_framing_size() {
    let _ = ph2d_panel_model3d::drain_intents();
    let key = "/tmp/outra-escultura.ply";
    let extent = register_a_sculpture(key);

    let mut sim = a_world();
    crate::field3d_scene::sync_scene(&mut sim, Some(&scene(2)), 0.0);
    crate::field3d_smoke::ask_spawn_sculpt(key.to_string());
    crate::field3d_smoke::ask_sculpt_extent(extent);
    crate::field3d_scene::sync_scene(&mut sim, None, 0.0);

    let root = the_root(&mut sim);
    let world = sim.world_mut();
    let e = ph2d_field_ecs::walk(world, root)
        .into_iter()
        .map(|(e, _)| e)
        .find(|e| {
            matches!(
                world.get::<FieldNode>(*e).map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Sampled { .. })
            )
        })
        .expect("a escultura está lá");
    let scale = world.get::<FieldPose>(e).expect("pose").xform.scale;
    let cam = crate::field3d_smoke::with_smoke(|s| s.vp().cam).unwrap_or_default();
    let want = crate::field3d_import::framing_scale(extent, cam.half_extent);
    assert!(
        (scale - want).abs() < 1e-4,
        "a escultura nasceu com escala {scale} e o enquadramento pede {want}"
    );
    assert!(
        (scale - 1.0).abs() > 1e-6,
        "e o gate só prova alguma coisa se a escala NÃO for 1 — senão ele passa sem o código"
    );
}
