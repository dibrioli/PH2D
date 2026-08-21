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

/// Uma vista qualquer, para os gates que precisam de uma. ⚠️ Explícita de propósito: a porta de
/// produção lê-a do estado do módulo, e um teste não encena esse estado.
fn a_view() -> (ph2d_field_render::Orbit, ph2d_field_render::Screen) {
    let cam = ph2d_field_render::Orbit::default();
    let screen = ph2d_field_render::Screen::new(800, 600, cam.half_extent);
    (cam, screen)
}

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

    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetParam {
        entity: root.to_bits(),
        // A união tem uma dimensão só: o raio da mistura.
        param: ph2d_field::Param::Dim(0),
        value: 0.2,
    });
    super::sync_scene_and_birth(&mut sim, None, &[root], 7.5);

    let world = sim.world_mut();
    assert!(
        (ph2d_field_ecs::radius_of(world, root).expect("a união tem raio") - 0.2).abs() < 1e-6,
        "o NÓ tem de guardar o raio novo — senão o salvar e o desfazer levam o antigo"
    );
    let snap = ph2d_panel_model3d::state::current();
    let row = snap
        .rows
        .iter()
        .find(|r| r.param == ph2d_field::Param::Dim(0))
        .expect("a linha do filete da união");
    assert_eq!(row.entity, root.to_bits());
    assert!((row.value - 0.2).abs() < 1e-6, "o retrato do MESMO quadro");
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

    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetParam {
        entity: root.to_bits(),
        // A caixa da cena 2 é uma FOLHA: o filete é a última dimensão dela.
        param: ph2d_field::Param::Dim(3),
        value: 5.0,
    });
    super::sync_scene_and_birth(&mut sim, None, &[root], 0.0);

    let snap = ph2d_panel_model3d::state::current();
    let row = snap
        .rows
        .iter()
        .find(|r| r.param == ph2d_field::Param::Dim(3))
        .expect("o filete");
    assert!(
        (row.value - before).abs() < 1e-6,
        "o retrato tem de publicar o valor REAL ({before}), e publicou {}",
        row.value
    );
}

/// ⭐ **O painel é o INSPETOR da seleção** — as linhas são as dimensões do que está escolhido.
///
/// ⚠️ Até a W10 elas eram uma linha por nó com o raio dele: uma segunda vista da estrutura, a
/// competir com a Hierarquia e sem onde pôr largura, altura e profundidade. A divisão passou a ser
/// a da casa — a Hierarquia mostra **o que existe**, o painel mostra **os números do escolhido**.
#[test]
fn the_panel_shows_the_dimensions_of_what_is_selected() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    // A cena 2 é UMA caixa: largura, altura, profundidade e filete.
    sync_scene(&mut sim, Some(&scene(2)), 0.0);
    let root = the_root(&mut sim);
    super::sync_scene_and_birth(&mut sim, None, &[root], 0.0);

    let keys: Vec<&str> = ph2d_panel_model3d::state::current()
        .rows
        .iter()
        .map(|r| r.key)
        .collect();
    assert_eq!(
        keys,
        vec![
            "field.dim.pos_x",
            "field.dim.pos_y",
            "field.dim.pos_z",
            "field.dim.width",
            "field.dim.height",
            "field.dim.depth",
            "field.dim.round",
        ],
        "uma caixa tem a POSE e quatro dimensões, nesta ordem"
    );
    // ⛔ **E NÃO tem `Scale`**: numa folha o tamanho visível são as dimensões, e mostrar as duas
    // coisas daria dois controles para a mesma coisa — sem forma de saber qual o gesto seguinte
    // mexe. Ver `ph2d_field::scale_primitive`.
    assert!(
        !keys.contains(&"field.dim.scale"),
        "uma folha não pode ter escala E dimensões: são duas verdades sobre o mesmo tamanho"
    );

    // ⚠️ **Sem seleção, o painel diz-lo** — em vez de mostrar uma lista de tudo que ninguém pediu.
    super::sync_scene_and_birth(&mut sim, None, &[], 0.0);
    assert!(ph2d_panel_model3d::state::current().rows.is_empty());
}

/// **Todo nome de dimensão tem uma tradução** — nenhuma vaza a chave crua na tela.
///
/// ⚠️ O `tr` da casa devolve a **própria chave** quando não conhece uma (de propósito: o
/// identificador feio na tela é o alarme). Então "traduziu" mede-se por *"o que voltou é diferente
/// da chave"*.
#[test]
fn every_dimension_name_has_a_translation() {
    let _ = ph2d_panel_model3d::drain_intents();
    for n in 1..=5 {
        let mut sim = a_world();
        sync_scene(&mut sim, Some(&scene(n)), 0.0);
        let root = the_root(&mut sim);
        // Cada nó da peça, um a um: é o que o artista alcança clicando.
        let all: Vec<bevy_ecs::entity::Entity> = {
            let world = sim.world_mut();
            ph2d_field_ecs::walk(world, root)
                .into_iter()
                .map(|(e, _)| e)
                .collect()
        };
        for e in all {
            super::sync_scene_and_birth(&mut sim, None, &[e], 0.0);
            for row in ph2d_panel_model3d::state::current().rows {
                assert_ne!(
                    ph2d_i18n::tr(row.key),
                    row.key,
                    "a cena {n} tem uma dimensão cuja chave `{}` não está na tabela",
                    row.key
                );
            }
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
    let (_, born) = super::sync_scene_and_birth(&mut sim, Some(&scene(1)), &[], 0.0);
    let super::SelectRequest::Entity(bits) = born.expect("nascer tem de pedir uma seleção")
    else {
        panic!("nascer pede uma ENTIDADE, não uma limpeza");
    };

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

    let (_, again) = super::sync_scene_and_birth(&mut sim, None, &[], 0.0);
    assert_eq!(
        again, None,
        "o quadro seguinte não volta a mandar selecionar"
    );
}

/// ⭐ **Um giro em torno de um eixo do MUNDO é em torno do eixo do mundo** — mesmo num filho cujo
/// pai está rodado.
///
/// ⚠️ A conta é a conjugação (`inv(R_pai) ⊗ Q ⊗ R_pai`), e sem o sanduíche um giro em torno do X do
/// mundo aplicado a um filho de pai rodado giraria em torno do X **do pai**. O eixo errado, e
/// ninguém diria que o culpado é o gizmo — diria que "a rotação está estranha".
#[test]
fn a_world_axis_spin_stays_on_the_world_axis_under_a_rotated_parent() {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let mut sim = a_world();
    let world = sim.world_mut();
    let doc = FieldDoc::new(
        vec![
            ph2d_field::Node {
                xform: Xform::at(0.3, 0.0, 0.0),
                kind: ph2d_field::NodeKind::Leaf(Primitive::Box {
                    half: [0.3, 0.1, 0.1],
                    round: 0.02,
                }),
            },
            ph2d_field::Node {
                xform: Xform::at(-0.3, 0.0, 0.0),
                kind: ph2d_field::NodeKind::Leaf(Primitive::Sphere { radius: 0.1 }),
            },
            ph2d_field::Node {
                // O pai roda um quarto de volta em torno de Z.
                xform: Xform {
                    translation: [0.0; 3],
                    rotation: [0.0, 0.0, s, s],
                    scale: 1.0,
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

    // Um ponto do nó, longe do centro dele, e o que ele faz sob um quarto de volta em torno do X
    // do MUNDO: `(0,1,0) → (0,0,1)`.
    let probe = [0.0f32, 0.5, 0.0];
    let before = ph2d_field_ecs::world_xform(world, child);
    ph2d_field_ecs::rotate_world(world, child, [1.0, 0.0, 0.0], std::f32::consts::FRAC_PI_2);
    let after = ph2d_field_ecs::world_xform(world, child);

    // A direção local `probe`, vista no mundo, tem de ter rodado em torno do X do mundo.
    let d0 = before.apply_dir(probe);
    let d1 = after.apply_dir(probe);
    let want = [d0[0], -d0[2], d0[1]];
    for k in 0..3 {
        assert!(
            (d1[k] - want[k]).abs() < 1e-5,
            "o giro saiu no eixo errado: {d0:?} -> {d1:?}, esperava {want:?}"
        );
    }
    // E o nó não SAIU do lugar: o pivô é o centro dele.
    for k in 0..3 {
        assert!(
            (after.translation[k] - before.translation[k]).abs() < 1e-6,
            "rodar não pode transladar: {:?} -> {:?}",
            before.translation,
            after.translation
        );
    }
}

/// ⭐ **Numa FOLHA, crescer é crescer as DIMENSÕES** — e a pose fica em 1.
///
/// ⚠️ As duas dariam a mesma forma, mas só uma delas é o número que o painel mostra: escalar a pose
/// deixaria uma caixa que mede 2 na tela e diz «1» no painel — duas verdades sobre o mesmo tamanho
/// visível, sem forma de o artista saber qual o gesto seguinte mexe.
///
/// ⛔ E o fator não-positivo é **recusado**: uma escala nula faria o campo deixar de ser uma
/// distância, e a invariante do módulo é *um nó que existe está válido*.
#[test]
fn scaling_a_leaf_grows_its_dimensions_and_leaves_the_pose_alone() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let child = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");

    let radius_of = |w: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity| {
        ph2d_field_ecs::dims_of(w, e)
            .iter()
            .find(|d| d.key == "field.dim.radius")
            .map(|d| d.value)
            .expect("um cilindro tem raio")
    };
    let before = radius_of(world, child);

    ph2d_field_ecs::scale_by(world, child, 1.5);
    ph2d_field_ecs::scale_by(world, child, 2.0);

    assert!(
        (radius_of(world, child) / before - 3.0).abs() < 1e-5,
        "1,5 x 2 = 3 sobre o RAIO, e deu {}",
        radius_of(world, child) / before
    );
    assert!(
        (world.get::<FieldPose>(child).expect("pose").xform.scale - 1.0).abs() < 1e-6,
        "a pose de uma folha não é onde o tamanho mora"
    );

    for bad in [0.0f32, -1.0, f32::NAN, f32::INFINITY] {
        ph2d_field_ecs::scale_by(world, child, bad);
        assert!(
            (radius_of(world, child) / before - 3.0).abs() < 1e-5,
            "o fator {bad} passou"
        );
    }
    assert!(
        ph2d_field_ecs::cook(world, root)
            .expect("não vazia")
            .is_ok()
    );
}

/// ⭐ **Numa OPERAÇÃO é a pose que escala** — ali ela não compete com nada, porque um grupo não tem
/// dimensões próprias.
#[test]
fn scaling_a_group_multiplies_its_pose() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");

    ph2d_field_ecs::scale_by(world, root, 1.5);
    ph2d_field_ecs::scale_by(world, root, 2.0);
    assert!(
        (world.get::<FieldPose>(root).expect("pose").xform.scale - 3.0).abs() < 1e-5,
        "1,5 x 2 = 3 na pose do grupo"
    );
    // E o painel mostra-a, porque ali ela é a única resposta.
    assert!(
        ph2d_field_ecs::params_of(world, root)
            .iter()
            .any(|(_, d)| d.key == "field.dim.scale"),
        "uma operação tem de mostrar a escala — é o único tamanho que ela tem"
    );
}

/// ⭐ **A ficha diz o que o MUNDO levou** — a lei que o `gizmo/readout.rs` da casa já escreveu, aqui
/// virada em assertiva.
///
/// ⚠️ O número que aparece durante um arrasto sai do `Grip::applied`. Isso só é honesto porque o que
/// o mundo recebe é exatamente esse valor — e "exatamente" é o tipo de afirmação que apodrece num
/// comentário. Se um dia a escrita recusar, limitar ou arredondar um pedido, a ficha passa a dizer
/// `0,503` enquanto a peça pousou em `0,500` — e este gate cai antes de alguém ver.
#[test]
fn the_readout_is_the_pose_the_world_took() {
    use crate::field3d_gizmo::Motion;

    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let child = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");

    let before = ph2d_field_ecs::world_xform(world, child);

    // Os três verbos, cada um com um total PRESO — que é o caso em que a ficha e o mundo mais
    // facilmente divergiriam.
    let moved = Motion::Translate([0.25, -0.1, 0.05]).snapped(0.05);
    let Motion::Translate(d) = moved else {
        panic!("translação")
    };
    ph2d_field_ecs::translate_world(world, child, d);
    let after = ph2d_field_ecs::world_xform(world, child);
    for k in 0..3 {
        assert!(
            (after.translation[k] - before.translation[k] - d[k]).abs() < 1e-5,
            "a ficha diz {d:?} e o mundo levou {:?}",
            [
                after.translation[0] - before.translation[0],
                after.translation[1] - before.translation[1],
                after.translation[2] - before.translation[2],
            ]
        );
    }

    let sized = Motion::Scale(1.47).snapped(0.05);
    let Motion::Scale(f) = sized else {
        panic!("escala")
    };
    // ⚠️ Numa FOLHA o tamanho mora nas dimensões, não na pose — então é ali que a ficha se confere.
    let radius = |w: &bevy_ecs::world::World| {
        ph2d_field_ecs::dims_of(w, child)
            .iter()
            .find(|d| d.key == "field.dim.radius")
            .map(|d| d.value)
            .expect("um cilindro tem raio")
    };
    let s0 = radius(world);
    ph2d_field_ecs::scale_by(world, child, f);
    let s1 = radius(world);
    assert!(
        (s1 / s0 - f).abs() < 1e-5,
        "a ficha diz x{f} e o mundo levou x{}",
        s1 / s0
    );

    let turned = Motion::Rotate {
        axis: [0.0, 0.0, 1.0],
        angle: 0.80,
    }
    .snapped(0.05);
    let Motion::Rotate { axis, angle } = turned else {
        panic!("rotação")
    };
    let r0 = ph2d_field_ecs::world_xform(world, child).rotation;
    ph2d_field_ecs::rotate_world(world, child, axis, angle);
    let r1 = ph2d_field_ecs::world_xform(world, child).rotation;
    // O ângulo entre duas orientações: `2·acos(|<q0, q1>|)`.
    let dot: f32 = (0..4).map(|k| r0[k] * r1[k]).sum();
    let swept = 2.0 * dot.abs().clamp(0.0, 1.0).acos();
    assert!(
        (swept - angle.abs()).abs() < 1e-4,
        "a ficha diz {}° e o mundo girou {}°",
        angle.to_degrees(),
        swept.to_degrees()
    );
}

/// ⭐ **Acrescentar uma forma** — o gesto que faltava para o módulo ser um modelador e não um
/// visualizador.
///
/// ⚠️ Ela nasce **onde a câmera olha** e **no tamanho do enquadramento**, e as duas metades são a
/// mesma condição: uma forma nova tem de ser **vista**. Um tamanho fixo em unidades de mundo nasce
/// invisível numa peça grande e tapa a janela numa pequena, e nos dois casos o artista conclui que o
/// botão não funcionou.
#[test]
fn a_new_shape_is_born_where_the_camera_looks_and_big_enough_to_see() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let before = ph2d_field_ecs::walk(world, root).len();

    let at = [0.4f32, -0.2, 0.1];
    let e = ph2d_field_ecs::add_leaf(world, root, Primitive::Sphere { radius: 0.2 }, at)
        .expect("a raiz aceita filhos");

    assert_eq!(ph2d_field_ecs::walk(world, root).len(), before + 1);
    let pose = ph2d_field_ecs::world_xform(world, e).translation;
    for k in 0..3 {
        assert!(
            (pose[k] - at[k]).abs() < 1e-5,
            "a forma nasceu em {pose:?} e devia nascer em {at:?}"
        );
    }
    // E ela ENTRA na peça: o cozimento tem de a conter.
    let cooked = ph2d_field_ecs::cook(world, root)
        .expect("não vazia")
        .expect("válida");
    assert!(
        cooked
            .nodes()
            .iter()
            .any(|n| matches!(n.kind, ph2d_field::NodeKind::Leaf(Primitive::Sphere { .. }))),
        "a forma nova não chegou ao documento — o botão criou um objeto invisível"
    );
}

/// ⚠️ **Uma forma pendurada FORA da peça é recusada.** Ela apareceria na Hierarquia e o traçado
/// ignorá-la-ia — um objeto que existe e não existe.
#[test]
fn a_shape_cannot_be_hung_outside_the_part() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let alien = world.spawn(ph2d_ecs::Name::new("nota")).id();
    assert!(
        ph2d_field_ecs::add_leaf(world, alien, Primitive::Sphere { radius: 0.1 }, [0.0; 3])
            .is_err()
    );
}

/// ⭐ **Trocar a operação de um nó**, e o RAIO da mistura sobrevive.
///
/// ⚠️ O raio é do nó, não da operação. Perdê-lo ao trocar de união para subtração obrigaria a
/// re-encontrá-lo, e o gesto passaria a custar dois.
#[test]
fn changing_the_operation_keeps_the_blend_radius() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let before = ph2d_field_ecs::radius_of(world, root).expect("a união tem raio");
    assert!(before > 0.0, "a cena 1 tem filete");

    ph2d_field_ecs::set_op(
        world,
        root,
        ph2d_field::Op::Difference(ph2d_field::Blend::Sharp),
    )
    .expect("a raiz é uma combinação");

    assert!(
        matches!(
            world.get::<FieldNode>(root).map(|n| &n.shape),
            Some(NodeShape::Combine(ph2d_field::Op::Difference(_)))
        ),
        "a operação não mudou"
    );
    assert!(
        (ph2d_field_ecs::radius_of(world, root).expect("continua a ter raio") - before).abs()
            < 1e-6,
        "o raio da mistura evaporou na troca"
    );
}

/// ⭐ **Embrulhar duas formas numa operação** — a autoria da booleana.
///
/// ⚠️ **A ORDEM que entra é o significado** numa subtração: `children[0]` menos os seguintes.
#[test]
fn wrapping_two_siblings_makes_a_new_operation_in_their_place() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let kids: Vec<bevy_ecs::entity::Entity> = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();
    assert_eq!(kids.len(), 3);

    let group = ph2d_field_ecs::wrap_in_op(
        world,
        &kids[..2],
        ph2d_field::Op::Difference(ph2d_field::Blend::Sharp),
    )
    .expect("dois irmãos embrulham");

    // O grupo é filho da raiz, e os dois passaram a ser filhos DELE.
    assert_eq!(world.get::<ChildOf>(group).map(|c| c.0), Some(root));
    for k in &kids[..2] {
        assert_eq!(world.get::<ChildOf>(*k).map(|c| c.0), Some(group));
    }
    // E a ordem que entrou é a que ficou — é ela que diz o que é subtraído de quê.
    assert_eq!(
        world
            .get::<Children>(group)
            .expect("o grupo tem filhos")
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        kids[..2].to_vec()
    );
    assert!(
        ph2d_field_ecs::cook(world, root)
            .expect("não vazia")
            .is_ok()
    );
}

/// ⚠️ **Embrulhar exige PAI COMUM**, e não é conveniência: mover um nó para debaixo de outra
/// operação muda o que ele é subtraído de — um segundo gesto, com o seu próprio desfazer. Um
/// «embrulhar» que o fizesse em silêncio seria dois gestos com um nome só.
#[test]
fn wrapping_refuses_nodes_that_do_not_share_a_parent() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let first = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");
    let op = ph2d_field::Op::Union(ph2d_field::Blend::Sharp);
    let others: Vec<bevy_ecs::entity::Entity> = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .filter(|e| *e != first)
        .collect();

    // ⚠️ **O caso que de facto exercita a regra:** dois nós que TÊM pai, e pais DIFERENTES.
    //
    // A primeira versão deste gate usava a raiz e um filho dela — e passava **por acidente**, porque
    // a raiz não tem pai nenhum e a função sai mais cedo. Uma prova de mutação apanhou-o: retirar a
    // exigência de pai comum deixava-o verde. *Um gate que passa pelo motivo errado não prova nada.*
    let group = ph2d_field_ecs::wrap_in_op(world, &others, op).expect("dois irmãos embrulham");
    let inside = world
        .get::<Children>(group)
        .expect("o grupo tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro de dentro");
    assert_eq!(world.get::<ChildOf>(first).map(|c| c.0), Some(root));
    assert_eq!(world.get::<ChildOf>(inside).map(|c| c.0), Some(group));
    assert!(
        ph2d_field_ecs::wrap_in_op(world, &[first, inside], op).is_none(),
        "dois nós de pais diferentes não se embrulham — mover um deles é outro gesto"
    );

    // A raiz não tem pai, então também não se embrulha.
    assert!(ph2d_field_ecs::wrap_in_op(world, &[root, first], op).is_none());
    // E um só nunca é «dois».
    assert!(ph2d_field_ecs::wrap_in_op(world, &[first], op).is_none());
}

/// **Dois irmãos com o mesmo tipo não ficam com o mesmo nome.**
///
/// ⚠️ A Hierarquia é a única superfície em que estes objetos têm identidade legível, e três linhas
/// «Sphere» tornam-na inútil exatamente quando a peça começa a ficar interessante.
#[test]
fn siblings_of_the_same_kind_get_distinct_names() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    for _ in 0..3 {
        ph2d_field_ecs::add_leaf(world, root, Primitive::Sphere { radius: 0.1 }, [0.0; 3])
            .expect("nasce");
    }
    let names: Vec<String> = ph2d_field_ecs::walk(world, root)
        .into_iter()
        .filter_map(|(e, _)| {
            world
                .get::<ph2d_ecs::Name>(e)
                .map(|n| n.as_str().to_string())
        })
        .collect();
    let unique: std::collections::BTreeSet<&String> = names.iter().collect();
    assert_eq!(unique.len(), names.len(), "nomes repetidos: {names:?}");
}

/// ⭐ **Duplicar copia a SUBÁRVORE inteira**, como irmã, e com nomes próprios.
///
/// ⚠️ A subárvore, e não só o nó: o caso útil é copiar um *furo* que já é ele próprio uma subtração
/// de várias formas. Copiar só o topo daria um grupo vazio — que não é nada, e o artista veria o
/// botão «funcionar» sem nada aparecer.
#[test]
fn duplicating_copies_the_whole_subtree_as_a_sibling() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let kids: Vec<bevy_ecs::entity::Entity> = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();
    // Um grupo com dois dentro: é o que torna a cópia recursiva observável.
    let group = ph2d_field_ecs::wrap_in_op(
        world,
        &kids[..2],
        ph2d_field::Op::Difference(ph2d_field::Blend::Sharp),
    )
    .expect("embrulha");
    let before = ph2d_field_ecs::walk(world, root).len();

    let copy = ph2d_field_ecs::duplicate(world, group, [0.3, 0.0, 0.0]).expect("duplica");

    assert_eq!(
        world.get::<ChildOf>(copy).map(|c| c.0),
        Some(root),
        "é IRMÃ"
    );
    assert_eq!(
        ph2d_field_ecs::walk(world, root).len(),
        before + 3,
        "o grupo e os dois filhos dele — três nós novos"
    );
    assert_eq!(
        world.get::<Children>(copy).map(|c| c.len()),
        Some(2),
        "a cópia tem os filhos dela"
    );
    // ⚠️ E a ORDEM dos filhos sobrevive: é ela que diz o que é subtraído de quê.
    let kind_of = |e: bevy_ecs::entity::Entity, w: &bevy_ecs::world::World| {
        w.get::<FieldNode>(e)
            .map(|n| ph2d_field_ecs::shape_name(&n.shape))
    };
    let orig: Vec<_> = world
        .get::<Children>(group)
        .expect("original")
        .iter()
        .copied()
        .map(|e| kind_of(e, world))
        .collect();
    let made: Vec<_> = world
        .get::<Children>(copy)
        .expect("cópia")
        .iter()
        .copied()
        .map(|e| kind_of(e, world))
        .collect();
    assert_eq!(orig, made, "a ordem dos filhos baralhou-se na cópia");

    // A cópia saiu do sítio do original.
    let a = ph2d_field_ecs::world_xform(world, group).translation;
    let b = ph2d_field_ecs::world_xform(world, copy).translation;
    assert!((b[0] - a[0] - 0.3).abs() < 1e-5, "{a:?} -> {b:?}");
    // E a peça continua válida.
    assert!(
        ph2d_field_ecs::cook(world, root)
            .expect("não vazia")
            .is_ok()
    );
}

/// ⛔ **A RAIZ não se duplica nem se apaga pelo painel.**
///
/// Ela *é* a peça. Apagá-la deixaria o módulo sem nada para onde voltar (a cena inicial só existe no
/// primeiro quadro), e duplicá-la seria criar uma segunda peça — um gesto da **cena**, não uma
/// edição desta.
#[test]
fn the_root_is_neither_duplicated_nor_deleted_from_the_panel() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    assert!(ph2d_field_ecs::duplicate(world, root, [0.1, 0.0, 0.0]).is_none());
    assert!(!ph2d_field_ecs::remove(world, root));
    assert!(world.get::<FieldNode>(root).is_some(), "a peça continua lá");
}

/// ⭐ **Apagar leva o que está debaixo junto**, e a peça continua válida.
#[test]
fn deleting_a_group_takes_its_children_with_it() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let kids: Vec<bevy_ecs::entity::Entity> = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();
    let group = ph2d_field_ecs::wrap_in_op(
        world,
        &kids[..2],
        ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
    )
    .expect("embrulha");

    assert!(ph2d_field_ecs::remove(world, group));
    assert!(world.get_entity(group).is_err(), "o grupo saiu");
    for k in &kids[..2] {
        assert!(world.get_entity(*k).is_err(), "os filhos foram com ele");
    }
    // Sobra o terceiro cilindro, e a peça é válida.
    assert_eq!(ph2d_field_ecs::walk(world, root).len(), 2);
    assert!(
        ph2d_field_ecs::cook(world, root)
            .expect("não vazia")
            .is_ok()
    );
}

/// ⭐ **Apagar a peça na Hierarquia apaga-a de VERDADE** — ela não volta no quadro seguinte.
///
/// ⚠️ Era um bug, e o comentário do código afirmava o contrário do que o código fazia. A ponte
/// oferecia o documento **cozido** como semente («a peça inicial»), e o comentário dizia que ele
/// *"deixa de existir"* — o que nunca foi verdade: ele é reescrito a cada quadro. Apagar a raiz
/// deixava a ponte sem raiz, e ela **replantava o que tinha acabado de cozer**.
///
/// *Uma semente usa-se uma vez.*
/// ⚠️ **Passa pelo `ecs_bridge`**, e não pela metade de baixo: a decisão que estava errada era
/// *o que a ponte oferece como semente*, e um gate que passasse `None` à mão nunca lhe chegaria.
/// (Foi assim que a primeira versão deste gate ficou verde com o bug reposto.)
#[test]
fn deleting_the_part_does_not_replant_it_next_frame() {
    let _ = ph2d_panel_model3d::drain_intents();
    crate::field3d_smoke::set_armed_by_panel(true);
    let mut sim = a_world();

    // Quadro 1: a ponte planta a semente.
    super::ecs_bridge(&mut sim, None, &[]);
    let root = {
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldObject)>();
        q.iter(world).next().map(|(e, _)| e).expect("a peça nasceu")
    };

    // A Hierarquia apaga a peça (cascata: a raiz leva os filhos).
    sim.world_mut().despawn(root);

    // Quadro 2: a ponte corre outra vez — e **não replanta**.
    super::ecs_bridge(&mut sim, None, &[]);
    let world = sim.world_mut();
    let mut q = world.query::<&FieldObject>();
    assert_eq!(
        q.iter(world).count(),
        0,
        "a peça voltou — a ponte replantou o que tinha acabado de cozer"
    );
}

/// ⭐ **Duplicar pela Hierarquia e pelo painel é a MESMA porta.**
///
/// ⚠️ O braço genérico da Hierarquia copia `Transform` + `Sprite` + `Name`. Um nó de campo **não tem
/// nenhum dos dois** — sairia uma linha na Hierarchy sobre geometria nenhuma, invisível para o
/// traçado. É o mesmo defeito que a nota vetorial daquele bloco já descreve, no módulo seguinte.
///
/// O gate mede o que a cópia **é**, e não que ela existe: sem `FieldNode` ela é o sósia.
#[test]
fn a_duplicate_is_a_real_node_not_a_nameless_twin() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let first = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");

    let bits = super::duplicate_with_view(world, first, &a_view().0, a_view().1).expect("duplica");
    let copy = bevy_ecs::entity::Entity::from_bits(bits);

    assert!(
        world.get::<FieldNode>(copy).is_some(),
        "a cópia tem de SER um nó — sem isto ela é uma linha na Hierarchy sobre nada"
    );
    assert!(world.get::<FieldPose>(copy).is_some(), "e ter pose própria");
    assert_eq!(world.get::<ChildOf>(copy).map(|c| c.0), Some(root));
    // E ela entra na peça: o cozimento tem de a conter.
    let cooked = ph2d_field_ecs::cook(world, root)
        .expect("não vazia")
        .expect("válida");
    assert_eq!(
        cooked.nodes().len(),
        5,
        "três cilindros + a cópia + a união"
    );
}

/// ⚠️ **A porta da Hierarquia é a MESMA do painel**, e o gate prende-o: as duas cópias saem no mesmo
/// sítio relativo.
///
/// Duas contas para *"onde vai a cópia?"* divergiriam no primeiro ajuste — com o artista a ver o
/// mesmo gesto fazer duas coisas conforme por onde o pediu.
#[test]
fn both_doors_put_the_copy_in_the_same_place() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let kids: Vec<bevy_ecs::entity::Entity> = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();

    let offset_of = |world: &mut bevy_ecs::world::World, src: bevy_ecs::entity::Entity| {
        let before = ph2d_field_ecs::world_xform(world, src).translation;
        let bits =
            super::duplicate_with_view(world, src, &a_view().0, a_view().1).expect("duplica");
        let after = ph2d_field_ecs::world_xform(world, bevy_ecs::entity::Entity::from_bits(bits))
            .translation;
        [
            after[0] - before[0],
            after[1] - before[1],
            after[2] - before[2],
        ]
    };
    let a = offset_of(world, kids[0]);
    let b = offset_of(world, kids[1]);
    for k in 0..3 {
        assert!(
            (a[k] - b[k]).abs() < 1e-6,
            "as duas cópias saíram em sítios diferentes: {a:?} e {b:?}"
        );
    }
    // E o deslocamento não é zero — senão o gate passaria com as duas portas a não fazer nada.
    assert!(
        a.iter().any(|v| v.abs() > 1e-6),
        "a cópia ficou em cima do original"
    );
}

/// ⭐ **Digitar uma posição move o objeto** — o par da W10, que deu o tamanho e não a pose.
///
/// ⚠️ A posição é **LOCAL**, e é a convenção da casa: o Inspector dela mostra o `Transform`, que é
/// local, e o readout do gizmo 2D diz por extenso que o delta é local *"porque é isso que o
/// Inspector mostra"*. Um painel que mostrasse mundo contradiria o número ao lado no dia em que
/// alguém agrupasse — e o gate prova-o com um pai deslocado.
#[test]
fn typing_a_position_moves_the_node_in_its_parents_frame() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    // O grupo sai do zero: se as linhas mostrassem MUNDO, elas passariam a incluir este offset.
    ph2d_field_ecs::set_param(world, root, ph2d_field::Param::Pos(0), 1.0)
        .expect("a raiz tem pose");

    let child = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");
    ph2d_field_ecs::set_param(world, child, ph2d_field::Param::Pos(2), 0.4).expect("escreve Z");

    let local = world
        .get::<FieldPose>(child)
        .expect("pose")
        .xform
        .translation;
    assert!((local[2] - 0.4).abs() < 1e-6, "o número escrito é o local");

    // E o painel mostra o mesmo número — não o de mundo, que aqui é outro.
    let shown = ph2d_field_ecs::params_of(world, child)
        .into_iter()
        .find(|(p, _)| *p == ph2d_field::Param::Pos(0))
        .map(|(_, d)| d.value)
        .expect("a linha de X");
    assert!(
        (shown - local[0]).abs() < 1e-6,
        "o painel mostra {shown} e a pose local é {}",
        local[0]
    );
    let world_x = ph2d_field_ecs::world_xform(world, child).translation[0];
    assert!(
        (world_x - local[0]).abs() > 0.5,
        "a fixture tem de ter mundo != local, senão o gate não distingue os dois"
    );
}

/// ⛔ **Uma escala não-positiva é recusada pela porta do painel**, e o nó fica como estava.
#[test]
fn typing_a_non_positive_scale_is_refused() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    for bad in [0.0f32, -2.0, f32::NAN] {
        assert!(ph2d_field_ecs::set_param(world, root, ph2d_field::Param::Scale, bad).is_err());
    }
    assert!((world.get::<FieldPose>(root).expect("pose").xform.scale - 1.0).abs() < 1e-6);
}
