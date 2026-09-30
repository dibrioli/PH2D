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

/// ⭐⭐⭐ **A BARRA vai e volta, os onze campos** (plano 28, W4) — e um placar SÓ com barra tem
/// secção sem ter vida nem corpo.
///
/// **Mutações que devem sangrar:** tirar qualquer braço do `apply_bar` · o `build_vida_info` voltar
/// a exigir vida ou dano.
#[test]
fn a_barra_vai_e_volta_e_um_placar_so_com_barra_tem_seccao() {
    let mut sim = SimWorld::new();
    let placar = sim.world_mut().spawn(HealthBar::default()).id();
    let b = placar.to_bits();
    let i = build_vida_info(sim.world(), b, 1, true).expect("um placar SÓ com barra tem secção");
    assert!(i.health.is_none() && i.damage.is_none() && i.bar.is_some());
    assert_eq!(
        i.queixa(),
        None,
        "uma barra sozinha não se queixa de corpo — ela não o precisa"
    );
    let edicoes = [
        E::BarTarget("  Heroi ".into()),
        E::BarWidth(2.0),
        E::BarHeight(0.3),
        E::BarOffsetX(-0.5),
        E::BarOffsetY(-1.5),
        E::BarFill([0.1, 0.2, 0.3, 1.0]),
        E::BarTrail([0.9, 0.8, 0.7, 1.0]),
        E::BarBack([0.0, 0.0, 0.0, 0.5]),
        E::BarTrailDelayS(0.9),
        E::BarTrailSpeed(3.0),
        E::BarHideWhenFull(true),
    ];
    for e in &edicoes {
        assert!(apply(&mut sim, b, e), "a edição {e:?} não tocou na barra");
    }
    let lida = build_vida_info(sim.world(), b, 1, true)
        .unwrap()
        .bar
        .unwrap();
    assert_eq!(lida.target, "Heroi", "o nome é aparado");
    assert_eq!(
        (lida.width, lida.height, lida.offset_x, lida.offset_y),
        (2.0, 0.3, -0.5, -1.5),
        "⚠️ os deslocamentos NEGATIVOS ficam — uma barra pode ir abaixo e à esquerda"
    );
    assert_eq!(lida.fill, [0.1, 0.2, 0.3, 1.0]);
    assert_eq!(lida.trail, [0.9, 0.8, 0.7, 1.0]);
    assert_eq!(lida.back, [0.0, 0.0, 0.0, 0.5]);
    assert_eq!((lida.trail_delay_s, lida.trail_speed), (0.9, 3.0));
    assert!(lida.hide_when_full);
}

/// ⚠️ **As cercas da barra** — um `NaN` num deslocamento cai a zero, um tamanho negativo a zero, e
/// um canal fora de `0..1` é preso (outra rota que não o painel não pode pôr lixo no tinte).
#[test]
fn as_cercas_da_barra() {
    let mut sim = SimWorld::new();
    let b = sim.world_mut().spawn(HealthBar::default()).id().to_bits();
    apply(&mut sim, b, &E::BarOffsetY(f32::NAN));
    apply(&mut sim, b, &E::BarWidth(-3.0));
    apply(&mut sim, b, &E::BarFill([2.0, -1.0, f32::NAN, 0.5]));
    let lida = build_vida_info(sim.world(), b, 1, true)
        .unwrap()
        .bar
        .unwrap();
    assert_eq!(lida.offset_y, 0.0);
    assert_eq!(lida.width, 0.0);
    assert_eq!(lida.fill, [1.0, 0.0, 0.0, 0.5]);
}

/// ⭐⭐⭐ **O que a barra ENCONTROU sai das MESMAS portas que a ponte usa ao desenhar** — as três
/// respostas, e o molde homónimo que não conta.
///
/// **Mutação que deve sangrar:** o `bar_info` resolver o alvo por outra via (ex.: aceitar moldes).
#[test]
fn o_painel_diz_o_que_a_barra_encontrou() {
    use ph2d_ecs::MasterPiece;
    let mut sim = SimWorld::new();
    let placar = sim
        .world_mut()
        .spawn(HealthBar {
            target: "Heroi".into(),
            ..HealthBar::default()
        })
        .id();
    let b = placar.to_bits();
    let alvo = |sim: &SimWorld| {
        build_vida_info(sim.world(), b, 1, true)
            .unwrap()
            .bar
            .unwrap()
            .alvo
    };
    assert_eq!(alvo(&sim), BarraAlvo::SemAlvo, "ninguém com esse nome");
    // Um MOLDE com o nome e com vida não conta — a ponte também não o desenharia.
    sim.world_mut()
        .spawn((Name::new("Heroi"), Health::default(), MasterPiece));
    assert_eq!(alvo(&sim), BarraAlvo::SemAlvo, "um molde não é o alvo");
    let heroi = sim.world_mut().spawn(Name::new("Heroi")).id();
    assert_eq!(
        alvo(&sim),
        BarraAlvo::SemVida,
        "o herói existe e não tem vida"
    );
    sim.world_mut().entity_mut(heroi).insert((
        Health {
            max: 40.0,
            start: 40.0,
            ..Health::default()
        },
        HealthNow {
            pontos: 10.0,
            escudo: 0.0,
            morta: false,
        },
    ));
    assert_eq!(
        alvo(&sim),
        BarraAlvo::Mostra {
            pontos: 10.0,
            max: 40.0
        }
    );
}

/// ⭐⭐⭐ **As nove edições do IMPACTO vão e voltam** (plano 28, W5) — e as cercas: o empurrão é
/// LIVRE (negativo puxa), o que a vida aceita não desce de zero, e a cor fica em `0..=1`.
///
/// **Mutações que devem sangrar:** tirar qualquer braço novo do `apply_health`/`apply_damage` ·
/// prender o empurrão a `≥ 0`.
#[test]
fn as_edicoes_do_impacto_vao_e_voltam() {
    let (mut sim, b) = mundo(true, true, true);
    for e in [
        E::DeathHitstopS(0.15),
        E::BlinkS(0.07),
        E::KnockbackTaken(0.5),
        E::Numbers(true),
        E::NumbersColor([0.1, 0.2, 0.3, 1.0]),
        E::NumbersSize(0.6),
        E::HitstopS(0.05),
        E::Knockback(-3.0),
        E::KnockbackLift(2.0),
    ] {
        assert!(apply(&mut sim, b, &e), "{e:?} não tocou no mundo");
    }
    let i = build_vida_info(sim.world(), b, 1, true).expect("tem os dois");
    let h = i.health.expect("vida");
    let d = i.damage.expect("dano");
    assert_eq!(
        (
            h.death_hitstop_s,
            h.blink_s,
            h.knockback_taken,
            h.numbers,
            h.numbers_size
        ),
        (0.15, 0.07, 0.5, true, 0.6)
    );
    assert_eq!(h.numbers_color, [0.1, 0.2, 0.3, 1.0]);
    assert_eq!(
        (d.hitstop_s, d.knockback, d.knockback_lift),
        (0.05, -3.0, 2.0)
    );
    // As cercas.
    for e in [
        E::KnockbackTaken(-2.0),
        E::NumbersColor([2.0, -1.0, 0.5, 1.0]),
        E::Knockback(f32::NAN),
    ] {
        assert!(apply(&mut sim, b, &e));
    }
    let i = build_vida_info(sim.world(), b, 1, true).expect("tem os dois");
    let (h, d) = (i.health.unwrap(), i.damage.unwrap());
    assert_eq!(
        h.knockback_taken, 0.0,
        "a vida não aceita um empurrão NEGATIVO"
    );
    assert_eq!(h.numbers_color, [1.0, 0.0, 0.5, 1.0], "a cor fica em 0..=1");
    assert_eq!(d.knockback, 0.0, "um empurrão não finito cai a zero");
}

/// ⭐⭐⭐ **O tipo e o dano que DURA vão e voltam** (plano 28, W6) — e as cercas: o tipo perde os
/// espaços da ponta (a dobra da lei já os ignora, e gravá-los faria o painel mostrar um nome que a
/// lei não lê assim), e um número negativo ou não finito cai a zero.
///
/// **Mutação que deve sangrar:** tirar qualquer braço novo do `apply_damage`.
#[test]
fn o_tipo_e_o_dano_que_dura_vao_e_voltam() {
    let (mut sim, b) = mundo(false, true, true);
    for e in [
        E::DamageKind("  fogo ".into()),
        E::OverTimePerS(4.0),
        E::OverTimeS(2.5),
        E::OverTimeEveryS(0.0),
    ] {
        assert!(apply(&mut sim, b, &e), "{e:?} não tocou no mundo");
    }
    let d = build_vida_info(sim.world(), b, 1, true)
        .and_then(|i| i.damage)
        .expect("dano");
    assert_eq!(d.kind, "fogo");
    assert_eq!(
        (d.over_time_per_s, d.over_time_s, d.over_time_every_s),
        (4.0, 2.5, 0.0),
        "o intervalo `0` é legítimo (a lava pulsa a cada tique)"
    );
    assert!(d.dura());
    for e in [E::OverTimePerS(-1.0), E::OverTimeS(f32::NAN)] {
        assert!(apply(&mut sim, b, &e));
    }
    let d = build_vida_info(sim.world(), b, 1, true)
        .and_then(|i| i.damage)
        .expect("dano");
    assert_eq!((d.over_time_per_s, d.over_time_s), (0.0, 0.0));
    assert!(
        !d.dura(),
        "o CONTROLO: sem pontos por segundo, ele não dura"
    );
}

/// ⭐⭐⭐ **A tabela de resistências: juntar, escrever, tirar** — e o instantâneo lê-a de volta
/// pela ordem da LEI.
///
/// **Mutações que devem sangrar:** tirar qualquer braço de resistência do `apply_health`; o
/// `RemoveResistance` tirar outra linha.
#[test]
fn a_tabela_de_resistencias_vai_e_volta() {
    let (mut sim, b) = mundo(true, false, true);
    for e in [
        E::AddResistance,
        E::AddResistance,
        E::ResistanceKind(0, "fogo".into()),
        E::ResistanceRate(0, 0.0),
        E::ResistanceKind(1, "gelo".into()),
        E::ResistanceRate(1, 2.0),
        E::ResistanceAbsorbs(1, true),
    ] {
        assert!(apply(&mut sim, b, &e), "{e:?} não tocou no mundo");
    }
    let rs = |sim: &SimWorld| {
        build_vida_info(sim.world(), b, 1, true)
            .and_then(|i| i.health)
            .expect("vida")
            .resistances
    };
    let r = rs(&sim);
    assert_eq!(r.len(), 2);
    assert_eq!(
        (r[0].kind.as_str(), r[0].rate, r[0].absorbs),
        ("fogo", 0.0, false)
    );
    assert_eq!(
        (r[1].kind.as_str(), r[1].rate, r[1].absorbs),
        ("gelo", 2.0, true)
    );
    // Uma linha NOVA nasce NEUTRA (taxa 1, sem tipo): uma linha a meio de ser escrita não muda
    // nada até o artista lhe dar um tipo.
    assert!(apply(&mut sim, b, &E::AddResistance));
    let r = rs(&sim);
    assert_eq!(
        (r[2].kind.as_str(), r[2].rate, r[2].absorbs),
        ("", 1.0, false)
    );
    // Tirar a do MEIO leva a do meio.
    assert!(apply(&mut sim, b, &E::RemoveResistance(1)));
    let kinds: Vec<String> = rs(&sim).into_iter().map(|r| r.kind).collect();
    assert_eq!(kinds, vec!["fogo".to_string(), String::new()]);
    // As cercas: um índice fora da tabela não toca em nada, e uma taxa negativa cai a zero.
    assert!(!apply(&mut sim, b, &E::ResistanceKind(9, "x".into())));
    assert!(!apply(&mut sim, b, &E::RemoveResistance(9)));
    assert!(apply(&mut sim, b, &E::ResistanceRate(0, -3.0)));
    assert_eq!(rs(&sim)[0].rate, 0.0);
}

/// ⭐⭐ **Juntar pára no TECTO** — o `RESISTANCES_MAX`, que um gate na shell amarra à lista pintada.
///
/// **Mutação que deve sangrar:** tirar a cerca do `AddResistance`.
#[test]
fn juntar_para_no_tecto() {
    let (mut sim, b) = mundo(true, false, true);
    for _ in 0..RESISTANCES_MAX {
        assert!(apply(&mut sim, b, &E::AddResistance));
    }
    assert!(
        !apply(&mut sim, b, &E::AddResistance),
        "a 9.ª linha entrou — um estado que o painel não mostra"
    );
    let n = build_vida_info(sim.world(), b, 1, true)
        .and_then(|i| i.health)
        .expect("vida")
        .resistances
        .len();
    assert_eq!(n, RESISTANCES_MAX);
}

/// ⭐⭐ **A REPETIDA marca-se pela dobra da LEI** — `FOGO` depois de `fogo` é a mesma linha para a
/// lei, e o painel tem de o dizer; uma linha SEM tipo nunca é repetida (está a meio de ser escrita).
///
/// **Mutação que deve sangrar:** comparar os nomes crus no `resistance_rows`.
#[test]
fn a_repetida_marca_se_pela_dobra_da_lei() {
    let (mut sim, b) = mundo(true, false, true);
    sim.world_mut()
        .get_mut::<Health>(Entity::from_bits(b))
        .expect("vida")
        .resistances = ["fogo", "", "FÓGO", "", "gelo"]
        .iter()
        .map(|k| Resistance {
            kind: (*k).to_string(),
            ..Resistance::default()
        })
        .collect();
    let rep: Vec<bool> = build_vida_info(sim.world(), b, 1, true)
        .and_then(|i| i.health)
        .expect("vida")
        .resistances
        .into_iter()
        .map(|r| r.repetida)
        .collect();
    assert_eq!(rep, vec![false, false, true, false, false]);
}
