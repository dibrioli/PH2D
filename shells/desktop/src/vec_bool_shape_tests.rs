//! Os gates do **PAPEL de cada forma** dentro de uma booleana viva.
//!
//! O que só se pode afirmar aqui é a TRIAGEM — quem recebe o seletor e quem não recebe. O que
//! cada verbo desenha é do `bool_live`; o que a cadeia calcula é do motor.
//!
//! ⚠️ Os quatro `None` são gates separados de propósito, e não um só com quatro asserts: cada um é
//! uma decisão distinta (*é de UMA forma* · *a base não tem verbo* · *uma receita é da pilha* · *o
//! escritor reconfere*), e uma implementação que perdesse só uma delas tem de sangrar só um gate.

use super::*;
use ph2d_ecs::VecBoolGroup;
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecScene, VecXforms, rectangle};

/// Três retângulos sobrepostos num grupo booleano `op`, já cozidos. `ids[0]` é a BASE.
fn cooked(op: u8) -> (SimWorld, VecEntityMap, BoolLive, [VecPathId; 3]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let b = scene.push_path(rectangle([10.0, 0.0], [30.0, 20.0]));
    let c = scene.push_path(rectangle([15.0, 5.0], [25.0, 15.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b], map[&c]], "B".into())
            .unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });
    let mut live = LiveGeometry::new();
    let mut bl = BoolLive::default();
    bl.recook(&scene, &sim, &map, &VecXforms::default(), &mut live);
    // Pré-condição: o grupo cozinhou. Sem isto todos os gates leriam `None` e seriam verdes por
    // AUSÊNCIA — o modo de falha mais barato de um gate de triagem.
    assert!(
        bl.plan_containing(a).is_some(),
        "a fixture não cozinhou a booleana"
    );
    (sim, map, bl, [a, b, c])
}

/// **A BASE NÃO TEM VERBO** — o papel dela é próprio, e o seletor não é oferecido.
#[test]
fn the_base_has_no_verb_to_offer() {
    let (sim, map, bl, ids) = cooked(0);
    assert_eq!(role_of(&sim, &map, &bl, ids[0]), Some(BoolRole::Base));
    assert_eq!(shape_op_of_selection(&sim, &map, &bl, &[ids[0]]), None);
}

/// **HERANÇA: sem override, a forma mostra o verbo do GRUPO aceso.** É o que mantém os oito
/// botões do grupo vivos — eles continuam a ser o padrão de quem não se pronunciou.
#[test]
fn an_operand_inherits_the_groups_verb() {
    for op in 0..=3u8 {
        let (sim, map, bl, ids) = cooked(op);
        assert_eq!(
            shape_op_of_selection(&sim, &map, &bl, &[ids[2]]),
            Some(op),
            "op {op}: a forma devia herdar o verbo do grupo"
        );
    }
}

/// **E o override vence a herança** — a capacidade, do lado da projeção.
#[test]
fn an_operand_with_an_override_offers_its_own_verb() {
    let (mut sim, map, bl, ids) = cooked(0); // grupo em Union
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&ids[2]]))
        .insert(ph2d_ecs::VecBoolOp { op: 1 });
    assert_eq!(shape_op_of_selection(&sim, &map, &bl, &[ids[2]]), Some(1));
}

/// ⛔ **UMA RECEITA NO GRUPO NÃO OFERECE VERBO A FORMA NENHUMA**, e o papel diz isso em vez de
/// devolver um verbo que o motor vai ignorar.
///
/// `Trim`/`Crop`/`Merge`/`MinusBack` são afirmações sobre a pilha inteira. Oferecer o seletor
/// sobre elas seria pintar um controlo inerte como vivo — e o `bool_live` tem o gate gêmeo a
/// provar que o override de facto não muda nada ali.
#[test]
fn a_recipe_group_offers_no_per_shape_verb() {
    let (sim, map, bl, ids) = cooked(5); // Trim
    assert_eq!(role_of(&sim, &map, &bl, ids[2]), Some(BoolRole::Recipe));
    assert_eq!(shape_op_of_selection(&sim, &map, &bl, &[ids[2]]), None);
}

/// **O VERBO É DE UMA FORMA** — uma seleção múltipla não o oferece.
///
/// ⚠️ Aceso sobre um conjunto, o chip teria de responder por várias respostas ao mesmo tempo, e o
/// clique escreveria em todas sem o artista ter pedido.
#[test]
fn a_multi_selection_offers_nothing() {
    let (sim, map, bl, ids) = cooked(0);
    assert_eq!(
        shape_op_of_selection(&sim, &map, &bl, &[ids[1], ids[2]]),
        None
    );
    assert_eq!(shape_op_of_selection(&sim, &map, &bl, &[]), None);
}

/// **Fora de uma booleana viva não há papel nenhum** — o mundo de quem nunca criou um grupo
/// booleano fica intocado.
#[test]
fn a_shape_outside_a_live_boolean_has_no_role() {
    let (mut sim, map, bl, ids) = cooked(0);
    // O grupo perde o componente: o plano velho continua na mão, e a pergunta tem de mudar de
    // resposta na mesma — é o `VecBoolGroup` que manda, não a memória do último cozimento.
    let g = sim
        .world()
        .get::<ph2d_ecs::ChildOf>(Entity::from_bits(map[&ids[0]]))
        .map(|c| c.parent())
        .unwrap();
    sim.world_mut().entity_mut(g).remove::<VecBoolGroup>();
    assert_eq!(role_of(&sim, &map, &bl, ids[2]), None);
}

/// **O ESCRITOR RECONFERE** em vez de confiar no que o painel pintou.
///
/// ⚠️ Entre pintar a fileira e o clique chegar passa um frame, e nele a seleção pode ter mudado.
/// Escrever sem reconferir é como um clique numa forma acaba a mudar outra — e aqui o alvo mais
/// perigoso é a BASE, cujo verbo o motor ignora: o componente ficaria gravado no arquivo, inerte,
/// à espera de que uma reordenação o tornasse subitamente visível.
#[test]
fn the_writer_refuses_what_the_projection_would_not_offer() {
    let (mut sim, map, bl, ids) = cooked(0);
    assert!(
        !set_selected_shape_op(&mut sim, &map, &bl, &[ids[0]], 1),
        "escreveu na BASE"
    );
    assert!(
        sim.world()
            .get::<ph2d_ecs::VecBoolOp>(Entity::from_bits(map[&ids[0]]))
            .is_none(),
        "a base ficou com um verbo gravado"
    );
    // E o caminho feliz continua a escrever.
    assert!(set_selected_shape_op(&mut sim, &map, &bl, &[ids[2]], 1));
    assert_eq!(shape_op_of_selection(&sim, &map, &bl, &[ids[2]]), Some(1));
}

/// **O selo nomeia o papel** — e o da base nomeia a AUSÊNCIA de verbo em vez de a deixar em
/// branco. Um branco lê-se como *"nada escolhido"*; `BSE` diz o que aquela linha é.
#[test]
fn the_badge_names_the_role() {
    assert_eq!(BoolRole::Base.badge(), "BSE");
    assert_eq!(BoolRole::Recipe.badge(), "RCP");
    for (code, want) in [(0u8, "UNI"), (1, "SUB"), (2, "INT"), (3, "EXC")] {
        assert_eq!(BoolRole::Verb(code).badge(), want, "código {code}");
    }
}
