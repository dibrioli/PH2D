//! **A BOOLEANA VIVA A MEIO DE UMA TROCA DE VERBO** — os gates do que o artista vê quando um
//! estado de UI muda a operação (Enio, 2026-08-23).
//!
//! # A fixture é a DONUT, e a escolha é a metade do trabalho
//!
//! Um retângulo grande com outro **inteiramente dentro**. `Union` desenha **1** contorno (área
//! 400); `Subtract` desenha **2** — o de fora e o buraco de 64. É o único par em que a TOPOLOGIA
//! muda, e portanto o único que decide se o morph serve para alguma coisa.
//!
//! ⚠️ **A fixture do irmão (dois retângulos que se cruzam) NÃO contém o fenômeno:** ali os quatro
//! verbos dão 1 contorno, e um gate escrito sobre ela ficaria verde sem nunca exercitar um buraco
//! a nascer. Medido antes de escrever uma linha destes gates.
//!
//! # ⛔ O que foi MEDIDO e recusado (sonda `probe_boolean_morph.rs`)
//!
//! | tentativa | por quê não |
//! |---|---|
//! | Saltar o verbo (Blender / After Effects / Rive) | move **64,0** de tinta num quadro com a peça PARADA, contra 3,1 do morph |
//! | Crossfade das duas formas (o recuo do Figma) | mostra as duas ao mesmo tempo; o morph não tem fantasma |
//! | Perseguir a partir do que está na tela | cura o único quadro que salta e paga com o desenho a FICAR PARA TRÁS do movimento: 793,0 de tinta de afastamento numa peça que viaja |

use super::tests::area_of;
use super::*;
use ph2d_ui_state::BoolMorph;
use ph2d_vec_scene::{Xform, rectangle};

/// A área do buraco que o desenho tem: o de fora mede 400 nos dois extremos, então o que falta
/// para 400 **é** o buraco.
const HOLE: f64 = 64.0;
/// Metade de um buraco que cresce por `t²` mede `16`. A barra separa isso das duas pontas (`0` e
/// `64`) com folga de 14 para cada lado — ela não precisa de ser apertada, precisa de ser
/// DECISIVA.
const TOL: f64 = 2.0;

/// A DONUT: o de fora, o de dentro, agrupados, com o grupo em `op`.
fn donut(op: u8) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>, Entity) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let outer = scene.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let inner = scene.push_path(rectangle([6.0, 6.0], [14.0, 14.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&outer], map[&inner]], "Bool".into())
            .unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });
    (sim, scene, map, vec![outer, inner], g)
}

/// O recado que a ponte publica: *esta forma vai para o verbo `op`, e o grupo para `group_op`*.
fn morph(id: VecPathId, op: Option<u8>, group_op: u8, t: f64) -> BoolMorph {
    BoolMorph {
        id,
        op,
        group_op: Some(group_op),
        t,
    }
}

/// ⭐ **QUANTA TINTA difere entre dois desenhos** — a área da diferença simétrica.
///
/// ⚠️ **A primeira versão encadeava os dois lados num `Exclude` só** (`a.chain(b)`), e isso é
/// **errado assim que um lado tem mais de uma peça**: `Exclude` sobre três caminhos é `a ⊕ b ⊕ c`,
/// não a diferença entre `(a ∪ b)` e `c`. Medido: com o `Trim` (que devolve 2 peças) ela dizia
/// `0,000` de diferença entre desenhos cujas áreas eram 400 e 272 — e com um lado VAZIO dizia
/// `0,000` também, porque o motor recusa uma booleana de um só operando.
///
/// ⇒ cada lado é primeiro colapsado numa forma composta (as peças de um resultado booleano são
/// disjuntas, então juntá-las é exato), e só então os DOIS entram. Um lado vazio é o caso que a
/// booleana não sabe responder, e a resposta é a área do outro.
fn ink_between(a: &[VecPath], b: &[VecPath]) -> f64 {
    match (super::morph::as_one(a), super::morph::as_one(b)) {
        (None, None) => 0.0,
        (Some(x), None) | (None, Some(x)) => ph2d_vec_boolean::area(&x).abs(),
        (Some(x), Some(y)) => {
            ph2d_vec_boolean::pathfinder(&[&x, &y], ph2d_vec_boolean::PathfinderOp::Exclude)
                .map_or(f64::INFINITY, |o| {
                    o.iter().map(|p| ph2d_vec_boolean::area(p).abs()).sum()
                })
        }
    }
}

/// O quanto de buraco o desenho tem.
fn hole_of(items: &[VecPath]) -> f64 {
    400.0 - area_of(items)
}

/// O canto de baixo-esquerda do que o desenho ocupa.
///
/// ⚠️ **A régua é a CAIXA da peça inteira, e não o centro do buraco.** Tentei o centro primeiro e
/// ele mediu a coisa errada: a meio caminho o buraco está entre o PONTO em que nasce e o buraco
/// real, então mover o operando 4 unidades move o centro do meio-buraco 2 — o número certo para
/// uma régua que eu tinha lido como estando errada. *Foi a régua, não o motor.*
fn min_corner(items: &[VecPath]) -> [f64; 2] {
    let mut lo = [f64::INFINITY, f64::INFINITY];
    for p in items {
        for c in 0..p.contour_count() {
            if let Some((verts, _)) = p.contour(c) {
                for v in verts {
                    lo = [lo[0].min(v.anchor[0]), lo[1].min(v.anchor[1])];
                }
            }
        }
    }
    lo
}

/// Roda um quadro com os operandos postos por `xf`, num `BoolLive` novo.
fn frame(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    xf: &VecXforms,
    morphs: &[BoolMorph],
) -> Vec<VecPath> {
    frame_on(&mut BoolLive::default(), sim, scene, map, xf, morphs)
}

/// Roda um quadro **no mesmo `BoolLive`** — o que o app faz, e a única forma de o memo entrar na
/// conta.
///
/// ⚠️ A distinção não é cosmética: um cozimento que servisse a resposta MEMOIZADA a meio de uma
/// transição congelaria o desenho, e um gate que constrói um `BoolLive` novo por quadro nunca o
/// veria — o memo nasce vazio.
fn frame_on(
    bl: &mut BoolLive,
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    xf: &VecXforms,
    morphs: &[BoolMorph],
) -> Vec<VecPath> {
    let mut live = LiveGeometry::new();
    bl.recook(scene, sim, map, xf, morphs, &mut live);
    live.get(&scene.paths()[0].id).cloned().unwrap_or_default()
}

/// ⭐ **O MEIO DE UMA TROCA DE VERBO NÃO É NENHUMA DAS DUAS PONTAS — o buraco CRESCE DE UM PONTO.**
///
/// É o gate que a feature inteira existe para ter. As duas pontas medem `0` e `64` de buraco; o
/// meio tem de medir `16`, que é `t²·64` — um buraco a crescer **linearmente na dimensão** a
/// partir de um ponto, e não a piscar.
///
/// ⚠️ **As duas pontas entram no mesmo gate de propósito.** Sem elas, um `16` sozinho não separa
/// *"o morph desenha o meio"* de *"a fixture desenha 16 de qualquer maneira"* — e uma barra sem os
/// seus controlos é uma barra que fica verde no dia em que o cozimento parar de correr.
#[test]
fn the_middle_of_a_verb_change_is_neither_end() {
    let (sim, scene, map, ids, _g) = donut(0); // o grupo em Union
    let xf = VecXforms::new();

    let start = frame(&sim, &scene, &map, &xf, &[]);
    assert!(
        hole_of(&start) < TOL,
        "a partida tem de ser a UNIÃO, sem buraco: mediu {:.2}",
        hole_of(&start)
    );

    let end = frame(&sim, &scene, &map, &xf, &[morph(ids[1], Some(1), 0, 1.0)]);
    assert!(
        (hole_of(&end) - HOLE).abs() < TOL || hole_of(&end) < TOL,
        "a fixture não contém o fenômeno: a chegada mediu {:.2} de buraco",
        hole_of(&end)
    );

    let mid = frame(&sim, &scene, &map, &xf, &[morph(ids[1], Some(1), 0, 0.5)]);
    let hole = hole_of(&mid);
    assert!(
        (hole - HOLE * 0.25).abs() < TOL,
        "o meio mediu {hole:.2} de buraco, esperado {:.2} (0 = ainda é a união, {HOLE} = já saltou \
         para a subtração)",
        HOLE * 0.25
    );
}

/// ⭐⭐ **O DESENHO SEGUE OS OPERANDOS ENQUANTO O VERBO TROCA** (Enio, 2026-08-23: *"as formas além
/// de mudar o modo do boolean também podem estar animadas em pos, scl e rot"*).
///
/// ⚠️ **É o gate que separa este desenho do único outro que passaria nos de cima:** cozinhar as
/// duas pontas UMA vez, no início da transição, e morfar entre elas. Aquele desenharia o mesmo meio
/// numa cena parada e **congelaria** a peça assim que ela se mexesse — e nada ficaria vermelho.
///
/// A fixture move o operando pela MESMA porta que a animação usa (o `Xform` que sai do `Transform`
/// que o `install` escreve), e exige que o buraco a meio caminho ande com ele.
#[test]
fn the_drawing_follows_the_operands_while_the_verb_changes() {
    let (sim, scene, map, ids, _g) = donut(0);
    let recado = [morph(ids[1], Some(1), 0, 0.5)];
    // ⚠️ **Um `BoolLive` só para os quadros todos**, como no app — e o PRIMEIRO deles é sem recado
    // nenhum, que é o que o artista vê antes de tocar no botão. É esse quadro que **aquece o
    // memo**, e sem ele o gate não teria como apanhar um cozimento que servisse a resposta velha a
    // meio da transição: um memo vazio nunca acerta.
    let mut bl = BoolLive::default();
    frame_on(&mut bl, &sim, &scene, &map, &VecXforms::new(), &[]);

    let still = frame_on(&mut bl, &sim, &scene, &map, &VecXforms::new(), &recado);
    let a = min_corner(&still);

    // **A PEÇA INTEIRA viaja 30 unidades**, os dois operandos pela MESMA porta que a animação usa
    // (o `Xform` que sai do `Transform` que o `install` escreve).
    let mut travel = VecXforms::new();
    for id in &ids {
        travel.insert(*id, Xform([1.0, 0.0, 0.0, 1.0, 30.0, 0.0]));
    }
    let travelled = frame_on(&mut bl, &sim, &scene, &map, &travel, &recado);
    let b = min_corner(&travelled);
    assert!(
        (b[0] - a[0] - 30.0).abs() < 1e-6,
        "o desenho não acompanhou o movimento: a peça andou {:.4} em x, esperado 30 \
         (as pontas foram cozidas uma vez e congeladas?)",
        b[0] - a[0]
    );

    // **E a ESCALA também**: só o de dentro encolhe para metade, e o buraco do meio encolhe com ele.
    let mut shrunk = VecXforms::new();
    shrunk.insert(ids[1], Xform([0.5, 0.0, 0.0, 0.5, 5.0, 5.0]));
    let smaller = frame_on(&mut bl, &sim, &scene, &map, &shrunk, &recado);
    assert!(
        hole_of(&smaller) < hole_of(&still) * 0.5,
        "o operando encolheu para metade e o buraco do meio não encolheu: {:.2} contra {:.2}",
        hole_of(&smaller),
        hole_of(&still)
    );
}

/// ⭐ **TROCAR A OPERAÇÃO DO GRUPO anima** — aqui entre duas operações de CONJUNTO.
///
/// ⚠️ **O doc deste gate afirmava «inclusive para uma RECEITA» e a fixture passava `2` =
/// `Intersect`** (auditoria de 2026-08-23): o ramo de receita do `cook_side` — o que chama o
/// `pathfinder` em vez da cadeia — **nunca corria num morph**, e o argumento que justificava o
/// canal do grupo estava escrito por cima de uma fixture que não o exercitava. A receita tem
/// agora gate PRÓPRIO, o irmão logo abaixo.
#[test]
fn the_group_can_change_operation_mid_flight() {
    let (sim, scene, map, ids, _g) = donut(0); // Union
    let xf = VecXforms::new();
    let union = frame(&sim, &scene, &map, &xf, &[]);
    // O grupo vai para Intersect (2): a peça inteira encolhe para o de dentro.
    let mid = frame(&sim, &scene, &map, &xf, &[morph(ids[1], None, 2, 0.5)]);
    let inter = frame(&sim, &scene, &map, &xf, &[morph(ids[1], None, 2, 1.0)]);

    let (a, m, b) = (area_of(&union), area_of(&mid), area_of(&inter));
    assert!(
        b < a * 0.5,
        "a fixture não separa as pontas: união {a:.1}, interseção {b:.1}"
    );
    assert!(
        m < a && m > b,
        "o meio ({m:.1}) tem de estar ENTRE a união ({a:.1}) e a interseção ({b:.1})"
    );
}

/// **INÉRCIA: sem recado nenhum, o desenho é o de sempre.**
///
/// ⚠️ É o gate que faz esta feature não custar nada a quem não a usa — e o que apanha um cozimento
/// duplo ligado por omissão, que não muda um pixel e paga o dobro em todo quadro do app.
#[test]
fn without_a_morph_the_drawing_is_untouched() {
    let (sim, scene, map, _ids, _g) = donut(1); // Subtract
    let xf = VecXforms::new();
    let plain = frame(&sim, &scene, &map, &xf, &[]);
    // Um recado que fala de OUTRA forma não é deste grupo.
    let other = frame(&sim, &scene, &map, &xf, &[morph(9999, Some(0), 0, 0.5)]);
    assert_eq!(
        plain, other,
        "um recado de outro grupo mudou o desenho deste"
    );
    let mut bl = BoolLive::default();
    frame_on(&mut bl, &sim, &scene, &map, &xf, &[]);
    assert_eq!(
        bl.morphed(),
        0,
        "sem recado nenhum o grupo pagou um cozimento a mais"
    );
    assert!(
        (hole_of(&plain) - HOLE).abs() < TOL,
        "a fixture não está a subtrair: {:.2} de buraco",
        hole_of(&plain)
    );
}

/// **UMA CHEGADA QUE DESENHA O MESMO NÃO CUSTA UM SEGUNDO COZIMENTO.**
///
/// ⚠️ O caso é real: um estado pode largar o override de uma forma (`Some(0)` ⟶ `None`) sem mudar
/// o verbo EFETIVO dela, porque o grupo já era aquele. Sem a conferência, o grupo pagaria dois
/// cozimentos e um casamento por quadro para desenhar exactamente o que um cozimento desenha — e
/// o resultado passaria por um `Plan`, que reamostra a forma e a devolve **diferente ao bit**.
///
/// É por isso que a barra aqui é a IGUALDADE EXATA: ela mede as duas coisas de uma vez.
#[test]
fn an_arrival_that_draws_the_same_costs_nothing() {
    let (mut sim, scene, map, ids, _g) = donut(0); // o grupo em Union
    let inner = Entity::from_bits(map[&ids[1]]);
    sim.world_mut()
        .entity_mut(inner)
        .insert(ph2d_ecs::VecBoolOp { op: 0 }); // ...e a forma a repetir o Union
    let xf = VecXforms::new();

    // ⚠️ **A régua é a CONTAGEM DE COZIMENTOS, e não a igualdade do desenho.** Tentei a igualdade
    // primeiro e ela é CEGA a este defeito: morfar duas pontas iguais devolve a mesma forma ao
    // bit, então o dobro do trabalho fica ligado para sempre com o gate verde. Um custo declarado
    // precisa de um gate que o MEÇA (a emenda do HR-13).
    let mut idle = BoolLive::default();
    frame_on(
        &mut idle,
        &sim,
        &scene,
        &map,
        &xf,
        &[morph(ids[1], None, 0, 0.5)],
    );
    assert_eq!(
        idle.morphed(),
        0,
        "uma chegada que desenha o mesmo pagou dois cozimentos"
    );

    // O CONTROLE POSITIVO: uma troca de verbo a sério paga os dois, e sem ele o `0` acima não
    // separa *"a conferência funciona"* de *"o morph nunca corre"*.
    let mut busy = BoolLive::default();
    frame_on(
        &mut busy,
        &sim,
        &scene,
        &map,
        &xf,
        &[morph(ids[1], Some(1), 0, 0.5)],
    );
    assert_eq!(
        busy.morphed(),
        1,
        "uma troca de verbo a sério não cozinhou as duas pontas"
    );
}

/// ⭐⭐⭐ **A COMPOSIÇÃO QUE O QUADRO CORRE** — a ponte dos estados publica, a booleana consome, e o
/// desenho fica entre as duas operações.
///
/// ⚠️ **Ele mora aqui de propósito, e nenhuma das duas metades sozinha o mostraria.** Os gates de
/// cima entregam recados escritos à mão a um cozimento; os da `ph2d-ui-state` provam que a
/// transição os publica. O que só existe na costura é *o recado chegar* — e esta casa já pagou
/// essa lição com vinte testes verdes sobre um `draw` cravado em `true`.
///
/// A fixture é a da cena `=74`: o grupo booleano pendurado num CHIP, que é o hospedeiro dos
/// estados — a única disposição em que o artista consegue selecionar um hospedeiro ÚNICO com uma
/// booleana dentro dele.
#[test]
fn the_frame_composes_the_bridge_and_the_cook() {
    use ph2d_anim::{Easing, EasingFamily, EasingMode};
    use ph2d_ui_state::{StateRole, StateSets};

    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let chip = scene.push_path(rectangle([-2.0, -2.0], [22.0, 22.0]));
    let outer = scene.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let inner = scene.push_path(rectangle([6.0, 6.0], [14.0, 14.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&outer], map[&inner]], "Bool".into())
            .unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op: 0 });
    crate::vec_transform::reparent_keeping_world(&mut sim, g, Entity::from_bits(map[&chip]));

    // As duas poses, pela porta do PRODUTO — nunca escrevendo a tabela à mão.
    let mut states = StateSets::default();
    let rec = |sim: &mut SimWorld, scene: &mut VecScene, states: &mut StateSets, role| {
        crate::vec_ui_state_edit::apply(
            sim,
            scene,
            &map,
            &[chip],
            states,
            crate::vec_ui_state_edit::UiStateEdit::Record(role),
        );
    };
    rec(&mut sim, &mut scene, &mut states, StateRole::Default);
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op: 1 }); // Subtract no Hover
    rec(&mut sim, &mut scene, &mut states, StateRole::Hover);
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op: 0 }); // e a cena volta ao repouso

    // ⚠️ Curva LINEAR: com a de fábrica o `t` de meio caminho é deformado, e a barra passaria a
    // medir a curva em vez do desenho.
    states.set_easing(chip, Easing::new(EasingFamily::Linear, EasingMode::InOut));
    let (duration, _) = states.timing(chip);

    let mut machines = crate::render_loop::ui_state_bridge::UiMachines::new();
    crate::render_loop::ui_state_bridge::request(&mut machines, &states, chip, StateRole::Hover);
    let mut morphs = Vec::new();
    let animating = crate::render_loop::ui_state_bridge::dispatch(
        &mut machines,
        &mut states,
        &mut sim,
        &mut scene,
        &map,
        duration * 0.5,
        &mut morphs,
    );
    assert!(animating, "a ponte não pôs a máquina no ar");
    assert!(
        !morphs.is_empty(),
        "a ponte não publicou recado nenhum: o buraco vai APARECER de uma vez no fim"
    );

    // ⚠️ A leitura é pelo id do OUTER, e não pelo primeiro caminho da cena: o portador do
    // resultado é a BASE do grupo, e aqui o primeiro caminho é o CHIP — que não é operando de
    // nada. Ler o índice 0 media uma entrada VAZIA e dizia *"o buraco é a peça inteira"*.
    let mut live = LiveGeometry::new();
    BoolLive::default().recook(&scene, &sim, &map, &VecXforms::new(), &morphs, &mut live);
    let drawn = live.get(&outer).cloned().unwrap_or_default();
    let hole = hole_of(&drawn);
    assert!(
        (hole - HOLE * 0.25).abs() < TOL,
        "a meio caminho o desenho mediu {hole:.2} de buraco, esperado {:.2} \
         (0 = o recado não chegou ao cozimento, {HOLE} = ele saltou)",
        HOLE * 0.25
    );
}

/// ⛔ **UM RECADO DO GRUPO DE DENTRO NÃO MANDA NO GRUPO DE FORA.**
///
/// ⚠️ **A base de um grupo INTERNO é também operando do EXTERNO** — e o `bool_group_op` da pose
/// dela fala do grupo **dela**. Lido pelo externo, ele faria o grupo de fora adotar a operação do
/// de dentro, em silêncio e **só em documentos aninhados** — que são precisamente os que ninguém
/// smoka.
///
/// A régua é a CONTAGEM DE COZIMENTOS, e não o desenho: com o defeito os DOIS grupos morfam; com a
/// leitura certa, só o de dentro. Um gate sobre a forma teria de saber qual é o desenho "errado",
/// e o número diz a mesma coisa sem essa dúvida.
#[test]
fn a_morph_from_the_inner_group_does_not_command_the_outer_one() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let b = scene.push_path(rectangle([6.0, 6.0], [14.0, 14.0]));
    let c = scene.push_path(rectangle([10.0, -4.0], [16.0, 24.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);

    // O grupo de DENTRO: `a` e `b`. A base dele é `a`, que carrega o resultado.
    let inner = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b]], "In".into()).unwrap(),
    );
    sim.world_mut()
        .entity_mut(inner)
        .insert(VecBoolGroup { op: 0 });
    // O de FORA: o grupo de dentro e mais o `c`.
    let outer = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[inner.to_bits(), map[&c]], "Out".into())
            .unwrap(),
    );
    sim.world_mut()
        .entity_mut(outer)
        .insert(VecBoolGroup { op: 0 });

    // O CONTROLE: sem recado nenhum, ninguém morfa — e os dois grupos cozinham.
    let mut idle = BoolLive::default();
    let mut live = LiveGeometry::new();
    idle.recook(&scene, &sim, &map, &VecXforms::new(), &[], &mut live);
    assert!(
        idle.plan(inner).is_some() && idle.plan(outer).is_some(),
        "a fixture não montou os dois grupos aninhados — o gate não mede nada"
    );
    assert_eq!(idle.morphed(), 0, "sem recado nenhum, ninguém pode morfar");

    // O recado é da BASE do grupo de dentro, e fala da operação DELE.
    let mut bl = BoolLive::default();
    let mut live2 = LiveGeometry::new();
    bl.recook(
        &scene,
        &sim,
        &map,
        &VecXforms::new(),
        &[morph(a, None, 2, 0.5)], // o grupo de DENTRO vai para Intersect
        &mut live2,
    );
    assert_eq!(
        bl.morphed(),
        1,
        "morfaram {} grupos: o recado do grupo de dentro foi lido também pelo de fora",
        bl.morphed()
    );
}

/// ⭐ **E TROCAR PARA UMA RECEITA TAMBÉM ANIMA** — o ramo do cozimento que o irmão acima não toca.
///
/// ⚠️ As quatro receitas (`MinusBack`/`Trim`/`Crop`/`Merge`) são verbos da PILHA INTEIRA e não têm
/// decomposição por forma nenhuma: `verbs_of` devolve **vazio** para elas e o `cook_side` cai no
/// ramo do `pathfinder`. É este o ramo que justifica a pose carregar o canal do GRUPO — e ele
/// esteve por medir enquanto o doc do irmão afirmava tê-lo medido.
///
/// ⚠️ **`Crop`, e só ele.** Medido nesta fixture: `Trim` e `Merge` desenham **a MESMA REGIÃO** que
/// a união (`0,000` de tinta de diferença — as peças do `Trim` recobrem a peça inteira), e
/// `MinusBack` desenha **nada**. Um gate sobre qualquer um dos três seria tautológico ou mediria a
/// recusa em vez do cozimento. *Um par que não separa não é fixture.*
///
/// ⚠️ **E o oráculo é o COZIMENTO DIRETO**, medido em TINTA. Duas armadilhas caíram aqui: uma
/// desigualdade (*"o meio está entre as pontas"*) deixou passar um mutante que devolvia a entrada
/// crua; e o `area_of` — soma de áreas absolutas — lê `400` e `272` para dois desenhos que cobrem
/// **exactamente a mesma região**, porque um vem em duas peças e o outro num composto.
#[test]
fn the_group_can_change_to_a_recipe_mid_flight() {
    const CROP: u8 = 6;
    let (sim, scene, map, ids, _g) = donut(0); // Union
    let xf = VecXforms::new();
    let union = frame(&sim, &scene, &map, &xf, &[]);

    let (osim, oscene, omap, _oids, _og) = donut(CROP);
    let oracle = frame(&osim, &oscene, &omap, &xf, &[]);
    assert!(
        ink_between(&oracle, &union) > TOL,
        "a fixture não separa a receita da união — um gate assim passaria com o morph desligado"
    );

    let end = frame(&sim, &scene, &map, &xf, &[morph(ids[1], None, CROP, 1.0)]);
    assert!(
        ink_between(&end, &oracle) < TOL,
        "a chegada do morph difere do cozimento direto da receita em {:.2} de tinta — o ramo de \
         RECEITA não cozinhou o que devia",
        ink_between(&end, &oracle)
    );

    let mid = frame(&sim, &scene, &map, &xf, &[morph(ids[1], None, CROP, 0.5)]);
    assert!(
        ink_between(&mid, &union) > TOL && ink_between(&mid, &end) > TOL,
        "o meio coincide com uma das pontas — o morph não desenhou meio nenhum"
    );
}

/// ⛔ **UMA CHEGADA QUE DESENHA NADA NÃO SE MORFA: o desenho FICA na partida e troca na chegada.**
///
/// ⚠️ `Ok(vazio)` é uma RESPOSTA do motor, não uma recusa (a interseção de duas formas disjuntas é
/// o vazio). Mas não há forma para onde interpolar: o `Plan` casa dois desenhos, e um deles não
/// existe. A lei é a mesma do par degenerado do `Transition::at` — *sem plano, fica-se na partida*
/// —, e o preço é **um** salto no quadro da chegada.
///
/// Medido: `MinusBack` na DONUT desenha **nada** (0 peças), e a transição para ele segura os 400
/// da união até o fim. É a única das quatro receitas com esse comportamento nesta fixture, e está
/// aqui **nomeada** em vez de descoberta.
#[test]
fn an_arrival_that_draws_nothing_holds_the_start() {
    const MINUS_BACK: u8 = 4;
    let (sim, scene, map, ids, _g) = donut(0);
    let xf = VecXforms::new();
    let union = frame(&sim, &scene, &map, &xf, &[]);

    let (osim, oscene, omap, _oids, _og) = donut(MINUS_BACK);
    assert!(
        frame(&osim, &oscene, &omap, &xf, &[]).is_empty(),
        "a fixture não contém o fenômeno: esta receita desenha alguma coisa"
    );

    for t in [0.5, 0.999] {
        let drawn = frame(
            &sim,
            &scene,
            &map,
            &xf,
            &[morph(ids[1], None, MINUS_BACK, t)],
        );
        assert!(
            ink_between(&drawn, &union) < TOL,
            "em t={t} o desenho afastou-se da partida em {:.2} de tinta — sem chegada para onde \
             ir, ele tem de FICAR",
            ink_between(&drawn, &union)
        );
    }
}

/// ⭐ **O PRIMEIRO QUADRO DE UM MORPH DESENHA O QUE A PARTIDA DESENHA.**
///
/// ⚠️ Ele existe por um risco que a auditoria de 2026-08-23 nomeou e **não conseguiu reproduzir**:
/// o `Plan::at` decide a `fill_rule` pelo número de contornos PAREADOS (`fill_rule_for`), então um
/// resultado de contorno único (que a booleana coze `NonZero`) sai do morph como `EvenOdd` assim
/// que a chegada tiver mais contornos — medido, `Union -> Trim` em `t≈0` desenha **3** contornos
/// onde a união desenha 1, com os dois extras degenerados.
///
/// ⇒ em vez de remendar às cegas um caso que ninguém sabe produzir, a diferença fica **medida**: a
/// régua é a TINTA (a área da diferença simétrica), que é cega a contornos degenerados e a
/// `fill_rule` — e é a única coisa que o artista vê. Se um dia mudar, este gate diz onde.
#[test]
fn the_first_frame_of_a_morph_draws_what_the_start_draws() {
    let (sim, scene, map, ids, _g) = donut(0); // Union
    let xf = VecXforms::new();
    let start = frame(&sim, &scene, &map, &xf, &[]);

    // As duas chegadas com mais contornos que a partida: a que abre buraco e a receita.
    for (name, code) in [("Subtract", 1u8), ("Trim", 5)] {
        let first = frame(&sim, &scene, &map, &xf, &[morph(ids[1], None, code, 1e-6)]);
        let moved = ink_between(&start, &first);
        assert!(
            moved < TOL,
            "{name}: o primeiro quadro da transição moveu {moved:.3} de tinta — a transição \
             começa com um SALTO, que é exactamente o que ela existe para não ter"
        );
    }
}
