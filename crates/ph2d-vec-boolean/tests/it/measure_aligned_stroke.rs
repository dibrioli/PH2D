//! **Quanto custa alinhar um traço** — a medição que decide o desenho, antes do código.
//!
//! Inner/Outer não é geometria nova: é `outline_stroke(2w)` cortado contra o preenchimento
//! (`∩` para dentro, `−` para fora), a mesma composição que o Pathfinder usa. A pergunta que
//! só um relógio responde é se isso pode rodar **por frame** — como o offset vivo — ou se
//! precisa de memo desde o primeiro dia.
//!
//! Rodar: `cargo test -p ph2d-vec-boolean --test measure_aligned_stroke -- --ignored --nocapture`

use ph2d_vec_boolean::{BoolOp, apply, outline_stroke};
use ph2d_vec_scene::{Contour, FillRule, LineCap, LineJoin, Rgba8, StrokeSpec, VecPath, VecVertex};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex::corner([x, y])
}

fn spec(width: f64) -> StrokeSpec {
    StrokeSpec {
        cap: LineCap::Butt,
        join: LineJoin::Miter,
        ..StrokeSpec::new(Rgba8::new(0, 0, 0, 255), width)
    }
}

/// Um retângulo com furo — a rosquinha do smoke, a forma em que a queda de FPS do offset foi
/// vista. Fixture deliberadamente NÃO trivial: uma forma convexa simples mediria o melhor caso.
fn donut(width: f64) -> VecPath {
    let hole = 0.7;
    VecPath {
        verts: vec![v(2.8, -1.2), v(5.2, -1.2), v(5.2, 1.2), v(2.8, 1.2)],
        closed: true,
        subpaths: vec![Contour::new_closed(vec![
            v(4.0 - hole, -hole),
            v(4.0 + hole, -hole),
            v(4.0 + hole, hole),
            v(4.0 - hole, hole),
        ])],
        fill_rule: FillRule::EvenOdd,
        stroke: Some(spec(width)),
        ..VecPath::default()
    }
}

/// Uma estrela de N pontas — quinas reentrantes, onde o miter do traço trabalha de verdade.
fn star(points: usize, width: f64) -> VecPath {
    let mut verts = Vec::with_capacity(points * 2);
    for i in 0..points * 2 {
        let a = std::f64::consts::TAU * (i as f64) / (points as f64 * 2.0);
        let r = if i % 2 == 0 { 2.0 } else { 0.9 };
        verts.push(v(a.cos() * r, a.sin() * r));
    }
    VecPath {
        verts,
        closed: true,
        stroke: Some(spec(width)),
        ..VecPath::default()
    }
}

fn median<F: FnMut() -> usize>(mut f: F) -> std::time::Duration {
    let mut runs: Vec<std::time::Duration> = (0..5)
        .map(|_| {
            let t0 = std::time::Instant::now();
            let n = f();
            let dt = t0.elapsed();
            assert!(n > 0, "a fixture nao pode produzir geometria vazia");
            dt
        })
        .collect();
    runs.sort_unstable();
    runs[2]
}

/// O que uma forma custa em CADA alinhamento, lado a lado.
fn table(name: &str, make: impl Fn(f64) -> VecPath, width: f64) {
    // (a) Centre: o que já shipa. `outline_stroke` na largura autorada — é o que o Expand faz
    //     hoje, e o que o pintor NÃO paga (Vello traça direto). Serve de piso da comparação.
    let centre = median(|| outline_stroke(&make(width)).len());

    // (b) A banda dupla: o traço a `2w`, que é o insumo dos dois alinhamentos.
    let band = median(|| outline_stroke(&make(width * 2.0)).len());

    // (c) Inner = banda ∩ preenchimento. (d) Outer = banda − preenchimento.
    let mut fill = make(width);
    fill.stroke = None;
    let inner = median(|| {
        let band = outline_stroke(&make(width * 2.0));
        band.iter()
            .flat_map(|b| apply(b, &fill, BoolOp::Intersect))
            .count()
            .max(1)
    });
    let outer = median(|| {
        let band = outline_stroke(&make(width * 2.0));
        band.iter()
            .flat_map(|b| apply(b, &fill, BoolOp::Subtract))
            .count()
            .max(1)
    });

    let ms = |d: std::time::Duration| d.as_secs_f64() * 1e3;
    println!(
        "{name:>14} w={width:<5} | centre {:>7.3} ms | banda 2w {:>7.3} | INNER {:>7.3} | OUTER {:>7.3}  (inner/centre {:.2}x)",
        ms(centre),
        ms(band),
        ms(inner),
        ms(outer),
        ms(inner) / ms(centre).max(1e-6),
    );
}

#[test]
#[ignore = "sonda de medicao, nao gate"]
fn what_an_aligned_stroke_costs() {
    println!("\n--- custo do alinhamento de traco (debug; o que importa e a ORDEM e a razao) ---");
    for w in [0.05, 0.2, 0.5] {
        table("rosquinha", donut, w);
    }
    for w in [0.05, 0.2, 0.5] {
        table("estrela 5", |x| star(5, x), w);
    }
    table("estrela 24", |x| star(24, x), 0.2);
    println!(
        "\nOrcamento de um frame a 60 fps: 16.6 ms. A pergunta e se INNER cabe nele \
         POR FORMA TRACADA, com a cena inteira re-cozida a cada arrasto de slider.\n"
    );
}
