//! B06.A4 — o oráculo do levante: os laços de tela inteira de antes (`begin_transform` e
//! `lift_transform_relief`), verbatim, e o gate de paridade ao bit da caixa em faixas.

use super::*;
use std::cell::Cell;

thread_local! {
    static FAIXAS: Cell<Option<usize>> = const { Cell::new(None) };
}

pub(super) fn faixas_forcadas() -> Option<usize> {
    FAIXAS.with(Cell::get)
}

type Levante = (Vec<u8>, Vec<u8>, Region);

/// O laço de `begin_transform` de antes do B06.A4, verbatim (`cov` no lugar de `selection_coverage_at`).
fn levante_serial(
    cur: &[u8],
    w: u32,
    h: u32,
    cov: Option<&dyn Fn(u32, u32) -> u8>,
) -> Option<Levante> {
    let n = (w as usize) * (h as usize);
    let restrict = cov.is_some();
    let mut patch = vec![0u8; n * 4];
    let mut base = cur.to_vec();
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let mut any = false;
    for y in 0..h {
        for x in 0..w {
            let m = if restrict {
                f32::from(cov.unwrap()(x, y)) / 255.0
            } else {
                1.0
            };
            if m <= 0.0 {
                continue;
            }
            let i = ((y * w + x) as usize) * 4;
            let a = f32::from(cur[i + 3]);
            patch[i] = cur[i];
            patch[i + 1] = cur[i + 1];
            patch[i + 2] = cur[i + 2];
            patch[i + 3] = (a * m).round() as u8;
            base[i + 3] = (a * (1.0 - m)).round() as u8;
            if patch[i + 3] > 0 {
                any = true;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if !any {
        return None;
    }
    Some((
        patch,
        base,
        Region {
            x: x0,
            y: y0,
            w: x1 - x0 + 1,
            h: y1 - y0 + 1,
        },
    ))
}

/// A partição da cobertura de `lift_transform_relief` de antes, verbatim.
fn cobertura_serial(
    covers: &[u8],
    w: u32,
    h: u32,
    mask: &dyn Fn(u32, u32) -> u8,
) -> (Vec<u8>, Vec<u8>) {
    let n = (w as usize) * (h as usize);
    let mut cp = vec![0u8; n];
    let mut cb = vec![0u8; n];
    for y in 0..h {
        for x in 0..w {
            let gi = (y * w + x) as usize;
            let mk = f32::from(mask(x, y)) / 255.0;
            let c = f32::from(covers[gi]);
            cp[gi] = (c * mk).round().clamp(0.0, 255.0) as u8;
            cb[gi] = (c * (1.0 - mk)).round().clamp(0.0, 255.0) as u8;
        }
    }
    (cp, cb)
}

fn ruido(n: usize, semente: u32) -> Vec<u8> {
    let mut s = semente.wrapping_mul(2_654_435_761).max(1);
    (0..n)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            (s >> 24) as u8
        })
        .collect()
}

/// A caixa em faixas dá o levante e a partição do corpo de antes, ao bit: camada inteira, retângulo
/// duro, seleção macia com bordas a zero e meio, seleção vazia e máscara mais curta que a tela.
#[test]
fn o_levante_na_caixa_em_faixas_e_o_de_tela_inteira_ao_bit() {
    let (w, h) = (61u32, 47u32);
    let n = (w * h) as usize;
    let mut cur = ruido(n * 4, 7);
    for i in (0..n).step_by(5) {
        cur[i * 4 + 3] = 0; // texels sem alfa: o patch não os conta na caixa
    }
    let cur = Arc::new(cur);
    let covers = Arc::new(ruido(n, 11));
    let duro: Vec<u8> = (0..n)
        .map(|i| {
            let (x, y) = ((i as u32) % w, (i as u32) / w);
            if (9..33).contains(&x) && (5..29).contains(&y) {
                255
            } else {
                0
            }
        })
        .collect();
    let mut macio = ruido(n, 3);
    for (i, v) in macio.iter_mut().enumerate() {
        let (x, y) = ((i as u32) % w, (i as u32) / w);
        if !(4..50).contains(&x) || !(13..40).contains(&y) {
            *v = 0;
        }
    }
    let curta: Vec<u8> = macio[..n - 3 * w as usize - 7].to_vec();
    let vazia = vec![0u8; n];
    let mut casos = 0;
    for faixas in [1usize, 3, 7] {
        FAIXAS.with(|c| c.set(Some(faixas)));
        assert_eq!(
            levante_serial(&cur, w, h, None),
            lift_colour(&cur, None, w, h).map(|(p, b, r)| (p.to_vec(), b.to_vec(), r)),
            "camada inteira, {faixas} faixas"
        );
        for (nome, m) in [
            ("duro", &duro),
            ("macio", &macio),
            ("curta", &curta),
            ("vazia", &vazia),
        ] {
            let cov = |x: u32, y: u32| m.get((y * w + x) as usize).copied().unwrap_or(0);
            let velho = levante_serial(&cur, w, h, Some(&cov));
            let novo = mask_bbox(m, w, h).and_then(|sb| {
                lift_colour(&cur, Some((m, sb)), w, h).map(|(p, b, r)| (p.to_vec(), b.to_vec(), r))
            });
            assert_eq!(velho, novo, "{nome}, {faixas} faixas");
            let (cp, cb) = cobertura_serial(&covers, w, h, &cov);
            if let Some(sb) = mask_bbox(m, w, h) {
                let (np, nb) = split_covers(&covers, Some((m, sb)), w);
                assert_eq!(
                    (cp, cb),
                    (np.to_vec(), nb.unwrap().to_vec()),
                    "corpo {nome}, {faixas} faixas"
                );
                casos += 1;
            } else {
                assert!(
                    cp.iter().all(|&v| v == 0) && cb == covers[..],
                    "vazia: o corpo fica na base"
                );
            }
        }
    }
    FAIXAS.with(|c| c.set(None));
    assert_eq!(casos, 9);
}

/// Sonda B06.A4 (`--profile smoke -- --ignored`): o levante 4096² com uma seleção pequena (256²),
/// cor + corpo, o laço de antes × a caixa em faixas, no MESMO processo, intercalados, o mínimo.
#[test]
#[ignore = "sonda de tempo: --profile smoke -- --ignored"]
fn sonda_levante_4096_selecao_pequena() {
    use std::time::Instant;
    let (w, h) = (4096u32, 4096u32);
    let n = (w * h) as usize;
    let cur = Arc::new(vec![200u8; n * 4]);
    let covers = Arc::new(vec![180u8; n]);
    let mut mask = vec![0u8; n];
    for y in 1900..2156u32 {
        mask[(y * w + 1900) as usize..(y * w + 2156) as usize].fill(255);
    }
    let cov = |x: u32, y: u32| mask[(y * w + x) as usize];
    let (mut velho, mut novo) = (f64::MAX, f64::MAX);
    for volta in 0..10 {
        for v in [volta % 2 == 0, volta % 2 != 0] {
            let t0 = Instant::now();
            if v {
                let r = levante_serial(&cur, w, h, Some(&cov));
                let c = cobertura_serial(&covers, w, h, &cov);
                std::hint::black_box((r, c));
                velho = velho.min(t0.elapsed().as_secs_f64() * 1e3);
            } else {
                let sb = mask_bbox(&mask, w, h).expect("seleção");
                let r = lift_colour(&cur, Some((&mask, sb)), w, h);
                let c = split_covers(&covers, Some((&mask, sb)), w);
                std::hint::black_box((r, c));
                novo = novo.min(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
    }
    eprintln!(
        "levante 4096², seleção 256²: antes {velho:.1} ms → depois {novo:.1} ms (mín. de 10)"
    );
}

/// Sonda B06.A4: `begin_transform` inteiro (entrar no Transform) a 4096² com corpo e uma seleção 256².
#[test]
#[ignore = "sonda de tempo: --profile smoke -- --ignored"]
fn sonda_begin_transform_4096_selecao_pequena() {
    use crate::tool::PainterTool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    use std::time::Instant;
    let s = 4096u32;
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (s * s * 4) as usize], s, s);
    t.paint.brush.radius_px = 14.0;
    t.paint.brush.impasto = true;
    let b = t.paint.brush;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
    t.on_canvas_pointer(cp([1950.0, 2000.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([2100.0, 2000.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([2100.0, 2000.0], PointerPhase::Up));
    t.set_paint_tool_mode("liquify");
    let layer = t.layers.active().expect("camada");
    assert!(t.heights.contains_key(&layer), "sonda: sem corpo");
    let mut ms = f64::MAX;
    for _ in 0..6 {
        t.set_rect_selection(1900, 1900, 256, 256);
        let t0 = Instant::now();
        t.set_deform_transform_on(true);
        ms = ms.min(t0.elapsed().as_secs_f64() * 1e3);
        assert!(
            t.paint
                .deform
                .xform_patch
                .as_ref()
                .is_some_and(|p| p.relief.is_some())
        );
        t.set_deform_transform_on(false);
    }
    eprintln!("begin_transform 4096², corpo, seleção 256²: {ms:.1} ms (mín. de 6)");
}
