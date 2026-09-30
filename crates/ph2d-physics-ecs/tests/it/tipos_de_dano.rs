//! ⭐⭐⭐ **Os TIPOS de dano e o dano que DURA, na ponte** (plano 28, W6) — pela API pública.
//!
//! A lei (a taxa, o absorver, as aflições) tem os gates dela em `ph2d-health`, contra o oráculo do
//! addon MIT do Godot. Estes medem o que só a ponte faz: **o tipo do `Damage` encontra a
//! resistência do `Health`**, a aflição ENTRA com o golpe e continua depois do toque, a morte a
//! cura, e o anel devolve-a num scrub.
//!
//! Cada gate traz o CONTROLO ao lado — a mesma cena com a peça que ele afirma retirada.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::health::HealthEventKind;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, Health, PhysicsBridge, Resistance, RigidBody,
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

/// Um alvo cinemático e parado, com a vida dada.
fn alvo(sim: &mut SimWorld, vida: Health) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Alvo"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            caixa(0.4, false),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            vida,
        ))
        .id()
}

/// Um espinho SENSOR estático, SOBREPOSTO ao alvo desde o tique 0 (fere uma vez, no 1.º tique).
fn espinho(sim: &mut SimWorld, dano: Damage) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Espinho"),
            RigidBody {
                kind: BodyKind::Static,
            },
            caixa(0.3, true),
            Transform::from_translation(Vec2::new(0.2, 0.0)),
            dano,
        ))
        .id()
}

fn tipado(amount: f32, kind: &str) -> Damage {
    Damage {
        amount,
        kind: kind.to_owned(),
        ..Damage::default()
    }
}

fn resistente(linhas: &[(&str, f32, bool)]) -> Health {
    Health {
        resistances: linhas
            .iter()
            .map(|&(kind, rate, absorbs)| Resistance {
                kind: kind.to_owned(),
                rate,
                absorbs,
            })
            .collect(),
        ..Health::default()
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

/// Uma cena de um golpe só: o alvo com as `resistencias`, um espinho do tipo `kind`.
fn um_golpe(resistencias: &[(&str, f32, bool)], start: f32, kind: &str) -> f64 {
    let mut sim = SimWorld::new();
    let a = alvo(
        &mut sim,
        Health {
            start,
            ..resistente(resistencias)
        },
    );
    espinho(&mut sim, tipado(10.0, kind));
    let (ponte, _) = corre(&mut sim, 5);
    pontos(&ponte, a)
}

/// ⭐⭐⭐ **A salamandra: imune ao fogo, fraca ao gelo** — o smoke da wave, em números.
///
/// **Mutações que devem sangrar:** passar `Taxa::NEUTRA` no lugar de `h.taxa(&dano.kind)` no
/// `drive_health`; e fazer o `Health::taxa` devolver sempre `NEUTRA`.
#[test]
fn a_salamandra_e_imune_ao_fogo_e_fraca_ao_gelo() {
    let salamandra = [("fogo", 0.0, false), ("gelo", 2.0, false)];
    assert_eq!(um_golpe(&salamandra, 100.0, "fogo"), 100.0, "imune ao fogo");
    assert_eq!(um_golpe(&salamandra, 100.0, "gelo"), 80.0, "fraca ao gelo");
    // Um tipo que ela não lista, e um dano SEM tipo, passam normais.
    assert_eq!(um_golpe(&salamandra, 100.0, "veneno"), 90.0);
    assert_eq!(um_golpe(&salamandra, 100.0, ""), 90.0);
    // O CONTROLO: o mesmo fogo num alvo sem resistências tira os 10.
    assert_eq!(um_golpe(&[], 100.0, "fogo"), 90.0);
}

/// ⭐⭐ **O nome compara-se pela DOBRA da casa** — maiúsculas e acentos não importam (a lei das
/// tags, ordem do dono de 14/09). Sem isso `Fogo` num lado e `fogo` no outro seriam dois tipos, e
/// o artista veria uma resistência que não resiste.
///
/// **Mutação que deve sangrar:** comparar `r.kind == kind` cru no `Health::taxa`.
#[test]
fn o_tipo_compara_se_pela_dobra_da_casa() {
    assert_eq!(um_golpe(&[("FOGO", 0.0, false)], 100.0, "fógo"), 100.0);
    assert_eq!(um_golpe(&[("  Gelo ", 2.0, false)], 100.0, "gelo"), 80.0);
    // O CONTROLO: um nome DIFERENTE continua a não casar.
    assert_eq!(um_golpe(&[("fogo", 0.0, false)], 100.0, "fumo"), 90.0);
}

/// ⭐⭐ **A primeira linha com o nome ganha** — uma tabela com o mesmo tipo duas vezes é
/// ambígua, e a regra tem de ser UMA (a secção do Inspector avisa do duplicado).
#[test]
fn a_primeira_resistencia_com_o_nome_ganha() {
    let dupla = [("fogo", 0.0, false), ("fogo", 2.0, false)];
    assert_eq!(um_golpe(&dupla, 100.0, "fogo"), 100.0);
}

/// ⭐⭐⭐ **O elemental de fogo ABSORVE o fogo** — o golpe cura `dano × taxa` em vez de ferir, e
/// sai como `Healed` (a tabela do #5 ouve-o como qualquer cura).
///
/// **Mutação que deve sangrar:** passar `absorve: false` na `Taxa` do `Health::taxa`.
#[test]
fn o_elemental_de_fogo_absorve_o_fogo() {
    let elemental = [("fogo", 1.0, true)];
    let mut sim = SimWorld::new();
    let a = alvo(
        &mut sim,
        Health {
            start: 50.0,
            ..resistente(&elemental)
        },
    );
    espinho(&mut sim, tipado(10.0, "fogo"));
    let (ponte, factos) = corre(&mut sim, 5);
    assert_eq!(pontos(&ponte, a), 60.0);
    assert!(
        factos
            .iter()
            .any(|&(_, k)| matches!(k, HealthEventKind::Healed { amount } if amount == 10.0)),
        "a absorção sai como cura: {factos:?}"
    );
    // A metade: a vida CHEIA não tem o que curar, e não sai facto nenhum (a porta da cura).
    assert_eq!(um_golpe(&elemental, 100.0, "fogo"), 100.0);
    // O CONTROLO: o gelo fere o elemental normalmente.
    assert_eq!(um_golpe(&elemental, 50.0, "gelo"), 40.0);
}

/// Uma cena de veneno: o espinho dá `golpe` do tipo `kind` e deixa `por_s × dur_s`, a pulsar a
/// cada `cada_s`. Corre `ticks`.
fn envenena(
    vida: Health,
    kind: &str,
    golpe: f32,
    (por_s, dur_s, cada_s): (f32, f32, f32),
    ticks: u64,
) -> (PhysicsBridge, Entity, Vec<(u64, HealthEventKind)>) {
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, vida);
    espinho(
        &mut sim,
        Damage {
            over_time_per_s: por_s,
            over_time_s: dur_s,
            over_time_every_s: cada_s,
            ..tipado(golpe, kind)
        },
    );
    let (ponte, factos) = corre(&mut sim, ticks);
    (ponte, a, factos)
}

/// ⭐⭐⭐ **O veneno que DURA: o golpe, e depois três pulsos, um por segundo, que somam
/// `por segundo × duração`** — com o espinho ainda a tocar, porque um golpe único entra no COMEÇO
/// do toque e é a aflição, e não o toque, que continua a morder.
///
/// **Mutações que devem sangrar:** apagar o `st.aflicoes.aplica(…)`; apagar o laço do
/// `st.aflicoes.anda(dt)`.
#[test]
fn o_veneno_morde_depois_do_golpe_e_soma_o_que_promete() {
    let (ponte, a, factos) = envenena(Health::default(), "veneno", 10.0, (4.0, 3.0, 1.0), 300);
    let d = danos(&factos);
    assert_eq!(d.len(), 4, "o golpe e três pulsos: {d:?}");
    // ⚠️ O golpe cai no 1.º tique que ANDA (o `dispatch` do tique 0 só monta o mundo — é o que o
    // `quem_nasce_sobreposto_leva_o_golpe_no_primeiro_tique` mede), logo os pulsos contam dele.
    let (t0, golpe) = d[0];
    assert_eq!(golpe, 10.0, "o golpe: {d:?}");
    // Um pulso por segundo — o 1.º UM INTERVALO depois do golpe, nunca no mesmo tique.
    let tiques: Vec<u64> = d[1..].iter().map(|&(t, _)| t - t0).collect();
    assert_eq!(tiques, vec![60, 120, 180], "os pulsos caem a cada segundo");
    let soma: f64 = d[1..].iter().map(|&(_, x)| x).sum();
    assert!(
        (soma - 12.0).abs() < 1e-9,
        "a aflição tira 4 × 3 = 12, tirou {soma}"
    );
    assert!((pontos(&ponte, a) - 78.0).abs() < 1e-9);
    // Acabou: não fica aflição viva a pulsar para sempre.
    assert!(
        ponte
            .health_of(a)
            .expect("tem vida")
            .aflicoes()
            .0
            .is_empty()
    );

    // O CONTROLO: o mesmo golpe SEM dano que dura tira só os 10.
    let (ponte, a, factos) = envenena(Health::default(), "veneno", 10.0, (0.0, 3.0, 1.0), 300);
    assert_eq!(danos(&factos).len(), 1);
    assert_eq!(pontos(&ponte, a), 90.0);
}

/// ⭐⭐ **A LAVA queima enquanto se pisa** — um dano POR SEGUNDO golpeia em cada tique do toque, e
/// cada golpe RENOVA a queimadura: parado dentro dela três segundos, a aflição de UM segundo ainda
/// está viva e ainda pulsa. ⚠️ A renovação não adia o pulso (a fase fica), logo os pulsos caem a
/// cada meio segundo como numa aflição que ninguém renovou.
///
/// **Mutação que deve sangrar:** aplicar a aflição só no COMEÇO do toque (`comecou &&`) — a lava
/// queimaria o primeiro segundo e depois só feriria pelo dano por segundo.
#[test]
fn a_lava_renova_a_queimadura_enquanto_se_pisa() {
    let queima = |por_segundo: bool| {
        let mut sim = SimWorld::new();
        let a = alvo(
            &mut sim,
            Health {
                max: 1000.0,
                start: 1000.0,
                ..Health::default()
            },
        );
        espinho(
            &mut sim,
            Damage {
                per_second: por_segundo,
                over_time_per_s: 4.0,
                over_time_s: 1.0,
                over_time_every_s: 0.5,
                ..tipado(1.0, "fogo")
            },
        );
        let (ponte, factos) = corre(&mut sim, 180);
        let vivas = ponte.health_of(a).expect("tem vida").aflicoes().0.len();
        // Os pulsos são os danos de `2` pontos (`4/s × 0,5 s`); o golpe da lava é `1 × dt`.
        let pulsos = danos(&factos)
            .iter()
            .filter(|&&(_, x)| (x - 2.0).abs() < 1e-9)
            .count();
        (vivas, pulsos)
    };
    let (vivas, pulsos) = queima(true);
    assert_eq!(vivas, 1, "parado na lava, a queimadura continua viva");
    assert!(
        pulsos >= 5,
        "três segundos de lava pulsam a cada meio segundo: {pulsos}"
    );
    // O CONTROLO: o mesmo toque com um golpe ÚNICO queima o segundo dele e acaba.
    let (vivas, pulsos) = queima(false);
    assert_eq!(vivas, 0);
    assert_eq!(pulsos, 2, "um segundo a meio segundo por pulso");
}

/// ⭐⭐ **A resistência vale para os PULSOS também** — o veneno de fogo numa salamandra não morde,
/// e o de gelo morde a dobrar. ⚠️ A aflição de um tipo a que a vida é imune ENTRA na mesma e sai a
/// zero pela taxa (a regra de quem é afligido é o golpe; a de quanto dói é a taxa).
///
/// **Mutação que deve sangrar:** passar `Taxa::NEUTRA` no `pulso` do `drive_health`.
#[test]
fn a_resistencia_vale_para_os_pulsos() {
    let salamandra = || resistente(&[("fogo", 0.0, false), ("gelo", 2.0, false)]);
    let (ponte, a, _) = envenena(salamandra(), "fogo", 0.0, (4.0, 3.0, 1.0), 300);
    assert_eq!(
        pontos(&ponte, a),
        100.0,
        "a queimadura não morde a salamandra"
    );
    let (ponte, a, _) = envenena(salamandra(), "gelo", 0.0, (4.0, 3.0, 1.0), 300);
    assert!(
        (pontos(&ponte, a) - 76.0).abs() < 1e-9,
        "o frio morde a dobrar: {}",
        pontos(&ponte, a)
    );
}

/// ⭐⭐ **O que o golpe não atravessa também não deixa a aflição** — dois espinhos no MESMO tique,
/// cada um com um veneno de tipo seu: o 1.º a chegar arma a invencibilidade e o 2.º é barrado
/// INTEIRO, logo entra UMA aflição; sem invencibilidade entram as duas.
///
/// ⚠️ **A 1.ª redacção dependia de QUAL espinho chegava primeiro** (um limpo e um envenenado) e leu
/// o contrário do que esperava: a ordem das fontes é a do `BTreeMap` de entidades, e no bevy ela é
/// a criação INVERTIDA (a lei que o `assign_missing_root_order` pagou em 15/09). *Uma fixtura cuja
/// resposta depende da ordem interna de um mapa mede o mapa* ⇒ os dois envenenam, e o número conta.
///
/// **Mutação que deve sangrar:** tirar o `!barrado` da condição do `aplica`.
#[test]
fn a_invencibilidade_barra_a_aflicao_com_o_golpe() {
    let cena = |invencivel_s: f32| -> usize {
        let mut sim = SimWorld::new();
        let a = alvo(
            &mut sim,
            Health {
                invincible_s: invencivel_s,
                ..Health::default()
            },
        );
        for kind in ["veneno", "fogo"] {
            espinho(
                &mut sim,
                Damage {
                    over_time_per_s: 4.0,
                    over_time_s: 3.0,
                    ..tipado(5.0, kind)
                },
            );
        }
        let mut ponte = PhysicsBridge::new();
        for t in 0..=2 {
            ponte.dispatch(&mut sim, true, t);
        }
        ponte.health_of(a).expect("tem vida").aflicoes().0.len()
    };
    assert_eq!(
        cena(1.0),
        1,
        "o 2.º golpe foi barrado, e o veneno dele com ele"
    );
    // O CONTROLO: sem invencibilidade os dois golpes entram, e os dois venenos com eles.
    assert_eq!(cena(0.0), 2);
}

/// ⭐⭐ **Uma ESQUIVA leva o veneno com ela** — a outra metade de *«o que o golpe não atravessa»*.
///
/// **Mutação que deve sangrar:** tirar o `!fs.contains(&HealthEventKind::Dodged)` do `aplica`.
#[test]
fn uma_esquiva_leva_o_veneno_com_ela() {
    let cena = |dodge: f32| -> usize {
        let vida = Health {
            dodge,
            ..Health::default()
        };
        let (ponte, a, _) = envenena(vida, "veneno", 5.0, (4.0, 3.0, 1.0), 2);
        ponte.health_of(a).expect("tem vida").aflicoes().0.len()
    };
    assert_eq!(cena(1.0), 0, "esquivou tudo, nada entrou");
    // O CONTROLO: sem esquiva o veneno entra.
    assert_eq!(cena(0.0), 1);
}

/// ⭐⭐ **A morte cura** — uma aflição que sobrevivesse faria o renascimento acordar envenenado, e
/// uma vida morta não pulsa.
///
/// **Mutação que deve sangrar:** apagar o `st.aflicoes.limpa()` da morte.
#[test]
fn a_morte_cura_as_aflicoes() {
    let (ponte, a, factos) = envenena(
        Health {
            start: 15.0,
            ..Health::default()
        },
        "veneno",
        0.0,
        (10.0, 5.0, 1.0),
        // ⚠️ DENTRO da duração (morre no 2.º pulso, ~2 s; o veneno duraria 5): a `400` tiques a
        // aflição já tinha acabado sozinha e a asserção de baixo ficava verde sem a limpeza.
        150,
    );
    assert_eq!(pontos(&ponte, a), 0.0);
    assert_eq!(
        factos
            .iter()
            .filter(|&&(_, k)| k == HealthEventKind::Died)
            .count(),
        1
    );
    assert!(
        ponte
            .health_of(a)
            .expect("tem vida")
            .aflicoes()
            .0
            .is_empty(),
        "a morte limpou o veneno"
    );
}

/// ⭐⭐⭐ **O veneno vai no ANEL** — um scrub a meio da aflição devolve a vida exacta do tique, e
/// andar dali para a frente reproduz os pulsos que faltavam, ao bit.
///
/// **Mutação que deve sangrar:** construir o `HealthState` do scrub com `Aflicoes::default()` (o
/// replay perde o veneno, e os pulsos depois do tique de volta desaparecem).
#[test]
fn um_scrub_devolve_o_veneno_do_tique() {
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, Health::default());
    espinho(
        &mut sim,
        Damage {
            over_time_per_s: 3.0,
            over_time_s: 4.0,
            over_time_every_s: 0.5,
            ..tipado(10.0, "veneno")
        },
    );
    let mut ponte = PhysicsBridge::new();
    let mut corrida = Vec::new();
    for t in 0..=300 {
        ponte.dispatch(&mut sim, true, t);
        corrida.push(pontos(&ponte, a).to_bits());
    }
    // O PISO: o veneno mexeu na vida depois do golpe.
    assert!(f64::from_bits(corrida[300]) < f64::from_bits(corrida[5]) - 5.0);
    for alvo_t in [100_u64, 37, 200, 61, 0] {
        ponte.dispatch(&mut sim, true, alvo_t);
        assert_eq!(
            pontos(&ponte, a).to_bits(),
            corrida[alvo_t as usize],
            "o scrub para {alvo_t}"
        );
    }
    for t in 1..=300 {
        ponte.dispatch(&mut sim, true, t);
        assert_eq!(
            pontos(&ponte, a).to_bits(),
            corrida[t as usize],
            "tique {t}"
        );
    }
}
