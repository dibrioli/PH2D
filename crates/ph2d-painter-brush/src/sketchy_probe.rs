//! **SONDA (não é gate): o orçamento do Sketchy pela porta do PRODUTO.**
//!
//! A W0.3 mediu o comprimento de fio contra o arco do traço num harness próprio. Esta sonda mede o
//! que o motor de fato produz, e responde às duas perguntas que decidem constantes: **quanto custa
//! um dab** (a varredura de vizinhança é o único trabalho super-linear da wave) e **quanto fio a
//! densidade no teto de fato deposita**.
//!
//! Rodar: `cargo test -p ph2d-painter-brush --release measure_the_sketchy -- --ignored --nocapture`

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::{LineKind, SKETCHY_DENSITY_MAX};
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};

fn sp(reach: f32, density: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: LineKind::Sketchy,
        sketchy_reach: reach,
        sketchy_density: density,
        ..Default::default()
    }
}

fn dyn_() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

/// Um traço em ESPIRAL: ele volta sobre si mesmo, que é o gesto para o qual o Sketchy existe e o
/// único em que a memória antiga tem vizinhos legítimos.
fn spiral(spec: BrushSpec, turns: usize) -> (usize, f32, f32, std::time::Duration) {
    let mut s = Stroke::new(spec, dyn_(), 7);
    let mut out = Vec::new();
    let mut threads = Vec::new();
    let mut total_thread = 0.0f32;
    let mut arc = 0.0f32;
    let steps = turns * 64;
    let pt = |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        let t = i as f32 / 64.0;
        // Espiral por rotor unitário composto (HR-5: sem transcendental).
        let step = [0.995_184_7f32, 0.098_017_1]; // ~5,625° por passo (64 por volta)
        let mut r = [1.0f32, 0.0];
        for _ in 0..i {
            r = crate::heading::rotate(r, step);
        }
        let rad = 20.0 + 6.0 * t;
        [400.0 + r[0] * rad, 400.0 + r[1] * rad]
    };
    let t0 = std::time::Instant::now();
    s.begin(
        StrokePoint {
            pos: pt(0),
            pressure: 1.0,
        },
        &mut out,
    );
    let mut prev = pt(0);
    let mut dabs = out.len();
    s.take_threads(&mut threads);
    for i in 1..=steps {
        let p = pt(i);
        arc += (p[0] - prev[0]).hypot(p[1] - prev[1]);
        prev = p;
        s.extend(
            StrokePoint {
                pos: p,
                pressure: 1.0,
            },
            &mut out,
        );
        dabs += out.len();
        s.take_threads(&mut threads);
        for t in &threads {
            total_thread += (t[2] - t[0]).hypot(t[3] - t[1]);
        }
    }
    (dabs, arc, total_thread, t0.elapsed())
}

#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_sketchy_scan() {
    println!(
        "[sketchy] ESPIRAL que volta sobre si mesma (o gesto que a wave existe para costurar)"
    );
    println!(
        "{:>8} {:>8} {:>10} {:>10} {:>12} {:>12} {:>10}",
        "voltas", "dabs", "arco px", "fio px", "fio/arco", "ms", "us/dab"
    );
    for turns in [4usize, 16, 64] {
        let (dabs, arc, thread, dt) = spiral(sp(1.0, SKETCHY_DENSITY_MAX), turns);
        #[allow(clippy::cast_precision_loss)]
        let per = dt.as_secs_f64() * 1e6 / dabs as f64;
        println!(
            "{:>8} {:>8} {:>10.0} {:>10.0} {:>12.2} {:>12.2} {:>10.2}",
            turns,
            dabs,
            arc,
            thread,
            thread / arc,
            dt.as_secs_f64() * 1e3,
            per
        );
    }
    println!("[sketchy] leitura: `fio/arco` e o ORCAMENTO (o alvo derivado da W0.3 e ~2x);");
    println!("[sketchy]          `us/dab` CONSTANTE = o custo e linear no traco, nao quadratico.");
}

/// E o que a densidade cheia custaria — o número que torna o teto da W0.3 uma medição e não um
/// palpite.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_sketchy_budget_at_full_density() {
    println!("[sketchy] o que a DENSIDADE cheia custa, contra o teto orcado");
    println!("{:>10} {:>12} {:>12}", "densidade", "fio/arco", "us/dab");
    for d in [SKETCHY_DENSITY_MAX, 0.25, 1.0] {
        let (dabs, arc, thread, dt) = spiral(sp(1.0, d), 16);
        #[allow(clippy::cast_precision_loss)]
        let per = dt.as_secs_f64() * 1e6 / dabs as f64;
        println!("{d:>10.3} {:>12.2} {per:>12.2}", thread / arc);
    }
}
