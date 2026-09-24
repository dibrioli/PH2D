//! Os gates do instantâneo e do dreno das secções HEALTH e DAMAGE.

use super::*;
use ph2d_ecs::{Name, SimWorld};
use ph2d_physics_ecs::{BodyKind, RigidBody};

fn mundo(health: bool, damage: bool, corpo: bool) -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let mut e = sim.world_mut().spawn(Name::new("Alvo"));
    if health {
        e.insert(Health::default());
    }
    if damage {
        e.insert(Damage::default());
    }
    if corpo {
        e.insert(RigidBody {
            kind: BodyKind::Kinematic,
        });
    }
    let bits = e.id().to_bits();
    (sim, bits)
}

/// ⚠️ **Sem vida nem dano, sem secção** (ADR-0166) — e com UMA delas, só a metade dela.
#[test]
fn a_secao_so_existe_para_quem_tem_um_dos_dois() {
    let (sim, b) = mundo(false, false, true);
    assert!(build_vida_info(sim.world(), b, 1, true).is_none());
    let (sim, b) = mundo(true, false, true);
    let i = build_vida_info(sim.world(), b, 1, true).expect("tem vida");
    assert!(i.health.is_some() && i.damage.is_none());
    let (sim, b) = mundo(false, true, false);
    let i = build_vida_info(sim.world(), b, 1, true).expect("tem dano");
    assert!(i.health.is_none() && i.damage.is_some());
    assert!(
        !i.has_body,
        "o CONTROLO: sem corpo, o painel tem de o poder dizer"
    );
}

/// ⭐⭐⭐ **Toda edição chega ao componente, e o instantâneo lê-a de volta** — ida e volta, os 26
/// campos. ⚠️ Um campo que o dreno ignorasse lia-se como um knob morto, e só a VOLTA o apanha.
#[test]
fn toda_edicao_vai_e_volta() {
    let (mut sim, b) = mundo(true, true, true);
    let edicoes = [
        E::Max(150.0),
        E::Start(80.0),
        E::InvincibleS(0.5),
        E::Overheal(true),
        E::Regen(3.0),
        E::RegenDelayS(1.5),
        E::ShieldStart(20.0),
        E::ShieldMax(40.0),
        E::ShieldDurationS(7.0),
        E::ShieldRegen(2.0),
        E::ShieldRegenDelayS(0.25),
        E::ShieldBlocksExcess(true),
        E::ArmorFlat(4.0),
        E::ArmorPercent(0.3),
        E::Dodge(0.2),
        E::Team("herois".into()),
        E::OnDamage("ai".into()),
        E::OnHeal("ufa".into()),
        E::OnDeath("morri".into()),
        E::Seed(42),
        E::DamageAmount(25.0),
        E::DamageTeam("monstros".into()),
        E::PerSecond(true),
        E::IgnoresShield(true),
        E::IgnoresArmor(true),
        E::Vanish(true),
    ];
    for e in &edicoes {
        assert!(apply(&mut sim, b, e), "{e:?} não tocou no mundo");
    }
    let i = build_vida_info(sim.world(), b, 1, true).expect("tem os dois");
    let h = i.health.expect("vida");
    let d = i.damage.expect("dano");
    assert_eq!(
        (
            h.max,
            h.start,
            h.invincible_s,
            h.overheal,
            h.regen,
            h.regen_delay_s
        ),
        (150.0, 80.0, 0.5, true, 3.0, 1.5)
    );
    assert_eq!(
        (
            h.shield_start,
            h.shield_max,
            h.shield_duration_s,
            h.shield_regen,
            h.shield_regen_delay_s,
            h.shield_blocks_excess
        ),
        (20.0, 40.0, 7.0, 2.0, 0.25, true)
    );
    assert_eq!(
        (h.armor_flat, h.armor_percent, h.dodge, h.seed),
        (4.0, 0.3, 0.2, 42)
    );
    assert_eq!(
        (
            h.team.as_str(),
            h.on_damage.as_str(),
            h.on_heal.as_str(),
            h.on_death.as_str()
        ),
        ("herois", "ai", "ufa", "morri")
    );
    assert_eq!(
        (
            d.amount,
            d.team.as_str(),
            d.per_second,
            d.ignores_shield,
            d.ignores_armor,
            d.vanish
        ),
        (25.0, "monstros", true, true, true, true)
    );
}

/// ⚠️ **As cercas da lei valem também no dreno** — o campo é alcançável por outra rota.
#[test]
fn as_cercas_da_lei_valem_no_dreno() {
    let (mut sim, b) = mundo(true, true, true);
    for e in [
        E::ArmorPercent(3.0),
        E::Dodge(-1.0),
        E::Max(-5.0),
        E::DamageAmount(f32::NAN),
    ] {
        apply(&mut sim, b, &e);
    }
    let i = build_vida_info(sim.world(), b, 1, true).expect("tem os dois");
    let h = i.health.expect("vida");
    assert_eq!((h.armor_percent, h.dodge, h.max), (1.0, 0.0, 0.0));
    assert_eq!(
        i.damage.expect("dano").amount,
        0.0,
        "um NaN não chega à lei"
    );
}

/// ⚠️ **Uma edição sem sujeito não faz nada, e DIZ que não fez** — a do dano num objecto só com
/// vida.
#[test]
fn uma_edicao_sem_sujeito_devolve_false() {
    let (mut sim, b) = mundo(true, false, true);
    assert!(!apply(&mut sim, b, &E::DamageAmount(9.0)));
    assert!(apply(&mut sim, b, &E::Max(9.0)), "o CONTROLO");
}

/// ⭐ **O readout lê o `HealthNow`** — e sem ele (antes do 1.º tique) diz `None`, nunca um zero.
#[test]
fn o_readout_le_a_vida_agora() {
    let (mut sim, b) = mundo(true, false, true);
    let i = build_vida_info(sim.world(), b, 1, true).expect("vida");
    assert!(i.health.expect("vida").agora.is_none());
    sim.world_mut()
        .entity_mut(Entity::from_bits(b))
        .insert(HealthNow {
            pontos: 70.0,
            escudo: 5.0,
            morta: false,
        });
    let i = build_vida_info(sim.world(), b, 1, true).expect("vida");
    let a = i.health.expect("vida").agora.expect("agora");
    assert_eq!((a.pontos, a.escudo, a.morta), (70.0, 5.0, false));
}
