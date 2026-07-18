//! O gate que pina o **espelho**: `ph2d-flip-fill` percorre os segmentos de uma
//! polilinha por conta própria (ela só depende de `ph2d-core` — não conhece o
//! documento), enquanto `FlipStroke::segments()` é a porta única daquela pergunta para
//! um traço do documento.
//!
//! Duas implementações da mesma pergunta divergem — a menos que algo as force a
//! concordar. Este arquivo é esse algo, e mora no shell porque **é o único lugar onde os
//! dois tipos coexistem**.

use super::boundaries;
use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, Point, Rgba};

fn stroke(pts: &[Vec2], width: f32, closed: bool) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &p in pts {
        s.push_point(Point {
            pos: p,
            width,
            opacity: 1.0,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
        });
    }
    s.closed = closed;
    s
}

/// A distância de `p` ao eixo do traço, computada pela porta do MODELO — o oráculo.
fn distance_via_the_models_door(s: &FlipStroke, p: Vec2) -> f32 {
    let mut best = f32::INFINITY;
    for (_, a, b) in s.segments() {
        let ab = Vec2::new(b.x - a.x, b.y - a.y);
        let l2 = ab.x * ab.x + ab.y * ab.y;
        let t = if l2 <= 0.0 {
            0.0
        } else {
            (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
        };
        let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
        best = best.min((dx * dx + dy * dy).sqrt());
    }
    best
}

/// **As duas caminhadas de segmento concordam** — em traço ABERTO e em traço FECHADO.
///
/// O caso que morde é o fechado: a costura (último → primeiro) é uma aresta inteira.
/// Se o espelho da crate a perdesse, todo ponto de contorno que a abraça cairia no
/// fallback da média — e num quadrado isso é 1/4 da arte com a dilatação errada.
///
/// Mutação que ele mata: tirar o `.chain(seam...)` do `dilate::segments` — a distância
/// da crate salta para a do vértice mais próximo, e o `assert` sangra no traço fechado.
#[test]
fn the_two_segment_walks_agree() {
    let square = [
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        Vec2::new(100.0, 100.0),
        Vec2::new(0.0, 100.0),
    ];
    let zig = [
        Vec2::new(0.0, 0.0),
        Vec2::new(40.0, 30.0),
        Vec2::new(80.0, -10.0),
        Vec2::new(120.0, 20.0),
    ];

    for (name, pts, closed) in [
        ("quadrado FECHADO", &square[..], true),
        ("quadrado ABERTO", &square[..], false),
        ("zigue-zague aberto", &zig[..], false),
    ] {
        let mut d = FlipDrawing::new();
        d.strokes.push(stroke(pts, 6.0, closed));
        let bounds = boundaries(&d);

        // Sondas espalhadas, inclusive em cima da aresta da costura (x≈0, y no meio) —
        // que é exatamente onde as duas caminhadas podem discordar.
        for gx in 0..9 {
            for gy in 0..9 {
                let p = Vec2::new(-10.0 + gx as f32 * 17.5, -10.0 + gy as f32 * 17.5);
                let want = distance_via_the_models_door(&d.strokes[0], p);
                let got = ph2d_flip_fill::local_line(&bounds, p)
                    .map(|(_, dist)| dist)
                    .unwrap_or(f32::INFINITY);
                assert!(
                    (got - want).abs() < 1e-3,
                    "{name}: em ({}, {}) a crate diz {got} e a porta do modelo diz {want}",
                    p.x,
                    p.y
                );
            }
        }
    }
}

/// **A espessura que a crate devolve é a CHEIA** — a lista de fronteiras fala MEIA
/// espessura (a convenção do `fill_at`), e a dilatação veste o diâmetro.
///
/// Sem este gate, um espelho que esquecesse o `2.0 *` daria metade da dilatação e o
/// sintoma seria sutil: a cor pararia no meio da linha em vez de na borda dela.
#[test]
fn the_width_that_comes_back_is_the_full_one() {
    let mut d = FlipDrawing::new();
    d.strokes.push(stroke(
        &[Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0)],
        7.0,
        false,
    ));
    let bounds = boundaries(&d);
    assert!(
        (bounds[0].1[0] - 3.5).abs() < 1e-6,
        "premissa: a lista de fronteiras fala MEIA espessura"
    );
    let (w, _) = ph2d_flip_fill::local_line(&bounds, Vec2::new(50.0, 1.0)).expect("acha");
    assert!(
        (w - 7.0).abs() < 1e-6,
        "a dilatacao veste o DIAMETRO (7,0), veio {w}"
    );
}
