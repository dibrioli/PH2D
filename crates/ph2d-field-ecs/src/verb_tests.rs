//! ⭐⭐⭐ **O VERBO ATRAVESSA A CENA** (W97) — os gates da metade que só o mundo sabe responder.
//!
//! A lei da dobra vive na `ph2d-field-eval`; aqui mede-se o que ela **recebe**: que a ausência do
//! componente atravesse como ausência, que a presença chegue ao documento, e que a *base* seja a
//! primeira forma que **contribui** — que não é a primeira da lista.

use super::*;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use ph2d_field::{Blend, Node, NodeId, NodeKind, Op, Primitive, Xform};

/// Uma peça com uma união e três esferas por baixo dela.
fn piece() -> (
    World,
    bevy_ecs::entity::Entity,
    Vec<bevy_ecs::entity::Entity>,
) {
    let ball = |r: f32| {
        Node::new(
            Xform::IDENTITY,
            NodeKind::Leaf(Primitive::Sphere { radius: r }),
        )
    };
    let doc = FieldDoc::new(
        vec![
            ball(0.3),
            ball(0.4),
            ball(0.5),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1), NodeId(2)],
                },
            ),
        ],
        NodeId(3),
    )
    .expect("peça");
    let mut world = World::new();
    let root = crate::spawn_doc(&mut world, &doc, "peça");
    let kids: Vec<_> = world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    assert_eq!(kids.len(), 3, "a fixtura tem de ter três formas");
    (world, root, kids)
}

/// ⭐⭐ **Sem componente, o documento sai com `None`** — e é isso que faz *ausência = herança*
/// atravessar sem se traduzir a meio.
#[test]
fn silence_in_the_world_is_silence_in_the_document() {
    let (world, root, _) = piece();
    let doc = crate::cook(&world, root).expect("peça").expect("válida");
    assert!(
        doc.nodes().iter().all(|n| n.verb.is_none()),
        "um nó sem `FieldVerb` chegou ao documento com verbo"
    );
}

/// ⭐⭐ **E o verbo escrito no mundo CHEGA ao documento**, no nó certo.
///
/// ⚠️ O gate afirma **qual** nó o recebeu, e não só que «algum» o tem: a travessia é pós-ordem e um
/// erro de índice ali poria o verbo na forma errada — a peça mudaria e nada apontaria a causa.
#[test]
fn a_verb_written_in_the_world_reaches_the_document() {
    let (mut world, root, kids) = piece();
    let corte = Op::Difference(Blend::Exact { radius: 0.07 });
    crate::set_verb(&mut world, kids[2], Some(corte)).expect("é um nó");

    let doc = crate::cook(&world, root).expect("peça").expect("válida");
    let com_verbo: Vec<_> = doc
        .nodes()
        .iter()
        .filter(|n| n.verb.is_some())
        .map(|n| (n.kind.shape(), n.verb))
        .collect();
    assert_eq!(com_verbo.len(), 1, "o verbo espalhou-se por mais de um nó");
    assert_eq!(com_verbo[0].1, Some(corte), "o verbo chegou alterado");
    assert!(
        matches!(
            &com_verbo[0].0,
            ph2d_field::NodeShape::Leaf(Primitive::Sphere { radius }) if (*radius - 0.5).abs() < 1e-6
        ),
        "o verbo pousou noutra forma que não a terceira"
    );
}

/// ⭐ **`None` devolve a forma à herança, e o componente SAI.**
///
/// ⚠️ Um valor guardado a fingir de ausente deixaria o nó a discordar do pai em silêncio no dia em
/// que o padrão do grupo mudasse — é por isso que o gate mede o componente, e não só o documento.
#[test]
fn asking_to_inherit_removes_the_component() {
    let (mut world, root, kids) = piece();
    crate::set_verb(&mut world, kids[1], Some(Op::Union(Blend::Sharp))).expect("é um nó");
    assert!(world.get::<FieldVerb>(kids[1]).is_some());

    crate::set_verb(&mut world, kids[1], None).expect("é um nó");
    assert!(
        world.get::<FieldVerb>(kids[1]).is_none(),
        "o componente ficou lá a fingir de ausente"
    );
    let doc = crate::cook(&world, root).expect("peça").expect("válida");
    assert!(doc.nodes().iter().all(|n| n.verb.is_none()));
}

/// ⭐⭐⭐ **A BASE é a primeira que CONTRIBUI — esconder a primeira PROMOVE a segunda.**
///
/// ⚠️ É a lei que uma segunda cópia da regra no painel quebraria em silêncio: a Hierarquia diria
/// `BSE` numa linha escondida e `SUB` na que de facto semeia o acumulado. A resposta sai do
/// [`crate::contributes`], que é a **mesma** função que o cozimento usa.
#[test]
fn the_base_is_the_first_that_contributes_not_the_first_in_the_list() {
    let (mut world, _, kids) = piece();
    assert_eq!(
        crate::verb_role(&world, kids[0]),
        Some(VerbRole::Base),
        "a primeira forma é a base"
    );
    assert_eq!(
        crate::verb_role(&world, kids[1]),
        Some(VerbRole::Inherited(Op::Union(Blend::Sharp))),
        "quem não se pronuncia herda o verbo do pai"
    );

    world
        .entity_mut(kids[0])
        .insert(ph2d_ecs::Visibility { hidden: true });
    assert_eq!(
        crate::verb_role(&world, kids[1]),
        Some(VerbRole::Base),
        "esconder a primeira tinha de promover a segunda a base"
    );
    assert_eq!(
        crate::verb_role(&world, kids[0]),
        None,
        "uma forma escondida não participa de receita nenhuma"
    );
}

/// ⭐ **Um GRUPO VAZIO não é a base** — ele não rende nada ao documento, e o cozimento nem o emite.
///
/// ⚠️ Sem isto, acrescentar um grupo vazio no topo da lista faria a Hierarquia selar `BSE` nele e a
/// forma que de facto semeia o acumulado aparecer a subtrair.
#[test]
fn an_empty_group_is_not_the_base() {
    let (mut world, root, kids) = piece();
    let vazio = world
        .spawn((
            ph2d_ecs::Name::new("Union"),
            FieldNode {
                shape: ph2d_field::NodeShape::Combine(Op::Union(Blend::Sharp)),
            },
            FieldPose::default(),
        ))
        .id();
    world.entity_mut(root).add_child(vazio);
    // ⚠️ E ele vai para o PRINCÍPIO da lista, que é o único sítio onde a confusão é observável.
    let ordem: Vec<_> = std::iter::once(vazio).chain(kids.iter().copied()).collect();
    for e in &ordem {
        world.entity_mut(*e).remove::<ChildOf>();
        world.entity_mut(*e).insert(ChildOf(root));
    }

    assert_eq!(
        crate::verb_role(&world, vazio),
        None,
        "um grupo vazio não participa"
    );
    assert_eq!(
        crate::verb_role(&world, kids[0]),
        Some(VerbRole::Base),
        "a base continua a ser a primeira forma que rende geometria"
    );
}

/// ⭐ **A raiz da peça não tem verbo** — não há nada acima dela com que dobrar.
#[test]
fn the_root_has_no_verb_to_choose() {
    let (world, root, _) = piece();
    assert_eq!(crate::verb_role(&world, root), None);
}

/// ⭐⭐ **A união de CENAS não herda o verbo de uma peça.**
///
/// ⚠️ Um verbo autorado dentro de uma peça fala dos **irmãos dela**; adoptado por
/// [`FieldDoc::union_all`] passaria a falar das **outras peças** — uma peça inteira a subtrair-se de
/// outra sem ninguém o ter pedido. A porta chama-se `union_all`, e a união é o contrato.
#[test]
fn joining_scenes_never_inherits_a_pieces_verb() {
    let uma = |r: f32| {
        let mut n = Node::new(
            Xform::IDENTITY,
            NodeKind::Leaf(Primitive::Sphere { radius: r }),
        );
        n.verb = Some(Op::Difference(Blend::Sharp));
        FieldDoc::new(vec![n], NodeId(0)).expect("esfera")
    };
    let cena = FieldDoc::union_all(&[uma(0.3), uma(0.4)], Blend::Sharp)
        .expect("duas peças")
        .expect("válida");
    assert!(
        cena.nodes().iter().all(|n| n.verb.is_none()),
        "uma peça adoptada pela cena trouxe o verbo dela"
    );
}

// ───────── W98: o RAIO DA JUNÇÃO por forma ─────────

/// Uma peça com uma união de filete `0,08` e duas **caixas** — a forma que tem filete próprio, que é
/// o que torna a colisão de nomes observável.
fn two_boxes() -> (
    World,
    bevy_ecs::entity::Entity,
    Vec<bevy_ecs::entity::Entity>,
) {
    let caixa = || {
        Node::new(
            Xform::IDENTITY,
            NodeKind::Leaf(Primitive::Box {
                half: [0.4; 3],
                round: 0.05,
            }),
        )
    };
    let doc = FieldDoc::new(
        vec![
            caixa(),
            caixa(),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Exact { radius: 0.08 }),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("peça");
    let mut world = World::new();
    let root = crate::spawn_doc(&mut world, &doc, "peça");
    let kids: Vec<_> = world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    (world, root, kids)
}

fn chaves(world: &World, e: bevy_ecs::entity::Entity) -> Vec<&'static str> {
    crate::params_of(world, e)
        .into_iter()
        .map(|(_, d)| d.key)
        .collect()
}

/// ⭐⭐⭐ **UMA FORMA TEM DOIS RAIOS, e eles têm nomes DIFERENTES.**
///
/// ⚠️ É a colisão que o verbo por forma criou: uma caixa arredondada que se junta ao resto mostra o
/// filete **das arestas dela** e o raio **do encontro** na mesma coluna. Dois rótulos iguais são dois
/// controles que o artista não sabe separar — e o painel deriva o rótulo da chave, então a chave é
/// onde isto se resolve.
#[test]
fn a_shape_shows_its_own_fillet_and_its_joint_under_different_names() {
    let (world, _, kids) = two_boxes();
    let ks = chaves(&world, kids[1]);
    assert!(
        ks.contains(&"field.dim.round"),
        "sumiu o filete das arestas da própria caixa: {ks:?}"
    );
    assert!(
        ks.contains(&"field.dim.joint"),
        "a forma não oferece o raio da junção dela: {ks:?}"
    );
}

/// ⭐⭐ **A BASE não tem raio de junção** — ela semeia o acumulado e não se junta a nada.
///
/// ⚠️ E a **raiz** também não. Oferecer a linha ali seria um controle a escrever num verbo que
/// ninguém lê — a lei da W34, aqui aplicada a uma linha de número em vez de a uma fileira de chips.
///
/// # ⚠️ A pergunta é sobre o `Param`, e NÃO sobre o rótulo
///
/// A 1.ª redacção deste gate procurava a chave `field.dim.joint`, e reprovou na raiz — **com razão**:
/// o **grupo** usa essa chave de propósito, porque o raio dele *é* o raio de junção **padrão**, o que
/// as formas caladas usam (*uma grandeza, uma palavra*). O que ele não tem é o [`Param::Joint`], que
/// é *«o meu próprio encontro»*. ⇒ **Duas coisas diferentes com o mesmo nome na tela, e é correcto:**
/// o que as separa é quem escreve onde, não como se chamam.
#[test]
fn the_base_and_the_root_have_no_joint_to_set() {
    let (world, root, kids) = two_boxes();
    let proprio = |e| {
        crate::params_of(&world, e)
            .into_iter()
            .any(|(p, _)| p == ph2d_field::Param::Joint)
    };
    assert!(
        !proprio(kids[0]),
        "a BASE recebeu um raio de junção próprio"
    );
    assert!(!proprio(root), "a RAIZ recebeu um raio de junção próprio");
    // ⭐ E o controlo do rótulo partilhado: a raiz **mostra** um raio de junção (o padrão dela).
    assert!(
        chaves(&world, root).contains(&"field.dim.joint"),
        "o grupo tinha de mostrar o raio de junção PADRÃO — é o que as formas caladas usam"
    );
}

/// ⭐⭐ **Quem HERDA vê o valor herdado** — e não um zero, nem uma linha ausente.
///
/// ⚠️ *«Quero a boca deste furo mais macia»* não pode exigir que o artista entenda o modelo do verbo
/// primeiro. A linha mostra o que **de facto acontece** àquela forma agora.
#[test]
fn an_inheriting_shape_shows_the_radius_it_actually_uses() {
    let (world, _, kids) = two_boxes();
    let joint = crate::params_of(&world, kids[1])
        .into_iter()
        .find(|(p, _)| *p == ph2d_field::Param::Joint)
        .expect("a linha existe");
    assert!(
        (joint.1.value - 0.08).abs() < 1e-6,
        "a linha tinha de mostrar o 0,08 do grupo, e mostrou {}",
        joint.1.value
    );
}

/// ⭐⭐⭐ **Escrever o raio MATERIALIZA o verbo — e NÃO toca no irmão que herda.**
///
/// ⚠️ É a razão de existir da wave inteira, num gate: sem a materialização, arrastar a linha de uma
/// forma calada escreveria no **grupo**, e as outras caladas mudariam com ela — que é exactamente o
/// defeito que o verbo por forma cura. ⛔ Um gate que só medisse *«o valor mudou»* passaria com essa
/// escrita no grupo; é o **irmão** que separa as duas hipóteses.
#[test]
fn writing_the_joint_speaks_for_this_shape_alone() {
    let (mut world, root, kids) = two_boxes();
    // Um terceiro, também calado — o controlo.
    let terceiro = crate::add_leaf(
        &mut world,
        root,
        Primitive::Sphere { radius: 0.3 },
        [0.0, 0.0, 0.0],
    )
    .expect("nasce");

    crate::set_param(&mut world, kids[1], ph2d_field::Param::Joint, 0.2).expect("escreve");

    assert_eq!(
        crate::verb_of(&world, kids[1]),
        Some(Op::Union(Blend::Exact { radius: 0.2 })),
        "a forma tinha de passar a ter o verbo POR ESCRITO, com o verbo que já usava"
    );
    assert_eq!(
        crate::verb_role(&world, terceiro),
        Some(VerbRole::Inherited(Op::Union(Blend::Exact {
            radius: 0.08
        }))),
        "o irmão CALADO mudou junto — a escrita foi para o grupo"
    );
    // ⚠️ E o grupo ficou como estava: o padrão de quem não se pronunciou não se mexe.
    let grupo = crate::params_of(&world, root)
        .into_iter()
        .find(|(p, _)| *p == ph2d_field::Param::Dim(0))
        .expect("o grupo tem o raio padrão");
    assert!(
        (grupo.1.value - 0.08).abs() < 1e-6,
        "o raio PADRÃO do grupo mudou: {}",
        grupo.1.value
    );
}

/// ⭐ **Zero é a aresta VIVA, e negativo é recusado** — a mesma lei do filete de uma forma.
///
/// ⚠️ Um zero recusado obrigaria o artista a apagar o verbo para conseguir uma quina, o que é outro
/// gesto e noutro sítio.
#[test]
fn a_zero_joint_is_a_live_edge_and_a_negative_one_is_refused() {
    let (mut world, _, kids) = two_boxes();
    crate::set_param(&mut world, kids[1], ph2d_field::Param::Joint, 0.0).expect("zero entra");
    assert_eq!(
        crate::verb_of(&world, kids[1]),
        Some(Op::Union(Blend::Sharp)),
        "zero tinha de dar aresta viva"
    );
    assert!(
        crate::set_param(&mut world, kids[1], ph2d_field::Param::Joint, -0.1).is_err(),
        "um raio negativo não existe"
    );
    assert_eq!(
        crate::verb_of(&world, kids[1]),
        Some(Op::Union(Blend::Sharp)),
        "e a recusa deixa o nó COMO ESTAVA — a invariante do módulo"
    );
}

/// ⭐ **O carácter da mistura SOBREVIVE ao raio novo** — quem era orgânica continua orgânica.
///
/// ⚠️ A lei é copiada do `set_shape_radius` de propósito: duas leis para o mesmo gesto divergem na
/// primeira wave que corrija uma delas. É o mesmo motivo por que o `set_op` preserva a mistura.///
/// ⛔⛔ **A primeira versão deste gate provava um carácter só, e foi por isso que um defeito passou:**
/// o `Param::Joint` tinha uma **cópia** da escada que não conhecia o chanfro, e mudar o raio de uma
/// junta chanfrada transformava-a em filete **em silêncio**. O comentário ao lado dela dizia que a
/// porta era única, e era falso. ⇒ *quem varre um carácter varre todos*, e a prova de mutação foi
/// quem o disse (o mutante de `with_amount` sobreviveu por não haver ninguém a chamá-la).
#[test]
fn the_character_of_the_blend_survives_a_new_radius() {
    let (mut world, _, kids) = two_boxes();
    // ⚠️ **Os DOIS caracteres que sobrevivem**, e não só o orgânico. O `Exact` não entra porque é
    // ele o destino de um erro — um gate que o incluísse passaria com a escada apagada.
    for (nome, antes, depois) in [
        (
            "orgânico",
            Blend::Organic { radius: 0.1 },
            Blend::Organic { radius: 0.3 },
        ),
        (
            "chanfro",
            Blend::Chamfer { radius: 0.1 },
            Blend::Chamfer { radius: 0.3 },
        ),
    ] {
        crate::set_verb(&mut world, kids[1], Some(Op::Difference(antes))).expect("é um nó");
        crate::set_param(&mut world, kids[1], ph2d_field::Param::Joint, 0.3).expect("escreve");
        assert_eq!(
            crate::verb_of(&world, kids[1]),
            Some(Op::Difference(depois)),
            "o carácter {nome} e o VERBO tinham de sobreviver ao raio novo"
        );
    }
}

// ───────── W99: o CARÁTER da mistura ─────────

/// ⭐⭐⭐ **TROCAR O CARÁTER NÃO MEXE NO NÚMERO** — nem numa forma, nem num grupo.
///
/// ⚠️ Quem carrega no chip escolheu a **forma** da junta; ver o raio saltar junto seria o painel a
/// decidir por ele. E o contrário — trocar o raio e perder o carácter — é o mesmo defeito ao
/// contrário, e tem gate próprio (`the_character_of_the_blend_survives_a_new_radius`).
#[test]
fn switching_character_keeps_the_number() {
    use ph2d_field::Character;
    let (mut world, root, kids) = two_boxes();

    // ── Numa FORMA: o carácter materializa o verbo, como o raio ──
    crate::set_param(&mut world, kids[1], ph2d_field::Param::Joint, 0.17).expect("escreve");
    crate::set_character(&mut world, kids[1], Character::Chamfer).expect("troca");
    assert_eq!(
        crate::verb_of(&world, kids[1]),
        Some(Op::Union(Blend::Chamfer { radius: 0.17 })),
        "o número tinha de sobreviver à troca de carácter"
    );
    assert_eq!(
        crate::character_of(&world, kids[1]),
        Some(Character::Chamfer),
        "e a leitura tem de devolver o que a escrita pôs"
    );

    // ── Num GRUPO: o raio dele é o padrão dos filhos calados, e também não se mexe ──
    crate::set_character(&mut world, root, Character::Organic).expect("troca");
    let grupo = crate::params_of(&world, root)
        .into_iter()
        .find(|(p, _)| *p == ph2d_field::Param::Dim(0))
        .expect("o grupo tem raio");
    assert!(
        (grupo.1.value - 0.08).abs() < 1e-6,
        "o raio PADRÃO do grupo mudou ao trocar de carácter: {}",
        grupo.1.value
    );
    assert_eq!(crate::character_of(&world, root), Some(Character::Organic));
}

/// ⭐⭐ **Trocar o carácter de UMA forma não toca no irmão que herda** — a mesma lei do raio.
///
/// ⛔ Sem a materialização, o chip escreveria no grupo e mudaria as outras caladas com ele — que é o
/// defeito que o verbo por forma existe para curar, agora pela segunda porta.
#[test]
fn switching_character_speaks_for_this_shape_alone() {
    use ph2d_field::Character;
    let (mut world, root, kids) = two_boxes();
    let terceiro = crate::add_leaf(
        &mut world,
        root,
        Primitive::Sphere { radius: 0.3 },
        [0.0, 0.0, 0.0],
    )
    .expect("nasce");

    crate::set_character(&mut world, kids[1], Character::Chamfer).expect("troca");

    assert_eq!(
        crate::verb_of(&world, terceiro),
        None,
        "o irmão CALADO ganhou um verbo — a escrita foi para o grupo"
    );
    assert_eq!(
        crate::character_of(&world, terceiro),
        Some(Character::Fillet),
        "e ele continua a ler o carácter herdado do grupo"
    );
}

/// ⭐ **A BASE e a RAIZ não têm carácter a escolher** — a mesma recusa do raio de junção.
///
/// ⚠️ A **raiz** deste teste é uma operação, então ela **tem** (o filete dela é o padrão dos
/// filhos); quem não tem é a **base**, que não se junta a nada.
#[test]
fn the_base_has_no_character_to_choose() {
    use ph2d_field::Character;
    let (mut world, _, kids) = two_boxes();
    assert_eq!(
        crate::character_of(&world, kids[0]),
        None,
        "a BASE não tem junta"
    );
    assert!(
        crate::set_character(&mut world, kids[0], Character::Chamfer).is_err(),
        "escrever o carácter da base tinha de ser recusado"
    );
}
