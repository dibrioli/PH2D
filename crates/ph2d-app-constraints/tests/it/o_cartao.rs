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
    assert_eq!(escolher_alvo(&mut sim, &[mao.to_bits()]), Escolha::Gravado);
    assert!(!armado(&sim));
    quadro(&mut sim, &mut drive);
    let depois = world_transform(sim.world(), espada).unwrap();
    assert!((depois.translation - antes.translation).length() < 1e-5, "saltou ao ligar");
    assert!((depois.rotation - antes.rotation).abs() < 1e-5);
    sim.world_mut().get_mut::<Transform>(mao).unwrap().translation.x += 1.0;
    quadro(&mut sim, &mut drive);
    let seguiu = world_transform(sim.world(), espada).unwrap();
    assert!((seguiu.translation.x - antes.translation.x - 1.0).abs() < 1e-4, "não seguiu a mão");
    let i = info(&sim, b, 1).unwrap();
    assert_eq!(i.rows[0].estado, EstadoDaConstraint::Activa);
    assert_eq!(i.rows[0].tipo, 0);
}

/// O clique no próprio dono e o que fecharia um ciclo são recusados; o pick fica armado.
#[test]
fn the_owner_and_a_cycle_are_refused_and_the_pick_stays_armed() {
    let mut sim = SimWorld::new();
    let a = sim.world_mut().spawn((StableId(1), em(0.0, 0.0, 0.0), Constraints::default())).id();
    let b = sim.world_mut().spawn((StableId(2), em(5.0, 0.0, 0.0), Constraints::default())).id();
    apply_all(&mut sim, &[(a.to_bits(), E::Adicionar(1))]);
    assert_eq!(escolher_alvo(&mut sim, &[a.to_bits()]), Escolha::Recusado);
    assert!(armado(&sim));
    assert_eq!(escolher_alvo(&mut sim, &[a.to_bits(), b.to_bits()]), Escolha::Gravado, "o de baixo do dono serve");
    apply_all(&mut sim, &[(b.to_bits(), E::Adicionar(1))]);
    let antes = sim.world().get::<Constraints>(b).unwrap().clone();
    assert_eq!(escolher_alvo(&mut sim, &[a.to_bits()]), Escolha::Recusado, "A copia B e B copiaria A");
    assert!(armado(&sim));
    assert_eq!(*sim.world().get::<Constraints>(b).unwrap(), antes, "a recusa não deixa rasto");
    assert_eq!(escolher_alvo(&mut sim, &[]), Escolha::Desistiu);
    assert!(!armado(&sim));
}

/// Mover e apagar endereçam o item pelo id, nunca pela posição; o Limit Rotation nasce sem mover.
#[test]
fn edits_address_the_item_by_id_and_a_new_limit_does_not_move() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let e = sim.world_mut().spawn((StableId(1), em(0.0, 0.0, 2.9), Constraints::default())).id();
    let b = e.to_bits();
    apply_all(&mut sim, &[(b, E::Adicionar(6)), (b, E::Adicionar(5)), (b, E::Adicionar(7))]);
    let ids: Vec<u32> = sim.world().get::<Constraints>(e).unwrap().items.iter().map(|i| i.id).collect();
    assert!(apply_all(&mut sim, &[(b, E::Mover(ids[2], -1))]));
    assert!(!apply_all(&mut sim, &[(b, E::Mover(ids[0], -1))]), "o primeiro não sobe");
    let ordem: Vec<u32> = sim.world().get::<Constraints>(e).unwrap().items.iter().map(|i| i.id).collect();
    assert_eq!(ordem, vec![ids[0], ids[2], ids[1]]);
    assert!(apply_all(&mut sim, &[(b, E::Apagar(ids[1]))]));
    assert!(apply_all(&mut sim, &[(b, E::Forca(ids[0], 0.5)), (b, E::Ligada(ids[2], false))]));
    assert!(!apply_all(&mut sim, &[(b, E::Forca(ids[0], 0.5))]), "o mesmo valor não é edição");
    quadro(&mut sim, &mut drive);
    assert!((sim.world().get::<Transform>(e).unwrap().rotation - 2.9).abs() < 1e-6, "o limite novo moveu");
    let i = info(&sim, b, 1).unwrap();
    assert_eq!(i.rows.iter().map(|r| r.id).collect::<Vec<_>>(), vec![ids[0], ids[2]]);
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
    escolher_alvo(&mut sim, &[mao.to_bits()]);
    let id = sim.world().get::<Constraints>(espada).unwrap().items[0].id;
    apply_all(&mut sim, &[(b, E::Forca(id, 0.0))]);
    let anda = |sim: &mut SimWorld, drive: &mut PreviewDrive, dx: f32| {
        let pos = |sim: &SimWorld| world_transform(sim.world(), espada).unwrap();
        let antes = pos(sim);
        sim.world_mut().get_mut::<Transform>(mao).unwrap().translation.x += dx;
        quadro(sim, drive);
        quadro(sim, drive);
        (antes, pos(sim))
    };
    let (a, d) = anda(&mut sim, &mut drive, 2.0);
    assert!((d.translation - a.translation).length() < 1e-5, "a força 0 seguiu a mão");
    let antes = world_transform(sim.world(), espada).unwrap();
    apply_all(&mut sim, &[(b, E::Forca(id, 1.0))]);
    quadro(&mut sim, &mut drive);
    let pegou = world_transform(sim.world(), espada).unwrap();
    assert!((pegou.translation - antes.translation).length() < 1e-4, "saltou ao pegar");
    let (a, d) = anda(&mut sim, &mut drive, 1.5);
    assert!((d.translation.x - a.translation.x - 1.5).abs() < 1e-4, "não foi com a mão");
    apply_all(&mut sim, &[(b, E::Forca(id, 0.0))]);
    quadro(&mut sim, &mut drive);
    quadro(&mut sim, &mut drive);
    let largou = world_transform(sim.world(), espada).unwrap();
    assert!((largou.translation - d.translation).length() < 1e-4, "saltou ao largar");
    let (a, d) = anda(&mut sim, &mut drive, -4.0);
    assert!((d.translation - a.translation).length() < 1e-5, "largada, seguiu a mão");
}
