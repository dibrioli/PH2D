//! ⭐⭐ **O RAIO DA JUNÇÃO por forma** (W98) — a metade do [`crate::verb_tests`] que fala de
//! **junções**, e não de verbos.
//!
//! ⚠️ **Por que ela vive num ficheiro próprio** (2026-09-10): o `verb_tests.rs` passou o tecto de
//! `700` LOC, e as duas metades já eram duas — *que VERBO uma forma tem* (herança, base, raiz) e
//! *que RAIO e que CARÁCTER a junção dela usa*. Elas nem partilham fixtura: uma monta `piece()`,
//! a outra `two_boxes()`. ⛔ A cura de um tecto é **cortar por responsabilidade**, nunca subir uma
//! tolerância.

use super::*;
use bevy_ecs::world::World;
use ph2d_field::{Blend, Node, NodeId, NodeKind, Op, Primitive, Xform};

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
                chamfer: 0.0,
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

// ─────────────────── W145: o SEGUNDO número de uma junta ───────────────────

/// ⭐⭐⭐ **A LINHA DO SEGUNDO NÚMERO SÓ EXISTE ONDE A JUNTA TEM UM** — a lei da W34, aplicada ao
/// controlo que esta wave trouxe.
///
/// ⚠️ **Os dois lados, e é por isso que o gate afirma duas coisas:** um filete que oferecesse uma
/// «meia-largura» seria um controle a escrever onde nada lê, e um sulco sem ela seria uma feição
/// inalcançável — *os dois defeitos leem-se igual numa foto do painel*.
#[test]
fn only_the_junctions_with_a_second_number_offer_the_row() {
    use ph2d_field::{Blend, Character, Param};
    let tem_seam = |world: &bevy_ecs::world::World, e| {
        crate::params_of(world, e)
            .into_iter()
            .any(|(p, _)| matches!(p, Param::Seam(_)))
    };
    for (c, esperado) in [
        (Character::Fillet, false),
        (Character::Organic, false),
        (Character::Soft, false),
        (Character::Bead, false),
        // ⛔ **O sulco PERDEU o segundo número** no report de 09/09: ele localiza-se pelo vinco, e um
        // tubo à volta de uma curva tem UM raio. *O número que saiu nunca descreveu nada.*
        (Character::Groove, false),
        // ⭐ O chanfro oferece-a com `1,0` dentro — sem isso o desequilíbrio nasceria inalcançável.
        (Character::Chamfer, true),
        (Character::Ridge, true),
    ] {
        let (mut world, _, kids) = two_boxes();
        crate::set_character(&mut world, kids[1], c).expect("troca");
        assert_eq!(
            tem_seam(&world, kids[1]),
            esperado,
            "o carácter {c:?} {} a linha do segundo número",
            if esperado {
                "devia oferecer"
            } else {
                "não devia oferecer"
            }
        );
    }
    // ⛔ **O CONTROLO da própria régua:** o valor tem de ser o que a junta guarda, e não um zero
    // que passaria o teste de presença sem nada do outro lado.
    let (mut world, _, kids) = two_boxes();
    crate::set_character(&mut world, kids[1], Character::Ridge).expect("troca");
    let (_, d) = crate::params_of(&world, kids[1])
        .into_iter()
        .find(|(p, _)| matches!(p, Param::Seam(_)))
        .expect("a linha existe");
    let esperado = 0.08 * Blend::SEAM_WIDTH_RATIO;
    assert!(
        (d.value - esperado).abs() < 1e-6,
        "o friso nasceu com meia-largura {} e não com o {esperado} que o `SEAM_WIDTH_RATIO` manda",
        d.value
    );
}

/// ⭐⭐⭐ **ESCREVER O SEGUNDO NÚMERO CHEGA AO DOCUMENTO — e não apaga o primeiro.**
///
/// ⚠️ **A metade que quase ficou por gatear:** um `with_second` que reconstruísse a variante do zero
/// perderia o tamanho, e o sintoma seria um sulco que muda de profundidade quando o artista arrasta
/// a largura. Os dois números são lidos **depois** da escrita, no mesmo documento.
#[test]
fn writing_the_second_number_keeps_the_first() {
    use ph2d_field::{Blend, Character, Param};
    let (mut world, _, kids) = two_boxes();
    crate::set_character(&mut world, kids[1], Character::Ridge).expect("troca");
    crate::set_param(&mut world, kids[1], Param::Joint, 0.2).expect("a altura");
    crate::set_param(&mut world, kids[1], Param::Seam(1), 0.03).expect("a largura");
    let op = crate::verb_of(&world, kids[1]).expect("o verbo materializou-se");
    match op.blend() {
        Blend::Ridge { radius, width } => {
            assert!(
                (radius - 0.2).abs() < 1e-6,
                "a altura virou {radius} ao escrever a largura"
            );
            assert!((width - 0.03).abs() < 1e-6, "a largura não chegou: {width}");
        }
        outro => panic!("o carácter mudou sozinho para {outro:?}"),
    }
    // ⛔ **Zero é RECUSADO**, ao contrário do raio de junção: um canal sem largura é a feição a
    // desaparecer com o chip aceso.
    assert!(
        crate::set_param(&mut world, kids[1], Param::Seam(1), 0.0).is_err(),
        "uma meia-largura de zero entrou — o carácter fica aceso sobre uma feição que não existe"
    );
}

/// ⭐⭐ **O DESEQUILÍBRIO DO CHANFRO volta a ser um chanfro simples quando regressa a `1,0`.**
///
/// ⚠️ *Um valor tem uma representação.* Sem esta lei, dois documentos que a tela mostra iguais
/// teriam bytes diferentes — e o caminho rápido do chanfro simétrico deixaria de ser tomado por
/// quem lá voltou.
#[test]
fn a_bevel_that_returns_to_balance_is_a_chamfer_again() {
    use ph2d_field::{Blend, Character, Param};
    let (mut world, _, kids) = two_boxes();
    crate::set_character(&mut world, kids[1], Character::Chamfer).expect("troca");
    crate::set_param(&mut world, kids[1], Param::Seam(1), 2.5).expect("desequilibra");
    assert!(
        matches!(
            crate::verb_of(&world, kids[1]).map(ph2d_field::Op::blend),
            Some(Blend::Bevel { .. })
        ),
        "escrever o desequilíbrio não promoveu o chanfro"
    );
    // ⭐ E o chip continua a dizer **Chamfer** — a forma é a mesma, o número é que mudou.
    assert_eq!(
        crate::character_of(&world, kids[1]),
        Some(Character::Chamfer),
        "o chanfro desigual passou a ler-se como outro carácter"
    );
    crate::set_param(&mut world, kids[1], Param::Seam(1), 1.0).expect("reequilibra");
    assert!(
        matches!(
            crate::verb_of(&world, kids[1]).map(ph2d_field::Op::blend),
            Some(Blend::Chamfer { .. })
        ),
        "voltar a 1,0 deixou um `Bevel` com um dentro — dois bytes para a mesma peça"
    );
}

/// ⭐⭐⭐ **A FAIXA DE UMA JUNTA VEM DA PEÇA, E NÃO DO ENQUADRAMENTO** (report do Enio, 2026-09-09:
/// *«os sliders das joints vão de 0 a 16 quando só precisa de 0 a 1»*).
///
/// ⚠️ **O defeito não era um número errado — era o número CERTO no sítio errado.** A
/// [`ph2d_field::Span::Positive`] entrega o tecto à vista, e a vista mede a **cena inteira**: numa
/// fileira de seis peças o slider de um raio de junta abria `0..16`. O valor útil dele vive abaixo de
/// `0,1`, logo **todo o curso do dedo cabia num pixel**.
///
/// ⭐ E o número certo já existia desde a W10 — a [`crate::radius_bound`] devolve a menor peça sob o
/// nó — com uma nota no painel a dizer que trocá-lo era *«número do Enio, com a peça à frente»*.
/// *Uma nota que espera um veredito precisa de ser perguntada.*
#[test]
fn the_joint_slider_is_bounded_by_the_piece_and_not_by_the_view() {
    use ph2d_field::{Param, Span};
    let (world, root, kids) = two_boxes();
    // A menor peça da fixtura é uma caixa de meia-extensão `0,4`.
    const ESCALA: f32 = 0.4;
    for (quem, e) in [("a forma", kids[1]), ("o grupo", root)] {
        let linhas = crate::params_of(&world, e);
        let juntas: Vec<_> = linhas
            .iter()
            .filter(|(p, d)| {
                matches!(p, Param::Joint | Param::Seam(_)) || d.key == "field.dim.joint"
            })
            .collect();
        assert!(
            !juntas.is_empty(),
            "{quem} não ofereceu linha de junta nenhuma — a fixtura mudou"
        );
        for (p, d) in juntas {
            match d.span {
                Span::SoftFromZero(top) => assert!(
                    (top - ESCALA).abs() < 1e-6,
                    "{quem}, {p:?}: o curso do slider e' {top} e a menor peca mede {ESCALA} — o \
                     tecto voltou a vir de outro sitio"
                ),
                outro => panic!(
                    "{quem}, {p:?}: a faixa e' {outro:?} — uma junta com `Span::Positive` recebe o \
                     tecto da CENA, e o curso do dedo colapsa"
                ),
            }
        }
    }
}

/// ⛔ **O CONTROLO: o desequilíbrio do chanfro NÃO é fechado pela peça** — ele é uma **razão**, e a
/// escala de um sólido não limita um adimensional.
///
/// ⚠️ Sem este gate, a porta que fecha as faixas passaria a fechar tudo o que lhe chegasse, e o
/// desequilíbrio ficaria com um curso em unidades de comprimento — *o mesmo defeito do report, do
/// outro lado*.
#[test]
fn the_chamfer_bias_keeps_its_own_range() {
    use ph2d_field::{Character, Param, Span};
    let (mut world, _, kids) = two_boxes();
    crate::set_character(&mut world, kids[1], Character::Chamfer).expect("troca");
    crate::set_param(&mut world, kids[1], Param::Joint, 0.08).expect("o raio");
    let (_, d) = crate::params_of(&world, kids[1])
        .into_iter()
        .find(|(p, _)| matches!(p, Param::Seam(_)))
        .expect("a linha do desequilíbrio");
    // O curso é `escala / raio` — o ponto em que o corte do lado longo alcança a peça.
    let esperado = 0.4 / 0.08;
    match d.span {
        Span::SoftFromZero(top) => assert!(
            (top - esperado).abs() < 1e-5,
            "o curso do desequilibrio e' {top} e devia ser {esperado} (escala / raio) — ele foi \
             fechado em unidades de COMPRIMENTO, que e' o defeito do report do outro lado"
        ),
        outro => panic!("o desequilibrio ficou com a faixa {outro:?}"),
    }
}
