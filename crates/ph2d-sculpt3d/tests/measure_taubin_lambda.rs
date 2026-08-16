//! **DE QUE RECURSO É O `λ`** — a sonda que responde se `0,33` é um número do
//! paper, um teto de estabilidade, ou um palpite.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_taubin_lambda \
//!   -- --ignored --nocapture --test-threads=1
//! ```
//!
//! O par de Taubin tem UM grau de liberdade depois de `k_PB` fixado: escolhido
//! o `λ`, o `μ` sai da relação `1/λ + 1/μ = k_PB`. O que o par FAZ a cada
//! frequência é a função de transferência
//!
//! ```text
//! f(k) = (1 − λk)·(1 − μk)
//! ```
//!
//! e o critério do próprio paper é: `f(k_PB) = 1` (por construção da relação) e
//! **`|f(k)| < 1` em todo o resto da banda** — se algum `k` da banda de corte
//! tem `|f| > 1`, aquela frequência é AMPLIFICADA e o filtro explode ao longo do
//! traço.
//!
//! ⚠️ **O espectro do laplaciano deste motor é `[0, 2]`.** Tanto o uniforme
//! (`I − D⁻¹A`) quanto o cotangente normalizado pelo `Σw` são operadores de
//! média com pesos que somam 1 ⇒ os autovalores vivem em `[0, 2]`, e o `k = 2`
//! é o padrão alternado — a *ruga de um vértice*, exactamente o que um artista
//! está a alisar. É por isso que o número que decide é o `f(2)`.

use ph2d_sculpt3d::{TAUBIN_LAMBDA, TAUBIN_MU, TAUBIN_PASS_BAND};

/// `μ` derivado, para qualquer `λ` — a MESMA relação que a const do produto usa.
fn mu_for(lambda: f64) -> f64 {
    1.0 / (f64::from(TAUBIN_PASS_BAND) - 1.0 / lambda)
}

fn transfer(lambda: f64, k: f64) -> f64 {
    (1.0 - lambda * k) * (1.0 - mu_for(lambda) * k)
}

/// O pior ganho da banda de CORTE — o critério de estabilidade do paper.
fn worst_stopband_gain(lambda: f64) -> (f64, f64) {
    let mut worst = 0.0f64;
    let mut at = 0.0f64;
    // A partir de um passo acima do `k_PB`: no próprio `k_PB` o ganho é 1 por
    // construção, e afirmar que ele passa de 1 mediria a construção.
    let mut k = f64::from(TAUBIN_PASS_BAND) + 1e-3;
    while k <= 2.0 {
        let g = transfer(lambda, k).abs();
        if g > worst {
            worst = g;
            at = k;
        }
        k += 1e-3;
    }
    (worst, at)
}

/// **A VARREDURA.** O que cada `λ` faz com a frequência mais alta que a malha
/// tem, e onde ele deixa de ser um filtro.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn where_the_lambda_stops_being_a_filter() {
    println!("\n  k_PB = {TAUBIN_PASS_BAND} (do paper)   lambda do produto = {TAUBIN_LAMBDA}\n");
    println!("     lambda        mu     f(2)    |f| pior   em k     veredito");
    println!("  ---------   -------   ------   ---------   -----   ----------");
    let mut lambda = 0.20f64;
    while lambda <= 0.80 + 1e-9 {
        let (worst, at) = worst_stopband_gain(lambda);
        let f2 = transfer(lambda, 2.0);
        let verdict = if worst > 1.0 { "AMPLIFICA" } else { "estavel" };
        println!(
            "  {lambda:>9.4}   {:>7.4}   {f2:>6.3}   {worst:>9.4}   {at:>5.2}   {verdict}",
            mu_for(lambda)
        );
        lambda += 0.025;
    }

    // O ponto em que o primeiro fator zera exactamente em k = 2 — o `λ` que
    // ANIQUILA a ruga de um vértice num par só.
    println!("\n  O zero do primeiro fator cai em k = 1/lambda:");
    for l in [0.33f64, 0.4, 0.5, 0.6] {
        println!("    lambda {l:>5.3}  ->  zero em k = {:>6.3}", 1.0 / l);
    }

    // A fronteira, por bisseção — o número que um teto legítimo cita.
    let (mut lo, mut hi) = (0.5f64, 0.9f64);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if worst_stopband_gain(mid).0 > 1.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    println!("\n  FRONTEIRA DA ESTABILIDADE: lambda = {lo:.6}");
    println!("  (acima disto |f| > 1 na banda de corte e o par AMPLIFICA a ruga)");
    println!(
        "  o produto usa {TAUBIN_LAMBDA}, que e {:.1}% dela",
        f64::from(TAUBIN_LAMBDA) / lo * 100.0
    );
    println!("  mu do produto = {TAUBIN_MU}");
}
