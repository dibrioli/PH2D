//! **COMO O IMPULSO DA PERNA É INTEGRADO** — a sonda que decide a W11.
//!
//! O `BUGS_physics.md` §7 nomeia o mecanismo da subida involuntária: *a perna
//! cancela a gravidade com um impulso no topo do tique, e o `rapier` integra a
//! gravidade AO LONGO dele*. O que aquela medição não fez foi descer um degrau e
//! perguntar **quem é "ao longo"** — e a resposta está no `world/step.rs`: o
//! tique é um laço de `substeps` sub-passos, e `drag`, `effector`, `blast` e as
//! polias são todos aplicados DENTRO dele, com o comentário que diz por quê
//! (*"per SUBSTEP, not per tick: a force applied once per tick would be wrong by
//! the substep count"*). O motor do player é aplicado **uma vez, antes do laço**.
//!
//! # A aritmética, e por que ela dá um discriminador exato
//!
//! Com `n` sub-passos de `h = dt/n`, o impulso no topo entra uma vez e a
//! gravidade `n` vezes:
//!
//! ```text
//!   x = x0 + dt·v0 + a·dt²          + g·dt²·(n+1)/(2n)
//!   ideal (a fatiado como g):
//!   x = x0 + dt·v0 + (a+g)·dt²·(n+1)/(2n)
//!   erro  =  a·dt²·( 1 − (n+1)/(2n) )
//! ```
//!
//! ⚠️ **Em `n = 1` o erro é EXATAMENTE zero** (`1 − 1 = 0`), e ele CRESCE com o
//! número de sub-passos (0 · 0,25 · 0,375 · 0,4375 para 1 · 2 · 4 · 8). Se a
//! deriva da rampa é este mecanismo, ela some com `substeps = 1` e engorda com
//! `substeps = 8` — e nenhuma outra hipótese sobre a perna prevê isso.
//!
//! ⚠️ **A velocidade cancela nos dois casos**, e é por isso que o defeito
//! sobreviveu: o impulso total é `a·dt` seja lá como for fatiado. O que difere é
//! só o DESLOCAMENTO, que é o que o artista vê.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_substep -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PhysicsSettings,
    PlatformPlayer, RigidBody,
};
use scene_fixture::pose;

/// A altura de flutuação dos gates de repouso — acima do mínimo geométrico
/// desta cápsula (0,5), então a perna de fato PAIRA em vez de disputar a rampa
/// com o solver de contato.
const FLOAT: f32 = 0.9;

/// Quantos segundos a perna tem para assentar antes de a medição começar — a
/// mesma premissa do `platform_idle.rs`, e pelo mesmo motivo: o assentamento
/// não é deriva.
const SETTLE_SECS: u64 = 2;

fn rig(slope_deg: f32, damping: f32, substeps: u32) -> (SimWorld, PhysicsBridge) {
    let slope = slope_deg.to_radians();
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 200.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform {
            rotation: slope,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    sim.world_mut().spawn((
        Name::new("Player"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            spring_damping: damping,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.5 / slope.cos() + FLOAT)),
    ));
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(PhysicsSettings {
        substeps,
        ..PhysicsSettings::default()
    });
    (sim, bridge)
}

/// O mesmo rig com a RIGIDEZ escolhida — ela é autorável (`spring_strength` só
/// é clampada em `>= 0`), então um piso medido numa rigidez só vale se ele não
/// se mover com ela.
fn rig_k(slope_deg: f32, damping: f32, substeps: u32, k: f32) -> (SimWorld, PhysicsBridge) {
    let (mut sim, bridge) = rig(slope_deg, damping, substeps);
    let mut q = sim.world_mut().query::<&mut PlatformPlayer>();
    for mut p in q.iter_mut(sim.world_mut()) {
        p.spring_strength = k;
    }
    (sim, bridge)
}

/// Quanto o personagem VIAJOU em `secs` segundos sem ninguém tocar nele.
fn idle_travel(slope_deg: f32, secs: u64, damping: f32, substeps: u32) -> f32 {
    let (mut sim, mut bridge) = rig(slope_deg, damping, substeps);
    for t in 1..=SETTLE_SECS * 60 {
        bridge.dispatch(&mut sim, true, t);
    }
    let start = pose(&sim);
    for t in SETTLE_SECS * 60 + 1..=(SETTLE_SECS + secs) * 60 {
        bridge.dispatch(&mut sim, true, t);
    }
    let end = pose(&sim);
    ((end.0 - start.0).powi(2) + (end.1 - start.1).powi(2)).sqrt()
}

/// **O DISCRIMINADOR:** a deriva contra o número de sub-passos.
///
/// A previsão da aritmética do topo é `erro ∝ 1 − (n+1)/(2n)`: **zero** em
/// `n = 1`, e crescendo daí. Se a tabela seguir essa forma, a causa é o
/// descasamento de integração e a cura é fatiar o impulso da perna como o
/// `drag` já é fatiado — não suprimir a gravidade.
#[test]
#[ignore = "sonda"]
fn measure_the_drift_against_the_substep_count() {
    println!("\n=== A DERIVA CONTRA O NUMERO DE SUB-PASSOS (30 graus, 10 s) ===");
    println!("previsao do erro por tique:  a·dt²·(1 − (n+1)/(2n))\n");
    println!(
        "{:>9}  {:>9}  {:>9}  {:>9}  {:>9}  {:>12}",
        "substeps", "d = 0.00", "d = 0.25", "d = 0.50", "d = 1.00", "fator prev."
    );
    for &n in &[1_u32, 2, 4, 8] {
        let predicted = 1.0 - (f64::from(n) + 1.0) / (2.0 * f64::from(n));
        println!(
            "{n:>9}  {:>9.4}  {:>9.4}  {:>9.4}  {:>9.4}  {predicted:>12.4}",
            idle_travel(30.0, 10, 0.0, n),
            idle_travel(30.0, 10, 0.25, n),
            idle_travel(30.0, 10, 0.5, n),
            idle_travel(30.0, 10, 1.0, n),
        );
    }
    println!("\n(o default do produto e' substeps = 4)");
    println!("previsao a d=0:  4.905 · dt² · fator · 600 tiques");
    for &n in &[1_u32, 2, 4, 8] {
        let f = 1.0 - (f64::from(n) + 1.0) / (2.0 * f64::from(n));
        println!(
            "  n={n}: {:.4} m",
            4.905 * (1.0 / 60.0f64).powi(2) * f * 600.0
        );
    }
}

/// **DE ONDE VEM A COMPONENTE HORIZONTAL** — a pergunta que a tabela acima
/// deixou aberta.
///
/// A `d = 0` o amortecedor está desligado, a mola empurra ao longo do `up` e a
/// gravidade também: **as duas são verticais**. O único ator horizontal que
/// sobra é o freio da caminhada — e um freio não cria movimento. Esta sonda
/// imprime o estado tique a tique para que a atribuição venha do produto.
#[test]
#[ignore = "sonda"]
fn measure_where_the_horizontal_comes_from() {
    for &(d, n) in &[(0.0_f32, 1_u32), (0.0, 4), (0.5, 4), (1.0, 4)] {
        let (mut sim, mut bridge) = rig(30.0, d, n);
        for t in 1..=SETTLE_SECS * 60 {
            bridge.dispatch(&mut sim, true, t);
        }
        println!("\n=== d = {d:.2}, substeps = {n} — dez tiques depois de assentar ===");
        println!(
            "{:>5}  {:>10}  {:>10}  {:>10}  {:>10}",
            "tique", "x", "y", "dx", "dy"
        );
        let mut prev = pose(&sim);
        for t in SETTLE_SECS * 60 + 1..=SETTLE_SECS * 60 + 10 {
            bridge.dispatch(&mut sim, true, t);
            let p = pose(&sim);
            println!(
                "{t:>5}  {:>10.6}  {:>10.6}  {:>10.7}  {:>10.7}",
                p.0,
                p.1,
                p.0 - prev.0,
                p.1 - prev.1
            );
            prev = p;
        }
        // A razão dy/dx diz se ele anda AO LONGO da rampa (tan 30° = 0,5774) ou
        // noutra direção qualquer.
        let start = pose(&sim);
        for t in SETTLE_SECS * 60 + 11..=SETTLE_SECS * 60 + 610 {
            bridge.dispatch(&mut sim, true, t);
        }
        let end = pose(&sim);
        let (dx, dy) = (end.0 - start.0, end.1 - start.1);
        println!(
            "  em 10 s:  dx = {dx:.4}  dy = {dy:.4}  dy/dx = {:.4}  (tan 30 = 0.5774)",
            dy / dx
        );
    }
}

/// **ONDE FICA O PISO DO AMORTECIMENTO** — o irmão medido do teto que o
/// [`ph2d_platformer::RideConfig::MAX_DAMPING`] já tem.
///
/// Uma mola integrada por FORÇA amostrada na posição do início do tique é
/// explícita, e uma mola explícita **sem amortecimento nenhum ganha energia** —
/// é livro-texto, não um acidente desta wave. O que a sonda responde é *a
/// partir de que valor ela para de ganhar*, para que o piso seja um número
/// medido e não um palpite.
///
/// A coluna que decide é o **erro de repouso no plano**: enquanto ele fica na
/// casa do milímetro a perna está a segurar; quando salta para a casa do metro
/// ela largou o personagem.
#[test]
#[ignore = "sonda"]
fn measure_where_the_damping_floor_is() {
    println!("\n=== O PISO DO AMORTECIMENTO (plano, 10 s, folga pedida 0,9 m) ===\n");
    println!(
        "{:>9}  {:>14}  {:>14}  {:>12}",
        "damping", "erro rep. (mm)", "deriva 30° (m)", "veredito"
    );
    for &d in &[0.0_f32, 0.02, 0.03, 0.04, 0.05, 0.06, 0.10, 0.50, 1.00] {
        let (mut sim, mut bridge) = rig(0.0, d, 4);
        for t in 1..=600 {
            bridge.dispatch(&mut sim, true, t);
        }
        let held = pose(&sim).1 - 0.5;
        let err_mm = (held - FLOAT) * 1000.0;
        let drift = idle_travel(30.0, 10, d, 4);
        let verdict = if err_mm.abs() < 20.0 {
            "segura"
        } else {
            "LARGOU"
        };
        println!("{d:>9.2}  {err_mm:>14.3}  {drift:>14.4}  {verdict:>12}");
    }
}

/// ⚠️ **O piso depende da RIGIDEZ?** — a pergunta que decide se ele pode ser um
/// número ou tem de ser uma fórmula.
///
/// `spring_strength` é autorável e só é clampado em `>= 0` (o
/// `inspector_player.rs`), então um piso medido a `k = 400` que não valha a
/// `k = 3200` seria um piso que mente. A forma que a teoria prevê para um
/// oscilador integrado por força AMOSTRADA na posição do início do tique é
/// `d ≥ k·dt²/(1 + k·dt²)` — se a medição seguir isso, o piso é a fórmula.
#[test]
#[ignore = "sonda"]
fn measure_whether_the_floor_moves_with_the_stiffness() {
    println!("\n=== O PISO CONTRA A RIGIDEZ (plano, 10 s) ===\n");
    println!(
        "{:>8}  {:>10}  {:>28}  {:>12}",
        "k", "piso medido", "k·dt²/(1+k·dt²)", "razao"
    );
    for &k in &[100.0_f32, 200.0, 400.0, 800.0, 1600.0, 3200.0] {
        let mut floor = f32::NAN;
        let mut d = 0.0_f32;
        while d <= 1.0 {
            let (mut sim, mut bridge) = rig_k(0.0, d, 4, k);
            for t in 1..=600 {
                bridge.dispatch(&mut sim, true, t);
            }
            let err = ((pose(&sim).1 - 0.5) - FLOAT).abs();
            if err < 0.02 {
                floor = d;
                break;
            }
            d += 0.01;
        }
        let dt2 = (1.0 / 60.0f32).powi(2);
        let predicted = k * dt2 / (1.0 + k * dt2);
        println!(
            "{k:>8.0}  {floor:>10.2}  {predicted:>28.4}  {:>12.2}",
            floor / predicted
        );
    }
}

/// E o **CONTROLE**: no plano não há componente tangente onde a sobra se
/// esconda, então nenhuma contagem de sub-passos pode produzir viagem.
///
/// Sem esta metade, *"a deriva some em `n = 1`"* seria compatível com *"a sonda
/// mede zero em toda parte"*.
#[test]
#[ignore = "sonda"]
fn measure_the_flat_control_across_substep_counts() {
    println!("\n=== O CONTROLE: o PLANO, a mesma varredura ===\n");
    println!("{:>9}  {:>12}", "substeps", "viagem (m)");
    for &n in &[1_u32, 2, 4, 8] {
        println!("{n:>9}  {:>12.6}", idle_travel(0.0, 10, 0.5, n));
    }
}

/// **O QUE SOBRA, no referencial da rampa** — a sonda que decide se o resíduo da
/// W11b fecha barato.
///
/// Ele é exactamente linear em `(1 − d)` e zero no teto, então é alguma coisa
/// que o amortecedor — que age na NORMAL — remove por completo. Duas explicações
/// óbvias já foram descartadas **pelo sinal**: a mola, no repouso, tem offset
/// NEGATIVO (o personagem paira acima do pedido) e o seu termo tangente aponta
/// rampa ABAIXO; e o freio da caminhada calcula `−8,7e−8`, que é nada.
///
/// Esta imprime o estado assentado no referencial `{tangente, normal}`, que é
/// onde as duas leis vivem.
#[test]
#[ignore = "sonda"]
fn measure_what_is_left_in_the_ramp_frame() {
    let n: [f32; 2] = [-0.5, 0.866_025_4];
    let t = [n[1], -n[0]]; // perp_cw
    for &d in &[0.0_f32, 0.25, 0.5, 1.0] {
        let (mut sim, mut bridge) = rig(30.0, d, 4);
        for tick in 1..=SETTLE_SECS * 60 {
            bridge.dispatch(&mut sim, true, tick);
        }
        println!("\n=== d = {d:.2} — dez tiques assentados, no referencial da rampa ===");
        println!(
            "{:>5}  {:>12}  {:>12}",
            "tique", "ds (tang.)", "dn (normal)"
        );
        let mut prev = pose(&sim);
        let (mut sum_s, mut sum_n) = (0.0_f32, 0.0_f32);
        for tick in SETTLE_SECS * 60 + 1..=SETTLE_SECS * 60 + 10 {
            bridge.dispatch(&mut sim, true, tick);
            let p = pose(&sim);
            let (dx, dy) = (p.0 - prev.0, p.1 - prev.1);
            let (ds, dn) = (dx * t[0] + dy * t[1], dx * n[0] + dy * n[1]);
            println!("{tick:>5}  {ds:>12.8}  {dn:>12.8}");
            sum_s += ds;
            sum_n += dn;
            prev = p;
        }
        println!("  soma: tangente {sum_s:.7}  normal {sum_n:.7}");
    }
}

/// **A VARREDURA QUE FALTAVA: a deriva contra a INCLINAÇÃO.**
///
/// ⚠️ A tabela do handoff foi medida **só a 30°**, e o resíduo tem DOIS termos
/// que não escalam igual com o ângulo:
///
/// - o agrupamento da mola, `D ∝ g·sen θ·(n−1)/(2n²)` — sobe a rampa;
/// - a queda da mola no repouso, `k·erro·(up·t) ∝ k·erro·sen θ` — desce, e o
///   `erro` de repouso é função da carga NORMAL, isto é de `cos θ`.
///
/// Se o segundo termo carrega um `cos θ` que o primeiro não tem, então **o
/// cancelamento no teto é função da rampa** e o `0,0000` da tabela é um fato
/// sobre 30°, não sobre o knob. É exactamente isso que esta sonda decide.
#[test]
#[ignore = "sonda"]
fn measure_the_drift_against_the_slope() {
    println!("\n=== A DERIVA CONTRA A INCLINACAO (10 s parado, 4 sub-passos) ===");
    println!(
        "{:>7}  {:>9}  {:>9}  {:>9}  {:>9}",
        "rampa", "d=0.25", "d=0.50", "d=0.75", "d=1.00"
    );
    for &slope in &[20.0_f32, 30.0, 35.0, 40.0, 45.0, 50.0] {
        let row: Vec<f32> = [0.25_f32, 0.5, 0.75, 1.0]
            .iter()
            .map(|&d| idle_travel(slope, 10, d, 4))
            .collect();
        println!(
            "{:>6.0}°  {:>9.4}  {:>9.4}  {:>9.4}  {:>9.4}",
            slope, row[0], row[1], row[2], row[3]
        );
    }
    println!(
        "\nO QUE ELA MEDIU (2026-08-05), e a hipotese acima morreu aqui:\n\
         · a coluna d=1.00 da' 0.0000 EXACTO de 20deg a 45deg -- o teto e' um\n\
           piso de verdade, nao um cruzamento calibrado numa rampa so';\n\
         · e as outras tres colunas colapsam numa lei unica,\n\
               deriva(10 s) = 0,153 · sen(theta) · (1 - d)   metros,\n\
           que reproduz os 24 valores ao quarto decimal. Daí a leitura de\n\
           produto: a deriva CRESCE com a rampa (40deg e' 1,29x a de 30deg),\n\
           que foi o que o smoke viu como \"um pouco mais rapido\".\n\
         · 50deg viaja centenas de metros porque ESCORREGA: o limite de rampa\n\
           autorado e' 45deg, e a cena =88 existe para esse par."
    );
}

/// O rig do POUSO: a mesma cápsula, no plano, largada de `drop` metros ACIMA da
/// altura de repouso da perna.
fn landing_rig(damping: f32, substeps: u32, drop: f32) -> (SimWorld, PhysicsBridge) {
    let (mut sim, bridge) = rig(0.0, damping, substeps);
    let mut q = sim.world_mut().query::<(&PlatformPlayer, &mut Transform)>();
    for (_, mut t) in q.iter_mut(sim.world_mut()) {
        t.translation.y += drop;
    }
    (sim, bridge)
}

/// **O QUIQUE DO POUSO, em milímetros** — quanto a perna sobe acima da altura de
/// repouso depois de tocar.
///
/// ⚠️ **O oráculo é o PICO depois do mínimo**, não a altura final: um pouso sem
/// quique e um pouso com quique acabam no mesmo lugar, e é o caminho entre os
/// dois que o artista vê.
fn landing_bounce_mm(damping: f32, substeps: u32) -> f32 {
    let rest = 0.5 + FLOAT;
    let (mut sim, mut bridge) = landing_rig(damping, substeps, 1.5);
    let mut lowest = f32::INFINITY;
    let mut peak_after = f32::NEG_INFINITY;
    for t in 1..=300 {
        bridge.dispatch(&mut sim, true, t);
        let y = pose(&sim).1;
        if y < lowest {
            lowest = y;
            peak_after = f32::NEG_INFINITY;
        } else if y > peak_after {
            peak_after = y;
        }
    }
    ((peak_after - rest) * 1000.0).max(0.0)
}

/// **⚠️ A TABELA DO `BUGS_physics.md` §7 ESTÁ STALE, E O SINAL DELA INVERTEU.**
///
/// Aquela tabela diz que a deriva **CRESCE** com o número de sub-passos
/// (`0,1533 → 0,1788 → 0,1916 → 0,1980` a `d = 0,5`) e ajusta
/// `A·(1 − 1/(4n))·(1 − d)`, cujo limite `n → ∞` é o próprio `A`. Ela foi medida
/// **antes da wave `gravity_hold`**, que passou a integrar o cancelamento da
/// gravidade como a gravidade — e depois dela a série **CAI pela metade a cada
/// dobra** (`1/n` exacto), com o `n = 1` idêntico.
///
/// A consequência é de PRODUTO, e é o contrário do que a nota do `RideConfig`
/// deixa entender: **a deriva e o quique deixaram de estar soldados.** Eles são
/// os dois `∝ (1 − d)`, mas só a deriva é `∝ 1/n` — então `substeps` é um
/// terceiro eixo que compra o quique de volta sem devolver a subida inteira.
///
/// ⚠️ **E ela reconcilia a tentativa REJEITADA:** fatiar o motor *"corta a
/// deriva 4×"* (BUGS §7 (3)) — que é exactamente **uma potência de `n`** no
/// default `n = 4`. Fatiar não some com o defeito; ele desloca a série de um
/// degrau, e o degrau já é comprável pelo knob que o artista tem.
#[test]
#[ignore = "sonda"]
fn measure_whether_substeps_buy_the_bounce_back() {
    println!("\n=== A DERIVA (30 graus, 10 s, m) x O QUIQUE DO POUSO (plano, mm) ===");
    println!("(⚠️ a tabela do BUGS §7 e' PRE-`gravity_hold`: la' a deriva CRESCE com n)\n");
    println!(
        "{:>9}  {:>22}  {:>22}",
        "substeps", "d=0.25  deriva / quique", "d=0.50  deriva / quique"
    );
    for &n in &[1_u32, 2, 4, 8, 12] {
        println!(
            "{n:>9}  {:>10.4} / {:>8.1}  {:>10.4} / {:>8.1}",
            idle_travel(30.0, 10, 0.25, n),
            landing_bounce_mm(0.25, n),
            idle_travel(30.0, 10, 0.5, n),
            landing_bounce_mm(0.5, n),
        );
    }
    println!(
        "\n{:>9}  {:>22}",
        "substeps", "d=1.00 (o default) deriva / quique"
    );
    for &n in &[4_u32, 12] {
        println!(
            "{n:>9}  {:>10.4} / {:>8.1}",
            idle_travel(30.0, 10, 1.0, n),
            landing_bounce_mm(1.0, n)
        );
    }
}
