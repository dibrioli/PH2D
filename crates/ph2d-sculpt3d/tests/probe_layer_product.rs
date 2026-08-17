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
fn armed_with(verb: Verb, hardness: f32, auto_smooth: f32) -> Brush {
    let mut b = armed(verb);
    b.hardness = hardness;
    b.auto_smooth = auto_smooth;
    b
}

/// O desvio de guarda-chuva sobre os vértices TOCADOS, em fração da ARESTA —
/// a régua do espeto. Varrer a malha inteira mediria os polos da uv_sphere, e
/// não o traço (a lição da sonda irmã).
fn umbrella_of(mesh: &ph2d_mesh::Mesh, verts: &[u32]) -> f32 {
    let (pos, adj) = (mesh.positions(), mesh.adjacency());
    let mut worst = 0.0f32;
    for &vi in verts {
        let i = vi as usize;
        let p = pos[i];
        let nb = adj.vert_verts.neighbours(i);
        if nb.len() < 3 {
            continue;
        }
        let (mut mid, mut len) = ([0.0f32; 3], 0.0f32);
        for &j in nb {
            let q = pos[j as usize];
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            len += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            for k in 0..3 {
                mid[k] += q[k];
            }
        }
        let n = nb.len() as f32;
        let d = [p[0] - mid[0] / n, p[1] - mid[1] / n, p[2] - mid[2] / n];
        let edge = len / n;
        if edge > 1e-9 {
            let dev = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() / edge;
            worst = worst.max(dev);
        }
    }
    worst
}

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

/// **OS DOIS EIXOS DO REPORT** — Hardness e Auto Smooth sobre a demão.
#[test]
#[ignore]
fn probe_hardness_and_auto_smooth_on_the_coat() {
    println!("\n  == HARDNESS sobre a DEMAO (uma pincelada) ==");
    println!("  hardness   relevo    espeto   espeto/relevo");
    println!("  ---------  --------  -------  -------------");
    for h in [0.0f32, 0.25, 0.5, 0.75, 0.9] {
        let b = armed_with(Verb::Layer, h, 0.0);
        let (_, r, u) = stroke_probe(&b, 1);
        let per = if r > 1e-6 { u / r } else { 0.0 };
        println!("  {h:<9.2}  {r:>8.5}  {u:>7.5}  {per:>13.3}");
    }
    println!("\n  == e o CONTROLE, o mesmo em Draw ==");
    for h in [0.0f32, 0.5, 0.9] {
        let b = armed_with(Verb::Draw, h, 0.0);
        let (_, r, u) = stroke_probe(&b, 1);
        let per = if r > 1e-6 { u / r } else { 0.0 };
        println!("  {h:<9.2}  {r:>8.5}  {u:>7.5}  {per:>13.3}");
    }
    println!("\n  == AUTO SMOOTH sobre a DEMAO (uma pincelada) ==");
    println!("  auto_sm    relevo    espeto   espeto/relevo");
    println!("  ---------  --------  -------  -------------");
    for a in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
        let b = armed_with(Verb::Layer, 0.0, a);
        let (_, r, u) = stroke_probe(&b, 1);
        let per = if r > 1e-6 { u / r } else { 0.0 };
        println!("  {a:<9.2}  {r:>8.5}  {u:>7.5}  {per:>13.3}");
    }
    println!("\n  == e o CONTROLE, o mesmo em Draw ==");
    for a in [0.0f32, 0.5, 1.0] {
        let b = armed_with(Verb::Draw, 0.0, a);
        let (_, r, u) = stroke_probe(&b, 1);
        let per = if r > 1e-6 { u / r } else { 0.0 };
        println!("  {a:<9.2}  {r:>8.5}  {u:>7.5}  {per:>13.3}");
    }
    println!();
}

/// Como o `stroke`, mas devolve TAMBEM o espeto sobre os tocados.
fn stroke_probe(brush: &Brush, passes: usize) -> (usize, f32, f32) {
    let mut mesh = shapes::uv_sphere(48, 72, 1.0);
    mesh.triangulate();
    mesh.rebuild();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    let mut dabs = 0usize;
    for _ in 0..passes {
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
    let touched = s.touched().to_vec();
    (dabs, relief(&mesh), umbrella_of(&mesh, &touched))
}
