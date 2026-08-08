//! **SONDA — a BOMBA DE ENERGIA, e a grandeza que a desarma.**
//!
//! A cena `=100` (smoke da W-Water) fechou com um item aberto que não é
//! polimento: um personagem largado DENTRO da poça não bóia — ele **bombeia**, e
//! sai da cena. Medido lá: `−1,05 / +4,71 / +12,08 / −20,31`.
//!
//! # ⚠️ O mecanismo, e por que ele é do PULO e não do empuxo
//!
//! Os multiplicadores de gravidade que moldam um arco balístico (`peak 0,5` no
//! ápice, `fall 2,0` na queda) descrevem um corpo em **voo livre**, onde a
//! gravidade é a única força e o arco é o produto. Quando é o **empuxo** quem o
//! segura, a mesma modelagem vira uma **amplificação paramétrica**: pesado ao
//! descer injeta mais energia do que leve ao subir devolve, ciclo após ciclo. O
//! empuxo é a mola; a modelagem é quem a empurra no ritmo dela.
//!
//! Esta sonda mede as duas metades ANTES de a lei mudar:
//!
//! 1. **quanto** de um personagem o fluido segura, em cada regime, e
//! 2. que a ablação dos multiplicadores — e só ela — o estabiliza.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_submersion -- --ignored --nocapture`

#[path = "platform_water_scene.rs"]
mod water;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer};

use water::{FLOAT, FLUID, floor, pool, subject, subject_tuned, y_of};

/// Um `PlatformPlayer` com os quatro multiplicadores de gravidade **neutros** —
/// a ablação que separa *"o pulo bombeia"* de *"o empuxo oscila"*.
fn neutral_gravity() -> PlatformPlayer {
    PlatformPlayer {
        float_height: FLOAT,
        takeoff_gravity: 1.0,
        peak_gravity: 1.0,
        fall_gravity: 1.0,
        cut_gravity: 1.0,
        ..PlatformPlayer::default()
    }
}

/// Simula e devolve `(t, y, empuxo÷peso)` a cada `every` tiques.
fn trace(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    who: Entity,
    ticks: u64,
    every: u64,
) -> Vec<(f32, f32, f32)> {
    let mut out = Vec::new();
    for t in 1..=ticks {
        bridge.dispatch(sim, true, t);
        if t % every == 0 {
            out.push((t as f32 / 60.0, y_of(sim, "Subject"), bridge.buoyed(who)));
        }
    }
    out
}

/// **Quanto o fluido segura, em cada regime.**
///
/// ⚠️ O CONTROLE é a cápsula idêntica sem `PlatformPlayer`. É ele que diz qual é
/// a submersão de EQUILÍBRIO desta densidade — o número que uma lei de flutuação
/// tem de produzir, e que nenhum literal deveria afirmar.
#[test]
#[ignore = "sonda de medição"]
fn measure_how_much_the_fluid_holds() {
    println!("\n=== QUANTO O FLUIDO SEGURA (densidade do fluido = {FLUID}x a do corpo) ===");
    println!(
        "{:<34} {:>9} {:>12}",
        "sujeito / regime", "y final", "empuxo/peso"
    );
    for (label, player, y0) in [
        ("capsula (CONTROLE), boiando", false, 0.5),
        ("capsula (CONTROLE), largada funda", false, -2.0),
        ("player, entrando de cima", true, 0.5),
        ("player, largado dentro", true, -2.0),
    ] {
        let mut sim = SimWorld::new();
        pool(&mut sim, 0.0);
        let who = subject(&mut sim, player, y0);
        let mut bridge = PhysicsBridge::new();
        for t in 1..=600u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        println!(
            "{label:<34} {:>9.4} {:>12.4}",
            y_of(&sim, "Subject"),
            bridge.buoyed(who)
        );
    }

    println!("\n  e o que a razao vale FORA da agua e SUBMERSO:");
    // ⚠️ O 2º caso mede DENTRO da poça, com chão para ele não cair pelo fundo —
    // a 1ª versão largava em y = -5,5 SEM chão e media um corpo que já tinha
    // saído da poça pela base (razão 0,0000, informativo sobre nada).
    for (label, y0, with_floor) in [
        ("em pe' no chao seco", 3.0, true),
        ("no fundo da poca", -4.0, true),
    ] {
        let mut sim = SimWorld::new();
        pool(&mut sim, 0.0);
        floor(&mut sim, if with_floor && y0 > 0.0 { 2.0 } else { -5.5 });
        let who = subject(&mut sim, true, y0);
        let mut bridge = PhysicsBridge::new();
        for t in 1..=300u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        println!(
            "  {label:<32} {:>9.4} {:>12.4}",
            y_of(&sim, "Subject"),
            bridge.buoyed(who)
        );
    }
}

/// **A BOMBA, e a ablação que a desarma.**
///
/// ⚠️ As duas colunas correm a MESMA cena, o MESMO número de tiques e a MESMA
/// poça — só os quatro multiplicadores mudam. Uma coluna que divergisse por
/// outro motivo não separaria nada.
#[test]
#[ignore = "sonda de medição"]
fn measure_the_pump() {
    let mut a = SimWorld::new();
    pool(&mut a, 0.0);
    let pa = subject(&mut a, true, -2.0);
    let mut ba = PhysicsBridge::new();
    let shipped = trace(&mut a, &mut ba, pa, 900, 120);

    let mut b = SimWorld::new();
    pool(&mut b, 0.0);
    let pb = subject_tuned(&mut b, true, -2.0, Some(neutral_gravity()));
    let mut bb = PhysicsBridge::new();
    let ablated = trace(&mut b, &mut bb, pb, 900, 120);

    let mut c = SimWorld::new();
    pool(&mut c, 0.0);
    let pc = subject(&mut c, false, -2.0);
    let mut bc = PhysicsBridge::new();
    let control = trace(&mut c, &mut bc, pc, 900, 120);

    println!("\n=== A BOMBA (largado em y = -2, dentro da poca) ===");
    println!(
        "{:>6} | {:>9} {:>7} | {:>9} {:>7} | {:>9}",
        "t (s)", "shipa y", "lift", "neutro y", "lift", "controle"
    );
    for i in 0..shipped.len() {
        println!(
            "{:>6.1} | {:>9.4} {:>7.4} | {:>9.4} {:>7.4} | {:>9.4}",
            shipped[i].0, shipped[i].1, shipped[i].2, ablated[i].1, ablated[i].2, control[i].1
        );
    }
}

/// **Ele ASSENTA depois de entrar na água?** — a pergunta do produto, e a que
/// separa *"a cura funcionou"* de *"a cura funcionou no repouso"*.
///
/// ⚠️ O CONTROLE é a cápsula idêntica, na MESMA altura de largada: se os dois
/// oscilarem igual, a oscilação é do fluido e não do player.
#[test]
#[ignore = "sonda de medição"]
fn measure_how_it_settles_after_entering() {
    println!("\n=== ENTRAR NA AGUA (amplitude nos ultimos 5 s de uma corrida de 20 s) ===");
    println!(
        "{:>8} | {:>9} {:>9} | {:>9} {:>9}",
        "largado", "player y", "amplit.", "controle", "amplit."
    );
    for y0 in [0.5_f32, 1.5, 3.0, 6.0, -2.0] {
        let mut row = [(0.0_f32, 0.0_f32); 2];
        for (slot, player) in [true, false].into_iter().enumerate() {
            let mut sim = SimWorld::new();
            pool(&mut sim, 0.0);
            let who = subject(&mut sim, player, y0);
            let mut bridge = PhysicsBridge::new();
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for t in 1..=1200u64 {
                bridge.dispatch(&mut sim, true, t);
                if t > 900 {
                    let y = y_of(&sim, "Subject");
                    lo = lo.min(y);
                    hi = hi.max(y);
                }
            }
            let _ = who;
            row[slot] = (y_of(&sim, "Subject"), hi - lo);
        }
        println!(
            "{y0:>8.1} | {:>9.4} {:>9.4} | {:>9.4} {:>9.4}",
            row[0].0, row[0].1, row[1].0, row[1].1
        );
    }
}

/// **QUAL multiplicador bombeia?** — a ablação, um de cada vez.
///
/// ⚠️ A cura da submersão desligou a modelagem DENTRO da água; o que sobra
/// acontece no AR, e um arco que sobe com `g` e desce com `2g` volta ao mesmo
/// nível com `√2` da velocidade — **o dobro da energia por ciclo**. Esta tabela
/// diz se a aritmética é a explicação.
#[test]
#[ignore = "sonda de medição"]
fn measure_which_multiplier_pumps() {
    println!("\n=== A ABLACAO (largado de y = 3, amplitude nos ultimos 5 s de 20 s) ===");
    println!("{:<28} {:>9} {:>9}", "config", "y final", "amplit.");
    let base = PlatformPlayer {
        float_height: FLOAT,
        ..PlatformPlayer::default()
    };
    for (label, tune) in [
        ("o que shipa", base),
        (
            "fall = 1 (o resto shipa)",
            PlatformPlayer {
                fall_gravity: 1.0,
                ..base
            },
        ),
        (
            "peak = 1 (o resto shipa)",
            PlatformPlayer {
                peak_gravity: 1.0,
                ..base
            },
        ),
        (
            "takeoff = 1 (o resto shipa)",
            PlatformPlayer {
                takeoff_gravity: 1.0,
                ..base
            },
        ),
        ("tudo neutro", neutral_gravity()),
    ] {
        let mut sim = SimWorld::new();
        pool(&mut sim, 0.0);
        subject_tuned(&mut sim, true, 3.0, Some(tune));
        let mut bridge = PhysicsBridge::new();
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for t in 1..=1200u64 {
            bridge.dispatch(&mut sim, true, t);
            if t > 900 {
                let y = y_of(&sim, "Subject");
                lo = lo.min(y);
                hi = hi.max(y);
            }
        }
        println!(
            "{label:<28} {:>9.4} {:>9.4}",
            y_of(&sim, "Subject"),
            hi - lo
        );
    }
}
