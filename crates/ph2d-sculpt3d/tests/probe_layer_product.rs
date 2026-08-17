//! **A DEMÃO CHEGA AO BARRO?** — a sonda do PRODUTO, não do kernel.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test probe_layer_product \
//!   -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Os doze gates do `verb_layer_tests` passam e o artista reporta *"nada
//! mudou"*. Um gate de kernel é cego à fiação, então esta sonda arma o pincel
//! pelas MESMAS portas que o `arm_verb_defaults` do painel chama
//! (`birth_for` · `default_strength` · `default_falloff`) e percorre o traço
//! pelo `walk`, que é o que a shell faz.

use ph2d_mesh::shapes;
use ph2d_sculpt3d::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb};

/// O relevo máximo para FORA da esfera unitária.
fn relief(mesh: &ph2d_mesh::Mesh) -> f32 {
    mesh.positions()
        .iter()
        .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - 1.0)
        .fold(f32::NEG_INFINITY, f32::max)
}

/// Arma o pincel como o painel arma ao escolher o verbo.
fn armed(verb: Verb) -> Brush {
    let mode = RefMode::birth_for(verb);
    Brush {
        verb,
        mode,
        strength: verb.default_strength(),
        falloff: verb.default_falloff(mode),
        radius: 0.30,
        ..Brush::default()
    }
}

/// Um traço reto, percorrido pelo `walk` como a shell faz.
///
/// ⚠️ **`fresh` é a diferença entre esfregar e SOLTAR** — o pen-up encerra o
/// traço, e a demão declara (gate `a_second_stroke_lays_a_second_coat`) que a
/// segunda pincelada deposita uma segunda camada. Uma sonda que só esfregasse
/// mediria a saturação e chamaria de inerte o que é o desenho.
fn stroke(brush: &Brush, passes: usize, fresh: bool) -> (usize, f32) {
    let mut mesh = shapes::uv_sphere(48, 72, 1.0);
    mesh.triangulate();
    mesh.rebuild();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    let mut dabs = 0usize;
    for _ in 0..passes {
        if fresh {
            s.begin(&mesh);
        }
        let (a, b) = ([-0.35_f32, 0.0], [0.35_f32, 0.0]);
        let Some(walk) = ph2d_sculpt3d::walk(a, b, 0.06) else {
            continue;
        };
        for [sx, sy] in walk {
            let ray = ph2d_mesh::Ray::new([sx, sy, 5.0], [0.0, 0.0, -1.0]);
            let Some(hit) = mesh.raycast(&ray) else { break };
            s.dab(
                &mut mesh,
                brush,
                &Dab::at(hit.point, brush.radius, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
            dabs += 1;
        }
    }
    (dabs, relief(&mesh))
}

#[test]
#[ignore]
fn probe_does_the_coat_reach_the_clay() {
    println!("\n  verbo      modo  força  curva        h      dabs   relevo");
    println!("  ---------  ----  -----  -----------  -----  -----  --------");
    for verb in [Verb::Layer, Verb::Draw] {
        let b = armed(verb);
        let (dabs, r) = stroke(&b, 1, false);
        println!(
            "  {:<9}  {:<4?}  {:.3}  {:<11}  {:.3}  {:>5}  {:>8.5}",
            verb.label(),
            b.mode,
            b.strength,
            b.falloff.label(),
            b.layer_height,
            dabs,
            r
        );
    }
    println!("\n  -- ESFREGANDO sem soltar (um traço só) --");
    for passes in [1usize, 2, 4, 8] {
        let (_, l) = stroke(&armed(Verb::Layer), passes, false);
        let (_, d) = stroke(&armed(Verb::Draw), passes, false);
        println!("  passadas={passes:<2}  Layer {l:>8.5}   Draw {d:>8.5}");
    }
    println!("\n  -- SOLTANDO entre as passadas (pincelada nova) --");
    for passes in [1usize, 2, 4, 8] {
        let (_, l) = stroke(&armed(Verb::Layer), passes, true);
        let (_, d) = stroke(&armed(Verb::Draw), passes, true);
        println!("  pinceladas={passes:<2}  Layer {l:>8.5}   Draw {d:>8.5}");
    }
    println!();
}
