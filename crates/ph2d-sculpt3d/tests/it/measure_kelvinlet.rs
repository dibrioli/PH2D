//! **AS SONDAS DO CAMPO ELÁSTICO** — os números que escolheram o [`POISSON`] e
//! a família de escalas, e os que dizem o preço.
//!
//! `cargo test -p ph2d-sculpt3d --release --test measure_kelvinlet -- --ignored
//! --nocapture`
//!
//! ⚠️ **Elas medem o campo, não o produto.** O que o produto faz com ele — o
//! `l-mode` do Grab — tem sonda própria (`measure_grab_profile`), porque a
//! diferença entre *"o kernel diz isto"* e *"o traço faz isto"* já custou a esta
//! casa uma nota errada mais de uma vez.

use ph2d_sculpt3d::kelvinlet::{POISSON, Scales, grab, scale, twist};

fn len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// **O RESÍDUO NA BORDA** — quanto do deslocamento do bico ainda sobra à
/// distância de um raio de pincel, que é onde a pegada do motor corta.
#[test]
#[ignore = "sonda"]
fn rim_residual() {
    println!("\n== resíduo na borda da pegada (ε = raio da pegada) ==");
    println!(
        "{:>6} | {:>10} | {:>10} | {:>10}",
        "r/ε", "Mono", "Bi", "Tri"
    );
    let f = [1.0, 0.0, 0.0];
    for k in [0.5, 1.0, 1.5, 2.0, 3.0, 4.0] {
        // À FRENTE do puxão: o pior caso, onde o termo radial soma.
        let at = |s| len(grab([k, 0.0, 0.0], 1.0, f, s));
        println!(
            "{k:>6.1} | {:>10.4} | {:>10.4} | {:>10.4}",
            at(Scales::Mono),
            at(Scales::Bi),
            at(Scales::Tri)
        );
    }
    println!("\n(o campo do bico vale 1,0000 por normalização)");
}

/// **A VARREDURA DE ν** — os três eixos que o [`POISSON`] arbitra, lado a lado.
#[test]
#[ignore = "sonda"]
fn poisson_sweep() {
    println!("\n== o que o coeficiente de Poisson decide ==");
    println!(
        "{:>6} | {:>14} | {:>14} | {:>12}",
        "ν", "ganho escala", "anisotropia", "divergência"
    );
    for nu in [0.0_f32, 0.1, 0.2, 0.3, 0.4, 0.45, 0.49, 0.5] {
        let b = 1.0 / (4.0 * (1.0 - nu));
        let gain = 2.5 - 5.0 * b;
        // A anisotropia do grab a um ε de distância, em forma fechada (o campo
        // cru, sem normalizar — a razão não depende da normalização).
        let re = f32::sqrt(2.0);
        let re3 = re * re * re;
        let iso = (1.0 - b) / re + 1.0 / (2.0 * re3);
        let ahead = iso + b / re3;
        let aniso = ahead / iso;
        // `div u = (2b − a)(r·f)/r³` no limite clássico: zero em ν = 1/2.
        let div = (2.0 * b - 1.0).abs();
        println!("{nu:>6.2} | {gain:>14.3} | {aniso:>13.3}× | {div:>12.4}");
    }
    println!("\nem uso: ν = {POISSON}");
}

/// **O PREÇO** — quanto custa uma avaliação de cada família, contra a lei do
/// `s-mode` que ela substitui.
#[test]
#[ignore = "sonda"]
fn what_a_field_evaluation_costs() {
    const N: usize = 2_000_000;
    let pts: Vec<[f32; 3]> = (0..1024)
        .map(|i| {
            let t = i as f32 * 0.0137;
            [t.sin() * 0.7, t.cos() * 0.5, (t * 1.7).sin() * 0.3]
        })
        .collect();
    let bench = |name: &str, f: &dyn Fn([f32; 3]) -> [f32; 3]| {
        let t0 = std::time::Instant::now();
        let mut acc = 0.0f32;
        for i in 0..N {
            let u = f(pts[i % pts.len()]);
            acc += u[0];
        }
        let ns = t0.elapsed().as_secs_f64() * 1e9 / N as f64;
        println!("{name:>28}: {ns:>7.2} ns/avaliação  (acc {acc:.3})");
    };
    println!("\n== custo por avaliação ==");
    let f = [1.0, 0.0, 0.0];
    bench("s-mode (gesto · escalar)", &|r: [f32; 3]| {
        let w = 1.0 - (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]);
        [f[0] * w, f[1] * w, f[2] * w]
    });
    bench("grab Mono", &|r| grab(r, 0.5, f, Scales::Mono));
    bench("grab Bi", &|r| grab(r, 0.5, f, Scales::Bi));
    bench("grab Tri", &|r| grab(r, 0.5, f, Scales::Tri));
    bench("twist Tri", &|r| {
        twist(r, 0.5, [0.0, 0.0, 1.0], Scales::Tri)
    });
    bench("scale Tri", &|r| scale(r, 0.5, 0.3, Scales::Tri));
    // ⚠️ **A linha do `pinch` saiu em 2026-08-15**: ele deixou de ter chamador
    // de produção (ver o doc dele) e vive sob `cfg(test)`, que um teste de
    // INTEGRAÇÃO não alcança. Cronometrar uma função congelada seria publicar o
    // custo de um caminho que ninguém percorre.
}

/// **O PERFIL, lado a lado com a curva que ele substitui** — é a tabela que
/// explica de olho o que o chip troca.
#[test]
#[ignore = "sonda"]
fn the_profile_against_the_curve() {
    println!("\n== deslocamento ao longo do raio (bico = 1,0) ==");
    println!(
        "{:>6} | {:>10} | {:>10} | {:>10}",
        "r/ε", "à frente", "ao lado", "curva S"
    );
    let f = [1.0, 0.0, 0.0];
    for k in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let ahead = len(grab([k, 0.0, 0.0], 1.0, f, Scales::Tri));
        let beside = len(grab([0.0, k, 0.0], 1.0, f, Scales::Tri));
        // A `Falloff::Smooth` do nosso pincel, para comparação.
        let t = 1.0 - k * k;
        let curve = t * t * t;
        println!("{k:>6.2} | {ahead:>10.4} | {beside:>10.4} | {curve:>10.4}");
    }
}

/// **O QUE ν FAZ COM A FORMA DO MODO DE ESCALA** — a coluna que faltava para a
/// varredura decidir, porque o *ganho* sozinho não diz nada (ele é normalizado
/// de volta) e o que pode estragar é a FORMA.
#[test]
#[ignore = "sonda"]
fn poisson_shapes_the_scale_field() {
    println!("\n== taxa de dilatação local |u(r)|/(s·r), bico = 1,0 ==");
    println!(
        "{:>6} | {:>9} | {:>9} | {:>9} | {:>9}",
        "ν", "r=0,5ε", "r=1ε", "r=2ε", "r=3ε"
    );
    for nu in [0.0_f32, 0.2, 0.3, 0.4, 0.45, 0.49] {
        // A varredura precisa de um ν que o `const` do módulo não move, então
        // ela re-monta o campo de escala a partir das mesmas duas fórmulas.
        let b = 1.0 / (4.0 * (1.0 - nu));
        let raw = |r: f32, eps: f32| {
            let re = (r * r + eps * eps).sqrt();
            let (re3, re5) = (re * re * re, re * re * re * re * re);
            let c1 = (1.0 - b) / re3 + 1.5 * eps * eps / re5;
            // `F = I` ⇒ `F r = r`, `rᵀFr = r²`, `tr = 3`, `Fᵀr = r`.
            c1 * r + 3.0 * b / re5 * r * r * r - b / re3 * 4.0 * r
        };
        let norm: f32 = Scales::Tri
            .taps()
            .iter()
            .map(|&(w, m)| w * (2.5 - 5.0 * b) / (m * m * m))
            .sum();
        let at = |r: f32| {
            let s: f32 = Scales::Tri.taps().iter().map(|&(w, m)| w * raw(r, m)).sum();
            s / norm / r
        };
        println!(
            "{nu:>6.2} | {:>9.4} | {:>9.4} | {:>9.4} | {:>9.4}",
            at(0.5),
            at(1.0),
            at(2.0),
            at(3.0)
        );
    }
}
