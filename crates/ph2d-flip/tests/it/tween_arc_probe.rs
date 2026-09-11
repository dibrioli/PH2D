//! **O bug do arco (report do Enio) — gate de regressão + sonda.**
//!
//! O smoke da fase usa dois blobs IDÊNTICOS (forma igual, só movida). O artista desenha as duas
//! chaves À MÃO: mesma forma geral, contagens diferentes, tremor diferente, e a costura (o ponto
//! onde ele fechou o traço) cai em lugares ligeiramente diferentes. A 1ª versão da fase
//! (correlação da VIRADA, invariante à rotação) escolhia uma fase que fazia o ajuste achar um
//! GIRO — e a espiral varria um ARCO em vez de deslizar. Uma costura 2 vértices fora já subia o
//! meio +20 unidades. A fase pelo TRAJETO (posições centradas) sobrepõe as formas com o menor
//! deslocamento ⇒ o ajuste vê ~translação ⇒ desliza.

use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, Point, Rgba, TweenOptions, tween_drawing};
use std::f32::consts::TAU;

/// Um blob-pêra desenhado "à mão": um ovo com um tremor determinístico por vértice, começando
/// pelo vértice `seam` (a costura) e transladado por `off`. `jitter` semeia o tremor (chaves
/// desenhadas em momentos diferentes têm tremor diferente).
fn hand_blob(n: usize, seam: usize, off: Vec2, jitter: u32) -> FlipStroke {
    let mut s = FlipStroke::new();
    for i in 0..n {
        let a = ((i + seam) % n) as f32 / n as f32 * TAU;
        let base = 30.0 * (1.0 + 0.35 * a.cos()); // ovo (assimétrico em X)
        let h = (i as u32)
            .wrapping_mul(2_654_435_761)
            .wrapping_add(jitter.wrapping_mul(40_503));
        let noise = ((h >> 9) & 0xff) as f32 / 255.0 - 0.5; // [-0.5, 0.5]
        let r = base + noise * 3.0; // tremor de mão, +/-1.5 px
        s.push_point(Point {
            pos: off + Vec2::new(a.cos() * r, a.sin() * r * 0.85),
            width: 0.5,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    s.closed = true;
    s
}

fn centroid(s: &FlipStroke) -> Vec2 {
    let p = s.positions();
    p.iter().fold(Vec2::ZERO, |a, &q| a + q) / p.len() as f32
}

/// Quantos pares de segmentos NÃO-adjacentes do anel se cruzam (0 = forma limpa; >0 = "oito").
fn self_intersections(s: &FlipStroke) -> usize {
    let p = s.positions();
    let n = p.len();
    let seg = |i: usize| (p[i], p[(i + 1) % n]);
    let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
    let inter = |a: Vec2, b: Vec2, c: Vec2, d: Vec2| -> bool {
        let d1 = cross(b - a, c - a);
        let d2 = cross(b - a, d - a);
        let d3 = cross(d - c, a - c);
        let d4 = cross(d - c, b - c);
        (d1 * d2 < 0.0) && (d3 * d4 < 0.0)
    };
    let mut count = 0;
    for i in 0..n {
        for j in (i + 2)..n {
            if i == 0 && j == n - 1 {
                continue; // o par de fecho é adjacente
            }
            let (a, b) = seg(i);
            let (c, d) = seg(j);
            if inter(a, b, c, d) {
                count += 1;
            }
        }
    }
    count
}

fn keys(off: usize, dx: f32) -> (FlipDrawing, FlipDrawing) {
    // A: 50 pts, costura 0, tremor #1. B: 54 pts, costura deslocada, tremor #2, movido dx.
    let mut a = FlipDrawing::new();
    a.strokes
        .push(hand_blob(50, 0, Vec2::new(-dx / 2.0, 0.0), 1));
    let mut b = FlipDrawing::new();
    b.strokes
        .push(hand_blob(54, off, Vec2::new(dx / 2.0, 0.0), 2));
    (a, b)
}

/// 🔴 **O bug do Enio, ponta a ponta:** um blob de mão MOVIDO na horizontal desliza reto — o
/// centróide fica perto da reta A→B (ambos em `y=0`) e o anel não se cruza. A fase pela VIRADA
/// subia o meio +20 numa costura 2 vértices fora; a fase pelo TRAJETO mantém em ~0.
///
/// Mutação que sangra: a fase devolvendo um deslocamento espúrio (`n/6`) ⇒ um giro de ~60° ⇒ o
/// meio arqueia bem além do teto. (Também sangra a versão da virada, o defeito original.)
#[test]
fn a_moved_hand_drawn_blob_glides_straight_instead_of_arcing() {
    let dx = 120.0f32; // as duas chaves, MOVIDAS na horizontal, ambas em y=0
    let o = TweenOptions::default();
    for off in [0usize, 1, 2, 3, 4, 5, 8] {
        let (a, b) = keys(off, dx);
        for t in [0.25f32, 0.5, 0.75] {
            let mid = tween_drawing(&a, &b, t, o);
            let y = centroid(&mid.strokes[0]).y;
            assert!(
                y.abs() < 8.0,
                "off={off} t={t}: o blob ARQUEOU para y={y:.1} (a reta A->B está em y=0; a fase \
                 pela virada dava ~+20)"
            );
            assert_eq!(
                self_intersections(&mid.strokes[0]),
                0,
                "off={off} t={t}: o anel do meio se cruzou num OITO"
            );
        }
    }
}

/// **A SONDA (render-and-look):** imprime o arco (centróide y) e os cruzamentos por deslocamento
/// de costura. `cargo test -p ph2d-flip --release --test tween_arc_probe -- --ignored --nocapture`
#[test]
#[ignore = "sonda: mede o arco/oito do tween de dois blobs de mão, por deslocamento de costura"]
fn probe_hand_drawn_blob_arcs() {
    let dx = 120.0f32;
    println!(
        "\n  Dois blobs de mão (ovo ~30px, tremor diferente), A em x=-{:.0} e B em x=+{:.0},\n  \
         AMBOS em y=0. Um tween correto desliza reto: centróide y ~ 0, 0 cruzamentos.\n",
        dx / 2.0,
        dx / 2.0
    );
    println!("  desloc.   |  y do centróide por t (0.25 / 0.50 / 0.75)  |  cruzamentos (t=0.5)");
    for off in [0usize, 2, 4, 8, 16, 24] {
        let (a, b) = keys(off, dx);
        let o = TweenOptions::default();
        let ys: Vec<f32> = [0.25, 0.5, 0.75]
            .iter()
            .map(|&t| centroid(&tween_drawing(&a, &b, t, o).strokes[0]).y)
            .collect();
        let xi = self_intersections(&tween_drawing(&a, &b, 0.5, o).strokes[0]);
        println!(
            "   {off:^6}   |   {:+6.1}  /  {:+6.1}  /  {:+6.1}              |   {xi:^3}",
            ys[0], ys[1], ys[2]
        );
    }
    println!();
}
