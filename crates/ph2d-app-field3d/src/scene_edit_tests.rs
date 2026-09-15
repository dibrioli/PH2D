//! Os gates da **autoria da peça** — criar, combinar, nomear, duplicar, apagar e digitar.
//!
//! ⚠️ Módulo-filho do arquivo de gates da cena, pela mesma razão do irmão: as fixtures moram no pai.

use super::*;

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
    // ⭐ **UM basta, desde a W31** — e esta linha dizia o contrário. Enio, 2026-08-22: *"ainda não
    // temos como criar novos grupos"*. Embrulhar uma forma sozinha **é** o gesto de criar grupo, e o
    // `>= 2` vinha de o gesto ter nascido como *«juntar os escolhidos»*. *Uma lei que muda muda o
    // gate; apagar o gate seria perder a metade dele que continua a valer* (pais diferentes, raiz).
    assert!(
        ph2d_field_ecs::wrap_in_op(world, &[first], op).is_some(),
        "uma forma sozinha TEM de se embrulhar — é assim que se cria um grupo"
    );
}

/// ⭐ **[`ph2d_field_ecs::can_wrap`] responde EXATAMENTE o que o `wrap_in_op` faz** (W34).
///
/// # Por que esta equivalência é um gate e não um comentário
///
/// O painel tem de saber, **antes** do clique, se o gesto vai acontecer — e não pode mutar o mundo
/// para descobrir. Enquanto a regra viveu só dentro do `wrap_in_op`, o painel escreveu a dele, as
/// duas divergiram, e o gesto de criar grupo ficou **inalcançável** durante uma wave inteira.
///
/// ⚠️ Este gate não mede o `can_wrap` contra uma tabela escrita à mão: mede-o contra a **função que
/// ele guarda**. Uma regra nova que entre num dos lados e não no outro fica vermelha aqui, mesmo que
/// ninguém se lembre de vir escrever o caso.
#[test]
fn can_wrap_answers_exactly_what_wrap_in_op_does() {
    let op = ph2d_field::Op::Union(ph2d_field::Blend::Sharp);
    // Cada caso monta um mundo NOVO: o `wrap_in_op` muta quando aceita, e reutilizar a árvore mediria
    // o segundo caso sobre a hierarquia que o primeiro deixou.
    let case = |pick: fn(
        &mut bevy_ecs::world::World,
        bevy_ecs::entity::Entity,
    ) -> Vec<bevy_ecs::entity::Entity>,
                name: &str| {
        let mut sim = a_world();
        let world = sim.world_mut();
        let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
        let sel = pick(world, root);
        let said = ph2d_field_ecs::can_wrap(world, &sel);
        let did = ph2d_field_ecs::wrap_in_op(world, &sel, op).is_some();
        assert_eq!(
            said, did,
            "«{name}»: o can_wrap disse {said} e o wrap_in_op fez {did} — \
             o painel vai oferecer o que o gesto não faz (ou esconder o que ele faz)"
        );
        did
    };

    fn kids(
        world: &bevy_ecs::world::World,
        root: bevy_ecs::entity::Entity,
    ) -> Vec<bevy_ecs::entity::Entity> {
        world
            .get::<Children>(root)
            .expect("tem filhos")
            .iter()
            .copied()
            .collect()
    }

    assert!(!case(|_, _| Vec::new(), "nada selecionado"));
    assert!(case(
        |w, r| vec![kids(w, r)[0]],
        "uma forma sozinha (cria grupo)"
    ));
    assert!(case(
        |w, r| kids(w, r).into_iter().take(2).collect(),
        "dois irmãos"
    ));
    assert!(!case(|_w, r| vec![r], "a raiz, que não tem pai"));
    assert!(!case(
        |w, r| vec![r, kids(w, r)[0]],
        "a raiz e um filho dela"
    ));
    // ⚠️ Um nó que **não é do campo** (uma anotação qualquer do mundo, irmã das formas) não se
    // embrulha: o grupo referenciá-lo-ia e o cozimento não teria o que emitir.
    assert!(!case(
        |w, r| {
            let alien = w.spawn(ph2d_ecs::Name::new("nota")).id();
            w.entity_mut(r).add_child(alien);
            vec![kids(w, r)[0], alien]
        },
        "um nó que não é do campo"
    ));
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

/// A segunda metade destes gates — ver [`b`].
#[path = "scene_edit_tests_b.rs"]
mod b;
