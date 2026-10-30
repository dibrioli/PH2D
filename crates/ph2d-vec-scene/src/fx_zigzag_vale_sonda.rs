//! **Sonda F40 — o Zig Zag num vale côncavo, três leis lado a lado** (F33 §6.3, decisão do dono).
//!
//! `A` = o produto (`zigzag_contour`). `B` = o mesmo laço com as cristas que cruzam o contorno
//! baixadas até deixarem de cruzar (a medida pelo raio local NÃO chega: numa quina viva a crista
//! encostada cruza o outro lado com qualquer altura — medido, 1 cruzamento a `0,8 R`). `C` = o
//! contorno de `A` com os nós cortados (o polígono perde o laço, a crista fica). `B` e `C` só vivem
//! aqui: escolher uma é do dono. Conta os segmentos que se cruzam em cada uma.
//!
//! ```text
//! PH2D_ZIGZAG_SONDA=<pasta> bash scripts/cargo-test-narrow.sh ph2d-vec-scene sonda_zigzag_vale -- --ignored
//! ```

#[path = "../tests/look/mod.rs"]
mod look;

use super::*;
use crate::VecPath;

const ESCALA: f64 = 5.0;
const MARGEM: f64 = 60.0;
const FUNDO: [u8; 3] = [24, 24, 28];
const PAPEL: [u8; 3] = [96, 132, 214];
const FIO: [u8; 3] = [235, 235, 240];
const NO: [u8; 3] = [255, 80, 80];

fn nuvem() -> VecPath {
    let k = crate::ShapeKind::Cloud;
    crate::cook(k, [0.0, 0.0], [120.0, 80.0], &k.defaults())
}

/// `A` com o trecho que CRUZA o contorno baixado a metade (em degradê), outra vez, até nada cruzar — a
/// cópia do laço do produto, com a altura de cada crista guardada à parte.
fn zigzag_b(verts: &[VecVertex], closed: bool, spec: &ZigZagSpec, ref_size: f64) -> Vec<VecVertex> {
    let amplitude = spec.amplitude / 100.0 * ref_size;
    let ap = ArcPath::from_contour(verts, closed).unwrap();
    let total = ap.total();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ridges = spec.ridges.floor().max(MIN_RIDGES) as usize;
    let count = (ridges * 2).min(MAX_SAMPLES);
    #[allow(clippy::cast_precision_loss)]
    let step = total / count as f64;
    let grid_n = if closed { count } else { count + 1 };
    #[allow(clippy::cast_precision_loss)]
    let grade = (0..grid_n).map(|k| (k as f64 * step).min(total));
    let at = fx_resample::amostras(&ap, grade, total * MERGE_FRACTION, MAX_SAMPLES);
    let base: Vec<_> = at.iter().map(|a| fx_resample::lados(&ap, a)).collect();
    let mut lift: Vec<f64> = at
        .iter()
        .map(|a| ridge_wave(a.s, total, ridges) * amplitude)
        .collect();
    let monta = |lift: &[f64]| -> Vec<VecVertex> {
        base.iter()
            .enumerate()
            .map(|(k, &(point, t_chega, tangent))| {
                let normal = [-tangent[1], tangent[0]];
                let anchor = [
                    normal[0].mul_add(lift[k], point[0]),
                    normal[1].mul_add(lift[k], point[1]),
                ];
                if spec.smooth {
                    fx_resample::vertice(
                        anchor,
                        t_chega,
                        tangent,
                        fx_resample::vaos(&at, k, closed, total),
                    )
                } else {
                    VecVertex::corner(anchor)
                }
            })
            .collect()
    };
    for _ in 0..40 {
        let v = monta(&lift);
        let Some((a, b)) = primeiro_cruzamento(&achata(&v)) else {
            return v;
        };
        let n = lift.len();
        let (p, q) = (a / PASSOS, b / PASSOS + 1);
        // Em degradê: o trecho que cruza a metade, e duas cristas de cada lado a 0,7 e 0,85.
        for k in p..=q {
            lift[k % n] *= 0.5;
        }
        for (d, f) in [(1, 0.7), (2, 0.85)] {
            lift[(p + n - d) % n] *= f;
            lift[(q + d) % n] *= f;
        }
    }
    monta(&lift)
}

const PASSOS: usize = 16;

fn primeiro_cruzamento(poly: &[[f64; 2]]) -> Option<(usize, usize)> {
    let n = poly.len();
    for i in 0..n {
        for j in i + 2..n {
            if (i == 0 && j == n - 1)
                || corta(poly[i], poly[(i + 1) % n], poly[j], poly[(j + 1) % n]).is_none()
            {
                continue;
            }
            return Some((i, j));
        }
    }
    None
}

fn achata(v: &[VecVertex]) -> Vec<[f64; 2]> {
    let mut p = Vec::new();
    for i in 0..v.len() {
        let (a, b) = (&v[i], &v[(i + 1) % v.len()]);
        for k in 0..PASSOS {
            #[allow(clippy::cast_precision_loss)]
            let t = k as f64 / PASSOS as f64;
            let u = 1.0 - t;
            let c = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            let q = [a.anchor, a.out_handle, b.in_handle, b.anchor];
            p.push([0, 1].map(|e| (0..4).map(|j| c[j] * q[j][e]).sum()));
        }
    }
    p
}

fn corta(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> Option<[f64; 2]> {
    let r = [b[0] - a[0], b[1] - a[1]];
    let s = [d[0] - c[0], d[1] - c[1]];
    let den = r[0] * s[1] - r[1] * s[0];
    if den.abs() < 1e-12 {
        return None;
    }
    let q = [c[0] - a[0], c[1] - a[1]];
    let t = (q[0] * s[1] - q[1] * s[0]) / den;
    let u = (q[0] * r[1] - q[1] * r[0]) / den;
    ((1e-9..1.0 - 1e-9).contains(&t) && (1e-9..1.0 - 1e-9).contains(&u))
        .then(|| [a[0] + t * r[0], a[1] + t * r[1]])
}

/// Os cruzamentos do polígono fechado (pares de segmentos não vizinhos).
fn cruzamentos(p: &[Vec<[f64; 2]>]) -> Vec<[f64; 2]> {
    let mut x = Vec::new();
    for poly in p {
        let n = poly.len();
        for i in 0..n {
            for j in i + 2..n {
                if i == 0 && j == n - 1 {
                    continue;
                }
                if let Some(q) = corta(poly[i], poly[(i + 1) % n], poly[j], poly[(j + 1) % n]) {
                    x.push(q);
                }
            }
        }
    }
    x
}

/// A lei C: corta cada nó PEQUENO (até um quarto do contorno) pelo ponto de cruzamento.
fn sem_nos(mut p: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    'de_novo: loop {
        let n = p.len();
        for i in 0..n {
            for j in i + 2..(i + n / 4).min(n) {
                if let Some(q) = corta(p[i], p[(i + 1) % n], p[j], p[(j + 1) % n]) {
                    let mut novo = p[..=i].to_vec();
                    novo.push(q);
                    novo.extend_from_slice(&p[j + 1..]);
                    p = novo;
                    continue 'de_novo;
                }
            }
        }
        return p;
    }
}

fn pinta(nome: &str, polys: &[Vec<[f64; 2]>], pasta: &std::path::Path) -> usize {
    let w = (120.0 * ESCALA + 2.0 * MARGEM) as usize;
    let h = (80.0 * ESCALA + 2.0 * MARGEM) as usize;
    let mut c = look::Canvas::new(w, h, FUNDO);
    let tela: Vec<Vec<[f64; 2]>> = polys
        .iter()
        .map(|p| {
            p.iter()
                .map(|q| [q[0] * ESCALA + MARGEM, q[1] * ESCALA + MARGEM])
                .collect()
        })
        .collect();
    look::fill(&mut c, &tela, PAPEL, false);
    look::stroke(&mut c, &tela, FIO);
    let x = cruzamentos(&tela);
    for q in &x {
        look::mark(&mut c, *q, NO);
    }
    look::write_png(&pasta.join(format!("{nome}.png")), &c).unwrap();
    x.len()
}

#[test]
#[ignore = "sonda: grava PNG"]
fn sonda_zigzag_vale() {
    let pasta = std::path::PathBuf::from(std::env::var("PH2D_ZIGZAG_SONDA").unwrap());
    std::fs::create_dir_all(&pasta).unwrap();
    let forma = nuvem();
    let r = crate::effect::FxCtx::of(&forma).ref_size;
    for (rotulo, size, smooth) in [("medido", 5.0, true), ("forte", 12.0, false)] {
        let spec = ZigZagSpec {
            amplitude: size,
            ridges: 10.0,
            smooth,
            ..ZigZagSpec::default()
        };
        let (a, _) = zigzag_contour(&forma.verts, true, &spec, r, None);
        let pa = vec![achata(&a)];
        let pb = vec![achata(&zigzag_b(&forma.verts, true, &spec, r))];
        let pc = vec![sem_nos(achata(&a))];
        let n = [
            pinta(&format!("A_{rotulo}"), &pa, &pasta),
            pinta(&format!("B_{rotulo}"), &pb, &pasta),
            pinta(&format!("C_{rotulo}"), &pc, &pasta),
        ];
        eprintln!(
            "{rotulo} (Size {size}, smooth {smooth}): cruzamentos A {} · B {} · C {}",
            n[0], n[1], n[2]
        );
    }
}
