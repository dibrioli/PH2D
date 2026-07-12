//! **Balões e símbolos** — módulo irmão de [`crate::shapes`].
//!
//! Balões de fala/pensamento (quadrinhos e animação) e o conjunto de símbolos que todo
//! app de diagrama traz (coração, nuvem, raio, explosão, escudo, engrenagem, cruz…).
//!
//! Tudo cabe na caixa do gesto e usa **frações** dela nos parâmetros. Onde a silhueta
//! pede curva (coração, nuvem, gota) os vértices saem com handles bézier; onde ela é
//! angular (explosão, cruz, raio) saem quinas.

use crate::{Contour, VecPath, VecVertex, VertexKind};

fn corners(pts: Vec<[f64; 2]>) -> VecPath {
    VecPath {
        verts: pts.into_iter().map(VecVertex::corner).collect(),
        closed: true,
        ..VecPath::default()
    }
}

fn smooth_path(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

fn v(anchor: [f64; 2], i: [f64; 2], o: [f64; 2]) -> VecVertex {
    VecVertex {
        anchor,
        in_handle: i,
        out_handle: o,
        kind: VertexKind::Corner,
    }
}

fn box_of(a: [f64; 2], b: [f64; 2]) -> (f64, f64, f64, f64) {
    (
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (b[0] - a[0]).abs() * 0.5,
        (b[1] - a[1]).abs() * 0.5,
    )
}

/// **Coração**: dois lóbulos e um bico. Silhueta clássica, em quatro cúbicas.
#[must_use]
pub fn heart(a: [f64; 2], b: [f64; 2], cleft: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let cl = cleft.clamp(0.0, 0.5); // quanto o vale do meio afunda
    let top = cy - hh + 2.0 * hh * (0.18 + cl * 0.3);
    smooth_path(vec![
        // bico de baixo
        v(
            [cx, cy + hh],
            [cx + hw * 0.55, cy + hh * 0.35],
            [cx - hw * 0.55, cy + hh * 0.35],
        ),
        // lóbulo esquerdo
        v(
            [cx - hw, cy - hh * 0.35],
            [cx - hw, cy + hh * 0.15],
            [cx - hw, cy - hh * 0.85],
        ),
        // vale do meio
        v(
            [cx, top],
            [cx - hw * 0.5, cy - hh],
            [cx + hw * 0.5, cy - hh],
        ),
        // lóbulo direito
        v(
            [cx + hw, cy - hh * 0.35],
            [cx + hw, cy - hh * 0.85],
            [cx + hw, cy + hh * 0.15],
        ),
    ])
}

/// **Raio** (bolt): o zigue-zague. `waist` = a estreiteza da cintura.
#[must_use]
pub fn bolt(a: [f64; 2], b: [f64; 2], waist: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let w = waist.clamp(0.05, 0.8);
    corners(vec![
        [cx + hw * 0.35, cy - hh],
        [cx - hw, cy + hh * 0.12],
        [cx - hw * w * 0.2, cy + hh * 0.12],
        [cx - hw * 0.35, cy + hh],
        [cx + hw, cy - hh * 0.12],
        [cx + hw * w * 0.2, cy - hh * 0.12],
    ])
}

/// **Lua crescente**: dois arcos — o de fora (meia elipse) e o de dentro, cuja curvatura
/// o `phase` controla. `0.5` = meia-lua reta; abaixo, crescente gordo; acima, fino.
///
/// **Não** é "uma elipse menos outra deslocada": aquilo põe a sombra FORA da caixa do
/// gesto, e a bbox (que é o que o gizmo desenha) passa a mentir sobre onde a forma está.
/// Dois arcos cabem por construção — e o path sai limpo, com quatro âncoras editáveis.
#[must_use]
pub fn moon(a: [f64; 2], b: [f64; 2], phase: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let p = phase.clamp(0.05, 0.95);
    // A "cintura" do arco interno, com SINAL: >0 curva para fora (crescente fino), <0
    // para dentro (gordo), 0 = corte reto.
    let waist = hw * (1.0 - 2.0 * p);
    let mut verts = crate::round::arc_verts([cx, cy], hw, hh, 90.0, 180.0, false);
    let inner = crate::round::arc_verts([cx, cy], waist, hh, 270.0, 180.0, false);
    verts.extend(inner);
    smooth_path(verts)
}

/// **Gota** (drop): círculo embaixo, bico em cima.
#[must_use]
pub fn drop(a: [f64; 2], b: [f64; 2], tip: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let t = tip.clamp(0.1, 0.9);
    smooth_path(vec![
        v(
            [cx, cy - hh],
            [cx, cy - hh],
            [cx + hw * t * 0.6, cy - hh * 0.2],
        ),
        v(
            [cx + hw, cy + hh * 0.35],
            [cx + hw, cy - hh * 0.2],
            [cx + hw, cy + hh],
        ),
        v(
            [cx, cy + hh],
            [cx + hw * 0.6, cy + hh],
            [cx - hw * 0.6, cy + hh],
        ),
        v(
            [cx - hw, cy + hh * 0.35],
            [cx - hw, cy + hh],
            [cx - hw, cy - hh * 0.2],
        ),
    ])
}

/// **Escudo**: topo reto, base em ponta.
#[must_use]
pub fn shield(a: [f64; 2], b: [f64; 2], curve: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let c = curve.clamp(0.0, 1.0);
    smooth_path(vec![
        v([cx - hw, cy - hh], [cx - hw, cy - hh], [cx - hw, cy - hh]),
        v([cx + hw, cy - hh], [cx + hw, cy - hh], [cx + hw, cy - hh]),
        v(
            [cx + hw, cy - hh * 0.1],
            [cx + hw, cy - hh * 0.1],
            [cx + hw, cy + hh * 0.45 * c],
        ),
        v(
            [cx, cy + hh],
            [cx + hw * 0.6, cy + hh],
            [cx - hw * 0.6, cy + hh],
        ),
        v(
            [cx - hw, cy - hh * 0.1],
            [cx - hw, cy + hh * 0.45 * c],
            [cx - hw, cy - hh * 0.1],
        ),
    ])
}

/// **Etiqueta / tag**: retângulo com um bico e o furo do cordão.
#[must_use]
pub fn tag(a: [f64; 2], b: [f64; 2], point: f64, hole: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let p = 2.0 * hw * point.clamp(0.05, 0.6);
    let mut path = corners(vec![
        [cx - hw, cy - hh],
        [cx + hw - p, cy - hh],
        [cx + hw, cy],
        [cx + hw - p, cy + hh],
        [cx - hw, cy + hh],
    ]);
    let hr = hh * hole.clamp(0.0, 0.4);
    if hr > 1e-6 {
        let h = crate::ellipse([cx - hw + p * 0.6, cy], hr, hr);
        path.subpaths.push(Contour {
            verts: h.verts,
            closed: true,
        });
        path.fill_rule = crate::FillRule::EvenOdd;
    }
    path
}

/// **Engrenagem**: dentes retangulares em volta + furo central.
#[must_use]
pub fn gear(a: [f64; 2], b: [f64; 2], teeth: f64, depth: f64, hole: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let n = teeth.clamp(4.0, 30.0).round() as usize;
    let d = depth.clamp(0.05, 0.5);
    let step = std::f64::consts::TAU / n as f64;
    let mut pts = Vec::with_capacity(n * 4);
    for i in 0..n {
        let a0 = step * i as f64;
        // Cada dente: dois pontos no raio cheio, dois no raio recuado (a base).
        for (frac, r) in [(0.10, 1.0 - d), (0.22, 1.0), (0.55, 1.0), (0.67, 1.0 - d)] {
            let ang = a0 + step * frac;
            let (s, c) = ang.sin_cos();
            pts.push([cx + hw * r * c, cy + hh * r * s]);
        }
    }
    let mut path = corners(pts);
    let hr = hole.clamp(0.0, 0.8);
    if hr > 1e-6 {
        let h = crate::ellipse([cx, cy], hw * hr, hh * hr);
        path.subpaths.push(Contour {
            verts: h.verts,
            closed: true,
        });
        path.fill_rule = crate::FillRule::EvenOdd;
    }
    path
}

/// **Cruz / plus**: `arm` = a espessura dos braços (fração da caixa).
#[must_use]
pub fn cross(a: [f64; 2], b: [f64; 2], arm: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let ax = hw * arm.clamp(0.05, 0.95);
    let ay = hh * arm.clamp(0.05, 0.95);
    corners(vec![
        [cx - ax, cy - hh],
        [cx + ax, cy - hh],
        [cx + ax, cy - ay],
        [cx + hw, cy - ay],
        [cx + hw, cy + ay],
        [cx + ax, cy + ay],
        [cx + ax, cy + hh],
        [cx - ax, cy + hh],
        [cx - ax, cy + ay],
        [cx - hw, cy + ay],
        [cx - hw, cy - ay],
        [cx - ax, cy - ay],
    ])
}

/// **Check** (o "certo"): um V grosso.
#[must_use]
pub fn check(a: [f64; 2], b: [f64; 2], weight: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let w = weight.clamp(0.05, 0.6);
    let t = hh * w;
    corners(vec![
        [cx - hw, cy],
        [cx - hw + hw * 0.35, cy - t * 0.6],
        [cx - hw * 0.25, cy + hh - t * 1.4],
        [cx + hw - hw * 0.2, cy - hh],
        [cx + hw, cy - hh + t],
        [cx - hw * 0.25, cy + hh],
    ])
}

/// **Fita / banner**: retângulo com as pontas em rabo-de-peixe.
#[must_use]
pub fn banner(a: [f64; 2], b: [f64; 2], notch: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let n = 2.0 * hw * notch.clamp(0.02, 0.4);
    corners(vec![
        [cx - hw, cy - hh],
        [cx + hw, cy - hh],
        [cx + hw - n, cy],
        [cx + hw, cy + hh],
        [cx - hw, cy + hh],
        [cx - hw + n, cy],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: [f64; 2] = [-2.0, -1.0];
    const B: [f64; 2] = [2.0, 1.0];

    /// Todos cabem na caixa do gesto (senão a bbox mente e o gizmo desalinha).
    #[test]
    fn every_symbol_fits_inside_the_gesture_box() {
        let all: [(&str, VecPath); 10] = [
            ("heart", heart(A, B, 0.2)),
            ("bolt", bolt(A, B, 0.4)),
            ("moon", moon(A, B, 0.45)),
            ("drop", drop(A, B, 0.5)),
            ("shield", shield(A, B, 0.6)),
            ("tag", tag(A, B, 0.25, 0.18)),
            ("gear", gear(A, B, 8.0, 0.2, 0.35)),
            ("cross", cross(A, B, 0.35)),
            ("check", check(A, B, 0.25)),
            ("banner", banner(A, B, 0.15)),
        ];
        for (name, p) in all {
            assert!(!p.verts.is_empty(), "{name}: vazio");
            for vx in p.verts_all() {
                assert!(
                    vx.anchor[0] >= -2.0 - 1e-6
                        && vx.anchor[0] <= 2.0 + 1e-6
                        && vx.anchor[1] >= -1.0 - 1e-6
                        && vx.anchor[1] <= 1.0 + 1e-6,
                    "{name}: ancora {:?} fora da caixa",
                    vx.anchor
                );
            }
        }
    }

    /// A engrenagem fura de verdade: o furo é um contorno próprio + `EvenOdd` (sem isso
    /// ela seria um disco dentado). E sem furo pedido, NÃO vira compound (nada de
    /// contorno degenerado no documento).
    #[test]
    fn the_gear_actually_has_a_hole() {
        let g = gear(A, B, 8.0, 0.2, 0.4);
        assert!(g.is_compound() && g.fill_rule == crate::FillRule::EvenOdd);
        assert!(!gear(A, B, 8.0, 0.2, 0.0).is_compound());
    }

    /// A lua é UM contorno (dois arcos), não uma elipse furada: o `phase` inverte a
    /// curvatura da cintura — abaixo de 0.5 o crescente é gordo, acima é fino. E ela
    /// cabe na caixa, que é o que o desenho "elipse menos elipse deslocada" não fazia.
    #[test]
    fn the_moon_is_two_arcs_and_the_phase_bends_the_waist() {
        let m = moon(A, B, 0.5);
        assert!(!m.is_compound(), "a lua e um contorno so");
        // Meia-lua (0.5): a cintura é reta — as âncoras do arco interno ficam em x = 0.
        let waist_x: Vec<f64> = m.verts[m.verts.len() / 2..]
            .iter()
            .map(|v| v.anchor[0])
            .collect();
        assert!(
            waist_x.iter().all(|x| x.abs() < 1e-9),
            "em 0.5 a cintura e reta"
        );
        // Gordo (0.2) e fino (0.8) curvam para lados opostos.
        let fat = moon(A, B, 0.2);
        let thin = moon(A, B, 0.8);
        let mid = |p: &VecPath| p.verts[p.verts.len() - 2].anchor[0];
        assert!(mid(&fat) > 0.0, "crescente gordo: cintura para a direita");
        assert!(mid(&thin) < 0.0, "crescente fino: cintura para a esquerda");
    }

    /// A cruz tem 12 quinas (o contorno em degrau) — é o que a separa de um retângulo.
    #[test]
    fn the_cross_has_twelve_corners() {
        assert_eq!(cross(A, B, 0.3).verts.len(), 12);
    }
}
