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
    bl.recook(&scene, &sim, &map, &VecXforms::default(), &[], &mut live);
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
    assert_eq!(
        shape_row_of_selection(&sim, &map, &bl, &[ids[0]], Some(ids[0])),
        None
    );
}

/// **HERANÇA: sem override, a forma mostra o verbo do GRUPO aceso.** É o que mantém os oito
/// botões do grupo vivos — eles continuam a ser o padrão de quem não se pronunciou.
#[test]
fn an_operand_inherits_the_groups_verb() {
    for op in 0..=3u8 {
        let (sim, map, bl, ids) = cooked(op);
        assert_eq!(
            shape_row_of_selection(&sim, &map, &bl, &[ids[2]], Some(ids[2])).map(|(c, _)| c),
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
    assert_eq!(
        shape_row_of_selection(&sim, &map, &bl, &[ids[2]], Some(ids[2])).map(|(c, _)| c),
        Some(1)
    );
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
    assert_eq!(
        shape_row_of_selection(&sim, &map, &bl, &[ids[2]], Some(ids[2])),
        None
    );
}

/// **O PRIMÁRIO TEM DE ESTAR NA SELEÇÃO.**
///
/// ⚠️ Este gate substituiu um que exigia *"exactamente uma forma selecionada"* — regra que
/// **nenhum clique deste editor podia satisfazer**, e que era o defeito. A multi-seleção passou a
/// ser o caso NORMAL (tocar um filho seleciona o grupo); o que continua a não poder acontecer é a
/// fileira falar de uma forma que já não está em mãos, porque o primário é pegajoso e sobrevive a
/// uma seleção nova que não o contém.
#[test]
fn a_primary_outside_the_selection_offers_nothing() {
    let (sim, map, bl, ids) = cooked(0);
    assert_eq!(
        shape_row_of_selection(&sim, &map, &bl, &[ids[0], ids[1]], Some(ids[2])),
        None,
        "o primário não estava na seleção"
    );
    assert_eq!(
        shape_row_of_selection(&sim, &map, &bl, &[], Some(ids[2])),
        None
    );
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
        !set_selected_shape_op(&mut sim, &map, &bl, &ids, Some(ids[0]), 1),
        "escreveu na BASE"
    );
    assert!(
        sim.world()
            .get::<ph2d_ecs::VecBoolOp>(Entity::from_bits(map[&ids[0]]))
            .is_none(),
        "a base ficou com um verbo gravado"
    );
    // E o caminho feliz continua a escrever.
    assert!(set_selected_shape_op(
        &mut sim,
        &map,
        &bl,
        &ids,
        Some(ids[2]),
        1
    ));
    assert_eq!(
        shape_row_of_selection(&sim, &map, &bl, &[ids[2]], Some(ids[2])).map(|(c, _)| c),
        Some(1)
    );
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

/// **A FILEIRA SEGUE O PRIMÁRIO, com o GRUPO INTEIRO selecionado** — o defeito que o Enio viu.
///
/// ⚠️ Este gate nasceu VERMELHO, e a causa era uma regra minha que **nenhum clique pode
/// satisfazer**: eu exigia *"exatamente UMA forma selecionada"*, e tocar um filho de um grupo
/// **seleciona o grupo inteiro** — é lei deliberada deste editor (`input_dispatch`: *"Tocar um
/// filho seleciona o GRUPO (a árvore é a Hierarquia)"*). A fileira nunca aparecia, e por isso os
/// quatro chips não respondiam: eles não estavam lá.
///
/// A porta certa é o **PRIMÁRIO** — `set_object_selection` preserva-o quando ele está na lista
/// (`selection.rs`), então depois do clique `selected()` continua a ser *a forma que o dedo
/// apontou*, mesmo com os irmãos todos selecionados.
#[test]
fn the_row_follows_the_primary_even_with_the_whole_group_selected() {
    let (sim, map, bl, ids) = cooked(0); // Union
    // Exactamente o que o canvas produz: a lista é o grupo, o primário é o clicado.
    let sel = ids.to_vec();
    assert_eq!(
        shape_row_of_selection(&sim, &map, &bl, &sel, Some(ids[2])).map(|(code, _)| code),
        Some(0),
        "com o grupo selecionado e um primário nomeado, a fileira tem de existir"
    );
}

/// **E ela NOMEIA a forma de que fala.** Com o grupo inteiro aceso no canvas, um rótulo genérico
/// («This Shape») não diz de QUAL das três ele fala — e o artista escolheria no escuro.
#[test]
fn the_row_names_the_shape_it_talks_about() {
    let (mut sim, map, bl, ids) = cooked(0);
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&ids[2]]))
        .insert(ph2d_ecs::Name("Ellipse 2".into()));
    assert_eq!(
        shape_row_of_selection(&sim, &map, &bl, &ids, Some(ids[2])).map(|(_, n)| n),
        Some("Ellipse 2".to_string())
    );
}

/// **O primário fora do grupo não abre fileira nenhuma** — o gêmeo de regressão.
#[test]
fn a_primary_that_is_not_an_operand_offers_nothing() {
    let (sim, map, bl, ids) = cooked(0);
    // A BASE continua sem verbo, mesmo sendo primária.
    assert_eq!(
        shape_row_of_selection(&sim, &map, &bl, &ids, Some(ids[0])),
        None
    );
    // E sem primário não há de quem falar.
    assert_eq!(shape_row_of_selection(&sim, &map, &bl, &ids, None), None);
}

/// **A COSTURA DO CLIQUE: id do chip → código do verbo.**
///
/// ⚠️ Este gate é a cura da causa-raiz, não do sintoma. Os quatro chips shiparam com o modelo, o
/// cozimento e a triagem gateados — e **zero** gates no caminho que o clique percorre. É a
/// primeira das quatro causas da `DIRETIVA_IMPLEMENTACAO` (*costura não-testada*), e ela só ficou
/// gateável quando o mapeamento saiu de dentro do `render_loop`, que exige janela e GPU.
///
/// A segunda metade é a que apanha um erro real: um id VIZINHO — os quatro botões de OPERAÇÃO da
/// seção, que agem sobre o grupo inteiro — não pode ser lido como verbo de forma. Os dois
/// conjuntos convivem na mesma seção, e partilhar um id faria um clique significar as duas.
#[test]
fn the_click_seam_maps_each_chip_to_its_verb() {
    use ph2d_editor::ids as i;
    for (id, want) in [
        (i::VECTOR_BOOL_SHAPE_UNION, 0u8),
        (i::VECTOR_BOOL_SHAPE_SUBTRACT, 1),
        (i::VECTOR_BOOL_SHAPE_INTERSECT, 2),
        (i::VECTOR_BOOL_SHAPE_EXCLUDE, 3),
    ] {
        assert_eq!(shape_op_for_id(id), Some(want));
    }
    // Os VIZINHOS — o verbo do grupo, o modo Live, o Apply — não são verbo de forma.
    for foreign in [
        i::VECTOR_BOOL_UNION,
        i::VECTOR_BOOL_SUBTRACT,
        i::VECTOR_BOOL_INTERSECT,
        i::VECTOR_BOOL_EXCLUDE,
        i::VECTOR_BOOL_TRIM,
        i::VECTOR_BOOL_LIVE_ON,
        i::VECTOR_BOOL_APPLY,
    ] {
        assert_eq!(
            shape_op_for_id(foreign),
            None,
            "um id de outra família foi lido como verbo de forma"
        );
    }
}

/// ⛔ **O SELO DESCREVE O DOCUMENTO, e não o que a tela mostra** (auditoria de 2026-08-23).
///
/// Durante uma transição de estados a booleana desenha um MEIO entre duas operações, e o selo
/// continua a dizer o verbo de PARTIDA até a chegada — porque é esse que o componente guarda.
///
/// ⚠️ **É uma lei, não um resto**, e este gate existe para a manter escrita: ligar o selo ao morph
/// dá-lhe um terceiro estado (*"a caminho de"*) que nenhum gesto pode editar, e um rótulo que muda
/// sozinho num controlo imóvel é ruído. A pergunta *"o que a tela mostra agora?"* responde-se
/// olhando para a tela.
#[test]
fn the_badge_names_the_document_not_the_frame() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let b = scene.push_path(rectangle([10.0, 0.0], [30.0, 20.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b]], "B".into()).unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op: 0 }); // Union
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&b]))
        .insert(ph2d_ecs::VecBoolOp { op: 1 }); // ...e esta forma subtrai

    let mut live = LiveGeometry::new();
    let mut bl = BoolLive::default();
    bl.recook(
        &scene,
        &sim,
        &map,
        &VecXforms::default(),
        // Um morph EM VOO, a caminho de `Intersect`: o desenho está no MEIO.
        &[ph2d_ui_state::BoolMorph {
            id: b,
            op: Some(2),
            group_op: Some(0),
            t: 0.5,
        }],
        &mut live,
    );
    assert_eq!(bl.morphed(), 1, "a fixture nao pos morph nenhum em voo");
    assert_eq!(
        role_of(&sim, &map, &bl, b),
        Some(BoolRole::Verb(1)),
        "o selo passou a descrever o desenho do quadro em vez do verbo que o documento guarda"
    );
}
