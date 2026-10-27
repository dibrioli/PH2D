//! ⭐ **O cartão, lado da família** — o que cada pedido faz ao documento.

use ph2d_app_constraints::inspector::{Escolha, apply_all, armado, escolher_alvo, info};
use ph2d_constraints_ecs::{Constraints, evaluate};
use ph2d_ecs::{SimWorld, StableId, Transform, world_transform};
use ph2d_editor_core::constraint_edits::{ConstraintFieldEdit as E, EstadoDaConstraint};
use ph2d_preview_drive::PreviewDrive;

fn em(x: f32, y: f32, rot: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t.rotation = rot;
    t
}

fn quadro(sim: &mut SimWorld, drive: &mut PreviewDrive) {
    evaluate(sim, drive);
    drive.settle();
}

/// Acrescentar um Child Of e escolher o alvo no palco: zero salto; mexer o alvo arrasta o dono.
#[test]
fn picking_a_child_of_target_never_jumps_and_then_follows() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let mao = sim.world_mut().spawn((StableId(1), em(3.0, 1.0, 0.7))).id();
    let espada = sim
        .world_mut()
        .spawn((StableId(2), em(-2.0, 0.5, -0.3), Constraints::default()))
        .id();
    let b = espada.to_bits();
    assert!(apply_all(&mut sim, &[(b, E::Adicionar(0))]));
    assert!(armado(&sim), "o Child Of nasce à espera do alvo");
    let antes = world_transform(sim.world(), espada).unwrap();
    assert_eq!(
        escolher_alvo(&mut sim, &[mao.to_bits()], [0.0, 0.0]),
        Escolha::Gravado
    );
    assert!(!armado(&sim));
    quadro(&mut sim, &mut drive);
    let depois = world_transform(sim.world(), espada).unwrap();
    assert!(
        (depois.translation - antes.translation).length() < 1e-5,
        "saltou ao ligar"
    );
    assert!((depois.rotation - antes.rotation).abs() < 1e-5);
    sim.world_mut()
        .get_mut::<Transform>(mao)
        .unwrap()
        .translation
        .x += 1.0;
    quadro(&mut sim, &mut drive);
    let seguiu = world_transform(sim.world(), espada).unwrap();
    assert!(
        (seguiu.translation.x - antes.translation.x - 1.0).abs() < 1e-4,
        "não seguiu a mão"
    );
    let i = info(&sim, b, 1).unwrap();
    assert_eq!(i.rows[0].estado, EstadoDaConstraint::Activa);
    assert_eq!(i.rows[0].tipo, 0);
}

/// O clique no próprio dono e o que fecharia um ciclo são recusados; o pick fica armado.
#[test]
fn the_owner_and_a_cycle_are_refused_and_the_pick_stays_armed() {
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((StableId(1), em(0.0, 0.0, 0.0), Constraints::default()))
        .id();
    let b = sim
        .world_mut()
        .spawn((StableId(2), em(5.0, 0.0, 0.0), Constraints::default()))
        .id();
    apply_all(&mut sim, &[(a.to_bits(), E::Adicionar(1))]);
    assert_eq!(
        escolher_alvo(&mut sim, &[a.to_bits()], [0.0, 0.0]),
        Escolha::Recusado
    );
    assert!(armado(&sim));
    assert_eq!(
        escolher_alvo(&mut sim, &[a.to_bits(), b.to_bits()], [0.0, 0.0]),
        Escolha::Gravado,
        "o de baixo do dono serve"
    );
    apply_all(&mut sim, &[(b.to_bits(), E::Adicionar(1))]);
    let antes = sim.world().get::<Constraints>(b).unwrap().clone();
    assert_eq!(
        escolher_alvo(&mut sim, &[a.to_bits()], [0.0, 0.0]),
        Escolha::Recusado,
        "A copia B e B copiaria A"
    );
    assert!(armado(&sim));
    assert_eq!(
        *sim.world().get::<Constraints>(b).unwrap(),
        antes,
        "a recusa não deixa rasto"
    );
    assert_eq!(escolher_alvo(&mut sim, &[], [0.0, 0.0]), Escolha::Desistiu);
    assert!(!armado(&sim));
}

/// Mover e apagar endereçam o item pelo id, nunca pela posição; o Limit Rotation nasce sem mover.
#[test]
fn edits_address_the_item_by_id_and_a_new_limit_does_not_move() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let e = sim
        .world_mut()
        .spawn((StableId(1), em(0.0, 0.0, 2.9), Constraints::default()))
        .id();
    let b = e.to_bits();
    apply_all(
        &mut sim,
        &[
            (b, E::Adicionar(6)),
            (b, E::Adicionar(5)),
            (b, E::Adicionar(7)),
        ],
    );
    let ids: Vec<u32> = sim
        .world()
        .get::<Constraints>(e)
        .unwrap()
        .items
        .iter()
        .map(|i| i.id)
        .collect();
    assert!(apply_all(&mut sim, &[(b, E::Mover(ids[2], -1))]));
    assert!(
        !apply_all(&mut sim, &[(b, E::Mover(ids[0], -1))]),
        "o primeiro não sobe"
    );
    let ordem: Vec<u32> = sim
        .world()
        .get::<Constraints>(e)
        .unwrap()
        .items
        .iter()
        .map(|i| i.id)
        .collect();
    assert_eq!(ordem, vec![ids[0], ids[2], ids[1]]);
    assert!(apply_all(&mut sim, &[(b, E::Apagar(ids[1]))]));
    assert!(apply_all(
        &mut sim,
        &[(b, E::Forca(ids[0], 0.5)), (b, E::Ligada(ids[2], false))]
    ));
    assert!(
        !apply_all(&mut sim, &[(b, E::Forca(ids[0], 0.5))]),
        "o mesmo valor não é edição"
    );
    quadro(&mut sim, &mut drive);
    assert!(
        (sim.world().get::<Transform>(e).unwrap().rotation - 2.9).abs() < 1e-6,
        "o limite novo moveu"
    );
    let i = info(&sim, b, 1).unwrap();
    assert_eq!(
        i.rows.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![ids[0], ids[2]]
    );
    assert_eq!(i.rows[1].estado, EstadoDaConstraint::Desligada);
    assert!(!i.rows[0].precisa_alvo);
}

/// ⭐ **Pegar e largar sem salto** (plano §5 `child_of_never_jumps_when_switched_on_or_off`): a mão
/// anda com a força a 0, pega (força 100 %), anda, larga (força 0) e anda — a espada nunca salta, e
/// depois de largada fica onde a mão a deixou.
#[test]
fn a_child_of_picks_up_and_drops_without_a_jump() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let mao = sim.world_mut().spawn((StableId(1), em(3.0, 1.0, 0.7))).id();
    let espada = sim
        .world_mut()
        .spawn((StableId(2), em(-2.0, 0.5, -0.3), Constraints::default()))
        .id();
    let b = espada.to_bits();
    apply_all(&mut sim, &[(b, E::Adicionar(0))]);
    escolher_alvo(&mut sim, &[mao.to_bits()], [0.0, 0.0]);
    let id = sim.world().get::<Constraints>(espada).unwrap().items[0].id;
    apply_all(&mut sim, &[(b, E::Forca(id, 0.0))]);
    let anda = |sim: &mut SimWorld, drive: &mut PreviewDrive, dx: f32| {
        let pos = |sim: &SimWorld| world_transform(sim.world(), espada).unwrap();
        let antes = pos(sim);
        sim.world_mut()
            .get_mut::<Transform>(mao)
            .unwrap()
            .translation
            .x += dx;
        quadro(sim, drive);
        quadro(sim, drive);
        (antes, pos(sim))
    };
    let (a, d) = anda(&mut sim, &mut drive, 2.0);
    assert!(
        (d.translation - a.translation).length() < 1e-5,
        "a força 0 seguiu a mão"
    );
    let antes = world_transform(sim.world(), espada).unwrap();
    apply_all(&mut sim, &[(b, E::Forca(id, 1.0))]);
    quadro(&mut sim, &mut drive);
    let pegou = world_transform(sim.world(), espada).unwrap();
    assert!(
        (pegou.translation - antes.translation).length() < 1e-4,
        "saltou ao pegar"
    );
    let (a, d) = anda(&mut sim, &mut drive, 1.5);
    assert!(
        (d.translation.x - a.translation.x - 1.5).abs() < 1e-4,
        "não foi com a mão"
    );
    apply_all(&mut sim, &[(b, E::Forca(id, 0.0))]);
    quadro(&mut sim, &mut drive);
    quadro(&mut sim, &mut drive);
    let largou = world_transform(sim.world(), espada).unwrap();
    assert!(
        (largou.translation - d.translation).length() < 1e-4,
        "saltou ao largar"
    );
    let (a, d) = anda(&mut sim, &mut drive, -4.0);
    assert!(
        (d.translation - a.translation).length() < 1e-5,
        "largada, seguiu a mão"
    );
}

/// O Max escrito ABAIXO do Min sobe ao Min (o espelho do Min que passa do Max): a faixa nunca se
/// inverte pelo campo do Max.
///
/// **Mutação que deve sangrar:** tirar o `.max(r[0])` da escrita do `LIMITE_MAX`.
#[test]
fn a_max_written_below_the_min_rises_to_the_min() {
    use ph2d_constraints::Kind;
    use ph2d_ecs::canal_constraint::LIMITE_MAX;
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((StableId(2), em(0.0, 0.0, 0.0), Constraints::default()))
        .id();
    let b = e.to_bits();
    assert!(apply_all(&mut sim, &[(b, E::Adicionar(6))]));
    let id = sim.world().get::<Constraints>(e).unwrap().items[0].id;
    assert!(apply_all(&mut sim, &[(b, E::Campo(id, LIMITE_MAX, -60.0))]));
    let c = sim.world().get::<Constraints>(e).unwrap();
    let Kind::Limit(l) = c.item(id).unwrap().kind else {
        panic!()
    };
    let [lo, hi] = l.rot.unwrap();
    assert!(
        (lo + 45f64.to_radians()).abs() < 1e-9 && hi == lo,
        "o Max desceu abaixo do Min: {lo} {hi}"
    );
}

/// ⭐⭐ **Os ajustes de cada tipo** (report do dono 25/10): o cartão mostra-os nas unidades dele e
/// cada edição chega ao item certo — o Min nunca passa do Max, a distância nunca é negativa, a
/// opção vai à válida; «Reset Offset» cola o Child Of ao alvo.
#[test]
fn the_settings_of_each_type_reach_the_item() {
    use ph2d_constraints::{DistanceMode, Kind, Mix, Space};
    use ph2d_ecs::canal_constraint::{
        DESVIO, DISTANCIA, ESPACO, LIMITE_MAX, LIMITE_MIN, MISTURA, MODO,
    };
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let mao = sim.world_mut().spawn((StableId(1), em(3.0, 1.0, 0.7))).id();
    let e = sim
        .world_mut()
        .spawn((StableId(2), em(0.0, 0.0, 0.0), Constraints::default()))
        .id();
    let b = e.to_bits();
    apply_all(
        &mut sim,
        &[
            (b, E::Adicionar(6)),
            (b, E::Adicionar(7)),
            (b, E::Adicionar(1)),
            (b, E::Adicionar(5)),
            (b, E::Adicionar(0)),
        ],
    );
    let ids: Vec<u32> = sim
        .world()
        .get::<Constraints>(e)
        .unwrap()
        .items
        .iter()
        .map(|i| i.id)
        .collect();
    let i = info(&sim, b, 1).unwrap();
    let campos = |k: usize| {
        i.rows[k]
            .numeros
            .iter()
            .map(|n| n.campo)
            .chain(i.rows[k].escolhas.iter().map(|e| e.campo))
            .collect::<Vec<_>>()
    };
    assert_eq!(campos(0), vec![LIMITE_MIN, LIMITE_MAX, ESPACO]);
    assert_eq!(campos(1), vec![DISTANCIA, MODO]);
    assert_eq!(campos(2), vec![MISTURA, ESPACO]);
    assert_eq!(campos(3), vec![DESVIO]);
    assert!(campos(4).is_empty() && i.rows[4].zera_desvio && !i.rows[0].zera_desvio);
    assert!(
        (i.rows[0].numeros[1].valor - 45.0).abs() < 1e-4,
        "o Max nasce a +45° em graus"
    );

    assert!(apply_all(
        &mut sim,
        &[
            (b, E::Campo(ids[0], LIMITE_MAX, 10.0)),
            (b, E::Campo(ids[0], LIMITE_MIN, 20.0))
        ]
    ));
    assert!(apply_all(
        &mut sim,
        &[
            (b, E::Campo(ids[1], DISTANCIA, -3.0)),
            (b, E::Campo(ids[1], MODO, 7.0))
        ]
    ));
    assert!(apply_all(
        &mut sim,
        &[
            (b, E::Campo(ids[2], MISTURA, 1.0)),
            (b, E::Campo(ids[2], ESPACO, 1.0))
        ]
    ));
    assert!(apply_all(&mut sim, &[(b, E::Campo(ids[3], DESVIO, 90.0))]));
    assert!(
        !apply_all(&mut sim, &[(b, E::Campo(ids[3], DISTANCIA, 1.0))]),
        "o Look At não tem distância"
    );
    let c = sim.world().get::<Constraints>(e).unwrap().clone();
    let Kind::Limit(l) = c.item(ids[0]).unwrap().kind else {
        panic!()
    };
    let [lo, hi] = l.rot.unwrap();
    assert!(
        (hi - 10f64.to_radians()).abs() < 1e-9 && lo == hi,
        "o Min passou do Max: {lo} {hi}"
    );
    let Kind::Distance(d) = c.item(ids[1]).unwrap().kind else {
        panic!()
    };
    assert_eq!((d.distance, d.mode), (0.0, DistanceMode::Surface));
    let Kind::CopyTransform(cp) = c.item(ids[2]).unwrap().kind else {
        panic!()
    };
    assert_eq!((cp.mix, cp.space), (Mix::Add, Space::Local));
    let Kind::LookAt(la) = c.item(ids[3]).unwrap().kind else {
        panic!()
    };
    assert!((la.offset - std::f64::consts::FRAC_PI_2).abs() < 1e-6);

    // Só o Child Of, com o alvo, e zerado: o dono cola-se à mão.
    for &id in &ids[..4] {
        apply_all(&mut sim, &[(b, E::Apagar(id))]);
    }
    escolher_alvo(&mut sim, &[mao.to_bits()], [0.0, 0.0]);
    quadro(&mut sim, &mut drive);
    assert!(
        (world_transform(sim.world(), e).unwrap().translation.x).abs() < 1e-4,
        "ligar saltou"
    );
    assert!(apply_all(&mut sim, &[(b, E::ZerarDesvio(ids[4]))]));
    assert!(
        !apply_all(&mut sim, &[(b, E::ZerarDesvio(ids[4]))]),
        "zerar duas vezes não é edição"
    );
    quadro(&mut sim, &mut drive);
    let colado = world_transform(sim.world(), e).unwrap();
    assert!(
        (colado.translation.x - 3.0).abs() < 1e-4 && (colado.rotation - 0.7).abs() < 1e-5,
        "{colado:?}"
    );
}
