//! **O OPERANDO ABSORVIDO** — os gates do pick sobre a booleana viva, módulo irmão de [`super`]
//! pelo teto de LOC.
//!
//! Um grupo booleano vivo dá a tinta do resultado ao operando mais ao FUNDO e escreve uma lista
//! **VAZIA** para todos os outros (`bool_live`). A lei do hit-test é *nada desenhado, nada pego* —
//! logo, até 2026-08-22, cada um desses operandos era **inalcançável pelo canvas**: o artista
//! configurava a booleana e passava a só conseguir agarrar UMA das formas (report do Enio).
//!
//! A cura não fura a lei, dá ao operando a porta que ele de facto tem: **a tinta do GRUPO que o
//! absorveu**. É por isso que estes gates vêm aos pares — um que prova o alcance, e o seu gêmeo
//! que prova que a tela limpa continua limpa. O par que interessa mais é o do `Subtract`: o
//! cortador ocupa exactamente o buraco, então uma cura que o alcançasse pelo próprio footprint
//! passaria no gate do alcance e **falharia** no da tela limpa.
//!
//! ⚠️ Mutação canônica: fazer `VecViewState::absorbed_door` devolver sempre `None` (o produto de
//! antes desta wave) tem de SANGRAR os gates de alcance e deixar VERDES os de regressão.

use super::*;
use ph2d_ecs::VecBoolGroup;
use ph2d_vec_scene::{VecXforms, rectangle};

/// O que uma cena booleana entrega aos gates: `a` é a BASE (fundo de z, carrega a tinta) e `b` o
/// operando absorvido.
struct Fixture {
    sim: SimWorld,
    scene: VecScene,
    map: VecEntityMap,
    a: VecPathId,
    b: VecPathId,
    live: LiveGeometry,
    view: VecViewState,
}

/// Dois retângulos SOBREPOSTOS num grupo booleano `op`, já cozidos pelo `bool_live` — a fixture
/// não pode inventar a absorção, senão o gate testaria a si mesmo.
///
/// `a = [0,20]²` e `b = [10,30]²` deixam três regiões com **10 unidades de folga** cada: só-`a`
/// em `(5,5)`, sobreposição em `(15,15)`, só-`b` em `(25,25)`.
fn boolean_group(op: u8) -> Fixture {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let b = scene.push_path(rectangle([10.0, 10.0], [30.0, 30.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b]], "Bool".into()).unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });
    let mut live = LiveGeometry::new();
    let mut bl = crate::bool_live::BoolLive::default();
    bl.recook(&scene, &sim, &map, &VecXforms::default(), &[], &mut live);
    // Pré-condição da fixture inteira: `b` foi mesmo ABSORVIDO. Sem isto todos os gates abaixo
    // mediriam uma forma normal e seriam verdes por acidente.
    assert_eq!(
        live.get(&b).map(Vec::len),
        Some(0),
        "a fixture não absorveu nada — o grupo não cozinhou"
    );
    let view = VecViewState {
        absorbed: bl.absorbed(),
        ..VecViewState::default()
    };
    Fixture {
        sim,
        scene,
        map,
        a,
        b,
        live,
        view,
    }
}

impl Fixture {
    /// A lista do clique-cíclico em `p`, com folga de traço ZERO — só interior conta.
    fn pick(&self, p: [f32; 2]) -> Vec<u64> {
        pick_all_at_world(
            &self.sim,
            &self.scene,
            &self.live,
            &self.view,
            &self.map,
            p,
            0.0,
        )
    }

    /// A mesma lista **sem a tabela de absorção** — literalmente o produto de antes da cura, e o
    /// controlo positivo de todo gate que afirma alcance.
    fn pick_blind(&self, p: [f32; 2]) -> Vec<u64> {
        pick_all_at_world(
            &self.sim,
            &self.scene,
            &self.live,
            &VecViewState::default(),
            &self.map,
            p,
            0.0,
        )
    }

    fn bits(&self, id: VecPathId) -> u64 {
        self.map[&id]
    }
}

/// **O OPERANDO ABSORVIDO É ALCANÇÁVEL** — o defeito que o Enio reportou, de frente.
///
/// Nasce VERMELHO no produto de antes: sem a tabela, `b` tem entrada vazia, a lei *nada
/// desenhado, nada pego* responde `false`, e a lista traz só a base. É o que o `pick_blind`
/// mede ao lado — **o controlo positivo é o que separa este gate de um verde por acidente**.
#[test]
fn an_absorbed_operand_is_reachable_through_the_groups_ink() {
    let f = boolean_group(0); // Union
    let hits = f.pick([5.0, 5.0]);
    assert!(
        hits.contains(&f.bits(f.b)),
        "o operando absorvido continua inalcançável pelo canvas: {hits:?}"
    );
    assert_eq!(
        f.pick_blind([5.0, 5.0]),
        vec![f.bits(f.a)],
        "controlo positivo: sem a tabela, o produto de antes traz SÓ a base"
    );
}

/// **QUEM ESTÁ SOB O DEDO VEM PRIMEIRO.** Em `(5,5)` só `a` cobre o ponto; `b` está lá dentro do
/// mesmo grupo, e é alcançável, mas não é ele que se está a apontar.
///
/// ⚠️ Sem a partição a ordem seria a de z pura — e `b` está **por cima** de `a`, logo clicar no
/// lobo esquerdo de uma união nomearia o círculo da direita. É o gate que só a ordem prova: o de
/// cima passa com a lista trocada.
#[test]
fn the_shape_under_the_finger_comes_before_the_one_that_is_merely_inside() {
    let f = boolean_group(0); // Union
    assert_eq!(
        f.pick([5.0, 5.0]),
        vec![f.bits(f.a), f.bits(f.b)],
        "a forma apontada tem de ser a primeira da lista"
    );
    // E do outro lado da união a resposta é a simétrica — senão o gate estaria a medir z, não o
    // dedo (`b` é o topo, e em `(25,25)` a ordem de z daria a mesma lista por acidente).
    assert_eq!(
        f.pick([25.0, 25.0]),
        vec![f.bits(f.b), f.bits(f.a)],
        "no lobo de `b`, é `b` que se aponta"
    );
}

/// **Duas formas sob o dedo mantêm a ordem de Z.** A partição separa *apontado* de *apenas lá
/// dentro*; dentro de cada classe quem manda continua a ser a pilha, do topo para o fundo.
#[test]
fn two_operands_under_the_finger_keep_the_z_order() {
    let f = boolean_group(0); // Union
    assert_eq!(
        f.pick([15.0, 15.0]),
        vec![f.bits(f.b), f.bits(f.a)],
        "na sobreposição as duas são apontadas, e o topo vem antes"
    );
}

/// **O CORTADOR de um `Subtract` alcança-se pela tinta que ele DEIXOU.**
///
/// É o caso que decide o desenho inteiro: `b` come o canto de `a`, então o footprint de `b` e a
/// tinta do grupo são **disjuntos**. Uma cura que alcançasse o operando pelo próprio footprint
/// nunca chegaria aqui.
#[test]
fn the_cutter_of_a_subtract_is_reached_through_the_ink_it_left() {
    let f = boolean_group(1); // Subtract: a − b
    let hits = f.pick([5.0, 5.0]);
    assert!(
        hits.contains(&f.bits(f.b)),
        "o cortador ficou inalcançável: {hits:?}"
    );
    assert_eq!(
        hits,
        vec![f.bits(f.a), f.bits(f.b)],
        "e quem está sob o dedo continua a ser a forma que sobrou"
    );
}

/// **NADA DESENHADO, NADA PEGO — a lei sobrevive à cura.** O gêmeo do gate acima, e o que
/// reprova a cura ingênua.
///
/// O buraco de um `Subtract` é exactamente o footprint do cortador, e ali a tela está **limpa**:
/// um clique no buraco tem de atravessar para quem estiver por baixo, nunca selecionar uma forma
/// invisível. `(25,25)` é a outra metade da mesma afirmação — dentro de `b`, fora de toda tinta.
#[test]
fn nothing_drawn_nothing_picked_survives_the_cure() {
    let f = boolean_group(1); // Subtract: a − b
    assert_eq!(
        f.pick([15.0, 15.0]),
        Vec::<u64>::new(),
        "o BURACO do subtract devolveu alguém — a tela limpa deixou de estar limpa"
    );
    assert_eq!(
        f.pick([25.0, 25.0]),
        Vec::<u64>::new(),
        "fora de toda tinta do grupo não há nada a pegar"
    );
}

/// **ANIQUILAÇÃO NÃO É ABSORÇÃO** — e no mapa as duas são o mesmo `Some(vec![])`.
///
/// Um offset que come a forma não deixou nada na tela e nada há a pegar; é a cerca que o
/// `offset_live` documenta e que os gates irmãos (`vec_offset_pick_tests`) defendem. A cura só
/// pode alcançar quem está na TABELA — sem ela, o vazio volta a ser o fim da linha.
///
/// ⚠️ Este gate é o que impede a cura de virar *"toda entrada vazia é clicável"*, que apagaria a
/// lei do offset sem uma linha vermelha em lado nenhum.
#[test]
fn an_annihilated_shape_is_not_an_absorbed_one() {
    let f = boolean_group(0); // Union
    assert_eq!(
        f.pick_blind([5.0, 5.0]),
        vec![f.bits(f.a)],
        "sem porta, um vazio no mapa tem de continuar a ser aniquilação"
    );
}

/// **O MARQUEE leva todos os operandos, não só a base.** A outra metade do mesmo gesto: se a
/// borracha apanhasse só a base, arrastar a seleção moveria uma forma e deixaria as outras para
/// trás — e a booleana partir-se-ia ao ser movida.
///
/// A caixa de um absorvido é a da FONTE, que é exactamente a que o gizmo desenha: as duas
/// metades do gesto têm de medir o mesmo retângulo.
#[test]
fn the_marquee_takes_every_operand_not_just_the_base() {
    let f = boolean_group(0); // Union
    let all = pick_in_world_rect(
        &f.sim,
        &f.scene,
        &f.live,
        &f.view,
        &f.map,
        [-1.0, -1.0],
        [31.0, 31.0],
    );
    assert!(
        all.contains(&f.bits(f.a)) && all.contains(&f.bits(f.b)),
        "o marquee sobre o grupo inteiro levou {all:?}"
    );
    let blind = pick_in_world_rect(
        &f.sim,
        &f.scene,
        &f.live,
        &VecViewState::default(),
        &f.map,
        [-1.0, -1.0],
        [31.0, 31.0],
    );
    assert_eq!(
        blind,
        vec![f.bits(f.a)],
        "controlo positivo: sem a tabela, o marquee levava só a base"
    );
}

/// **ANINHADO: a porta é quem de facto DESENHA, não o pai imediato.**
///
/// A base de um grupo interno é ela própria um operando do externo — logo o mapa dela também
/// acaba VAZIO. Um par direto `(operando → base interna)` apontaria para uma porta **sem tinta**,
/// e o defeito voltaria inteiro, mas só nos documentos aninhados: os que ninguém smoka.
///
/// Aqui `c` (o mais ao fundo) é a base do grupo externo, e a tinta é toda dele. `a` é a base do
/// interno — absorvida pelo externo — e `b` está a dois saltos da tinta.
#[test]
fn a_nested_operand_is_reached_through_the_ink_that_is_actually_drawn() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // `c` primeiro: ele fica no FUNDO da pilha e por isso é a base do grupo externo — é o que
    // força a cadeia a ter dois saltos (com o grupo interno no fundo, ela teria zero).
    let c = scene.push_path(rectangle([0.0, 0.0], [40.0, 40.0]));
    let a = scene.push_path(rectangle([10.0, 10.0], [20.0, 20.0]));
    let b = scene.push_path(rectangle([15.0, 15.0], [25.0, 25.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let inner = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b]], "In".into()).unwrap(),
    );
    sim.world_mut()
        .entity_mut(inner)
        .insert(VecBoolGroup { op: 0 }); // Union
    let outer = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&c], inner.to_bits()], "Out".into())
            .unwrap(),
    );
    sim.world_mut()
        .entity_mut(outer)
        .insert(VecBoolGroup { op: 0 }); // Union

    let mut live = LiveGeometry::new();
    let mut bl = crate::bool_live::BoolLive::default();
    bl.recook(&scene, &sim, &map, &VecXforms::default(), &[], &mut live);
    // Pré-condições: `c` é quem desenha, e as outras duas estão vazias — uma a um salto da
    // tinta, a outra a dois.
    assert_eq!(live.get(&a).map(Vec::len), Some(0), "`a` devia estar vazio");
    assert_eq!(live.get(&b).map(Vec::len), Some(0), "`b` devia estar vazio");
    assert_eq!(live.get(&c).map(Vec::len), Some(1), "`c` devia ter a tinta");

    let view = VecViewState {
        absorbed: bl.absorbed(),
        ..VecViewState::default()
    };
    let hits = pick_all_at_world(&sim, &scene, &live, &view, &map, [12.0, 12.0], 0.0);
    assert!(
        hits.contains(&map[&b]),
        "o operando a DOIS saltos da tinta ficou inalcançável: {hits:?}"
    );
    assert_eq!(
        hits,
        vec![map[&a], map[&c], map[&b]],
        "as duas apontadas primeiro (em z), a que só está lá dentro depois"
    );
}
