//! Os gates da **ponte com a cena** — o caminho de produção inteiro: mundo, entidades, hierarquia,
//! intents, retrato, cozimento.
//!
//! ⚠️ **É aqui que se prova que a peça É uma cena de objetos.** O gate da W1 media a metade errada
//! (que o componente *sobrevive* ao snapshot) e o da W4 media que *uma* entidade nasceu — as duas
//! passavam enquanto a Hierarquia mostrava um objeto onde há três cilindros.

use bevy_ecs::hierarchy::{ChildOf, Children};
use ph2d_ecs::SimWorld;
use ph2d_field::{FieldDoc, NodeShape, Primitive, Xform};
use ph2d_field_ecs::{FieldNode, FieldObject, FieldPose};

use super::sync_scene;
use crate::field3d_smoke::scene;

fn a_world() -> SimWorld {
    SimWorld::new()
}

/// O nome de cada linha da Hierarquia, em pré-ordem.
fn names(sim: &mut SimWorld, root: bevy_ecs::entity::Entity) -> Vec<String> {
    let world = sim.world_mut();
    ph2d_field_ecs::walk(world, root)
        .into_iter()
        .map(|(e, d)| {
            let n = world
                .get::<ph2d_ecs::Name>(e)
                .map_or("?", ph2d_ecs::Name::as_str);
            format!("{}{n}", "  ".repeat(d as usize))
        })
        .collect()
}

fn the_root(sim: &mut SimWorld) -> bevy_ecs::entity::Entity {
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldObject)>();
    q.iter(world).next().map(|(e, _)| e).expect("a peça existe")
}

/// ⭐ **A peça é uma CENA de objetos** — o gate do smoke reprovado.
///
/// Enio, 2026-08-19: *"na hierarchy apenas um objeto e não 3 cilindro"*. A cena 1 é a junção de
/// três cilindros; a Hierarquia tem de mostrar os três, **com nomes distintos**, aninhados sob a
/// operação que os junta.
#[test]
fn the_part_is_a_scene_of_objects_not_one_opaque_node() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    sync_scene(&mut sim, Some(&scene(1)), 0.0);
    let root = the_root(&mut sim);

    assert_eq!(
        names(&mut sim, root),
        vec!["Model", "  Cylinder", "  Cylinder 2", "  Cylinder 3"],
        "cada primitiva tem de ser um objeto próprio, com nome que a distingue dos irmãos"
    );

    // E cada um é um nó de verdade: tem forma e tem pose.
    let world = sim.world_mut();
    for (e, _) in ph2d_field_ecs::walk(world, root) {
        assert!(world.get::<FieldNode>(e).is_some(), "todo objeto é um nó");
        assert!(world.get::<FieldPose>(e).is_some(), "todo nó tem pose 3D");
    }
}

/// **A raiz é o que a Hierarquia enumera como objeto de topo**, e os filhos não precisam de ser.
///
/// ⚠️ A condição é concreta e MEDIDA no `build_hierarchy_snapshot`: raiz = `With<Transform>` +
/// `Without<ChildOf>`; os filhos são alcançados por `Children`, sem filtro nenhum. É por isso que
/// os nós **não** carregam o `Transform` 2D da casa — ele não tem onde guardar uma rotação 3D, e
/// meia pose lá seria a segunda verdade.
#[test]
fn only_the_root_carries_the_two_d_transform() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    sync_scene(&mut sim, Some(&scene(1)), 0.0);
    let root = the_root(&mut sim);
    let world = sim.world_mut();

    assert!(world.get::<ph2d_ecs::Transform>(root).is_some());
    assert!(world.get::<ChildOf>(root).is_none());
    assert!(world.get::<ph2d_ecs::RootOrder>(root).is_some());

    for (e, depth) in ph2d_field_ecs::walk(world, root) {
        if depth > 0 {
            assert!(
                world.get::<ph2d_ecs::Transform>(e).is_none(),
                "um nó 3D não tem pose 2D — dar-lhe uma seria escrever uma posição que a peça não tem"
            );
            assert!(world.get::<ChildOf>(e).is_some(), "e ele tem pai");
        }
    }
}

/// ⭐ **Ida e volta**: explodir um documento em objetos e voltar a cozê-los dá a **mesma** peça.
///
/// ⚠️ É o gate que impede as duas direções de divergirem. Sem ele, uma primitiva nova entra no
/// `spawn` e falta no `cook` (ou ao contrário) e o sintoma é a peça mudar de forma ao abrir o
/// arquivo — muito depois de a linha ter sido escrita.
#[test]
fn spawning_a_part_and_cooking_it_back_gives_the_same_part() {
    for n in 1..=5 {
        let doc = scene(n);
        let mut sim = a_world();
        let world = sim.world_mut();
        let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
        let back = ph2d_field_ecs::cook(world, root)
            .expect("a peça não está vazia")
            .expect("e é válida");
        assert_eq!(back, doc, "a cena {n} mudou de forma na ida e volta");
    }
}

/// **A ponte não cria uma peça por quadro.** Correr dez vezes deixa UMA raiz.
///
/// ⚠️ É o modo de falha natural de um "spawn se não existe" escrito ao contrário, e ele seria
/// invisível num quadro: dez peças empilhadas no mesmo sítio desenham como uma.
#[test]
fn the_bridge_spawns_the_part_once_not_once_per_frame() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    for _ in 0..10 {
        sync_scene(&mut sim, Some(&scene(1)), 0.0);
    }
    let world = sim.world_mut();
    let mut q = world.query::<&FieldObject>();
    assert_eq!(q.iter(world).count(), 1);
}

/// ⭐ **A edição do painel chega ao NÓ e ao retrato no mesmo quadro**, e ela viaja por **entidade**.
///
/// A ordem — drenar, aplicar, publicar — é load-bearing: se o retrato saísse primeiro, o painel
/// pintaria o valor antigo por um quadro e o controle daria um salto para trás debaixo do dedo.
#[test]
fn a_panel_edit_reaches_the_node_and_the_snapshot_in_the_same_frame() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    sync_scene(&mut sim, Some(&scene(1)), 0.0);
    let root = the_root(&mut sim);

    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetRadius {
        entity: root.to_bits(),
        radius: 0.2,
    });
    sync_scene(&mut sim, None, 7.5);

    let world = sim.world_mut();
    assert!(
        (ph2d_field_ecs::radius_of(world, root).expect("a união tem raio") - 0.2).abs() < 1e-6,
        "o NÓ tem de guardar o raio novo — senão o salvar e o desfazer levam o antigo"
    );
    let snap = ph2d_panel_model3d::state::current();
    let row = snap
        .rows
        .iter()
        .find(|r| r.entity == root.to_bits())
        .expect("a linha da raiz");
    assert!((row.radius - 0.2).abs() < 1e-6, "o retrato do MESMO quadro");
    assert!((snap.last_trace_ms - 7.5).abs() < 1e-6);
}

/// ⚠️ **Uma edição RECUSADA devolve o controle ao valor real**, em vez de deixar o painel a mostrar
/// um número que a peça não tem.
#[test]
fn a_refused_edit_publishes_the_value_the_document_actually_kept() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    // A cena 2 é um cubo de meia-extensão 0,45: o `round` não pode chegar a 0,45.
    sync_scene(&mut sim, Some(&scene(2)), 0.0);
    let root = the_root(&mut sim);
    let before = {
        let world = sim.world_mut();
        ph2d_field_ecs::radius_of(world, root).expect("o cubo tem round")
    };

    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetRadius {
        entity: root.to_bits(),
        radius: 5.0,
    });
    sync_scene(&mut sim, None, 0.0);

    let snap = ph2d_panel_model3d::state::current();
    assert!(
        (snap.rows[0].radius - before).abs() < 1e-6,
        "o retrato tem de publicar o valor REAL ({before}), e publicou {}",
        snap.rows[0].radius
    );
}

/// **As linhas do painel são a árvore**, na ordem e com a profundidade da Hierarquia.
#[test]
fn the_rows_are_the_hierarchy_tree_not_a_flat_list() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    sync_scene(&mut sim, Some(&scene(1)), 0.0);
    let rows = ph2d_panel_model3d::state::current().rows;

    assert_eq!(rows.first().map(|r| r.depth), Some(0), "a raiz é o nível 0");
    assert!(
        rows.iter().skip(1).all(|r| r.depth == 1),
        "os filhos da união estão um nível abaixo — uma lista plana esconderia a árvore"
    );
}

/// **Toda linha do painel tem uma chave de i18n que traduz** — nenhuma vaza o identificador cru.
///
/// ⚠️ O `tr` da casa devolve a **própria chave** quando não conhece uma (de propósito: o
/// identificador feio na tela é o alarme). Então "traduziu" mede-se por *"o que voltou é diferente
/// da chave"*.
#[test]
fn every_row_kind_has_a_translation() {
    for n in 1..=5 {
        let _ = ph2d_panel_model3d::drain_intents();
        let mut sim = a_world();
        sync_scene(&mut sim, Some(&scene(n)), 0.0);
        for row in ph2d_panel_model3d::state::current().rows {
            assert_ne!(
                ph2d_i18n::tr(row.kind_key),
                row.kind_key,
                "a cena {n} tem um nó cuja chave `{}` não está na tabela",
                row.kind_key
            );
        }
    }
}

/// ⭐ **Mover um nó move só aquele nó** — e a peça cozida sente-o.
///
/// É a metade de dados do gizmo: sem isto uma alça arrastaria um número que a superfície não lê.
#[test]
fn moving_a_node_moves_that_node_in_the_cooked_part() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    sync_scene(&mut sim, Some(&scene(1)), 0.0);
    let root = the_root(&mut sim);
    let world = sim.world_mut();
    let first = world
        .get::<Children>(root)
        .expect("a união tem filhos")
        .iter()
        .copied()
        .next()
        .expect("pelo menos um");

    let before = ph2d_field_ecs::world_xform(world, first).translation;
    world.get_mut::<FieldPose>(first).expect("tem pose").xform =
        Xform::at(before[0] + 0.3, before[1], before[2]);
    let after = ph2d_field_ecs::world_xform(world, first).translation;
    assert!((after[0] - before[0] - 0.3).abs() < 1e-6);

    let cooked = ph2d_field_ecs::cook(world, root)
        .expect("não vazia")
        .expect("válida");
    assert!(
        cooked
            .nodes()
            .iter()
            .any(|n| (n.xform.translation[0] - after[0]).abs() < 1e-6),
        "a peça cozida tem de conter a pose nova — senão o gizmo mexe num número morto"
    );
}

/// ⚠️ **Uma pose de MUNDO é a cadeia composta**, não a do próprio nó.
///
/// O gizmo desenha em mundo e o documento guarda local; se `world_xform` não compusesse, a alça
/// apareceria na origem enquanto a peça está noutro sítio — e o arrasto escreveria o valor errado.
#[test]
fn a_world_pose_is_the_whole_chain_composed() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let doc = FieldDoc::new(
        vec![
            ph2d_field::Node {
                xform: Xform::at(0.1, 0.0, 0.0),
                kind: ph2d_field::NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
            },
            ph2d_field::Node {
                xform: Xform::at(0.1, 0.0, 0.0),
                kind: ph2d_field::NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
            },
            ph2d_field::Node {
                xform: Xform::at(1.0, 2.0, 3.0),
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![ph2d_field::NodeId(0), ph2d_field::NodeId(1)],
                },
            },
        ],
        ph2d_field::NodeId(2),
    )
    .expect("documento válido");
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let child = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");

    let w = ph2d_field_ecs::world_xform(world, child).translation;
    assert!(
        (w[0] - 1.1).abs() < 1e-6 && (w[1] - 2.0).abs() < 1e-6 && (w[2] - 3.0).abs() < 1e-6,
        "esperava (1.1, 2, 3) e veio {w:?} — a cadeia não compôs"
    );
}

/// ⚠️ **Apagar o último filho esvazia a peça em vez de a partir.**
///
/// Apagar objetos na Hierarquia é um gesto normal. Devolver `EmptyCombine` aqui transformaria esse
/// gesto num erro que o artista teria de desfazer para entender; devolver *nada* é a resposta que
/// o gesto pediu.
#[test]
fn deleting_every_child_empties_the_part_instead_of_breaking_it() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let kids: Vec<_> = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();
    for k in kids {
        world.entity_mut(k).despawn();
    }
    assert!(
        ph2d_field_ecs::cook(world, root).is_none(),
        "uma união sem filhos não é geometria nenhuma"
    );
    // E a raiz continua na cena: quem a apaga é a Hierarquia, não o cozimento.
    assert!(world.get::<FieldNode>(root).is_some());
}

/// **Uma entidade que não é nó não entra na peça**, nem os filhos dela.
///
/// ⚠️ A cena é partilhada: qualquer sistema pode pendurar uma entidade debaixo de um objeto. Se o
/// cozimento a lesse, um sprite dentro de uma peça viraria geometria — ou, mais provável, um pânico.
#[test]
fn a_foreign_entity_under_the_part_is_ignored() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let before = ph2d_field_ecs::cook(world, root)
        .expect("não vazia")
        .expect("válida");

    let alien = world.spawn(ph2d_ecs::Name::new("nota do artista")).id();
    world.entity_mut(root).add_child(alien);

    let after = ph2d_field_ecs::cook(world, root)
        .expect("não vazia")
        .expect("válida");
    assert_eq!(before, after, "uma entidade estranha não pode mudar a peça");
}

/// **O nome de um nó sai do que ele é**, e dois irmãos iguais não ficam com o mesmo nome.
#[test]
fn a_node_is_named_after_what_it_is() {
    assert_eq!(
        ph2d_field_ecs::shape_name(&NodeShape::Leaf(Primitive::Sphere { radius: 1.0 })),
        "Sphere"
    );
    assert_eq!(
        ph2d_field_ecs::shape_name(&NodeShape::Combine(ph2d_field::Op::Difference(
            ph2d_field::Blend::Sharp
        ))),
        "Difference"
    );
}

/// ⭐ **Um deslocamento de MUNDO escrito num nó com pai rodado aterra onde o gizmo pediu.**
///
/// ⚠️ É a conversão que se esquece. Somar o deslocamento de mundo direto na translação **local**
/// funciona exatamente enquanto nenhum pai tiver rotação ou escala — o que é verdade na primeira
/// cena de smoke e em nenhuma peça real. O sintoma é a seta X mover a peça na diagonal, e ele só
/// aparece depois de alguém rodar um grupo.
#[test]
fn a_world_delta_lands_where_the_gizmo_asked_even_under_a_rotated_parent() {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let mut sim = a_world();
    let world = sim.world_mut();
    let doc = FieldDoc::new(
        vec![
            ph2d_field::Node {
                xform: Xform::at(0.1, 0.0, 0.0),
                kind: ph2d_field::NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
            },
            ph2d_field::Node {
                xform: Xform::at(0.2, 0.0, 0.0),
                kind: ph2d_field::NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
            },
            ph2d_field::Node {
                // Um quarto de volta em Z, e escala 2: as duas metades da inversa que faltavam.
                xform: Xform {
                    translation: [0.5, -0.3, 0.2],
                    rotation: [0.0, 0.0, s, s],
                    scale: 2.0,
                },
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![ph2d_field::NodeId(0), ph2d_field::NodeId(1)],
                },
            },
        ],
        ph2d_field::NodeId(2),
    )
    .expect("documento válido");
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let child = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");

    let before = ph2d_field_ecs::world_xform(world, child).translation;
    let asked = [0.37f32, -0.11, 0.05];
    ph2d_field_ecs::translate_world(world, child, asked);
    let after = ph2d_field_ecs::world_xform(world, child).translation;

    for k in 0..3 {
        assert!(
            (after[k] - before[k] - asked[k]).abs() < 1e-5,
            "no eixo {k} o gizmo pediu {} e a peça andou {}",
            asked[k],
            after[k] - before[k]
        );
    }
}

/// ⭐ **A peça nasce com um objeto selecionado** — as setas aparecem sem ninguém adivinhar o gesto.
///
/// ⚠️ E o selecionado é um **filho**, não a raiz: a raiz é o grupo inteiro, e um gizmo em cima dela
/// move a peça toda. Quem abre o módulo pela primeira vez quer ver o que uma seta faz a **uma**
/// forma.
///
/// ⚠️ **Uma vez, e só nessa.** Re-selecionar todo quadro tiraria da mão do artista o direito de
/// escolher outro objeto — o mesmo defeito que o painel de modelagem já pagou ao reabrir sozinho.
#[test]
fn the_part_is_born_with_an_object_selected_once_and_only_once() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    let (_, born) = super::sync_scene_and_birth(&mut sim, Some(&scene(1)), 0.0);
    let bits = born.expect("nascer tem de pedir uma seleção");

    let root = the_root(&mut sim);
    let world = sim.world_mut();
    let e = bevy_ecs::entity::Entity::from_bits(bits);
    assert!(world.get::<FieldNode>(e).is_some(), "o selecionado é um nó");
    assert_ne!(e, root, "e não é a raiz — a raiz é o grupo inteiro");
    assert_eq!(
        world.get::<ChildOf>(e).map(|c| c.0),
        Some(root),
        "é um filho direto da peça"
    );

    let (_, again) = super::sync_scene_and_birth(&mut sim, None, 0.0);
    assert_eq!(
        again, None,
        "o quadro seguinte não volta a mandar selecionar"
    );
}
