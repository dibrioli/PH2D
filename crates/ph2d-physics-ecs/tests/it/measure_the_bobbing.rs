//! **SONDA — de que é feito o bobeio do personagem na água.**
//!
//! O aberto que a W-KinFluid nomeou e não fechou: numa poça funda o personagem
//! oscila **1,44 m** de amplitude entre o 3.º e o 6.º segundo, **nos dois
//! modos** (dinâmico `1,4357` · cinemático `1,4394`), enquanto uma cápsula
//! solta — mesma forma, mesma densidade, mesma poça, sem lei de player nenhuma
//! — faz **0,81**.
//!
//! Ou seja: **a lei do player quase DOBRA a oscilação que a física sozinha
//! produz**, e a paridade entre modos prova que a causa não é do modo
//! cinemático. Esta sonda **não conserta nada** — ela atribui o excesso a um
//! termo, por **ablação da ENTRADA** (knobs do `PlatformPlayer`), nunca por
//! instrumentação: a mesma cena, o mesmo relógio, a mesma poça, e um termo de
//! cada vez fora do caminho.
//!
//! ⚠️ **O oráculo é a AMPLITUDE, não o `y` de um instante** — a lição que
//! custou o primeiro número desta frente (*"assenta a 0,4140, três milímetros
//! do dinâmico"* era uma amostra única de um sistema que oscila).
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_the_bobbing -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge,
    PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};

/// A cápsula das fixtures do player, e a poça 4× mais densa que ela — os mesmos
/// números do gate `player_in_water.rs`, para os dois falarem da mesma cena.
const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;
const FLUID: f32 = 4.0;
const START: f32 = 1.5;
const DRAG: f32 = 0.6;

fn pool(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 3.0,
            },
            ..Collider::default()
        },
        AreaBuoyancy(FLUID),
        AreaDrag(DRAG),
        Transform::from_translation(Vec2::new(0.0, -3.0)),
    ));
}

/// `law = None` ⇒ o CONTROLE (uma cápsula solta, sem lei de player).
fn subject(sim: &mut SimWorld, law: Option<PlatformPlayer>, kinematic: bool) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        Transform::from_translation(Vec2::new(0.0, START)),
    ));
    if let Some(p) = law {
        e.insert(p);
    }
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    e.id()
}

fn y_of(sim: &SimWorld) -> f32 {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Subject" {
            return t.translation.y;
        }
    }
    panic!("o sujeito tem de existir");
}

/// A média e a amplitude de `y` na **segunda metade** de seis segundos — a
/// mesma janela do gate (a primeira metade contém a entrada na água, que é
/// transiente e não regime).
fn stats(law: Option<PlatformPlayer>, kinematic: bool) -> (f32, f32) {
    let mut sim = SimWorld::new();
    pool(&mut sim);
    let who = subject(&mut sim, law, kinematic);
    let mut bridge = PhysicsBridge::new();
    if law.is_some() {
        bridge.set_player_input(who, PlayerInput::default());
    }
    let (mut lo, mut hi, mut sum, mut n) = (f32::MAX, f32::MIN, 0.0f64, 0u32);
    for t in 1..=360u64 {
        bridge.dispatch(&mut sim, true, t);
        if t > 180 {
            let y = y_of(&sim);
            lo = lo.min(y);
            hi = hi.max(y);
            sum += f64::from(y);
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((sum / f64::from(n)) as f32, hi - lo)
}

fn base() -> PlatformPlayer {
    PlatformPlayer {
        float_height: FLOAT,
        ..PlatformPlayer::default()
    }
}

/// Como a [`stats`], mas largando o sujeito de onde se pedir.
fn stats_from(law: Option<PlatformPlayer>, start: f32) -> (f32, f32) {
    let mut sim = SimWorld::new();
    pool(&mut sim);
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        Transform::from_translation(Vec2::new(0.0, start)),
    ));
    if let Some(p) = law {
        e.insert(p);
    }
    let who = e.id();
    let mut bridge = PhysicsBridge::new();
    if law.is_some() {
        bridge.set_player_input(who, PlayerInput::default());
    }
    let (mut lo, mut hi, mut sum, mut n) = (f32::MAX, f32::MIN, 0.0f64, 0u32);
    for t in 1..=360u64 {
        bridge.dispatch(&mut sim, true, t);
        if t > 180 {
            let y = y_of(&sim);
            lo = lo.min(y);
            hi = hi.max(y);
            sum += f64::from(y);
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((sum / f64::from(n)) as f32, hi - lo)
}

/// **DE ONDE VEM A ENERGIA: do ar antes do 1.º mergulho, ou de cada ciclo?**
///
/// A trava só DESARMA num contato com o chão, e nesta poça não há chão — então,
/// uma vez armada, ela devia calar a modelagem para sempre. Se for isso, largar
/// o personagem **já dentro da água** (a trava arma no tique 1) tem de o deixar
/// idêntico ao controle; e a diferença de `START = 1.5` seria só o transiente
/// aéreo de uma queda mais pesada.
#[test]
#[ignore = "sonda de medição"]
fn measure_where_the_extra_energy_enters() {
    println!("\n=== ONDE A ENERGIA ENTRA (mesma poca, alturas de largada) ===");
    println!(
        "{:<10} {:>10} {:>10} {:>12} {:>12}",
        "largado de", "ctrl amp", "player amp", "excesso", "player/ctrl"
    );
    for start in [1.5f32, 0.5, -0.5, -1.5] {
        let (_, c) = stats_from(None, start);
        let (_, p) = stats_from(Some(base()), start);
        println!(
            "{start:>10.1} {c:>10.4} {p:>10.4} {:>12.4} {:>12.2}x",
            p - c,
            p / c
        );
    }
    println!(
        "\nLEITURA: se o excesso SOBREVIVE a uma largada submersa (a trava arma no\n\
         tique 1), a modelagem nao esta' calada — e a cura e' na LEI. Se ele so'\n\
         existe na largada aerea, a energia entra ANTES da agua e a cura e' outra.\n"
    );
}

/// **QUANTO VALE, EM NÚMERO, a paridade APROXIMADA de arrasto entre os modos.**
///
/// O plano 07 nomeia a divergência pelo mecanismo — o solver amortece por
/// SUB-PASSO e a lei cinemática uma vez por TIQUE, `(1+d·h)⁻⁴` contra
/// `(1+d·4h)⁻¹` — e precifica-a por **analogia**: *"a mesma classe que a
/// W-AreaDrag mediu em 1,25%"*. ⚠️ **Uma analogia com outra medição não é a
/// medição desta**, e o §0 pede o número com a tabela ao lado.
///
/// A cena isola o termo: uma zona de arrasto **PURO** (sem empuxo — o empuxo
/// oscila e afogaria o sinal), os dois modos largados juntos, e o que se lê é a
/// diferença de queda ao longo do transiente. ⚠️ **A velocidade terminal NÃO
/// serve de oráculo:** ela é `g/d` nos dois por álgebra, então a divergência
/// vive só no caminho até lá — um gate na terminal seria verde por construção.
#[test]
#[ignore = "sonda de medição"]
fn measure_what_the_drag_parity_between_modes_is_worth() {
    println!("\n=== A PARIDADE DE ARRASTO ENTRE OS MODOS (zona de arrasto PURO) ===");
    println!(
        "{:>6} {:>12} {:>12} {:>12} {:>10}",
        "t (s)", "dinamico y", "cinematico y", "diferenca", "relativa"
    );

    let mut runs = Vec::new();
    for kinematic in [false, true] {
        let mut sim = SimWorld::new();
        // Arrasto puro: a MESMA poça, sem `AreaBuoyancy`.
        sim.world_mut().spawn((
            Name::new("Pool"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                is_sensor: true,
                shape: ColliderShape::Cuboid {
                    half_x: 20.0,
                    half_y: 40.0,
                },
                ..Collider::default()
            },
            AreaDrag(DRAG),
            Transform::from_translation(Vec2::new(0.0, -40.0)),
        ));
        let who = subject(&mut sim, Some(base()), kinematic);
        let mut bridge = PhysicsBridge::new();
        bridge.set_player_input(who, PlayerInput::default());
        let mut ys = Vec::new();
        for t in 1..=360u64 {
            bridge.dispatch(&mut sim, true, t);
            if t % 60 == 0 {
                ys.push(y_of(&sim));
            }
        }
        runs.push(ys);
    }

    let (d, k) = (&runs[0], &runs[1]);
    let mut worst = 0.0f32;
    for (i, (a, b)) in d.iter().zip(k.iter()).enumerate() {
        let travelled = START - a;
        let rel = if travelled.abs() > 1e-3 {
            (b - a).abs() / travelled.abs()
        } else {
            0.0
        };
        worst = worst.max(rel);
        println!(
            "{:>6} {a:>12.4} {b:>12.4} {:>12.4} {:>9.3}%",
            i + 1,
            b - a,
            rel * 100.0
        );
    }
    println!(
        "\nPIOR divergencia relativa: {:.3}%\n\
         LEITURA: a nota do plano 07 precifica isto por ANALOGIA com os 1,25% que\n\
         a W-AreaDrag mediu noutro sitio. Este e' o numero DESTA paridade.\n",
        worst * 100.0
    );
}

/// **O ELO QUE FECHA O MECANISMO: com que velocidade cada um ENTRA na água.**
///
/// `fall_gravity` vale `2.0` por default, e a modelagem age no ar — logo o
/// personagem tem de cruzar a superfície mais depressa do que o controle. Se
/// este número não sair, a explicação é plausível em vez de medida.
///
/// ⚠️ **E a minha previsão ingénua estava errada:** eu escrevi `√2 = 1,414×`
/// lendo só o `fall_gravity`, e o medido é **`1,299×`**. O que falta na conta é
/// o **`peak_gravity = 0.5`**, que torna o COMEÇO da queda mais LEVE que o
/// mundo — a queda não é pesada inteira, ela é leve no ápice e pesada depois. O
/// número honesto sai da medição, e ele fecha o resto: `1,299² = 1,687×` de
/// energia contra os `1,78×` de amplitude observados.
#[test]
#[ignore = "sonda de medição"]
fn measure_the_speed_at_which_each_enters_the_water() {
    println!("\n=== A VELOCIDADE DE ENTRADA (largado de y = 1.5, superficie em y = 0) ===");
    println!(
        "{:<26} {:>12} {:>12}",
        "sujeito", "v na entrada", "vs controle"
    );
    let mut control = 0.0f32;
    for (label, law) in [("capsula solta (CONTROLE)", None), ("player", Some(base()))] {
        let mut sim = SimWorld::new();
        pool(&mut sim);
        let who = subject(&mut sim, law, false);
        let mut bridge = PhysicsBridge::new();
        if law.is_some() {
            bridge.set_player_input(who, PlayerInput::default());
        }
        let mut prev = START;
        let mut entry = 0.0f32;
        for t in 1..=360u64 {
            bridge.dispatch(&mut sim, true, t);
            let y = y_of(&sim);
            // O tique em que a superfície é cruzada para baixo.
            if prev > 0.0 && y <= 0.0 && entry == 0.0 {
                entry = (prev - y) * 60.0;
            }
            prev = y;
        }
        if law.is_none() {
            control = entry;
            println!("{label:<26} {entry:>12.4} {:>12}", "-");
        } else {
            println!("{label:<26} {entry:>12.4} {:>11.3}x", entry / control);
        }
    }
    println!(
        "\nLEITURA: `fall_gravity = 2.0` sozinho preveria `sqrt(2) = 1,414x`, e o\n\
         medido e' 1,299x — a diferenca e' o `peak_gravity = 0.5`, que deixa o\n\
         COMECO da queda mais leve que o mundo. E o quadrado disso (1,687x de\n\
         energia) fecha com os 1,78x de amplitude, com a folga a vir da\n\
         saturacao do empuxo (submerso, ele e' constante em vez de linear).\n"
    );
}

/// **DECAI OU ACUMULA?** — a pergunta que decide se há defeito.
///
/// A modelagem do arco é **não-conservativa por construção** (subir com `g` e
/// descer com `2·g` devolve o corpo ao mesmo nível com `√2` da velocidade), e é
/// isso que a trava existe para conter. Se ela contém, o excesso da entrada é um
/// **transiente** e o arrasto do meio come-o; se escapa, a amplitude cresce e o
/// personagem sai de quadro — que é o modo de falha que o `Buoyed` documenta
/// (`−1,05 / +4,71 / +12,08 / −20,31`).
///
/// ⚠️ **Uma janela só não distingue as duas.** O oráculo é a SEQUÊNCIA de
/// amplitudes por janela de 3 s: monotónica a descer = transiente.
#[test]
#[ignore = "sonda de medição"]
fn measure_whether_the_bobbing_decays_or_pumps() {
    println!("\n=== DECAI OU ACUMULA? (30 s, amplitude por janela de 3 s) ===");
    println!("{:<26} amplitude por janela de 3 s", "sujeito");

    for (label, law) in [("capsula solta (CONTROLE)", None), ("player", Some(base()))] {
        let mut sim = SimWorld::new();
        pool(&mut sim);
        let who = subject(&mut sim, law, false);
        let mut bridge = PhysicsBridge::new();
        if law.is_some() {
            bridge.set_player_input(who, PlayerInput::default());
        }
        let mut out = String::new();
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for t in 1..=1800u64 {
            bridge.dispatch(&mut sim, true, t);
            let y = y_of(&sim);
            lo = lo.min(y);
            hi = hi.max(y);
            if t % 180 == 0 {
                out.push_str(&format!("{:>7.3}", hi - lo));
                lo = f32::MAX;
                hi = f32::MIN;
            }
        }
        println!("{label:<26}{out}");
    }
    println!(
        "\nLEITURA: a descer = transiente da entrada, e o meio come-o (nao ha' defeito).\n\
         A crescer, ou plana num valor alto = a modelagem esta' a bombear energia.\n"
    );
}

/// **A TABELA** — cada linha tira UM termo do caminho e mede a mesma poça.
#[test]
#[ignore = "sonda de medição"]
fn measure_what_the_bobbing_is_made_of() {
    println!("\n=== DE QUE E' FEITO O BOBEIO (poca funda, 6 s, regime = 2a metade) ===");
    println!("{:<34} {:>10} {:>12}", "sujeito", "y medio", "amplitude");

    let (c_mean, c_amp) = stats(None, false);
    println!(
        "{:<34} {c_mean:>10.4} {c_amp:>12.4}",
        "capsula solta (CONTROLE)"
    );

    let cases: [(&str, PlatformPlayer); 6] = [
        ("player default", base()),
        (
            "  sem multiplicadores (g = 1)",
            PlatformPlayer {
                takeoff_gravity: 1.0,
                peak_gravity: 1.0,
                fall_gravity: 1.0,
                cut_gravity: 1.0,
                ..base()
            },
        ),
        (
            "  sem PERNA (spring_strength 0)",
            PlatformPlayer {
                spring_strength: 0.0,
                ..base()
            },
        ),
        (
            "  sem amortecimento da perna",
            PlatformPlayer {
                spring_damping: 0.0,
                ..base()
            },
        ),
        (
            "  raio de chao mudo (cling 0)",
            PlatformPlayer {
                cling_distance: 0.0,
                ..base()
            },
        ),
        (
            "  perna INTEIRA fora",
            PlatformPlayer {
                spring_strength: 0.0,
                spring_damping: 0.0,
                cling_distance: 0.0,
                float_height: 0.0,
                ..base()
            },
        ),
    ];

    for (label, law) in cases {
        let (mean, amp) = stats(Some(law), false);
        println!(
            "{label:<34} {mean:>10.4} {amp:>12.4}   ({:+.4} vs controle)",
            amp - c_amp
        );
    }

    let (k_mean, k_amp) = stats(Some(base()), true);
    println!(
        "{:<34} {k_mean:>10.4} {k_amp:>12.4}",
        "player CINEMATICO (paridade)"
    );

    println!(
        "\nLEITURA: a linha cuja amplitude cai para perto do CONTROLE nomeia o termo.\n\
         Se NENHUMA cair, o excesso nao esta' nos knobs — esta' no caminho que a\n\
         lei escreve de volta ao corpo, e a proxima sonda tem de olhar para la'.\n"
    );
}

/// **A TRAVA ARMA?** — a segunda pergunta, depois de a tabela nomear os
/// multiplicadores.
///
/// A `JumpState::waterborne` existe para calar exactamente esses multiplicadores
/// (`extra = if waterborne { 0.0 } else { scale - 1.0 }`), e a tabela acima diz
/// que eles agem. Ou a trava não arma, ou ela arma e não cala — e as duas curas
/// são diferentes.
///
/// ⚠️ **A sonda não pode ler o `bool`** (ele é estado interno da lei), então lê
/// o que o ALIMENTA: a razão empuxo÷peso que a ponte publica. `carries_weight()`
/// é `> 0`, então uma série de zeros responde a pergunta sozinha.
#[test]
#[ignore = "sonda de medição"]
fn measure_whether_the_water_lock_ever_arms() {
    println!("\n=== A TRAVA DO FLUIDO ARMA? (razao empuxo/peso publicada pela ponte) ===");
    println!("{:>6} {:>10} {:>10}", "t", "y", "buoyed");

    let mut sim = SimWorld::new();
    pool(&mut sim);
    let who = subject(&mut sim, Some(base()), false);
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(who, PlayerInput::default());

    let mut armed_ticks = 0u32;
    for t in 1..=360u64 {
        bridge.dispatch(&mut sim, true, t);
        let b = bridge.buoyed(who);
        if b > 0.0 {
            armed_ticks += 1;
        }
        if t % 30 == 0 {
            println!("{t:>6} {:>10.4} {b:>10.4}", y_of(&sim));
        }
    }
    println!(
        "\ntiques com empuxo > 0: {armed_ticks} de 360\n\
         LEITURA: zero ⇒ a trava nunca arma e a cura e' na fiacao;\n\
         muitos ⇒ ela arma e o termo escapa por outro caminho.\n"
    );
}
