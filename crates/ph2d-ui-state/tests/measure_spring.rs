//! **A MOLA precisa de um solver, ou a curva basta?** (plano UI/UX W7)
//!
//! O plano deixou em aberto: *se a curva `Elastic` bastar, o solver não se constrói*. Isto mede a
//! pergunta em vez de a responder por gosto — e a comparação é contra a coisa de que a UI
//! moderna fala quando diz "spring": o oscilador **massa-mola-amortecedor** que o SwiftUI, o
//! Framer Motion e o `react-spring` expõem.
//!
//! Rode com: `cargo test -p ph2d-ui-state --test measure_spring -- --ignored --nocapture
//! --test-threads=1` — ⚠️ a última metade não é opcional: as duas sondas escrevem no MESMO
//! stdout, e em paralelo as tabelas saem intercaladas linha a linha.

use ph2d_anim::{Easing, EasingFamily, EasingMode};

/// A resposta a degrau de um oscilador massa-mola-amortecedor, integrada por Euler
/// semi-implícito — o mesmo integrador que o Motion usa, e por isso não é um segundo modelo.
///
/// `zeta < 1` sub-amortecido (passa do alvo e volta), `zeta = 1` crítico (chega sem passar).
fn spring(t: f64, omega: f64, zeta: f64) -> f64 {
    const DT: f64 = 1.0 / 2400.0;
    let (mut x, mut v) = (0.0_f64, 0.0_f64);
    let steps = (t / DT).round() as usize;
    for _ in 0..steps {
        let a = omega * omega * (1.0 - x) - 2.0 * zeta * omega * v;
        v += a * DT;
        x += v * DT;
    }
    x
}

/// Quanto uma curva passa do alvo, e quando ela assenta dentro de 1%.
fn shape(f: impl Fn(f64) -> f64) -> (f64, f64, usize) {
    const N: usize = 2000;
    let mut peak = 0.0_f64;
    let mut settle = 1.0_f64;
    let mut crossings = 0usize;
    let mut prev = f(0.0) - 1.0;
    // (a banda morta abaixo decide quando `prev` anda)
    for i in 0..=N {
        let u = i as f64 / N as f64;
        let y = f(u);
        peak = peak.max(y);
        if (y - 1.0).abs() > 0.01 {
            settle = 1.0;
        } else if settle > 0.99 {
            settle = u;
        }
        // ⚠️ Só conta uma travessia quando a excursão é VISÍVEL (1% do percurso). A primeira
        // versão contava o sinal cru de `y - 1`, e depois de assentar ele troca a cada amostra
        // por ruído de `f64` — a mola crítica saía com oito oscilações que não existem. *Um
        // contador de sinal precisa de uma banda morta, senão ele mede o ponto flutuante.*
        let cur = y - 1.0;
        if cur.abs() > 0.01 {
            if prev < 0.0 && cur > 0.0 || prev > 0.0 && cur < 0.0 {
                crossings += 1;
            }
            prev = cur;
        }
    }
    (peak, settle, crossings)
}

#[test]
#[ignore = "sonda de medição — roda a pedido"]
fn does_the_elastic_curve_stand_in_for_a_spring() {
    println!("\n  curva                       pico   assenta<1%  cruzamentos");
    println!("  ------------------------------------------------------------");

    for (name, e) in [
        (
            "Elastic Out",
            Easing {
                family: EasingFamily::Elastic,
                mode: EasingMode::Out,
            },
        ),
        (
            "Back Out",
            Easing {
                family: EasingFamily::Back,
                mode: EasingMode::Out,
            },
        ),
        (
            "Bounce Out",
            Easing {
                family: EasingFamily::Bounce,
                mode: EasingMode::Out,
            },
        ),
        (
            "Cubic Out (o default)",
            Easing {
                family: EasingFamily::Cubic,
                mode: EasingMode::Out,
            },
        ),
    ] {
        let (p, s, c) = shape(|u| e.eval(u));
        println!("  {name:<26} {p:>5.3}   {s:>8.3}   {c:>10}");
    }

    println!();
    for (name, omega, zeta) in [
        ("mola macia   (w=12, z=0.35)", 12.0, 0.35),
        ("mola media   (w=20, z=0.55)", 20.0, 0.55),
        ("mola critica (w=20, z=1.00)", 20.0, 1.00),
    ] {
        // Uma mola tem tempo PRÓPRIO: a normalização em `[0,1]` é feita contra o instante em que
        // ela assenta, que é como um `Easing` seria comparável a ela.
        let horizon = 1.6;
        let (p, s, c) = shape(|u| spring(u * horizon, omega, zeta));
        println!("  {name:<26} {p:>5.3}   {s:>8.3}   {c:>10}");
    }
    println!();
}

/// **A pergunta que decide o solver não é a FORMA da curva — é a INTERRUPÇÃO.**
///
/// O que uma mola dá e uma curva não dá é **continuidade de velocidade**: interrompida a meio, ela
/// carrega a velocidade que tinha. Uma curva reinicia em `t = 0` com a velocidade que a família
/// dela tiver ali — e se essa velocidade não casar com a que a cena trazia, o artista vê a forma
/// **parar e recomeçar** no meio do gesto.
///
/// Isto mede o salto: a velocidade imediatamente ANTES de interromper contra a velocidade com que
/// a transição de volta arranca, ambas em unidades de percurso por segundo, para uma ida
/// interrompida a 30% e revertida.
#[test]
#[ignore = "sonda de medição — roda a pedido"]
fn what_a_curve_costs_at_the_moment_of_interruption() {
    const D: f64 = 0.15; // a duração que shipa
    const AT: f64 = 0.30; // interrompe a 30% do caminho
    const H: f64 = 1e-4;

    // ⚠️ A REVERSÃO troca o sinal por definição — a cena ia e passa a voltar. O que se mede não é
    // o salto de sinal, é a RAZÃO das magnitudes: `1×` significa que a volta arranca à mesma
    // velocidade com que a ida chegou (nenhuma descontinuidade visível), e `0×` significa que a
    // cena PARA e recomeça, que é o *stutter* que faz alguém pedir um solver.
    println!("\n  curva                     |v| antes  |v| depois   razao");
    println!("  ------------------------------------------------------------");
    for (name, e) in [
        (
            "Cubic Out (o default)",
            Easing {
                family: EasingFamily::Cubic,
                mode: EasingMode::Out,
            },
        ),
        (
            "Cubic InOut",
            Easing {
                family: EasingFamily::Cubic,
                mode: EasingMode::InOut,
            },
        ),
        (
            "Elastic Out",
            Easing {
                family: EasingFamily::Elastic,
                mode: EasingMode::Out,
            },
        ),
    ] {
        let y = |u: f64| e.eval(u);
        // A ida: a cena está em `y(AT)` e a mover-se a esta velocidade.
        let before = (y(AT + H) - y(AT - H)) / (2.0 * H) / D;
        // A volta parte da pose VIVA e arranca no `t = 0` da MESMA curva; o percurso que ela tem
        // de percorrer é o que já foi andado.
        let span = y(AT);
        let after = -(y(H) - y(0.0)) / H / D * span;
        let (a, b) = (before.abs(), after.abs());
        println!(
            "  {name:<24} {a:>9.2}  {b:>10.2}  {:>6.2}x",
            if a > 0.0 { b / a } else { f64::NAN }
        );
    }
    // E a mola, pela definição: ela NÃO salta, porque a velocidade é estado dela.
    println!(
        "  {:<24} {:>9}  {:>10}  {:>6}",
        "mola (qualquer)", "v", "v", "1.00x"
    );
    println!();
}
