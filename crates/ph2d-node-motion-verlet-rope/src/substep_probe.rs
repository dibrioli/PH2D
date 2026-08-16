//! SONDAS dos sub-passos: quanto custam, e o que compram que as `iterations`
//! não compram.
//!
//! Rode com `cargo test -p ph2d-node-motion-verlet-rope --release -- --ignored
//! --nocapture`.

use super::*;

/// Uma corda pendurada, integrada `ticks` tiques, devolvendo o pior esticão
/// relativo dos últimos 20 tiques (o regime PERMANENTE — o transiente do
/// nascimento não é o que o artista vê).
pub(crate) fn steady_stretch(
    count: usize,
    gravity: f32,
    iterations: usize,
    substeps: usize,
) -> f32 {
    let length = 6.0f32;
    let seg = length / (count as f32 - 1.0);
    let p = Params {
        count,
        seg_rest: seg,
        gravity,
        iterations,
        damping: 0.0,
        pin_tail: false,
        bend: 0.0,
        substeps,
    };
    let mut pos: Vec<[f32; 2]> = (0..count).map(|i| [0.0, -(i as f32) * seg]).collect();
    let mut prev = pos.clone();
    let mut worst = 0.0f32;
    let ticks = 240;
    for k in 0..ticks {
        // Uma âncora que anda a velocidade CONSTANTE: a corda arrasta atrás com
        // um esticão de regime permanente, e o número REPRODUZ — ao contrário de
        // um chicote, cuja resposta é caótica e cujo "pior esticão" é ruído com
        // casas decimais.
        let anchor = [k as f32 * 0.08, 0.0];
        let (np, pp) = step(
            pos.clone(),
            &prev,
            &[],
            &[],
            anchor,
            [0.0, 0.0],
            1.0 / 60.0,
            &p,
        );
        pos = np;
        prev = pp;
        if k + 20 >= ticks {
            for w in pos.windows(2) {
                let d = ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
                worst = worst.max((d - seg).abs() / seg);
            }
        }
    }
    worst
}

/// **A ORÇAMENTO IGUAL, quem converge melhor?** — a pergunta do *Small Steps in
/// Physics Simulation* (Macklin et al., SCA 2019): para o MESMO número total de
/// correções, mais sub-passos com menos iterações batem um passo com muitas.
#[test]
#[ignore = "sonda"]
fn substeps_against_iterations_at_equal_budget() {
    for gravity in [9.8f32, 98.0] {
        println!("gravidade {gravity}:");
        for (sub, iters) in [(1usize, 24usize), (2, 12), (4, 6), (8, 3), (24, 1)] {
            let s = steady_stretch(24, gravity, iters, sub);
            println!(
                "  substeps {sub:>3} x iterations {iters:>3} (orcamento {:>3}): esticao {:>7.4}%",
                sub * iters,
                s * 100.0
            );
        }
    }
}

/// **O CUSTO de um sub-passo**, pela porta do produto (`step`), para o teto
/// nomear o recurso em vez de ser escolhido.
#[test]
#[ignore = "sonda"]
fn what_a_substep_costs() {
    for count in [24usize, 256, 2048] {
        for sub in [1usize, 2, 4, 8, 16, 32] {
            let seg = 6.0 / (count as f32 - 1.0);
            let p = Params {
                count,
                seg_rest: seg,
                gravity: 9.8,
                iterations: 24,
                damping: 0.0,
                pin_tail: false,
                bend: 0.0,
                substeps: sub,
            };
            let pos: Vec<[f32; 2]> = (0..count).map(|i| [0.0, -(i as f32) * seg]).collect();
            let prev = pos.clone();
            const REPS: u32 = 20;
            let t0 = std::time::Instant::now();
            for _ in 0..REPS {
                std::hint::black_box(step(
                    pos.clone(),
                    &prev,
                    &[],
                    &[],
                    [0.0, 0.0],
                    [0.0, 0.0],
                    1.0 / 60.0,
                    &p,
                ));
            }
            let ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(REPS);
            println!("count {count:>5} substeps {sub:>3}: {ms:>8.4} ms/tique");
        }
    }
}

/// **A INÉRCIA ATRAVESSA A FRONTEIRA DO TIQUE, e são DUAS metades.**
///
/// A **ENTRADA**: o `prev` que chega codifica um deslocamento de um TIQUE, e o
/// primeiro sub-passo levaria `N ×` a inércia se ninguém o re-escalasse. Mede-se
/// com uma corda a VIAJAR sob gravidade zero: a distância percorrida por tique
/// não pode depender do número de sub-passos.
///
/// A **SAÍDA**: o que o tique seguinte lê tem de ser a velocidade do ÚLTIMO
/// sub-passo, esticada a um tique. ⚠️ **A viagem é CEGA a esta metade** — a
/// velocidade constante faz o deslocamento MÉDIO ser igual ao FINAL —, então o
/// oráculo dela é a **queda LIVRE** (`iterations = 0`: sem restrição, cada ponto
/// é uma partícula, e uma restrição rígida esconderia o erro puxando o atrasado
/// de volta).
#[test]
#[ignore = "sonda"]
fn what_substeps_do_to_travel_and_to_fall() {
    println!("viagem (g=0, iterations 24, ancora a 3 u/s):");
    for sub in [1usize, 2, 4, 8, 16] {
        println!("  substeps {sub:>3}: dx {:>8.4}", travel(sub));
    }
    println!("queda LIVRE (g=9.8, iterations 0, 1 s):");
    for sub in [1usize, 2, 4, 8, 16] {
        println!("  substeps {sub:>3}: dy {:>9.4}", free_fall(sub));
    }
}

/// Quanto a cauda percorre em 60 tiques atrás de uma âncora a velocidade
/// constante, sem gravidade.
pub(crate) fn travel(substeps: usize) -> f32 {
    let count = 8;
    let seg = 1.0f32;
    let p = Params {
        count,
        seg_rest: seg,
        gravity: 0.0,
        iterations: 24,
        damping: 0.0,
        pin_tail: false,
        bend: 0.0,
        substeps,
    };
    let dt = 1.0f32 / 60.0;
    let mut pos: Vec<[f32; 2]> = (0..count).map(|i| [i as f32 * seg, 0.0]).collect();
    let mut prev: Vec<[f32; 2]> = pos.iter().map(|q| [q[0] - 3.0 * dt, q[1]]).collect();
    let start = pos[count - 1];
    for k in 0..60 {
        // ⚠️ A cabeça é SEMPRE pinada na âncora, então é a ÂNCORA que viaja — a
        // 1ª fixture ancorava no próprio ponto 0 e media uma corda CONGELADA
        // (`dx = -0,0000` em toda a varredura, inclusive no controle).
        let a = [k as f32 * 3.0 * dt, 0.0];
        let (np, pp) = step(pos.clone(), &prev, &[], &[], a, [0.0, 0.0], dt, &p);
        pos = np;
        prev = pp;
    }
    pos[count - 1][0] - start[0]
}

/// Quanto a cauda cai em 1 s SEM restrição nenhuma — a queda livre analítica.
pub(crate) fn free_fall(substeps: usize) -> f32 {
    let count = 8;
    let p = Params {
        count,
        seg_rest: 1.0,
        gravity: 9.8,
        iterations: 0,
        damping: 0.0,
        pin_tail: false,
        bend: 0.0,
        substeps,
    };
    let dt = 1.0f32 / 60.0;
    let mut pos: Vec<[f32; 2]> = (0..count).map(|i| [i as f32, 0.0]).collect();
    let mut prev = pos.clone();
    let start = pos[count - 1];
    for _ in 0..60 {
        let (np, pp) = step(pos.clone(), &prev, &[], &[], [0.0, 0.0], [0.0, 0.0], dt, &p);
        pos = np;
        prev = pp;
    }
    pos[count - 1][1] - start[1]
}

/// **AS `iterations` SOZINHAS ALCANÇAM O QUE OS SUB-PASSOS ALCANÇAM?**
#[test]
#[ignore = "sonda"]
fn can_iterations_alone_reach_it() {
    for iters in [24usize, 64, 128] {
        println!(
            "  iterations {iters:>4} x substeps 1: esticao {:>7.4}%",
            steady_stretch(24, 98.0, iters, 1) * 100.0
        );
    }
    for sub in [4usize, 8] {
        println!(
            "  iterations   24 x substeps {sub}: esticao {:>7.4}%",
            steady_stretch(24, 98.0, 24, sub) * 100.0
        );
    }
}

/// **QUANTO O AMORTECIMENTO MUDA COM OS SUB-PASSOS.**
///
/// O `keep = 1 - damping` e' computado UMA vez e aplicado DENTRO do laco de
/// sub-passos, entao um tique amortece `keep^N` em vez de `keep`. A pergunta que
/// esta sonda responde nao e' *"isso existe?"* (a aritmetica ja' o diz) e sim
/// *"quanto do desenho isso move no regime que o slider alcanca?"*.
///
/// Rodar: `cargo test -p ph2d-node-motion-verlet-rope --release -- --ignored
/// measure_what_substeps_do_to_the_damping --nocapture`
#[test]
#[ignore = "sonda"]
fn measure_what_substeps_do_to_the_damping() {
    println!("amortecimento x sub-passos (corda de 24, ancora chicoteada, 120 tiques)");
    println!("  damping | N |  keep^N  | excursao da cauda | vs N=1");
    for damping in [0.0f32, 0.02, 0.10, 0.30] {
        let mut base = None;
        for n in [1usize, 2, 4, 8, 16] {
            let p = Params {
                count: 24,
                seg_rest: 0.25,
                gravity: 9.8,
                iterations: 8,
                damping,
                pin_tail: false,
                bend: 0.0,
                substeps: n,
            };
            let mut pos: Vec<[f32; 2]> =
                (0..p.count).map(|i| [i as f32 * p.seg_rest, 0.0]).collect();
            let mut prev = pos.clone();
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for k in 0..120 {
                let t = k as f32 / 60.0;
                let a = [(t * 6.0).sin() * 1.5, 0.0];
                let (np, pp) = step(pos.clone(), &prev, &[], &[], a, [0.0, 0.0], 1.0 / 60.0, &p);
                pos = np;
                prev = pp;
                let y = pos[p.count - 1][1];
                lo = lo.min(y);
                hi = hi.max(y);
            }
            let span = hi - lo;
            let keep_n = (1.0f32 - damping).powi(n as i32);
            let rel = base.map_or(1.0, |b: f32| span / b);
            if base.is_none() {
                base = Some(span);
            }
            println!("     {damping:.2} | {n:2} | {keep_n:.6} | {span:17.4} | {rel:.4}x");
        }
    }
}
