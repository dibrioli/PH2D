//! ⭐⭐⭐ **A PERGUNTA QUE VEM ANTES DO LOTE 9** (`CLAUDE.md` §5.0): *antes de construir um item de
//! lista aberta, MEÇA se a composição já o exprime*.
//!
//! O levantamento diz que o **polígono de vértices arbitrários** e o **triângulo escaleno** «hoje
//! obrigam a desenhar, e desenhar custa por segmento». ⚠️ **A segunda metade dessa frase é uma
//! MEDIÇÃO que ninguém refez desde a W123** — e o `spike_formula_vs_profile` mediu `1,27×` com
//! **6** lados contra `134×` com **192**. *Se a `1,27×` a extrusão já entrega o polígono, o que
//! falta não é uma primitiva: é um BOTÃO.*

use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
use ph2d_field_eval::Field;

fn campo(p: Primitive) -> Field {
    Field::new(
        &FieldDoc::new(vec![ph2d_field_eval::leaf(p, Xform::IDENTITY)], NodeId(0)).expect("peça"),
    )
}

/// Um polígono REGULAR de `n` lados, pela porta do desenho.
fn extrusao(n: usize) -> Primitive {
    let contorno: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(0.35 * a.cos()) as f32, (0.35 * a.sin()) as f32]
        })
        .collect();
    Primitive::Extrude {
        profile: Profile::new(vec![contorno], FillRule::NonZero, 1e-4).expect("perfil"),
        half_height: 0.25,
        round: 0.0,
        chamfer: 0.0,
    }
}

fn cronometra(f: &Field) -> f64 {
    let n = 60;
    let at = |t: usize| -0.7 + 1.4 * (t as f64 + 0.5) / n as f64;
    let mut melhor = f64::INFINITY;
    for _ in 0..3 {
        let t0 = std::time::Instant::now();
        let mut acc = 0.0;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    acc += f.at(at(i), at(j), at(k));
                }
            }
        }
        std::hint::black_box(acc);
        melhor = melhor.min(t0.elapsed().as_secs_f64() * 1000.0);
    }
    melhor
}

#[test]
#[ignore = "sonda: a extrusão já entrega o polígono?"]
fn probe_is_a_polygon_already_reachable() {
    let base = cronometra(&campo(Primitive::Sphere { radius: 0.35 }));
    // O PRISMA de N lados, que é a forma por fórmula que a casa já tem — o tecto do que uma
    // primitiva de polígono REGULAR podia custar.
    println!("\n── o custo de um polígono, pelas DUAS portas (contra a esfera) ──");
    println!(
        "  {:<8} {:>12} {:>12}   {}",
        "lados", "extrusão", "prisma", "razão extrusão/prisma"
    );
    for n in [3_usize, 4, 5, 6, 8, 12, 16, 24, 32] {
        let ext = cronometra(&campo(extrusao(n)));
        let pri = if n <= 32 {
            #[allow(clippy::cast_possible_truncation)]
            Some(cronometra(&campo(Primitive::Prism {
                sides: n as u32,
                bottom: 0.35,
                top: 0.35,
                half_height: 0.25,
                round: 0.0,
                chamfer: 0.0,
            })))
        } else {
            None
        };
        println!(
            "  {n:<8} {:>11.2}× {:>11}   {}",
            ext / base,
            pri.map_or("—".into(), |p| format!("{:.2}×", p / base)),
            pri.map_or("—".into(), |p| format!("{:.2}×", ext / p))
        );
    }
    println!(
        "\n  ⚠️ load: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
