//! Gates do instantâneo e do dreno da secção PROJECTILE MOTION.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{Collider, ColliderShape};

fn cena() -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Bala"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.1 },
                ..Collider::default()
            },
            ProjectileMotion::default(),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    (sim, e.to_bits())
}

/// ⚠️ **Sem o componente não há secção** (ADR-0166).
#[test]
fn sem_o_componente_nao_ha_seccao() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(Transform::IDENTITY).id();
    assert!(build_projectile_info(sim.world(), e.to_bits(), 1, true, false).is_none());
}

/// ⭐ **O instantâneo lê a LEI, não os defaults do painel.**
#[test]
fn o_instantaneo_le_a_lei() {
    let (mut sim, bits) = cena();
    {
        let e = Entity::from_bits(bits);
        let mut c = sim.world_mut().get_mut::<ProjectileMotion>(e).expect("tem");
        c.gravity = 9.81;
        c.range = 12.5;
    }
    let i = build_projectile_info(sim.world(), bits, 1, true, false).expect("tem o componente");
    assert!((i.gravity - 9.81).abs() < 1.0e-5);
    assert!((i.range - 12.5).abs() < 1.0e-5);
    assert!(i.has_body && i.body_is_kinematic);
}

/// ⭐⭐ **O ALVO é um NOME nos DOIS sentidos**: texto → hash → texto.
#[test]
fn o_alvo_atravessa_como_nome_nos_dois_sentidos() {
    let (mut sim, bits) = cena();
    sim.world_mut()
        .spawn((Name::new("Heroi"), Transform::IDENTITY));
    assert!(apply_projectile_edit(
        sim.world_mut(),
        bits,
        &ProjectileFieldEdit::HomingTarget("Heroi".into())
    ));
    let i = build_projectile_info(sim.world(), bits, 1, true, false).unwrap();
    assert_eq!(i.homing_target, "Heroi");
    assert!(!i.homing_target_missing);
}

/// ⚠️⚠️ **Um alvo APAGADO não é um alvo VAZIO** — e o painel tem de poder dizer a diferença.
#[test]
fn um_alvo_apagado_nao_e_um_alvo_vazio() {
    let (mut sim, bits) = cena();
    let alvo = sim
        .world_mut()
        .spawn((Name::new("Heroi"), Transform::IDENTITY))
        .id();
    apply_projectile_edit(
        sim.world_mut(),
        bits,
        &ProjectileFieldEdit::HomingTarget("Heroi".into()),
    );
    sim.world_mut().despawn(alvo);
    let i = build_projectile_info(sim.world(), bits, 1, true, false).unwrap();
    assert!(
        i.homing_target_missing,
        "o alvo sumiu e o painel tem de o dizer"
    );

    // E o CONTROLO: sem alvo escrito, não há falta nenhuma.
    let (sim2, bits2) = cena();
    let i2 = build_projectile_info(sim2.world(), bits2, 1, true, false).unwrap();
    assert!(i2.homing_target.is_empty() && !i2.homing_target_missing);
}

/// ⚠️ **Vazio é ZERO, e não o hash de uma string vazia** — a convenção de «ninguém» desta casa.
#[test]
fn limpar_o_alvo_escreve_zero() {
    let (mut sim, bits) = cena();
    sim.world_mut()
        .spawn((Name::new("Heroi"), Transform::IDENTITY));
    apply_projectile_edit(
        sim.world_mut(),
        bits,
        &ProjectileFieldEdit::HomingTarget("Heroi".into()),
    );
    apply_projectile_edit(
        sim.world_mut(),
        bits,
        &ProjectileFieldEdit::HomingTarget("   ".into()),
    );
    let c = sim
        .world()
        .get::<ProjectileMotion>(Entity::from_bits(bits))
        .unwrap();
    assert_eq!(c.homing_target, 0, "so' espacos e' ninguem");
}

/// ⚠️⚠️ **As cercas do painel valem TAMBÉM no dreno** (script, ficheiro).
#[test]
fn as_cercas_do_painel_valem_no_dreno() {
    let (mut sim, bits) = cena();
    apply_projectile_edit(sim.world_mut(), bits, &ProjectileFieldEdit::Bounciness(4.0));
    let c = sim
        .world()
        .get::<ProjectileMotion>(Entity::from_bits(bits))
        .unwrap();
    assert!((c.bounciness - 1.0).abs() < 1.0e-6, "{}", c.bounciness);
    apply_projectile_edit(
        sim.world_mut(),
        bits,
        &ProjectileFieldEdit::Bounciness(-2.0),
    );
    let c = sim
        .world()
        .get::<ProjectileMotion>(Entity::from_bits(bits))
        .unwrap();
    assert!(c.bounciness.abs() < 1.0e-6);
}

/// ⚠️ **A aceleração pode ser NEGATIVA**: a bala trava no ar.
#[test]
fn a_aceleracao_pode_ser_negativa() {
    let (mut sim, bits) = cena();
    apply_projectile_edit(
        sim.world_mut(),
        bits,
        &ProjectileFieldEdit::Acceleration(-30.0),
    );
    let c = sim
        .world()
        .get::<ProjectileMotion>(Entity::from_bits(bits))
        .unwrap();
    assert!((c.acceleration + 30.0).abs() < 1.0e-5, "{}", c.acceleration);
}

/// ⚠️ Uma entidade que já não existe não parte o dreno.
#[test]
fn uma_entidade_morta_nao_parte_o_dreno() {
    let (mut sim, bits) = cena();
    sim.world_mut().despawn(Entity::from_bits(bits));
    assert!(!apply_projectile_edit(
        sim.world_mut(),
        bits,
        &ProjectileFieldEdit::Range(5.0)
    ));
}

fn lei(sim: &SimWorld, bits: u64) -> ph2d_projectile::ProjectileLaw {
    sim.world()
        .get::<ProjectileMotion>(Entity::from_bits(bits))
        .expect("tem")
        .law()
}

/// ⭐ GEF 06 — as caixas Bounce e Homing escrevem os números: ligar põe o valor de fábrica, desligar zera.
#[test]
fn as_caixas_bounce_e_homing_escrevem_os_numeros() {
    let (mut sim, bits) = cena();
    for (edit, ricochetes, puxa) in [
        (ProjectileFieldEdit::Bounce(true), BOUNCES_AO_LIGAR, 0.0),
        (
            ProjectileFieldEdit::Homing(true),
            BOUNCES_AO_LIGAR,
            PULL_AO_LIGAR,
        ),
        (ProjectileFieldEdit::Bounce(false), 0, PULL_AO_LIGAR),
        (ProjectileFieldEdit::Homing(false), 0, 0.0),
    ] {
        assert!(apply_projectile_edit(sim.world_mut(), bits, &edit));
        let l = lei(&sim, bits);
        assert_eq!(l.max_bounces, ricochetes, "{edit:?}");
        assert!((l.homing_accel - puxa).abs() < 1.0e-6, "{edit:?}");
    }
}

/// ⭐ GEF 06 — o conta-gotas: armar não toca no documento e só ESTA bala lê «armado»; o clique no palco escolhe o de
/// cima com nome (não a bala) pela porta do Inspector e desarma; o vazio só desarma; desarmado, o clique não é dele.
#[test]
fn o_conta_gotas_arma_escolhe_e_desarma() {
    let (mut sim, bits) = cena();
    let w = sim.world_mut();
    let outra = w.spawn(ProjectileMotion::default()).id().to_bits();
    let heroi = w.spawn(Name::new("Hero")).id().to_bits();
    let mut bus = ph2d_editor_core::action_bus::ActionBus::default();
    assert!(
        !escolher_alvo(sim.world_mut(), &mut bus, &[heroi]),
        "desarmado"
    );
    let antes = lei(&sim, bits);
    for (debaixo, escolhe) in [(vec![bits, heroi], true), (vec![], false)] {
        let arma = ProjectileFieldEdit::PickTarget(true);
        assert!(!apply_projectile_edit(sim.world_mut(), bits, &arma));
        assert_eq!(lei(&sim, bits), antes, "armar não toca no documento");
        let info = |b| build_projectile_info(sim.world(), b, 1, true, false).expect("tem");
        assert!(info(bits).target_pick_armed && !info(outra).target_pick_armed);
        assert!(escolher_alvo(sim.world_mut(), &mut bus, &debaixo));
        assert!(!armado(sim.world()) && desarma(sim.world_mut()).is_none());
        let nome = ProjectileFieldEdit::HomingTarget("Hero".into());
        let esperado = escolhe.then_some(EditorAction::InspectorComponentEdit {
            entity_bits: bits,
            edit: ComponentEdit::Projectile(nome),
        });
        assert_eq!(bus.drain().collect::<Vec<_>>(), Vec::from_iter(esperado));
    }
}
