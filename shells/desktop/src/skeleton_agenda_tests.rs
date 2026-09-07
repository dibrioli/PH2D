//! Os gates da **AGENDA** — o que acontece quando há MAIS DE UMA âncora.
//!
//! ⚠️ **Corte por RESPONSABILIDADE** (o teto de 600 LOC do HR-18 pediu-o a `680`), e é o mesmo
//! corte que o `bone_gesture_tests` já pagou: o irmão [`super`] mede **uma** restrição (ela nasce
//! sem mover, persiste, a mistura, o laço, o alvo apagado, a pré-visualização); aqui mede-se o que
//! só existe com **várias** — quem manda em que osso, e em que ordem.
//!
//! ⚠️ As fixturas partilhadas (`braco`, `osso`, `ponta`, `quadro`) vivem no irmão, numa porta só:
//! uma cópia por ficheiro divergiria no primeiro ajuste.

use super::*;

/// Um braço de QUATRO ossos, para as âncoras se sobreporem. `[b0(raiz), b1, b2, b3(ponta)]`.
fn braco_de_quatro() -> (SimWorld, [Entity; 4]) {
    let mut sim = SimWorld::default();
    let b0 = osso(&mut sim, "B0", [0.0, 0.0], 10.0, None);
    let b1 = osso(&mut sim, "B1", [10.0, 0.0], 10.0, Some(b0));
    let b2 = osso(&mut sim, "B2", [10.0, 0.0], 10.0, Some(b1));
    let b3 = osso(&mut sim, "B3", [10.0, 0.0], 10.0, Some(b2));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, [b0, b1, b2, b3])
}

/// ⭐⭐⭐ **DUAS ÂNCORAS NA MESMA CADEIA NÃO BRIGAM** (report do dono, 2026-09-07: *«múltiplos IKs
/// numa cadeia de bones tem resultado ruim»*).
///
/// ⚠️ Duas propriedades de PRODUTO: a cena assenta, e **cada âncora alcança o alvo dela**.
///
/// ⛔ **Este gate NÃO discrimina contra a ordem crua nesta fixtura, e isso está escrito de
/// propósito** — medido: sem agenda ele fica verde, porque o FABRIK parte da pose que encontra e
/// preserva o trabalho da âncora anterior. Quem discrimina é o irmão
/// [`the_plan_does_not_depend_on_the_order_the_anchors_are_found`]. ⭐ Ele fica na mesma, porque
/// defende o que o artista vê contra uma mudança FUTURA na lei — *um gate que hoje não separa duas
/// implementações ainda separa a certa de uma terceira*.
#[test]
fn two_anchors_on_one_chain_reach_a_fixed_point() {
    let (mut sim, [_, b1, _, b3]) = braco_de_quatro();
    // ⚠️⚠️ **A DE BAIXO NASCE PRIMEIRO, e isso é a metade que faz a fixtura discriminar.** Sem
    // agenda a ordem é a dos ARQUÉTIPOS, que segue a ordem em que o `IkGoal` foi inserido — e com a
    // de cima primeiro o resultado calha ser BOM, porque o FABRIK preserva a pose que encontra.
    // *Uma fixtura cuja ordem arbitrária calha ser a certa não testa a ordem.*
    add(&mut sim, b3).expect("a ancora de baixo");
    add(&mut sim, b1).expect("a ancora de cima");
    // ⚠️⚠️ **A SOBREPOSIÇÃO tem de ser construída**: com o `chain` de nascimento (`2`) as duas
    // correntes são `[b0,b1]` e `[b2,b3]` — **disjuntas por acaso**, e a cena converge mesmo sem
    // agenda nenhuma. A 1.ª redacção deste gate passava sob a mutação por isso. *Uma fixtura que
    // não produz o fenómeno devolve verde sobre o defeito.*
    sim.world_mut().get_mut::<IkGoal>(b3).expect("g").chain = 4;
    drag_anchor(&mut sim, b1, [6.0, 8.0]);
    drag_anchor(&mut sim, b3, [12.0, 22.0]);
    let mut pv = PreviewDrive::default();
    for _ in 0..30 {
        solve(&mut sim, &mut pv);
    }
    let assente = crate::skeleton_live::bone_segments(&sim);
    for _ in 0..30 {
        solve(&mut sim, &mut pv);
    }
    // ⭐⭐⭐ **E AS DUAS ALCANÇAM O PRÓPRIO ALVO** — é isto que o report chama de *«resultado bom»*,
    // e é a propriedade que a convergência sozinha NÃO mede: sem agenda a cena também assenta,
    // numa pose em que **uma das âncoras é ignorada**.
    let orcamento = 40.0 * f64::from(f32::EPSILON);
    for (osso, alvo) in [(b1, [6.0, 8.0]), (b3, [12.0, 22.0])] {
        let p = ponta(&sim, osso);
        let erro = (p[0] - alvo[0]).hypot(p[1] - alvo[1]);
        assert!(
            erro < orcamento,
            "a ponta de {osso:?} ficou a {erro} do alvo dela - a outra ancora atropelou-a"
        );
    }
    let pior = assente
        .iter()
        .zip(crate::skeleton_live::bone_segments(&sim))
        .flat_map(|(a, b)| {
            [
                (a.1[0] - b.1[0]).hypot(a.1[1] - b.1[1]),
                (a.2[0] - b.2[0]).hypot(a.2[1] - b.2[1]),
            ]
        })
        .fold(0.0_f64, f64::max);
    // ⚠️ **A barra é DERIVADA do recurso, não escolhida** (§0.0): a pose viaja pela rotação do
    // `Transform` da casa, que é **`f32`**, e uma corrente de `40` unidades assenta a
    // `40 × 1,19e-7 = 4,8e-6`. **Medido: `~2e-6`.**
    //
    // ⚠️⚠️ **E a igualdade EXACTA seria a barra errada aqui, ao contrário do que a 1.ª redacção
    // deste gate assumiu.** Uma corrente de 2 ossos assenta ao bit (a lei é fechada e o atalho da
    // recta é exacto); uma de 4 corre o FABRIK, que converge **assimptoticamente** — ela pousa
    // dentro do orçamento e nunca em cima dele. *Um gate que exige o zero de uma lei iterativa mede
    // o algoritmo, não o defeito.*
    assert!(
        pior < orcamento,
        "a cena andou {pior} depois de 30 passes (orcamento do f32: {orcamento}) - as duas ancoras \
         atropelam-se"
    );
}

/// ⭐⭐ **AS CORRENTES SÃO DISJUNTAS** — um osso obedece a UMA âncora.
///
/// A de baixo é a mais **específica** e reclama primeiro; a de cima fica com o que sobra. ⛔ Sem
/// isto as duas escrevem a mesma rotação em sequência, todo quadro.
#[test]
fn a_bone_obeys_exactly_one_anchor() {
    let (mut sim, [b0, b1, b2, b3]) = braco_de_quatro();
    // A de cima pede 2 (b0,b1); a de baixo pede 4 (b0..b3) e, sendo mais funda, reclama primeiro.
    add(&mut sim, b1).expect("cima");
    add(&mut sim, b3).expect("baixo");
    sim.world_mut().get_mut::<IkGoal>(b3).expect("g").chain = 4;
    let plano = schedule(
        &sim,
        vec![
            (b1, *sim.world().get::<IkGoal>(b1).expect("g")),
            (b3, *sim.world().get::<IkGoal>(b3).expect("g")),
        ],
        false,
    );
    let mut vistos: Vec<Entity> = plano.iter().flat_map(|(_, _, c)| c.clone()).collect();
    let n = vistos.len();
    vistos.sort();
    vistos.dedup();
    assert_eq!(n, vistos.len(), "um osso apareceu em DUAS correntes");
    // ⭐ A de CIMA reclama primeiro (é a mais rasa) e leva `[b0,b1]`; a de baixo fica com o que
    // sobra ABAIXO dela. ⛔ Ao contrário, a funda levava a corrente inteira e a de cima ficava
    // **inerte** — duas âncoras e uma delas sem efeito nenhum.
    let de_cima = plano.iter().find(|(t, ..)| *t == b1).expect("cima");
    assert_eq!(de_cima.2, vec![b0, b1], "a mais RASA reclama primeiro");
    let de_baixo = plano.iter().find(|(t, ..)| *t == b3).expect("baixo");
    assert_eq!(
        de_baixo.2,
        vec![b2, b3],
        "a funda fica com o que sobra ABAIXO"
    );
}

/// ⭐⭐⭐ **O PLANO NÃO DEPENDE DA ORDEM EM QUE AS ÂNCORAS SÃO ENCONTRADAS** — e é ESTA a propriedade
/// que a agenda compra.
///
/// ⛔⛔ **É também a única que discrimina, e descobri-lo custou três fixturas.** As duas primeiras
/// mediam *convergir* e *alcançar o alvo* — e a cena satisfaz as duas **mesmo sem agenda**, porque
/// o FABRIK parte da pose que encontra e tende a **preservar** o trabalho da âncora anterior. ⇒ o
/// defeito do report não é *«a cena não assenta»*: é *«ela assenta numa pose que depende da ordem
/// dos ARQUÉTIPOS»*, e essa ordem não é estável entre sessões. *Uma fixtura cuja ordem arbitrária
/// calha ser a certa não testa a ordem.*
///
/// ⚠️ O gate compara as **duas permutações** e exige o plano idêntico — a ordem de resolução **e** a
/// corrente de cada uma. Sem agenda o plano **é** a ordem da entrada, e as duas permutações diferem
/// por construção.
#[test]
fn the_plan_does_not_depend_on_the_order_the_anchors_are_found() {
    let (mut sim, [b0, b1, b2, b3]) = braco_de_quatro();
    add(&mut sim, b1).expect("cima");
    add(&mut sim, b3).expect("baixo");
    sim.world_mut().get_mut::<IkGoal>(b3).expect("g").chain = 4;
    let par = |a: Entity, b: Entity| {
        vec![
            (a, *sim.world().get::<IkGoal>(a).expect("g")),
            (b, *sim.world().get::<IkGoal>(b).expect("g")),
        ]
    };
    let so_o_essencial = |p: Vec<(Entity, IkGoal, Vec<Entity>)>| {
        p.into_iter().map(|(t, _, c)| (t, c)).collect::<Vec<_>>()
    };
    let numa = so_o_essencial(schedule(&sim, par(b1, b3), false));
    let noutra = so_o_essencial(schedule(&sim, par(b3, b1), false));
    assert_eq!(
        numa, noutra,
        "o plano mudou com a ordem da entrada - a pose passa a depender do arquetipo, e ele nao e' \
         estavel entre sessoes"
    );
    // E o plano é o que a lei diz: a rasa primeiro, com o segmento dela; a funda com o resto.
    assert_eq!(numa, vec![(b1, vec![b0, b1]), (b3, vec![b2, b3])]);
}
