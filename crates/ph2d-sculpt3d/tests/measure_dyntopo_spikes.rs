//! **DE QUE É FEITO O "HORRÍVEL" DEPOIS DO `P`** — a sonda que atribui o defeito.
//!
//! O report (2026-08-04, com foto): ligar a topologia dinâmica e esculpir devolve
//! uma superfície de **AGULHAS** — estrelas finas irradiando de dentro da região
//! trabalhada. A cena `=14` monta exatamente isso.
//!
//! Ela mede **desvio de guarda-chuva**: para cada vértice, a distância dele à
//! MÉDIA dos vizinhos, dividida pelo comprimento médio de aresta local. Numa
//! superfície lisa isso é a curvatura vezes meia aresta (~0,0x); numa agulha é da
//! ordem de 1 — o vértice está sozinho, longe do plano dos vizinhos.
//!
//! ⚠️ **O oráculo é uma RAZÃO por aresta local, não uma distância absoluta.** O
//! refino encurta as arestas, então uma medida absoluta ficaria menor
//! automaticamente com dyntopo ligado — a wave inteira mediria como melhoria.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --test measure_dyntopo_spikes --release
//! -- --ignored --nocapture`

use ph2d_mesh::{Birth, Mesh, edge_target, refine_in_sphere, shapes::uv_sphere};
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};

/// A estatística do guarda-chuva: `(pior, p99, média)` do desvio relativo.
fn umbrella(mesh: &Mesh) -> (f32, f32, f32) {
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let mut all: Vec<f32> = Vec::with_capacity(pos.len());
    for (i, p) in pos.iter().enumerate() {
        let nb = adj.vert_verts.neighbours(i);
        if nb.len() < 3 {
            continue;
        }
        let mut mid = [0.0f32; 3];
        let mut len = 0.0f32;
        for &j in nb {
            let q = pos[j as usize];
            for k in 0..3 {
                mid[k] += q[k];
            }
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            len += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        }
        let n = nb.len() as f32;
        let d = [p[0] - mid[0] / n, p[1] - mid[1] / n, p[2] - mid[2] / n];
        let dev = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let edge = len / n;
        if edge > 1e-9 {
            all.push(dev / edge);
        }
    }
    all.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let worst = all.last().copied().unwrap_or(0.0);
    let p99 = all
        .get((all.len() as f32 * 0.99) as usize)
        .copied()
        .unwrap_or(0.0);
    let mean = all.iter().sum::<f32>() / all.len().max(1) as f32;
    (worst, p99, mean)
}

/// A qualidade dos triângulos: `(pior angulo minimo, p1, fração abaixo de 10°)`.
///
/// ⚠️ **Uma lasca não desloca vértice nenhum, então o guarda-chuva é CEGO a
/// ela** — e é ela que a luz desenha como agulha, porque a normal por-vértice de
/// um triângulo fino aponta para qualquer lado.
fn sliver(mesh: &Mesh) -> (f32, f32, f32) {
    let pos = mesh.positions();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    mesh.triangle_indices(&mut tris);
    let mut mins: Vec<f32> = Vec::with_capacity(tris.len());
    for t in &tris {
        let p: Vec<[f32; 3]> = t.iter().map(|&i| pos[i as usize]).collect();
        let mut worst = 180.0f32;
        for k in 0..3 {
            let (o, u, v) = (p[k], p[(k + 1) % 3], p[(k + 2) % 3]);
            let a = [u[0] - o[0], u[1] - o[1], u[2] - o[2]];
            let b = [v[0] - o[0], v[1] - o[1], v[2] - o[2]];
            let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
            let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
            if la < 1e-12 || lb < 1e-12 {
                worst = 0.0;
                continue;
            }
            let c = ((a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (la * lb)).clamp(-1.0, 1.0);
            worst = worst.min(c.acos().to_degrees());
        }
        mins.push(worst);
    }
    mins.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let worst = mins.first().copied().unwrap_or(0.0);
    let p1 = mins.get(mins.len() / 100).copied().unwrap_or(0.0);
    let bad = mins.iter().filter(|m| **m < 10.0).count() as f32 / mins.len().max(1) as f32;
    (worst, p1, bad)
}

/// O traço do produto: um arrasto sobre o topo da esfera, com ou sem refino.
fn stroke_across(refine: bool) -> Mesh {
    let mut mesh = uv_sphere(10, 14, 1.0);
    // ⚠️ O produto TRIANGULA ao ligar (`toggle_dyntopo`), e sem esta linha o
    // refino devolve `NotTriangles` e a sonda mede duas cenas IDÊNTICAS — a
    // fixture que não contém o fenômeno, na sua forma mais barata de cometer.
    mesh.triangulate();
    mesh.rebuild();
    let brush = Brush {
        verb: Verb::Draw,
        falloff: Falloff::Smooth,
        radius: 0.30,
        strength: 0.6,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    let mut births: Vec<Birth> = Vec::new();
    stroke.begin(&mesh);
    const DABS: usize = 24;
    for k in 0..DABS {
        let t = k as f32 / (DABS - 1) as f32;
        let x = -0.6 + 1.2 * t;
        let y = (1.0 - x * x).max(0.0).sqrt();
        let center = [x, y, 0.0];
        // O olho olha de cima, do lado de fora para dentro.
        let eye = [-center[0], -center[1], -center[2]];
        if refine {
            let target = edge_target(brush.radius, 0.5);
            let _ = refine_in_sphere(&mut mesh, center, brush.radius, target, &mut births);
            stroke.grow_with(&mesh, &births);
        }
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at(center, brush.radius, eye),
            Symmetry::default(),
        );
    }
    mesh
}

/// **Até onde a propagação alcança**, em unidades de raio de pincel.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_how_far_the_propagation_reaches() {
    for (rings, segs) in [(8, 12), (10, 14), (16, 24), (24, 36)] {
        let mut mesh = uv_sphere(rings, segs, 1.0);
        mesh.triangulate();
        mesh.rebuild();
        let before = mesh.vert_count();
        let (centre, radius) = ([0.0, 1.0, 0.0], 0.35f32);
        let target = edge_target(radius, 0.7);
        let _ = refine_in_sphere(&mut mesh, centre, radius, target, &mut Vec::new());
        let far = mesh.positions()[before..]
            .iter()
            .map(|p| {
                let d = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() / radius
            })
            .fold(0.0f32, f32::max);
        println!(
            "  esfera {rings}x{segs}: {} verts novos, alcance maximo {far:.2}x o raio",
            mesh.vert_count() - before
        );
    }
}

/// **O gesto que contém o outro fenômeno: APERTAR no mesmo lugar.**
///
/// Com o pincel a andar, o refino dispara quase sempre em terreno virgem — os
/// pais ainda não foram deslocados, e a herança do `pre` quase não tem o que
/// corrigir. Apertando, a superfície SOBE, as arestas esticam e o refino passa a
/// nascer entre pais que o próprio traço já levantou: é ali que tratar o vértice
/// novo como nunca-visto conta o deslocamento duas vezes.
fn stroke_press() -> Mesh {
    let mut mesh = uv_sphere(10, 14, 1.0);
    mesh.triangulate();
    mesh.rebuild();
    let brush = Brush {
        verb: Verb::Draw,
        falloff: Falloff::Smooth,
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    let mut births: Vec<Birth> = Vec::new();
    stroke.begin(&mesh);
    const DABS: usize = 20;
    for k in 0..DABS {
        // A pressão SOBE: sem isso o envelope satura no primeiro dab e a
        // superfície não estica mais — a fixture não conteria o fenômeno.
        let pressure = 0.05 + 0.95 * (k as f32 / (DABS - 1) as f32);
        let center = [0.0, 1.0, 0.0];
        let target = edge_target(brush.radius, 0.7);
        let _ = refine_in_sphere(&mut mesh, center, brush.radius, target, &mut births);
        stroke.grow_with(&mesh, &births);
        let mut d = Dab::at(center, brush.radius, [0.0, -1.0, 0.0]);
        d.pressure = pressure;
        stroke.dab(&mut mesh, &brush, &d, Symmetry::default());
    }
    mesh
}

#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_what_pressing_in_one_place_leaves_behind() {
    let m = stroke_press();
    let (w, p99, mean) = umbrella(&m);
    let (sw, sp, sb) = sliver(&m);
    println!("\nAPERTAR NO MESMO LUGAR ({} verts)", m.vert_count());
    println!("  guarda-chuva  pior {w:.4}  p99 {p99:.4}  media {mean:.4}");
    println!(
        "  angulo minimo pior {sw:.2}  p1 {sp:.2}  <10 graus {:.1}%",
        sb * 100.0
    );
}

#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_what_the_dyntopo_stroke_leaves_behind() {
    let plain = stroke_across(false);
    let dyn_mesh = stroke_across(true);

    let (pw, pp, pm) = umbrella(&plain);
    let (dw, dp, dm) = umbrella(&dyn_mesh);

    println!("\nDESVIO DE GUARDA-CHUVA (fração da aresta local)");
    println!("  cena                         verts   pior     p99      media");
    println!(
        "  CONTROLE (sem dyntopo)      {:6}  {:.4}   {:.4}   {:.4}",
        plain.vert_count(),
        pw,
        pp,
        pm
    );
    println!(
        "  PRODUTO  (com dyntopo)      {:6}  {:.4}   {:.4}   {:.4}",
        dyn_mesh.vert_count(),
        dw,
        dp,
        dm
    );
    println!("  razao pior: {:.2}x", dw / pw.max(1e-9));

    let (psw, psp, psb) = sliver(&plain);
    let (dsw, dsp, dsb) = sliver(&dyn_mesh);
    println!("\nQUALIDADE DE TRIANGULO (angulo minimo, graus)");
    println!("  cena                          pior     p1      abaixo de 10 graus");
    println!(
        "  CONTROLE                     {psw:6.2}  {psp:6.2}   {:.1}%",
        psb * 100.0
    );
    println!(
        "  PRODUTO                      {dsw:6.2}  {dsp:6.2}   {:.1}%",
        dsb * 100.0
    );

    // ⚠️ **O VAZAMENTO**: o traço anda pelo topo (y >= 0,8). Um vértice no
    // hemisfério de baixo é território que o artista NUNCA tocou, e refinar ali
    // é quebrar a promessa da wave ("detalhe onde o pincel toca").
    let far = |m: &Mesh| m.positions().iter().filter(|p| p[1] < -0.2).count();
    println!("\nVAZAMENTO (vertices no hemisferio NAO tocado, y < -0,2)");
    println!("  base            {}", far(&uv_sphere(10, 14, 1.0)));
    println!("  CONTROLE        {}", far(&plain));
    println!("  PRODUTO         {}", far(&dyn_mesh));
}

/// A malha REFINADA sem traço nenhum: o refino sozinho deforma a superfície?
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_what_the_refinement_alone_does_to_a_clean_sphere() {
    let mut before = uv_sphere(10, 14, 1.0);
    before.triangulate();
    before.rebuild();
    let (bw, bp, bm) = umbrella(&before);

    let mut after = uv_sphere(10, 14, 1.0);
    after.triangulate();
    after.rebuild();
    let target = edge_target(0.30, 0.5);
    let out = refine_in_sphere(&mut after, [0.0, 1.0, 0.0], 0.30, target, &mut Vec::new());

    let (aw, ap, am) = umbrella(&after);
    // Quanto o refino afastou a superfície da esfera unitária?
    let worst_radial = after
        .positions()
        .iter()
        .map(|p| ((p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - 1.0).abs())
        .fold(0.0f32, f32::max);

    println!("\nREFINO SOZINHO, esfera limpa ({out:?})");
    println!(
        "  antes  {:6} verts  pior {:.4}  p99 {:.4}  media {:.4}",
        before.vert_count(),
        bw,
        bp,
        bm
    );
    println!(
        "  depois {:6} verts  pior {:.4}  p99 {:.4}  media {:.4}",
        after.vert_count(),
        aw,
        ap,
        am
    );
    println!("  maior desvio radial da esfera unitaria: {worst_radial:.5}");
    let (bw2, bp2, bb2) = sliver(&before);
    let (aw2, ap2, ab2) = sliver(&after);
    println!(
        "  angulo minimo  antes  pior {bw2:.2}  p1 {bp2:.2}  <10 graus {:.1}%",
        bb2 * 100.0
    );
    println!(
        "  angulo minimo  depois pior {aw2:.2}  p1 {ap2:.2}  <10 graus {:.1}%",
        ab2 * 100.0
    );
}
