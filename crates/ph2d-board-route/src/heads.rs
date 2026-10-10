//! ⭐ **As pontas de seta** (W9, lei XA2 do [estudo 07](../../../docs/MiroClone/07_fidelidade_excalidraw.md)
//! §2): cada tipo tem um TAMANHO de base e, numa seta curta, ENCOLHE para metade do último trecho —
//! `min(base, trecho / 2)`, igual em qualquer espessura (medido nas duas versões do oráculo:
//! `arrow` L 8 → 4 · 20 → 10 · ≥ 50 → 25). ⛔ Antes da W9 a caixa era fixa e uma seta ≤ 20 ficava SEM
//! ponta. A geometria é a do catálogo vectorial (a abertura fica a dele, W2); o tamanho é o do
//! Excalidraw. As de cardinalidade (que o catálogo não tem) desenham-se aqui.

use ph2d_board_model::Head;
use ph2d_vec_scene::Marker;
use ph2d_vector::{BezPath, Circle, Point, Shape as _};

/// O tamanho de base de cada ponta, como o Excalidraw o mede (`P_comprimento`, `P_tipos`): a risca
/// da `arrow` 25 · o lado do triângulo 15 · o diâmetro do círculo 15 · cada metade da barra 15 · o
/// comprimento do losango 24 · as riscas de cardinalidade 20 (o pé-de-galinha, [`CROWFOOT`]).
#[must_use]
pub fn base(h: Head) -> f64 {
    match h {
        Head::None => 0.0,
        Head::Arrow => 25.0,
        Head::Triangle | Head::TriangleOpen | Head::Circle | Head::CircleOpen | Head::Bar => 15.0,
        Head::Diamond | Head::DiamondOpen => 24.0,
        Head::One
        | Head::Many
        | Head::OneOrMany
        | Head::ExactlyOne
        | Head::ZeroOrOne
        | Head::ZeroOrMany => 20.0,
    }
}

/// O tamanho do pé-de-galinha (`cardinality_many`): 15, com a MESMA regra de encolher.
const CROWFOOT: f64 = 15.0;

/// A meia-abertura da `arrow` do Excalidraw (20°) — só para pôr a profundidade dela onde a medimos
/// (`25 · cos 20° = 23,49`, a do quadro desde a W2); a abertura desenhada é a do catálogo.
const ARROW_DEG: f64 = 20.0;
/// A meia-abertura do triângulo e das riscas de cardinalidade (25°, `P_tipos`).
const WIDE_DEG: f64 = 25.0;

/// ⭐ O tamanho da ponta `h` numa seta cujo trecho junto dela mede `leg`: `min(base, leg / 2)`.
#[must_use]
pub fn size(h: Head, leg: f64) -> f64 {
    base(h).min(leg.max(0.0) / 2.0)
}

/// A ponta do catálogo que desenha `h` (as de cardinalidade não estão lá).
#[must_use]
pub fn marker(h: Head) -> Marker {
    match h {
        Head::Arrow => Marker::Open,
        Head::Triangle | Head::TriangleOpen => Marker::Triangle,
        Head::Diamond => Marker::Diamond,
        Head::DiamondOpen => Marker::DiamondOpen,
        Head::Circle => Marker::Circle,
        Head::CircleOpen => Marker::CircleOpen,
        Head::Bar => Marker::Bar,
        Head::None
        | Head::One
        | Head::Many
        | Head::OneOrMany
        | Head::ExactlyOne
        | Head::ZeroOrOne
        | Head::ZeroOrMany => Marker::None,
    }
}

/// A largura EFECTIVA do catálogo (`w · scale`) que põe a ponta `h` no tamanho `size`: a caixa do
/// catálogo mede 4 de fundo (triângulo, «V»), 8 (losango), 4 de diâmetro (círculo), 2 de meia barra.
fn catalog_unit(h: Head, size: f64) -> f64 {
    match marker(h) {
        Marker::Open => size * ARROW_DEG.to_radians().cos() / 4.0,
        Marker::Triangle => size * WIDE_DEG.to_radians().cos() / 4.0,
        Marker::Diamond | Marker::DiamondOpen => size / 8.0,
        Marker::Circle | Marker::CircleOpen => size / 4.0,
        Marker::Bar => size / 2.0,
        Marker::None => 0.0,
    }
}

/// A escala do catálogo (`Marker::build`/`inset`) para a ponta `h` no tamanho `size` sobre um traço
/// de largura `width`.
#[must_use]
pub fn catalog_scale(h: Head, size: f64, width: f64) -> f64 {
    catalog_unit(h, size) / width.max(f64::EPSILON)
}

/// O tamanho em que a ponta `h` tem a escala FIXA `scale` do catálogo sobre o traço `width` (os
/// ícones da barra, `drawn_at`).
#[must_use]
pub fn size_for_scale(h: Head, scale: f64, width: f64) -> f64 {
    let per = catalog_unit(h, 1.0);
    if per > 0.0 {
        width * scale / per
    } else {
        width * scale * 4.0
    }
}

/// Quanto a ponta `h` no tamanho `size` ocupa AO LONGO da linha a partir do bico — para onde ela aponta
/// de volta (a direcção numa curva) e a haste recta debaixo dela. Nunca zero numa ponta que existe.
#[must_use]
pub fn along(h: Head, size: f64) -> f64 {
    let wide = WIDE_DEG.to_radians();
    match h {
        Head::None => 0.0,
        Head::Arrow => size * ARROW_DEG.to_radians().cos(),
        Head::Triangle | Head::TriangleOpen | Head::One | Head::ExactlyOne | Head::OneOrMany => {
            size * wide.cos()
        }
        Head::Diamond | Head::DiamondOpen | Head::Circle | Head::CircleOpen | Head::Bar => size,
        Head::Many | Head::ZeroOrOne | Head::ZeroOrMany => size.max(crowfoot_zero_center(size)),
    }
}

/// Quanto a LINHA recua debaixo da ponta (a vazada não pode ter o traço a atravessá-la).
#[must_use]
pub fn inset(h: Head, size: f64) -> f64 {
    let m = marker(h);
    if m == Marker::None {
        return 0.0;
    }
    m.inset(1.0) * catalog_unit(h, size)
}

/// A ponta é PINTADA (senão traça-se com o traço da linha)?
#[must_use]
pub fn filled(h: Head) -> bool {
    h != Head::TriangleOpen && marker(h).is_filled()
}

/// ⭐ Os contornos da ponta `h` com o bico em `tip`, a apontar `dir` (para FORA, unitária), no tamanho
/// `size` sobre um traço `width`: `(contorno, pintado?)`.
#[must_use]
pub fn build(h: Head, tip: [f64; 2], dir: [f64; 2], width: f64, size: f64) -> Vec<(BezPath, bool)> {
    let m = marker(h);
    if m != Marker::None {
        let scale = catalog_scale(h, size, width);
        return m
            .build(tip, dir, width, scale, 0.0)
            .map(|p| {
                let closed = p.closed;
                vec![(ph2d_vec_render::build_bezpath(&p), closed && filled(h))]
            })
            .unwrap_or_default();
    }
    cardinality(h, tip, dir, size)
}

/// O centro do círculo do «zero» atrás do bico: 22,5 com o pé-de-galinha a 15 (`P_tipos`, as duas
/// pontas «zero» a L 30 e 200: o círculo não muda com a seta, raio 6). Só esse tamanho foi medido:
/// a conta escala com ele.
fn crowfoot_zero_center(size: f64) -> f64 {
    size.min(CROWFOOT) * 1.5
}

/// ⭐ As pontas de CARDINALIDADE (o desenho do Excalidraw, `P_tipos`, as duas versões): as riscas a
/// 25° de tamanho `s` (`min(20, trecho/2)`), o pé-de-galinha de tamanho `m` (`min(15, trecho/2)`).
/// - um: uma barra a `s·cos 25°` do bico, de meia altura `s·sin 25°`;
/// - exactamente um: essa e outra `s/2` mais perto do bico;
/// - muitos: o pé-de-galinha — duas riscas do vértice a `m` do bico até `m·(cos, ±sin) 25°` à frente;
/// - um ou muitos: o pé-de-galinha e a barra `s/4` mais perto;
/// - zero ou um: a barra `s/2` mais perto e o círculo; zero ou muitos: o pé-de-galinha e o círculo.
fn cardinality(h: Head, tip: [f64; 2], dir: [f64; 2], size: f64) -> Vec<(BezPath, bool)> {
    let len = dir[0].hypot(dir[1]);
    if len < 1e-12 || size <= 0.0 {
        return Vec::new();
    }
    let (ux, uy) = (dir[0] / len, dir[1] / len);
    // `(a, b)` = `a` para trás do bico, `b` para o lado.
    let at = |a: f64, b: f64| Point::new(tip[0] - ux * a - uy * b, tip[1] - uy * a + ux * b);
    // `s = min(20, trecho/2)` ⇒ `m = min(15, trecho/2) = min(15, s)`.
    let (s, m) = (size, size.min(CROWFOOT));
    let (c, sn) = (WIDE_DEG.to_radians().cos(), WIDE_DEG.to_radians().sin());
    let seg = |a: Point, b: Point| {
        let mut p = BezPath::new();
        p.move_to(a);
        p.line_to(b);
        (p, false)
    };
    let bar = |back: f64| seg(at(back, s * sn), at(back, -s * sn));
    let crow = || {
        let apex = at(m, 0.0);
        vec![
            seg(at(m - m * c, m * sn), apex),
            seg(at(m - m * c, -m * sn), apex),
        ]
    };
    let zero = || {
        let r = crowfoot_zero_center(m) * 6.0 / 22.5;
        let ctr = at(crowfoot_zero_center(m), 0.0);
        (Circle::new(ctr, r).to_path(1e-3), false)
    };
    let one = s * c;
    match h {
        Head::One => vec![bar(one)],
        Head::ExactlyOne => vec![bar(one), bar(one - s / 2.0)],
        Head::Many => crow(),
        Head::OneOrMany => {
            let mut v = crow();
            v.push(bar(one - s / 4.0));
            v
        }
        Head::ZeroOrOne => vec![zero(), bar(one - s / 2.0)],
        Head::ZeroOrMany => {
            let mut v = crow();
            v.push(zero());
            v
        }
        _ => Vec::new(),
    }
}
