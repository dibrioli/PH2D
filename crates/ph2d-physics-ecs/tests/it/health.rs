//! ⭐⭐⭐ **A VIDA na ponte** (plano 28, W2) — os gates de comportamento, pela API pública.
//!
//! Cada gate tem o CONTROLO ao lado: a mesma cena com a peça que ele afirma retirada, senão uma
//! ponte que não fizesse nada passaria (a vida de fábrica é `100` e fica `100`).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::health::HealthEventKind;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, GravityScale, Health, OnHit, PhysicsBridge,
    ProjectileMotion, RigidBody,
};

fn caixa(h: f32, sensor: bool) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: h,
            half_y: h,
        },
        density: 1.0,
        is_sensor: sensor,
        ..Collider::default()
    }
}

/// Um alvo com vida — cinemático e parado (nada o empurra, logo a cena é a mesma em todo tique).
fn alvo(sim: &mut SimWorld, x: f32, vida: Health) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Alvo"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            caixa(0.4, false),
            Transform::from_translation(Vec2::new(x, 0.0)),
            vida,
        ))
        .id()
}

/// Um espinho: um SENSOR estático, com ou sem dano.
fn espinho(sim: &mut SimWorld, x: f32, dano: Option<Damage>) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Espinho"),
        RigidBody {
            kind: BodyKind::Static,
        },
        caixa(0.3, true),
        Transform::from_translation(Vec2::new(x, 0.0)),
    ));
    if let Some(d) = dano {
        e.insert(d);
    }
    e.id()
}

/// Uma bala: projéctil cinemático SÓLIDO a `v` m/s para a direita, com ou sem dano.
fn bala(sim: &mut SimWorld, x: f32, v: f32, dano: Option<Damage>) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Bala"),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        caixa(0.1, false),
        GravityScale(0.0),
        Transform::from_translation(Vec2::new(x, 0.0)),
        ProjectileMotion {
            initial_speed: v,
            range: 0.0,
            ..ProjectileMotion::default()
        },
    ));
    if let Some(d) = dano {
        e.insert(d);
    }
    e.id()
}

fn dano(amount: f32) -> Damage {
    Damage {
        amount,
        ..Damage::default()
    }
}

fn pontos(ponte: &PhysicsBridge, e: Entity) -> f64 {
    ponte.health_of(e).map_or(f64::NAN, |s| s.vida().pontos)
}

/// Corre `0..=ticks` e devolve a ponte e TODO facto de vida do caminho, com o tique.
fn corre(sim: &mut SimWorld, ticks: u64) -> (PhysicsBridge, Vec<(u64, HealthEventKind)>) {
    let mut ponte = PhysicsBridge::new();
    let mut factos = Vec::new();
    for t in 0..=ticks {
        ponte.dispatch(sim, true, t);
        factos.extend(ponte.health_events().iter().map(|e| (t, e.kind)));
    }
    (ponte, factos)
}

fn danos(factos: &[(u64, HealthEventKind)]) -> Vec<(u64, f64)> {
    factos
        .iter()
        .filter_map(|&(t, k)| match k {
            HealthEventKind::Damaged { amount } => Some((t, amount)),
            _ => None,
        })
        .collect()
}

/// ⭐⭐⭐ **Uma bala FERE quem acerta** — pelo canal do MOVER (plano 28 §8.1), porque a sonda
/// `mede_o_golpe_que_chega` mediu que ela nunca encosta no alvo.
///
/// **Mutação que deve sangrar:** apagar o `toques_do_mover.extend(…)` do `bridge/projectile.rs`.
#[test]
fn uma_bala_fere_quem_acerta() {
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, 0.0, Health::default());
    let _b = bala(&mut sim, -3.0, 30.0, Some(dano(30.0)));
    let (ponte, factos) = corre(&mut sim, 60);
    assert_eq!(pontos(&ponte, a), 70.0);
    let d = danos(&factos);
    assert_eq!(d.len(), 1, "um tiro, um golpe: {factos:?}");
    assert_eq!(d[0].1, 30.0);

    // O CONTROLO: a mesma bala SEM dano não tira nada.
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, 0.0, Health::default());
    let _b = bala(&mut sim, -3.0, 30.0, None);
    let (ponte, factos) = corre(&mut sim, 60);
    assert_eq!(pontos(&ponte, a), 100.0);
    assert!(factos.is_empty());
}

/// ⭐⭐⭐ **Quem NASCE sobreposto leva o golpe no 1.º tique** — a queixa do Godot, morta porque a
/// memória do toque da vida nasce vazia.
#[test]
fn quem_nasce_sobreposto_leva_o_golpe_no_primeiro_tique() {
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, 0.0, Health::default());
    let _e = espinho(&mut sim, 0.2, Some(dano(10.0)));
    let (ponte, factos) = corre(&mut sim, 5);
    assert_eq!(danos(&factos), vec![(1, 10.0)]);
    assert_eq!(pontos(&ponte, a), 90.0);
}

/// ⭐⭐⭐ **Um toque fere UMA vez** (o de fábrica), e o CONTÍNUO fere por segundo.
///
/// ⚠️ É o gate da queixa nº 1 da pesquisa (*dano a dobrar*): sem a memória do toque, cada tique de
/// sobreposição seria um golpe novo — `120` golpes de `10` em dois segundos.
///
/// **Mutação que deve sangrar:** `let comecou = true;` no `drive_health`.
#[test]
fn um_toque_fere_uma_vez_e_o_continuo_fere_por_segundo() {
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, 0.0, Health::default());
    let _e = espinho(&mut sim, 0.2, Some(dano(10.0)));
    let (ponte, _) = corre(&mut sim, 120);
    assert_eq!(
        pontos(&ponte, a),
        90.0,
        "um toque de dois segundos é UM golpe"
    );

    // O contínuo: 10 pontos por SEGUNDO durante 120 tiques = 20.
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, 0.0, Health::default());
    let _e = espinho(
        &mut sim,
        0.2,
        Some(Damage {
            per_second: true,
            ..dano(10.0)
        }),
    );
    let (ponte, _) = corre(&mut sim, 120);
    let p = pontos(&ponte, a);
    assert!((p - 80.0).abs() < 1e-4, "o contínuo tira 10/s: {p}");
}

/// ⭐⭐ **A invencibilidade apanha o 2.º golpe do MESMO tique** — dois espinhos tocam ao mesmo tempo.
#[test]
fn a_invencibilidade_apanha_o_segundo_golpe_do_mesmo_tique() {
    for (inv, esperado) in [(1.0_f32, 90.0), (0.0, 80.0)] {
        let mut sim = SimWorld::new();
        let a = alvo(
            &mut sim,
            0.0,
            Health {
                invincible_s: inv,
                ..Health::default()
            },
        );
        let _e1 = espinho(&mut sim, 0.2, Some(dano(10.0)));
        let _e2 = espinho(&mut sim, -0.2, Some(dano(10.0)));
        let (ponte, _) = corre(&mut sim, 5);
        assert_eq!(pontos(&ponte, a), esperado, "invencível {inv} s");
    }
}

/// ⭐⭐ **A mesma equipa não fere**; equipas diferentes (ou vazias) ferem.
#[test]
fn a_mesma_equipa_nao_fere() {
    for (vida, bate, esperado) in [
        ("herois", "herois", 100.0),
        ("herois", "monstros", 90.0),
        ("", "", 90.0),
        ("herois", "", 90.0),
    ] {
        let mut sim = SimWorld::new();
        let a = alvo(
            &mut sim,
            0.0,
            Health {
                team: vida.into(),
                ..Health::default()
            },
        );
        let _e = espinho(
            &mut sim,
            0.2,
            Some(Damage {
                team: bate.into(),
                ..dano(10.0)
            }),
        );
        let (ponte, _) = corre(&mut sim, 5);
        assert_eq!(pontos(&ponte, a), esperado, "{vida:?} × {bate:?}");
    }
}

/// ⭐⭐ **Morre UMA vez, e fica em zero** (a casa: um morto é final).
#[test]
fn morre_uma_vez_e_fica_em_zero() {
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, 0.0, Health::default());
    let _e = espinho(
        &mut sim,
        0.2,
        Some(Damage {
            per_second: true,
            ..dano(300.0)
        }),
    );
    let (ponte, factos) = corre(&mut sim, 120);
    assert_eq!(pontos(&ponte, a), 0.0);
    let mortes = factos
        .iter()
        .filter(|(_, k)| *k == HealthEventKind::Died)
        .count();
    assert_eq!(mortes, 1, "{factos:?}");
}

/// ⭐⭐ **Uma bala `Vanish` é anunciada ao bater** — a ponte anuncia, a shell remove.
#[test]
fn uma_bala_que_some_e_anunciada_ao_bater() {
    let mut sim = SimWorld::new();
    let _a = alvo(&mut sim, 0.0, Health::default());
    let b = bala(
        &mut sim,
        -3.0,
        30.0,
        Some(Damage {
            on_hit: OnHit::Vanish,
            ..dano(10.0)
        }),
    );
    let mut ponte = PhysicsBridge::new();
    let mut viu = Vec::new();
    for t in 0..=60 {
        ponte.dispatch(&mut sim, true, t);
        if ponte.damage_spent().contains(&b) {
            viu.push(t);
        }
    }
    assert_eq!(viu.len(), 1, "anunciada uma vez: {viu:?}");

    // O CONTROLO: `Stay` (o de fábrica) nunca é anunciada.
    let mut sim = SimWorld::new();
    let _a = alvo(&mut sim, 0.0, Health::default());
    let b = bala(&mut sim, -3.0, 30.0, Some(dano(10.0)));
    let mut ponte = PhysicsBridge::new();
    for t in 0..=60 {
        ponte.dispatch(&mut sim, true, t);
        assert!(!ponte.damage_spent().contains(&b));
    }
}

/// ⭐⭐⭐ **Um SCRUB devolve a vida EXACTA daquele tique** — a razão de a vida viver no passo da
/// física e ir no anel (plano 28 §2).
///
/// A cena mexe na vida de forma não trivial e em TRÊS memórias: um espinho que fere no 1.º tique
/// (a memória do toque), uma bala pela esquerda e outra pela direita (os golpes do mover e o
/// gerador da esquiva, que sorteia sempre), e a regeneração que arranca entre as duas (os relógios).
/// Cada tique de volta tem de ler **o mesmo `f64`** da corrida.
///
/// ⚠️ **A 1.ª redacção desta cena foi RECUSADA pelo próprio piso dela** (`min < 75`): tinha uma
/// lava contínua com invencibilidade, e a lava re-arma a invencibilidade a cada golpe — a lei do
/// alvo —, logo a bala chegava sempre dentro dela e a vida quase não se mexia. *Um gate de scrub
/// sobre uma vida parada mede nada.*
///
/// **Mutações que devem sangrar:** apagar o `self.depois_do_passo(sim, false)` do laço de replay,
/// apagar o `health_state.clear()` do `rebuild_from_rest`, e tirar o `health` do `seed` do anel.
#[test]
fn um_scrub_devolve_a_vida_exacta_do_tique() {
    let mut sim = SimWorld::new();
    let a = alvo(
        &mut sim,
        0.0,
        Health {
            regen: 4.0,
            regen_delay_s: 0.5,
            ..Health::default()
        },
    );
    let _espinho = espinho(&mut sim, 0.2, Some(dano(10.0)));
    let _b1 = bala(&mut sim, -6.0, 12.0, Some(dano(25.0)));
    // A 2.ª bala vem da DIREITA (virada ao contrário): pela esquerda ela bateria na 1.ª, que fica
    // parada encostada ao alvo.
    let b2 = bala(&mut sim, 20.0, 12.0, Some(dano(30.0)));
    sim.world_mut()
        .get_mut::<Transform>(b2)
        .expect("a bala tem pose")
        .rotation = std::f32::consts::PI;

    let mut ponte = PhysicsBridge::new();
    let mut corrida = Vec::new();
    for t in 0..=150 {
        ponte.dispatch(&mut sim, true, t);
        corrida.push(pontos(&ponte, a).to_bits());
    }
    // O PISO: a cena tem de mexer na vida, e com as três fontes — senão o gate mede nada.
    let lidas: Vec<f64> = corrida.iter().map(|&b| f64::from_bits(b)).collect();
    let min = lidas.iter().copied().fold(f64::MAX, f64::min);
    assert!(min < 50.0, "a cena não feriu o suficiente: {min}");
    assert!(
        lidas.windows(2).any(|w| w[1] > w[0]),
        "a regeneração nunca subiu a vida — os relógios não estão no gate"
    );

    let mut de = 150_u64;
    for alvo_t in [37_u64, 100, 3, 149, 0, 80] {
        ponte.dispatch(&mut sim, true, alvo_t);
        assert_eq!(
            pontos(&ponte, a).to_bits(),
            corrida[alvo_t as usize],
            "o scrub para o tique {alvo_t} devolveu {} contra {} da corrida",
            pontos(&ponte, a),
            f64::from_bits(corrida[alvo_t as usize])
        );
        // ⛔ Um replay NÃO publica: um scrub para TRÁS não é uma tempestade de golpes. ⚠️ Só para
        // trás — um salto para a FRENTE anda os tiques como um play (a mesma regra de todos os
        // canais desta ponte: *o estado é função do tique, não do botão*), e publica o que andou.
        if alvo_t < de {
            assert!(
                ponte.health_events().is_empty(),
                "o scrub para trás até {alvo_t} publicou {:?}",
                ponte.health_events()
            );
        }
        de = alvo_t;
    }
    // E andar outra vez para a frente a partir de um scrub reproduz o resto da corrida.
    for t in 81..=150 {
        ponte.dispatch(&mut sim, true, t);
        assert_eq!(
            pontos(&ponte, a).to_bits(),
            corrida[t as usize],
            "tique {t}"
        );
    }
}

/// ⭐⭐ **A esquiva é determinista:** a mesma cena esquiva igual, e uma semente diferente esquiva
/// diferente (o sorteio não é uma constante).
#[test]
fn a_esquiva_e_determinista_e_depende_da_semente() {
    let corrida = |seed: u64| -> f64 {
        let mut sim = SimWorld::new();
        let a = alvo(
            &mut sim,
            0.0,
            Health {
                dodge: 0.5,
                max: 0.0,
                start: 10_000.0,
                seed,
                ..Health::default()
            },
        );
        let _e = espinho(
            &mut sim,
            0.2,
            Some(Damage {
                per_second: true,
                ..dano(60.0)
            }),
        );
        let (ponte, _) = corre(&mut sim, 200);
        pontos(&ponte, a)
    };
    assert_eq!(corrida(7).to_bits(), corrida(7).to_bits());
    assert_ne!(corrida(7).to_bits(), corrida(8).to_bits());
    // E metade dos golpes esquiva, grosso modo — não zero, não todos.
    let tirado = 10_000.0 - corrida(7);
    assert!(
        tirado > 30.0 && tirado < 170.0,
        "tirado {tirado} de 200 golpes de 1"
    );
}

/// ⭐⭐⭐ **A vida GRITA os nomes que o artista escreveu** — pela porta de sinais da física, com
/// `source` = quem TEM a vida e `other` = quem bateu (é o que o `From Myself` da tabela lê).
///
/// **Mutação que deve sangrar:** trocar `source: ev.target` por `source: ev.source` no
/// `signal_events`.
#[test]
fn a_vida_grita_os_nomes_que_o_artista_escreveu() {
    let mut sim = SimWorld::new();
    let a = alvo(
        &mut sim,
        0.0,
        Health {
            on_damage: "ai".into(),
            on_death: "morri".into(),
            ..Health::default()
        },
    );
    let e = espinho(
        &mut sim,
        0.2,
        Some(Damage {
            per_second: true,
            ..dano(600.0)
        }),
    );
    let mut ponte = PhysicsBridge::new();
    let mut ouvidos = Vec::new();
    for t in 0..=30 {
        ponte.dispatch(&mut sim, true, t);
        for s in ponte.signal_events(&sim, &ph2d_tags::TagTree::new()) {
            ouvidos.push((s.name, s.source, s.other));
        }
    }
    assert!(ouvidos.contains(&("ai".into(), a, e)), "{ouvidos:?}");
    let mortes = ouvidos.iter().filter(|(n, ..)| n == "morri").count();
    assert_eq!(mortes, 1, "morre uma vez só: {ouvidos:?}");
    assert!(
        ouvidos.iter().all(|(_, s, o)| *s == a && *o == e),
        "quem grita é quem tem a vida: {ouvidos:?}"
    );

    // O CONTROLO: sem nomes, silêncio — a vida continua a ferir, mas ninguém ouve.
    let mut sim = SimWorld::new();
    let _a = alvo(&mut sim, 0.0, Health::default());
    let _e = espinho(&mut sim, 0.2, Some(dano(10.0)));
    let mut ponte = PhysicsBridge::new();
    for t in 0..=10 {
        ponte.dispatch(&mut sim, true, t);
        assert!(
            ponte
                .signal_events(&sim, &ph2d_tags::TagTree::new())
                .is_empty()
        );
    }
}

/// Corre `0..=ticks` e junta todas as mortes que a porta da shell anunciaria.
fn anunciadas(sim: &mut SimWorld, ticks: u64) -> Vec<(u64, ph2d_ecs::Death)> {
    let mut ponte = PhysicsBridge::new();
    let mut out = Vec::new();
    for t in 0..=ticks {
        ponte.dispatch(sim, true, t);
        out.extend(
            ponte
                .mortes_anunciadas(sim.world())
                .into_iter()
                .map(|d| (t, d)),
        );
    }
    out
}

/// ⭐⭐⭐ **Só sai da cena quem NASCEU numa corrida** — a lei que protege o trabalho do artista, e
/// ela mora na porta que a shell drena ([`PhysicsBridge::mortes_anunciadas`]).
///
/// Um inimigo e uma bala de DOCUMENTO morrem e param, e **ficam**; os mesmos dois com
/// `ph2d_ecs::Spawned` saem, cada um UMA vez e com a causa certa (`Killed` a vida, `Spent` a bala).
///
/// ⚠️ **A porta nasceu de um corte da shell e não tinha gate nenhum** — o `fase_fabrica_e_morte` a
/// chama e nenhum teste a alcança por ali (a fase pede a `App`).
///
/// **Mutações que devem sangrar:** apagar o `is_transient(world, entity) &&` · apagar o ramo do
/// `Died` · trocar o `Killed` por `Spent`.
#[test]
fn so_sai_da_cena_quem_nasceu_numa_corrida() {
    use ph2d_ecs::{DeathCause, Spawned};
    let nascido = Spawned {
        by: 0,
        born_tick: 0,
    };
    let espinho_mortal = || Damage {
        per_second: true,
        ..dano(600.0)
    };

    // A VIDA: um inimigo nascido morre e é anunciado; o de documento morre e fica.
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, 0.0, Health::default());
    sim.world_mut().entity_mut(a).insert(nascido);
    let _e = espinho(&mut sim, 0.2, Some(espinho_mortal()));
    let mortes = anunciadas(&mut sim, 30);
    assert_eq!(mortes.len(), 1, "morre UMA vez: {mortes:?}");
    assert_eq!(mortes[0].1.entity, a);
    assert_eq!(mortes[0].1.why, DeathCause::Killed);
    assert!(mortes[0].1.signal.is_empty(), "a porta é calada");

    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, 0.0, Health::default());
    let _e = espinho(&mut sim, 0.2, Some(espinho_mortal()));
    let mortes = anunciadas(&mut sim, 30);
    assert!(
        mortes.is_empty(),
        "um inimigo de DOCUMENTO fica: {mortes:?}"
    );
    let mut ponte = PhysicsBridge::new();
    for t in 0..=30 {
        ponte.dispatch(&mut sim, true, t);
    }
    assert!(
        ponte.health_of(a).is_some_and(|s| s.vida().morta()),
        "o CONTROLO: ele morreu na mesma — só não sai"
    );

    // A BALA `Vanish`: a nascida sai ao bater; a de documento pára.
    let bala_que_some = || Damage {
        on_hit: OnHit::Vanish,
        ..dano(10.0)
    };
    let mut sim = SimWorld::new();
    let _a = alvo(&mut sim, 0.0, Health::default());
    let b = bala(&mut sim, -3.0, 30.0, Some(bala_que_some()));
    sim.world_mut().entity_mut(b).insert(nascido);
    let mortes = anunciadas(&mut sim, 60);
    assert_eq!(mortes.len(), 1, "a bala sai UMA vez: {mortes:?}");
    assert_eq!(mortes[0].1.entity, b);
    assert_eq!(mortes[0].1.why, DeathCause::Spent);

    let mut sim = SimWorld::new();
    let _a = alvo(&mut sim, 0.0, Health::default());
    let _b = bala(&mut sim, -3.0, 30.0, Some(bala_que_some()));
    assert!(
        anunciadas(&mut sim, 60).is_empty(),
        "uma bala de DOCUMENTO pára"
    );

    // ⛔⛔ E quem some ao bater SEM ser projéctil — uma pedra que cai. ⚠️ **Nasceu de uma mutação
    // SOBREVIVENTE:** a bala acima também acaba o VOO ao bater (`projectile_done`), logo apagar o
    // ramo do `damage_spent` deixava este gate verde. Só um `Vanish` que não voa separa os dois.
    let pedra = |some: bool| {
        let mut sim = SimWorld::new();
        let _a = alvo(&mut sim, 0.0, Health::default());
        let p = sim
            .world_mut()
            .spawn((
                Name::new("Pedra"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.2 },
                    density: 1.0,
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(0.0, 3.0)),
                Damage {
                    on_hit: if some { OnHit::Vanish } else { OnHit::Stay },
                    ..dano(10.0)
                },
                nascido,
            ))
            .id();
        (p, anunciadas(&mut sim, 180))
    };
    let (p, mortes) = pedra(true);
    assert_eq!(mortes.len(), 1, "a pedra que some sai UMA vez: {mortes:?}");
    assert_eq!(mortes[0].1.entity, p);
    assert_eq!(mortes[0].1.why, DeathCause::Spent);
    let (_, mortes) = pedra(false);
    assert!(mortes.is_empty(), "o CONTROLO `Stay` fica: {mortes:?}");
}

/// Um mover com dano (`quem`) anda para a DIREITA contra um alvo parado; devolve a vida do alvo.
///
/// `anda = false` é o CONTROLO: o mesmo mover, parado a `2` m do alvo, não o fere.
fn mover_contra_o_alvo(
    mover: (RigidBody, Collider, impl bevy_ecs::bundle::Bundle),
    anda: bool,
    y: f32,
) -> f64 {
    let mut sim = SimWorld::new();
    // O chão: o controlador de plataforma precisa dele para andar; o de vista de cima ignora-o.
    sim.world_mut().spawn((
        Name::new("Chao"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    let a = alvo(&mut sim, 1.5, Health::default());
    sim.world_mut()
        .get_mut::<Transform>(a)
        .expect("o alvo tem pose")
        .translation = Vec2::new(1.5, 0.45);
    let quem = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            mover.0,
            mover.1,
            mover.2,
            ph2d_physics_ecs::LockRotation,
            dano(10.0),
            Transform::from_translation(Vec2::new(-1.0, y)),
        ))
        .id();
    let mut ponte = PhysicsBridge::new();
    for t in 0..=240 {
        ponte.set_player_input(
            quem,
            ph2d_platformer::PlayerInput {
                drive: if anda { 1.0 } else { 0.0 },
                ..ph2d_platformer::PlayerInput::default()
            },
        );
        ponte.dispatch(&mut sim, true, t);
    }
    pontos(&ponte, a)
}

/// ⭐⭐⭐ **Os DOIS controladores cinemáticos ferem pelo canal do MOVER** — o de vista de cima e o
/// de plataforma em modo cinemático, os dois irmãos do projéctil que também param rente ao que
/// batem (plano 28 §8.1).
///
/// ⚠️ **Nasceu da prova de mutação:** o `toques_do_mover.extend(…)` vive em TRÊS pontes, e só a do
/// projéctil tinha gate — apagar as outras duas deixava a suíte verde. *Um canal ligado em um dos
/// três ramos lê-se como ligado.*
///
/// **Mutações que devem sangrar:** apagar o `extend` do `bridge/topdown.rs` · e o do
/// `bridge/player_kinmove.rs`.
#[test]
fn os_dois_controladores_ferem_pelo_canal_do_mover() {
    let bola = || {
        (
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                ..Collider::default()
            },
        )
    };
    let capsula = || {
        (
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
        )
    };
    let vista_de_cima = |anda| {
        let (r, c) = bola();
        // À altura do MEIO do alvo: numa vista de cima não há chão por onde subir.
        mover_contra_o_alvo(
            (r, c, ph2d_physics_ecs::TopDownPlayer::default()),
            anda,
            0.45,
        )
    };
    let plataforma = |anda| {
        let (r, c) = capsula();
        mover_contra_o_alvo(
            (
                r,
                c,
                (
                    ph2d_physics_ecs::PlatformPlayer {
                        float_height: 1.1,
                        ..ph2d_physics_ecs::PlatformPlayer::default()
                    },
                    ph2d_physics_ecs::PlayerMode::Kinematic,
                ),
            ),
            anda,
            1.1,
        )
    };
    assert_eq!(
        vista_de_cima(true),
        90.0,
        "vista de cima: um toque, um golpe"
    );
    assert_eq!(vista_de_cima(false), 100.0, "o CONTROLO parado não fere");
    assert_eq!(
        plataforma(true),
        90.0,
        "plataforma cinemática: um toque, um golpe"
    );
    assert_eq!(plataforma(false), 100.0, "o CONTROLO parado não fere");
}

/// ⭐⭐ **Uma pedra que CAI fere pelo contacto SÓLIDO** — a terceira fonte (o `tick_contacts` do
/// solver), a única que só um corpo DINÂMICO produz: as balas e os controladores são movers e
/// param rente, e os espinhos são sensores.
///
/// ⚠️ E ela assenta e FICA em cima: um toque que dura não é um golpe por tique (o de fábrica).
///
/// **Mutação que deve sangrar:** apagar o `toques.push` do laço do `tick_contacts`.
#[test]
fn uma_pedra_que_cai_fere_pelo_contacto_solido() {
    let pedra = |com_dano: bool| {
        let mut sim = SimWorld::new();
        let a = alvo(&mut sim, 0.0, Health::default());
        let mut e = sim.world_mut().spawn((
            Name::new("Pedra"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 3.0)),
        ));
        if com_dano {
            e.insert(dano(10.0));
        }
        let (ponte, _) = corre(&mut sim, 180);
        pontos(&ponte, a)
    };
    assert_eq!(pedra(true), 90.0, "cai, bate, fica em cima: UM golpe");
    assert_eq!(pedra(false), 100.0, "o CONTROLO sem dano não fere");
}
