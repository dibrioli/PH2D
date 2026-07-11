//! T1.6 — preenchimento (fill) de um traço FECHADO: triangulação por
//! **ear-clipping** + o vértice GPU do fill.
//!
//! Ear-clipping é suficiente para um polígono simples (o caso do W1: um traço
//! fechado). É transcendental-free e determinístico (HR-5 limpo). Fills COMPOSTOS
//! (várias curvas com buracos, `fill_id`) e polígonos auto-intersectantes pedem
//! CDT robusto — isso é do W4 (a `02_referencia §1` os agrupa por `fill_id`).
//!
//! O fill é rasterizado ANTES/ABAIXO do traço: a profundidade dele é
//! `(2·sid+1)·2e-7`, contra `(2·sid+2)·2e-7` do traço (o traço ganha o GREATER
//! sobre o próprio fill), sem colidir com os vizinhos.

use ph2d_core::Vec2;

/// Um vértice de fill: posição em MUNDO (o vertex shader transforma direto, sem
/// expansão de fita), profundidade da ordem 2D, cor RGBA linear **premultiplicada**
/// (a opacidade do fill já entra no alpha). `repr(C)` + Pod = 8×f32 = 32 bytes.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FillVertex {
    pub pos: [f32; 2],
    pub depth: f32,
    pub _pad: f32,
    pub color: [f32; 4],
}

/// Área com sinal do polígono (shoelace). `> 0` = CCW na convenção Y-up.
fn signed_area(p: &[Vec2]) -> f32 {
    let n = p.len();
    let mut a = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        a += p[i].x * p[j].y - p[j].x * p[i].y;
    }
    a * 0.5
}

/// `true` se `q` está dentro do triângulo (a,b,c) (bordas incluídas), via sinais
/// de produto cruzado. Assume (a,b,c) em CCW.
fn point_in_tri(q: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let cross = |u: Vec2, v: Vec2, w: Vec2| (v.x - u.x) * (w.y - u.y) - (v.y - u.y) * (w.x - u.x);
    let d1 = cross(a, b, q);
    let d2 = cross(b, c, q);
    let d3 = cross(c, a, q);
    // Todos ≥ 0 (dentro ou na borda), já que o triângulo é CCW.
    d1 >= 0.0 && d2 >= 0.0 && d3 >= 0.0
}

/// Triangula o polígono FECHADO `points` por ear-clipping. Devolve triplas de
/// índices (flat: `[i0,i1,i2, i3,i4,i5, …]`) em CCW. Vazio se `< 3` pontos ou
/// degenerado.
#[must_use]
pub fn triangulate(points: &[Vec2]) -> Vec<u32> {
    let n = points.len();
    if n < 3 {
        return Vec::new();
    }
    // Normaliza para CCW (ear-clipping abaixo assume CCW). Se CW, inverte a ordem
    // de leitura via o mapa `idx`.
    let ccw = signed_area(points) >= 0.0;
    let mut idx: Vec<usize> = if ccw {
        (0..n).collect()
    } else {
        (0..n).rev().collect()
    };

    let mut out: Vec<u32> = Vec::with_capacity((n - 2) * 3);
    // Teto de segurança contra polígono degenerado (evita loop infinito).
    let mut guard = n * n + 8;
    while idx.len() > 3 && guard > 0 {
        guard -= 1;
        let m = idx.len();
        let mut clipped = false;
        for k in 0..m {
            let iprev = idx[(k + m - 1) % m];
            let icur = idx[k];
            let inext = idx[(k + 1) % m];
            let a = points[iprev];
            let b = points[icur];
            let c = points[inext];
            // Convexo? (produto cruzado ≥ 0 em CCW).
            let convex = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) >= 0.0;
            if !convex {
                continue;
            }
            // Nenhum outro vértice dentro do triângulo (a,b,c)?
            let mut empty = true;
            for &oi in &idx {
                if oi == iprev || oi == icur || oi == inext {
                    continue;
                }
                if point_in_tri(points[oi], a, b, c) {
                    empty = false;
                    break;
                }
            }
            if !empty {
                continue;
            }
            // Orelha: emite o triângulo e remove o vértice.
            out.push(iprev as u32);
            out.push(icur as u32);
            out.push(inext as u32);
            idx.remove(k);
            clipped = true;
            break;
        }
        if !clipped {
            break; // sem orelha achável (degenerado) — para com o que tem
        }
    }
    if idx.len() == 3 {
        out.push(idx[0] as u32);
        out.push(idx[1] as u32);
        out.push(idx[2] as u32);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulates_a_square_into_two_triangles() {
        let sq = [
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        let t = triangulate(&sq);
        assert_eq!(t.len(), 6, "2 triângulos = 6 índices");
        // Cada índice é válido.
        assert!(t.iter().all(|&i| (i as usize) < 4));
    }

    #[test]
    fn cw_polygon_is_normalized() {
        // Mesmo quadrado, ordem HORÁRIA (CW).
        let sq = [
            Vec2::new(0.0, 2.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(0.0, 0.0),
        ];
        let t = triangulate(&sq);
        assert_eq!(t.len(), 6);
    }

    #[test]
    fn concave_l_shape_triangulates() {
        // Um "L" côncavo (6 vértices → 4 triângulos).
        let l = [
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 4.0),
            Vec2::new(0.0, 4.0),
        ];
        let t = triangulate(&l);
        assert_eq!(t.len(), 12, "6-gon → 4 triângulos");
    }

    #[test]
    fn degenerate_returns_empty() {
        assert!(triangulate(&[]).is_empty());
        assert!(triangulate(&[Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)]).is_empty());
    }

    #[test]
    fn fill_vertex_is_32_bytes() {
        assert_eq!(std::mem::size_of::<FillVertex>(), 32);
    }
}
