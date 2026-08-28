//! Os gates da ponte ECS.

use super::*;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_field::{Node, NodeId, NodeKind, NodeShape, Primitive, Xform};

fn doc(radius: f32) -> FieldDoc {
    FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Sphere { radius }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("esfera")
}

/// ⭐ **O gate que justifica a chave existir.** A mesma cena, entregue em ordens diferentes — como
/// uma consulta ECS de facto a devolve — tem de produzir o **mesmo documento**, byte a byte.
///
/// Sem isto, cada quadro diferiria do anterior, o snapshot veria bytes novos, e **todo quadro
/// viraria um passo de undo**. É o bug que o `canonicalize()` do shell já pagou uma vez, e a cura
/// ali foi a mesma: ordenar por CONTEÚDO/nome, nunca pela ordem de alocação.
#[test]
fn the_scene_field_does_not_depend_on_the_order_the_query_returns() {
    let a = ("alfa", doc(0.3));
    let b = ("beta", doc(0.4));
    let c = ("gama", doc(0.5));

    let one = scene_field(vec![a.clone(), b.clone(), c.clone()], Blend::Sharp)
        .expect("três objetos")
        .expect("válido");
    let two = scene_field(vec![c, a, b], Blend::Sharp)
        .expect("três objetos")
        .expect("válido");

    assert_eq!(one, two, "a ordem da consulta vazou para o documento");
}

/// ⭐ **A porta RECUSA um modificador sobre uma escultura** — e o mundo fica como estava.
///
/// ⚠️ **A regra já vivia no documento** ([`ph2d_field::FieldError::ModsOnSampled`]) e ninguém a
/// consultava antes de escrever: o componente entrava, o cozimento do quadro seguinte recusava o
/// documento **inteiro**, e a peça desaparecia da tela com a Hierarquia intacta. *Uma invariante que
/// só o validador conhece é uma invariante que a UI descobre partindo-se.*
///
/// O gate mede as **duas** metades — a recusa **e** que uma forma normal continua a aceitar —, senão
/// passaria com `add_mod` a recusar toda a gente.
#[test]
fn a_sculpture_refuses_a_modifier_and_the_world_is_left_alone() {
    let mut world = bevy_ecs::world::World::new();
    let sculpture = world
        .spawn(FieldNode {
            shape: NodeShape::Sampled {
                key: "/tmp/uma.obj".into(),
            },
        })
        .id();
    let leaf = world
        .spawn(FieldNode {
            shape: NodeShape::Leaf(Primitive::Sphere { radius: 0.3 }),
        })
        .id();

    assert!(
        !add_mod(&mut world, sculpture, ph2d_field::UnaryKind::Shell),
        "uma escultura não aceita modificadores — a recusa é da porta, não do validador"
    );
    assert!(
        world.get::<FieldMods>(sculpture).is_none(),
        "…e uma recusa não pode deixar rasto: um componente escrito e recusado é o documento \
         inválido que a wave existe para impedir"
    );
    assert!(
        add_mod(&mut world, leaf, ph2d_field::UnaryKind::Shell),
        "uma forma normal TEM de continuar a aceitar — senão o gate acima passa com tudo partido"
    );
    assert_eq!(mods_of(&world, leaf).len(), 1);
}

/// ⭐ **O OLHO da Hierarquia apaga o nó da peça** — e a subárvore com ele (W28).
///
/// ⚠️ **O defeito era um controle mudo**: a Hierarquia escreve [`ph2d_ecs::Visibility`] em qualquer
/// entidade, o ícone acendia, o componente entrava no mundo — e a peça na tela ficava igual, porque
/// o cozimento nunca perguntava. É a mesma família do modificador da W25 e da seleção da W27.
///
/// O gate mede as três metades: o nó some, o **grupo** leva os filhos consigo, e **a ausência do
/// componente é visível** (a lei do próprio `Visibility`, HR-5).
#[test]
fn the_hierarchy_eye_takes_the_node_out_of_the_piece() {
    let mut world = bevy_ecs::world::World::new();
    let ball = |x: f32| Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
        mods: Vec::new(),
        verb: None,
    };
    let union = |children: Vec<NodeId>| Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Combine {
            op: ph2d_field::Op::Union(Blend::Sharp),
            children,
        },
        mods: Vec::new(),
        verb: None,
    };
    // ⚠️ **Um grupo ANINHADO, e não uma união rasa** — é ele que separa *recusar na descida* de
    // *recusar na subida*: com a pergunta na subida, esconder o grupo deixaria os filhos dele
    // emitidos na arena (órfãos) e a contagem daria 4 em vez de 2. Uma fixture rasa não contém o
    // fenómeno e passaria com as duas implementações.
    let doc = FieldDoc::new(
        vec![
            ball(-0.6),
            ball(0.2),
            ball(0.6),
            union(vec![NodeId(1), NodeId(2)]),
            union(vec![NodeId(0), NodeId(3)]),
        ],
        NodeId(4),
    )
    .expect("três esferas em dois níveis");
    let root = spawn_doc(&mut world, &doc, "peça");
    let count =
        |w: &bevy_ecs::world::World| cook(w, root).map(|r| r.expect("válida").nodes().len());
    assert_eq!(
        count(&world),
        Some(5),
        "sem esconder nada, a peça é inteira"
    );

    let rows: Vec<bevy_ecs::entity::Entity> =
        walk(&world, root).into_iter().map(|(e, _)| e).collect();
    let leaf = rows
        .iter()
        .copied()
        .find(|e| {
            matches!(
                world.get::<FieldNode>(*e).map(|n| &n.shape),
                Some(NodeShape::Leaf(_))
            )
        })
        .expect("há folhas");
    let group = rows
        .iter()
        .copied()
        .find(|e| {
            *e != root
                && matches!(
                    world.get::<FieldNode>(*e).map(|n| &n.shape),
                    Some(NodeShape::Combine(_))
                )
        })
        .expect("há um grupo aninhado");

    world
        .entity_mut(leaf)
        .insert(ph2d_ecs::Visibility::hidden());
    assert_eq!(
        count(&world),
        Some(4),
        "o nó escondido não pode entrar no documento — era isto que o olho não fazia"
    );

    // ⚠️ E a ausência é VISÍVEL: repor o componente a `visible` traz o nó de volta.
    world
        .entity_mut(leaf)
        .insert(ph2d_ecs::Visibility::visible());
    assert_eq!(
        count(&world),
        Some(5),
        "`Visibility::visible` conta como visível — a lei do componente é a ausência ser visível"
    );

    // ⭐ Esconder o GRUPO leva os filhos consigo: a travessia nem desce.
    world
        .entity_mut(group)
        .insert(ph2d_ecs::Visibility::hidden());
    assert_eq!(
        count(&world),
        Some(2),
        "esconder um grupo tem de levar a subárvore inteira — sobram a esfera de fora e a raiz"
    );

    // …e esconder a RAIZ esvazia a peça, que é `None` e não um erro.
    world
        .entity_mut(root)
        .insert(ph2d_ecs::Visibility::hidden());
    assert!(cook(&world, root).is_none(), "peça vazia é `None`");
}

/// Cena vazia não tem campo — e um documento vazio inventado aqui seria uma forma que ninguém pediu.
#[test]
fn an_empty_scene_has_no_field() {
    let empty: Vec<(&str, FieldDoc)> = Vec::new();
    assert!(scene_field(empty, Blend::Sharp).is_none());
}

/// Os componentes sobrevivem à ida e volta que o `WorldSnapshot` faz — se um deles não serializar,
/// o objeto some ao desfazer, e o sintoma não é um erro: é o desaparecimento.
#[test]
fn the_components_round_trip_through_serde() {
    let node = FieldNode {
        shape: NodeShape::Leaf(Primitive::Sphere { radius: 0.42 }),
    };
    let bytes = postcard::to_allocvec(&node).expect("serializa");
    assert_eq!(
        postcard::from_bytes::<FieldNode>(&bytes).expect("desserializa"),
        node
    );

    let pose = FieldPose {
        xform: Xform::at(0.1, -0.2, 0.3),
    };
    let bytes = postcard::to_allocvec(&pose).expect("serializa");
    assert_eq!(
        postcard::from_bytes::<FieldPose>(&bytes).expect("desserializa"),
        pose
    );

    let bytes = postcard::to_allocvec(&FieldObject).expect("serializa");
    assert_eq!(
        postcard::from_bytes::<FieldObject>(&bytes).expect("desserializa"),
        FieldObject
    );
}

/// ⚠️ **O nome canônico é PARTE DO FORMATO SALVO**, não um rótulo: o id do componente é derivado
/// dele (`stable_type_id`), então renomeá-lo faz todo projeto já salvo perder o objeto — em
/// silêncio, porque um id desconhecido é simplesmente ignorado ao carregar.
#[test]
fn the_canonical_name_is_pinned_because_the_saved_id_derives_from_it() {
    let mut reg = ComponentRegistry::new();
    register_field_components(&mut reg);
    for name in [
        "ph2d::field::FieldObject",
        "ph2d::field::FieldNode",
        "ph2d::field::FieldPose",
    ] {
        assert!(
            reg.get_by_name(name).is_some(),
            "o nome canônico `{name}` mudou — todo projeto salvo perde o objeto ao abrir"
        );
    }
}

/// ⚠️ **A generalização da W38 é INERTE na raiz de verdade.**
///
/// O `cook` passou a compor no nó de topo a pose da cadeia **acima** dele — a metade que faz
/// *isolar* um nó dentro de um grupo movido não atirar a peça para a origem. A raiz da peça não tem
/// nada acima, então o documento tem de sair **byte-a-byte** o mesmo de antes.
///
/// *Uma generalização que muda o caso que já funcionava não é uma generalização, é uma regressão.*
#[test]
fn cooking_from_the_real_root_is_unchanged_by_the_chain() {
    let mut world = bevy_ecs::world::World::new();
    // Uma peça com pose NÃO-identidade no topo e nos filhos: com identidades em todo o lado, uma
    // composição errada sairia igual e o gate não provaria nada.
    let doc = FieldDoc::new(
        vec![
            Node {
                xform: Xform::at(0.3, -0.2, 0.1),
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
                mods: Vec::new(),
                verb: None,
            },
            Node {
                xform: Xform::at(-0.4, 0.5, 0.0),
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.15 }),
                mods: Vec::new(),
                verb: None,
            },
            Node {
                xform: Xform::at(1.0, 2.0, -3.0),
                kind: NodeKind::Combine {
                    op: ph2d_field::Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("a peça");
    let root = crate::spawn_doc(&mut world, &doc, "peça");
    let cooked = cook(&world, root).expect("não vazia").expect("válida");
    assert_eq!(
        cooked, doc,
        "cozer a raiz verdadeira tem de devolver exactamente o documento que a semeou"
    );
}

/// ⭐ **ISOLAR É COZER A PARTIR DO NÓ — e a peça NÃO salta.**
///
/// ⚠️ Este gate é o que dá sentido à linha nova do `cook`. O nó isolado vive debaixo de um grupo
/// com pose própria; cozer a subárvore dele **sem** a cadeia acima daria a forma no sítio errado, e
/// da cadeira isso lê como *"isolar mexeu no meu modelo"*.
#[test]
fn isolating_a_node_keeps_it_where_it_was_in_the_world() {
    let mut world = bevy_ecs::world::World::new();
    let doc = FieldDoc::new(
        vec![
            Node {
                xform: Xform::at(0.25, 0.0, 0.0),
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
                mods: Vec::new(),
                verb: None,
            },
            Node {
                // ⭐ O GRUPO está deslocado: é o que a cadeia tem de trazer.
                xform: Xform::at(1.5, -0.5, 0.75),
                kind: NodeKind::Combine {
                    op: ph2d_field::Op::Union(Blend::Sharp),
                    children: vec![NodeId(0)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(1),
    )
    .expect("a peça");
    let root = crate::spawn_doc(&mut world, &doc, "peça");
    let leaf = *world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("o grupo tem um filho")
        .iter()
        .next()
        .expect("a folha");

    let isolated = cook(&world, leaf).expect("não vazia").expect("válida");
    let top = &isolated.nodes()[isolated.root().0 as usize];
    let expected = crate::world_xform(&world, leaf);
    for k in 0..3 {
        assert!(
            (top.xform.translation[k] - expected.translation[k]).abs() < 1e-6,
            "o eixo {k} saiu em {} e o nó está em {} no mundo — isolar teleportou a peça",
            top.xform.translation[k],
            expected.translation[k]
        );
    }
    // E o controle: sem a cadeia, o topo sairia na pose LOCAL — que aqui é outra.
    assert!(
        (expected.translation[0] - 0.25).abs() > 1e-3,
        "a fixture só prova alguma coisa se a pose local e a de mundo DIFERIREM"
    );
}

/// ⭐⭐ **UMA CÓPIA LEVA TODOS OS COMPONENTES OPCIONAIS DO NÓ** (W55) — e a primeira metade deste
/// gate é uma **censura**, não uma asserção sobre a cópia.
///
/// # O defeito que ele fecha, e o que estava errado
///
/// O `copy_subtree` nascia com uma lista escrita à mão — `Name`, [`FieldNode`], [`FieldPose`] — e o
/// [`FieldMods`] **nunca lá esteve** (desde a W26). Duplicar um cilindro oco devolvia um maciço, sem
/// erro e sem aviso. *Uma lista escrita à mão ao lado do registo de componentes são duas respostas à
/// pergunta «o que é um nó».*
///
/// ⚠️ **A contagem é o que impede a próxima.** O `bevy_ecs` não copia componentes sem reflexão, então
/// a enumeração no `copy_optional` é inevitável — o que se pode fazer é prendê-la ao registo. Quem
/// acrescentar o sexto componente do módulo vê **este** gate vermelho, com o que fazer escrito na
/// mensagem, antes de o artista ver a cópia mutilada.
#[test]
fn a_duplicate_carries_every_optional_component_of_a_node() {
    let mut reg = ComponentRegistry::new();
    crate::register_field_components(&mut reg);
    assert_eq!(
        reg.len(),
        6,
        "o módulo passou a ter outro componente — ensine-o ao `copy_optional` (a cópia da \
         Hierarquia) e acrescente-o à fixture abaixo, senão duplicar um nó perde-o em silêncio"
    );

    let mut world = bevy_ecs::world::World::new();
    // ⚠️ Uma peça com **grupo**: `duplicate` recusa a raiz (ela *é* a peça), então a fixture só prova
    // alguma coisa se houver um nó com pai.
    let grouped = FieldDoc::new(
        vec![
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.3 }),
                mods: Vec::new(),
                verb: None,
            },
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: ph2d_field::Op::Union(Blend::Sharp),
                    children: vec![NodeId(0)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(1),
    )
    .expect("a peça");
    let root = crate::spawn_doc(&mut world, &grouped, "peça");
    let leaf = *world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("a raiz tem filho")
        .iter()
        .next()
        .expect("a folha");
    // Os dois opcionais, no mesmo nó: a cópia tem de trazer os dois.
    assert!(crate::add_mod(
        &mut world,
        leaf,
        ph2d_field::UnaryKind::Shell
    ));
    world
        .entity_mut(leaf)
        .insert(crate::FieldProfileSource { path: 77, level: 3 });
    // ⭐⭐ **E o VERBO** (W97): duplicar um furo tem de dar outro furo. Sem a linha no
    // `copy_optional`, a cópia caía na herança do pai e passava a SOMAR — sem erro e sem aviso.
    crate::set_verb(
        &mut world,
        leaf,
        Some(ph2d_field::Op::Difference(Blend::Exact { radius: 0.04 })),
    )
    .expect("é um nó");

    let copy = crate::duplicate(&mut world, leaf, [0.5, 0.0, 0.0]).expect("duplicou");
    assert_eq!(
        crate::mods_of(&world, copy).len(),
        1,
        "a cópia de um nó OCO tinha de sair oca — a lista escrita à mão deixava-a maciça"
    );
    assert_eq!(
        world.get::<crate::FieldProfileSource>(copy).copied(),
        Some(crate::FieldProfileSource { path: 77, level: 3 }),
        "e a cópia continua ligada ao mesmo desenho, no mesmo nível"
    );
    assert_eq!(
        crate::verb_of(&world, copy),
        Some(ph2d_field::Op::Difference(Blend::Exact { radius: 0.04 })),
        "a cópia de um FURO tinha de sair furo — sem o verbo ela cai na herança e passa a somar"
    );
}
