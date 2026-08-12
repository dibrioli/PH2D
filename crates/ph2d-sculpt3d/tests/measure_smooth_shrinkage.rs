//! **QUANTO O SMOOTH ENCOLHE** — a sonda que decide se a W4 tem alvo.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_smooth_shrinkage \
//!   -- --ignored --nocapture
//! ```
//!
//! O nosso Smooth é `lerp(p, média do anel, w)` — o laplaciano umbrella puro.
//! Vollmer, Mencl & Müller (*Improved Laplacian Smoothing of Noisy Surface
//! Meshes*, EG 1999) abrem o paper com exatamente esta objeção: o operador
//! **contrai o volume**, e num objeto fechado a contração é monotônica e não
//! satura — alisar até a superfície ficar lisa é alisar até ela sumir.
//!
//! ⚠️ **A sonda mede pela porta do PRODUTO** (`SculptStroke::dab`), não por um
//! laço próprio sobre o `ring_average`: um laço próprio mediria a fórmula e
//! ficaria CEGO ao que o traço de fato faz com ela — a lição que a `line/Painter`
//! pagou três vezes.
//!
//! ⚠️ **E o oráculo é o RAIO MÉDIO, não o volume:** um volume por shoelace 3D
//! pede a orientação das faces e mediria a tesselação junto; o raio médio de uma
//! esfera unitária é `1,0` por construção, então todo desvio dele é o
//! encolhimento e nada mais.

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};

fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(64, 128, 1.0)
}

/// O raio MÉDIO da malha — `1,0` numa esfera unitária intacta.
fn mean_radius(mesh: &Mesh) -> f64 {
    let p = mesh.positions();
    let n = p.len();
    p.iter()
        .map(|v| f64::from(v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2]))).sqrt())
        .sum::<f64>()
        / n as f64
}

/// Um dab que cobre a esfera INTEIRA — o *Filter Layer*, que é onde o
/// encolhimento é o efeito e não um detalhe de borda.
fn whole_sphere_dab() -> Dab {
    Dab::at([0.0, 0.0, 0.0], 4.0, [0.0, 0.0, -1.0])
}

/// **A MEDIÇÃO.** N passadas de Smooth em força cheia, e o raio médio a cada uma.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn how_much_the_laplacian_smooth_shrinks_a_sphere() {
    let mut mesh = sphere();
    let b = Brush {
        verb: Verb::Smooth,
        radius: 4.0,
        strength: 1.0,
        // ⚠️ **`Constant` de propósito:** com uma curva macia o peso cairia com a
        // distância e o número falaria do FALLOFF junto. O que se mede aqui é o
        // operador.
        falloff: Falloff::Constant,
        ..Brush::default()
    };
    let r0 = mean_radius(&mesh);
    println!("\n  passadas   raio medio   encolhimento");
    println!("  --------   ----------   ------------");
    println!("  {:>8}   {r0:>10.6}   {:>11.2}%", 0, 0.0);
    for pass in 1..=40 {
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
        if pass % 5 == 0 || pass <= 3 {
            let r = mean_radius(&mesh);
            println!("  {pass:>8}   {r:>10.6}   {:>11.2}%", (r0 - r) / r0 * 100.0);
        }
    }
}

/// **A CURA DE TAUBIN, MEDIDA ANTES DE SER CONSTRUÍDA.**
///
/// Taubin (*A signal processing approach to fair surface design*, SIGGRAPH 1995)
/// alterna um passo laplaciano de peso `λ > 0` com um de peso `μ < 0`, com
/// `|μ| > λ`: o par é um passa-baixa cuja banda de passagem **devolve** o que o
/// laplaciano sozinho tira.
///
/// ⚠️ **E as duas metades JÁ EXISTEM neste motor como VERBOS:** o passo `λ` é o
/// nosso Smooth e o passo `μ` é o nosso Sharpen (*"o laplaciano com o sinal
/// trocado"*, `stroke_target.rs:234`). A sonda os alterna pela porta do produto —
/// se o raio segurar, a wave tem fundamento; se não, ela não tem, e é melhor
/// descobrir aqui do que depois de uma UI.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn whether_alternating_the_two_verbs_holds_the_radius() {
    // Os pesos do paper. `kpb = 0,1` ⇒ `λ ≈ 0,33`, `μ ≈ −0,34`.
    for (lambda, mu) in [(0.33f32, 0.34f32), (0.5, 0.53), (0.6, 0.64)] {
        let mut mesh = sphere();
        let step = |verb, w| Brush {
            verb,
            radius: 4.0,
            strength: w,
            falloff: Falloff::Constant,
            ..Brush::default()
        };
        let r0 = mean_radius(&mesh);
        let mut worst = 0.0f64;
        for _ in 0..20 {
            for b in [step(Verb::Smooth, lambda), step(Verb::Sharpen, mu)] {
                let mut s = SculptStroke::default();
                s.begin(&mesh);
                s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
            }
            let d = (r0 - mean_radius(&mesh)) / r0 * 100.0;
            worst = worst.max(d.abs());
        }
        let r = mean_radius(&mesh);
        println!(
            "  lambda {lambda:.2} / mu {mu:.2}: raio {r:.6}  desvio final {:.3}%  pior {worst:.3}%",
            (r0 - r) / r0 * 100.0
        );
    }
}
