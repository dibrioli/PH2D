//! **A sonda que escolhe os números da fonte de largura** (W1d do plano 25).
//!
//! `cargo test -p ph2d-vec-edit --release measure_pencil_width -- --ignored --nocapture`
//!
//! Ela existe para que as consts do [`ph2d_vec_edit::pencil_width`] tenham uma tabela ao lado —
//! e para que a próxima pessoa que quiser mexer nelas mexa no número que a medição deu, não num
//! palpite.

use ph2d_vec_edit::pencil_width::{PenDynamics, WidthSource, width_stops, width_stops_with_budget};

/// Um S desenhado por uma mão sintética que **acelera no meio** — é o gesto em que a fonte
/// *Speed* tem o que dizer; um traço de velocidade constante é o CONTROLE, e sai uniforme.
///
/// ⚠️ **A fixture TEM de conter o fenômeno, e a 1ª versão não continha.** Ela derivava `dt` de
/// uma função de velocidade lisa mais um jitter de ±0,9 ms sobre uma base de 1,5–10 ms — e a
/// medição disse que o suavizador não fazia nada (2,7× cru contra 2,6× filtrado), o que teria me
/// levado a escrever no doc do produto que ele é opcional.
///
/// O ruído REAL de um `ds/dt` de app tem duas fontes que aquela fixture omitia:
///
/// 1. **o `dt` de um laço de eventos varia por um FATOR**, não por uma margem — coalescing do
///    winit, escalonamento do SO, um frame que demorou. Aqui: `0,45×` a `2,2×` do nominal.
/// 2. **o `ds` é QUANTIZADO** pelo `MIN_SAMPLE_PX` do lápis (1,5 px): num trecho lento toda
///    amostra anda quase exatamente o passo mínimo, então o numerador é ~constante e a razão
///    herda o factor inteiro do denominador.
///
/// `jitter` é o expoente do factor (0 = relógio perfeito).
fn gesture(n: usize, jitter: f64) -> (Vec<[f64; 2]>, Vec<PenDynamics>) {
    let mut pts = Vec::with_capacity(n);
    let mut dyns = Vec::with_capacity(n);
    let mut t_ns: u128 = 0;
    for i in 0..n {
        #[allow(clippy::cast_precision_loss)]
        let u = i as f64 / (n - 1) as f64;
        // A velocidade do gesto: lento nas pontas, rápido no meio (um floreio).
        let speed = 0.4 + 2.6 * (u * std::f64::consts::PI).sin();
        pts.push([-1.5 + 3.0 * u, 0.6 * (u * std::f64::consts::TAU).sin()]);
        // O factor de jitter: determinístico (a cena tem de ser a mesma toda vez) e
        // multiplicativo, que é a forma que um laço de eventos de facto tem.
        #[allow(clippy::cast_precision_loss)]
        let f = ((i as f64 * 12.9898).sin() * 43758.5453).fract().abs();
        let factor = 1.0 + jitter * (f - 0.5);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let dt = (4_000_000.0 / speed * factor).max(1.0) as u128;
        t_ns += dt;
        dyns.push(PenDynamics {
            pressure: 0.2 + 0.8 * u as f32,
            t_ns,
        });
    }
    (pts, dyns)
}

/// **A RUGOSIDADE** de uma série: quanto ela sobe-e-desce entre amostras vizinhas, em fração da
/// própria faixa.
///
/// ⚠️ É esta a métrica, e não pico/mediana — o gesto tem um pico LEGÍTIMO (ele acelera no meio),
/// então pico/mediana mede o SINAL e responde ~o mesmo antes e depois do filtro. O que distingue
/// ruído de sinal é a frequência: o sinal varia devagar ao longo do arco, o ruído troca de
/// direção a cada amostra. Foi a métrica errada que quase me fez escrever o número errado.
fn roughness(v: &[f64]) -> f64 {
    let (lo, hi) = v
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    if hi - lo <= 0.0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let n = (v.len() - 1) as f64;
    v.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (n * (hi - lo))
}

#[test]
#[ignore = "sonda de medição; roda à mão"]
fn measure_pencil_width() {
    let (pts, dyns) = gesture(240, 1.75);

    // (1) O RUÍDO da velocidade crua, e o que o suavizador faz com ele.
    let mut raw = vec![0.0; pts.len()];
    for i in 1..pts.len() {
        let ds = ((pts[i][0] - pts[i - 1][0]).powi(2) + (pts[i][1] - pts[i - 1][1]).powi(2)).sqrt();
        #[allow(clippy::cast_precision_loss)]
        let secs = (dyns[i].t_ns - dyns[i - 1].t_ns) as f64 * 1e-9;
        raw[i] = ds / secs;
    }
    raw[0] = raw[1];
    println!("velocidade CRUA: rugosidade = {:.3}", roughness(&raw));
    for half in [1usize, 2, 3, 4, 6, 10] {
        let sm: Vec<f64> = (0..raw.len())
            .map(|i| {
                let lo = i.saturating_sub(half);
                let hi = (i + half + 1).min(raw.len());
                #[allow(clippy::cast_precision_loss)]
                let n = (hi - lo) as f64;
                raw[lo..hi].iter().sum::<f64>() / n
            })
            .collect();
        println!("  janela {half}: rugosidade = {:.3}", roughness(&sm));
    }

    // (2) A RUGOSIDADE do perfil PRODUZIDO, por orçamento de paradas — é ela que escolhe o
    // `STOP_BUDGET`: pouca demais e o perfil perde a forma do gesto; muita e ele volta a
    // descrever o jitter do relógio.
    let full = width_stops(WidthSource::Speed, &pts, &dyns);
    let n_full = full.as_slice().len();
    println!("paradas produzidas (orçamento em vigor): {n_full}");
    // A métrica tem de ser COMPARÁVEL entre orçamentos: rugosidade de uma série de 6 pontos e
    // de uma de 48 não se comparam (ela é por-passo). Então mede-se contra o IDEAL — o perfil que
    // a velocidade ANALÍTICA da mão sintética implica, sem jitter nenhum — em 101 pontos fixos, e
    // conta-se quantas REVERSÕES espúrias o perfil tem na metade em que a mão só acelera.
    let ideal = |t: f64| {
        let v = 0.4 + 2.6 * (t * std::f64::consts::PI).sin();
        let (lo, hi) = (0.4, 3.0);
        0.35 + (1.0 - (v - lo) / (hi - lo)) * (1.45 - 0.35)
    };
    println!("orçamento | erro médio vs ideal | reversões na subida");
    for budget in [4usize, 6, 8, 12, 16, 24, 48] {
        let st = width_stops_with_budget(WidthSource::Speed, &pts, &dyns, budget);
        let err: f64 = (0..=100)
            .map(|k| {
                let t = f64::from(k) / 100.0;
                (st.at(t) - ideal(t)).abs()
            })
            .sum::<f64>()
            / 101.0;
        let m: Vec<f64> = st.as_slice().iter().map(|x| x.mult).collect();
        let half = m.len() / 2;
        let rev = m[..half].windows(2).filter(|w| w[1] > w[0]).count();
        println!("       {budget:2} |              {err:.4} | {rev}");
    }
    for st in full.as_slice() {
        println!("    parada pos={:.3} mult={:.3}", st.pos, st.mult);
    }
    let probe: Vec<f64> = (0..=100).map(|k| f64::from(k) / 100.0).collect();
    let reference: Vec<f64> = probe.iter().map(|&t| full.at(t)).collect();
    println!(
        "faixa do perfil: {:.3} .. {:.3}  (razão grosso/fino = {:.2}x)",
        reference.iter().copied().fold(f64::MAX, f64::min),
        reference.iter().copied().fold(f64::MIN, f64::max),
        reference.iter().copied().fold(f64::MIN, f64::max)
            / reference.iter().copied().fold(f64::MAX, f64::min)
    );

    // (3) O CONTROLE: velocidade constante tem de sair uniforme (lista vazia).
    let n: usize = 120;
    let pts_c: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let u = i as f64 / (n - 1) as f64;
            [-1.5 + 3.0 * u, 0.0]
        })
        .collect();
    let dyns_c: Vec<PenDynamics> = (0..n)
        .map(|i| PenDynamics {
            pressure: 1.0,
            t_ns: i as u128 * 4_000_000,
        })
        .collect();
    let flat = width_stops(WidthSource::Speed, &pts_c, &dyns_c);
    println!(
        "CONTROLE velocidade constante: {} paradas (0 = uniforme, o esperado)",
        flat.as_slice().len()
    );

    // (4) E a pressão, pela mesma porta.
    let pres = width_stops(WidthSource::Pressure, &pts, &dyns);
    println!(
        "pressão 0,2->1,0: {} paradas, {:.3} .. {:.3}",
        pres.as_slice().len(),
        pres.at(0.0),
        pres.at(1.0)
    );
}
