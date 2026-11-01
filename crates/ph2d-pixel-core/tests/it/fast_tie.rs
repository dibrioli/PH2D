//! **#10 (P6g) — o empate da *Fast Rotation* no ramo da HOMOGRAFIA**: quando os cantos assentados
//! não fazem um paralelogramo, cada célula copia a da fonte sob o seu centro e o empate exacto
//! resolve-se como se o centro estivesse um nada à ESQUERDA (a lei do ramo inteiro, medida no
//! Trial). A referência daqui avalia a homografia do quadrilátero num ponto deslocado `1e-6` para a
//! esquerda, sem viés nenhum: nas células de empate o sinal do viés do motor decide.

use ph2d_pixel_core::affine::{Affine, Resample, transform_image};
use ph2d_pixel_core::transform::Image;

/// O quadrado `[0,w]×[0,h]` → o quadrilátero `q` (↖ ↗ ↘ ↙), Heckbert; devolve a INVERSA (3×3).
fn inversa(w: f64, h: f64, q: [(f64, f64); 4]) -> [f64; 9] {
    let [p0, p1, p2, p3] = q;
    let (dx1, dx2, dx3) = (p1.0 - p2.0, p3.0 - p2.0, p0.0 - p1.0 + p2.0 - p3.0);
    let (dy1, dy2, dy3) = (p1.1 - p2.1, p3.1 - p2.1, p0.1 - p1.1 + p2.1 - p3.1);
    let den = dx1 * dy2 - dx2 * dy1;
    let g = (dx3 * dy2 - dx2 * dy3) / den;
    let k = (dx1 * dy3 - dx3 * dy1) / den;
    let m = [
        (p1.0 - p0.0 + g * p1.0) / w,
        (p3.0 - p0.0 + k * p3.0) / h,
        p0.0,
        (p1.1 - p0.1 + g * p1.1) / w,
        (p3.1 - p0.1 + k * p3.1) / h,
        p0.1,
        g / w,
        k / h,
        1.0,
    ];
    let [a, b, c, d, e, f, g, h, i] = m;
    let (aa, bb, cc) = (e * i - f * h, f * g - d * i, d * h - e * g);
    let det = a * aa + b * bb + c * cc;
    [
        aa / det,
        (c * h - b * i) / det,
        (b * f - c * e) / det,
        bb / det,
        (a * i - c * g) / det,
        (c * d - a * f) / det,
        cc / det,
        (b * g - a * h) / det,
        (a * e - b * d) / det,
    ]
}

fn fonte(m: &[f64; 9], ox: f64, oy: f64) -> (f64, f64) {
    let z = m[6] * ox + m[7] * oy + m[8];
    (
        (m[0] * ox + m[1] * oy + m[2]) / z,
        (m[3] * ox + m[4] * oy + m[5]) / z,
    )
}

#[test]
fn a_fast_homography_tie_resolves_a_hair_to_the_left() {
    let (mut empates, mut casos) = (0usize, 0usize);
    for (w, h) in [(4u32, 4u32), (5, 3), (6, 5), (3, 6)] {
        let src = Image::map(w, h, 4, |x, y| 1 + y * w + x);
        for num in 1..16 {
            for (sx, sy) in [(1.0, 1.0), (1.5, 1.0), (2.0, 1.25)] {
                let shx = f64::from(num) / 8.0;
                let t = Affine {
                    a: sx,
                    b: 0.0,
                    c: shx,
                    d: sy,
                    e: 0.25,
                    f: 0.0,
                };
                let q = t
                    .quad(f64::from(w), f64::from(h))
                    .map(|p| (p.x.floor(), p.y.floor()));
                let paralelo =
                    q[1].0 - q[0].0 == q[2].0 - q[3].0 && q[1].1 - q[0].1 == q[2].1 - q[3].1;
                if paralelo {
                    continue;
                }
                let Some((r, out)) = transform_image(&src, &t, Resample::Fast, 0, None) else {
                    continue;
                };
                casos += 1;
                let m = inversa(f64::from(w), f64::from(h), q);
                for y in 0..out.h {
                    for x in 0..out.w {
                        let (ox, oy) = (
                            f64::from(r.x) + f64::from(x) + 0.5,
                            f64::from(r.y) + f64::from(y) + 0.5,
                        );
                        let (u0, v0) = fonte(&m, ox, oy);
                        let perto = |s: f64| (s - s.round()).abs() < 1e-9;
                        empates += usize::from(perto(u0) || perto(v0));
                        let (u, v) = fonte(&m, ox - 1e-6, oy);
                        let (u, v) = (u.floor(), v.floor());
                        let quer = if u >= 0.0 && v >= 0.0 && u < f64::from(w) && v < f64::from(h) {
                            src.get(u as u32, v as u32)
                        } else {
                            0
                        };
                        assert_eq!(
                            out.get(x, y),
                            quer,
                            "{w}×{h} shx {shx} esc ({sx}, {sy}) célula ({x}, {y})"
                        );
                    }
                }
            }
        }
    }
    assert!(
        casos >= 10,
        "controlo: {casos} quadriláteros no ramo da homografia"
    );
    assert!(
        empates > 0,
        "controlo: nenhum empate exacto — o teste não decide o sinal"
    );
}
