//! B40.A3 — o caminho de recomposição da seleção ANTES da cura, verbatim, como oráculo; o gate de
//! identidade ao byte e a sonda 4096² (arrasto de gizmo e tique do slider de Offset).

use super::super::PainterTool;
use super::super::selection_offset::{ordered, paint_band_masks};
use super::{
    SelectionEntry, SelectionShape, combine_into, sel_polygon_vertices, shape_coverage_eq,
};
use ph2d_editor_core::tool::RasterEditTool as _;
use std::sync::Arc;

impl PainterTool {
    fn recompose_selection_mask_antigo(&mut self) {
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize);
        if n == 0 {
            return;
        }
        if self.paint.selection_ring_stack {
            self.recompose_ring_stack_antigo(n);
            return;
        }
        let entries = self.paint.selection_shapes.clone();
        let old_cache = std::mem::take(&mut self.paint.selection_raster_cache);
        let mut crisp = vec![0u8; n];
        let mut new_cache: Vec<(SelectionShape, Arc<Vec<u8>>)> = Vec::with_capacity(entries.len());
        for (i, e) in entries.iter().enumerate() {
            let raster = match old_cache.get(i) {
                Some((s, r)) if r.len() == n && shape_coverage_eq(s, &e.shape) => Arc::clone(r),
                _ => Arc::new(self.rasterize_selection_shape_antigo(&e.shape)),
            };
            if raster.len() == n {
                combine_into(&mut crisp, &raster, e.op);
            }
            new_cache.push((e.shape.clone(), raster));
        }
        self.paint.selection_raster_cache = new_cache;
        self.set_selection_offset_source_antigo(&crisp);
        self.apply_selection_offset_antigo();
    }

    fn recompose_ring_stack_antigo(&mut self, n_pixels: usize) {
        let entries = self.paint.selection_shapes.clone();
        let n_curves = entries.len();
        let mut count = vec![0u16; n_pixels];
        for e in &entries {
            let region = self.rasterize_selection_shape_antigo(&e.shape);
            if region.len() != n_pixels {
                continue;
            }
            for (c, &r) in count.iter_mut().zip(region.iter()) {
                if r >= 128 {
                    *c += 1;
                }
            }
        }
        let mut crisp = vec![0u8; n_pixels];
        for (out, &c) in crisp.iter_mut().zip(count.iter()) {
            let c = c as usize;
            if c > 0 && (n_curves - c).is_multiple_of(2) {
                *out = 255;
            }
        }
        self.set_selection_offset_source_antigo(&crisp);
        self.apply_selection_offset_antigo();
    }

    fn rasterize_selection_shape_antigo(&self, shape: &SelectionShape) -> Vec<u8> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let mut cov = vec![0u8; w * h];
        if w == 0 || h == 0 {
            return cov;
        }
        match shape {
            SelectionShape::Ellipse { center, u, rx, ry } => {
                rasterize_ellipse_antigo(*center, *u, *rx, *ry, w, h, &mut cov);
                cov
            }
            SelectionShape::Polygon {
                center,
                u,
                rx,
                ry,
                sides,
            } => self.raster_lasso_antigo(&sel_polygon_vertices(*center, *u, *rx, *ry, *sides)),
            SelectionShape::Freehand { model, .. } => {
                let spine = self.freehand_spine(&model.points, &model.handles);
                if spine.len() < 3 {
                    return cov;
                }
                self.raster_lasso_antigo(&spine)
            }
            SelectionShape::Raster { crisp } => {
                if crisp.len() == w * h {
                    (**crisp).clone()
                } else {
                    cov
                }
            }
        }
    }

    fn raster_lasso_antigo(&self, pts: &[[f32; 2]]) -> Vec<u8> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let mut cov = vec![0u8; w * h];
        if w == 0 || h == 0 || pts.len() < 3 {
            return cov;
        }
        for yy in 0..h {
            let yc = yy as f32 + 0.5;
            let mut xs: Vec<f32> = Vec::new();
            for i in 0..pts.len() {
                let p = pts[i];
                let q = pts[(i + 1) % pts.len()];
                let (py, qy) = (p[1], q[1]);
                if (py <= yc && yc < qy) || (qy <= yc && yc < py) {
                    let t = (yc - py) / (qy - py);
                    xs.push(p[0] + t * (q[0] - p[0]));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mut i = 0;
            while i + 1 < xs.len() {
                let xa = xs[i].max(0.0).round() as usize;
                let xb = (xs[i + 1].min(w as f32).round() as usize).min(w);
                for xx in xa..xb {
                    cov[yy * w + xx] = 255;
                }
                i += 2;
            }
        }
        cov
    }

    fn set_selection_offset_source_antigo(&mut self, crisp: &[u8]) {
        self.paint.selection_offset_source = Arc::new(crisp.to_vec());
        self.paint.selection_offset_curves.clear();
        self.paint.selection_offset_level_cache.clear();
    }

    fn apply_selection_offset_antigo(&mut self) {
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize);
        if n == 0 || self.paint.selection_offset_source.len() != n {
            return;
        }
        let live = self.selection_slider_offset_px();
        if !self.paint.selection_offset_active && live == 0.0 {
            let src = (*self.paint.selection_offset_source).clone();
            self.set_selection_from_crisp(src);
            return;
        }
        self.ensure_selection_offset_curves();
        let mut eff: Vec<u8>;
        if !self.paint.selection_offset_active {
            eff = (*self.offset_level_mask_cached(live)).clone();
        } else {
            let rings = self.paint.selection_offset_rings.clone();
            let mut levels = vec![0.0f32];
            levels.extend_from_slice(&rings);
            eff = (*self.offset_level_mask_cached(levels[0])).clone();
            for k in 1..levels.len() {
                let paint = k.is_multiple_of(2);
                let (lo, hi) = ordered(levels[k - 1], levels[k]);
                let m_lo = self.offset_level_mask_cached(lo);
                let m_hi = self.offset_level_mask_cached(hi);
                paint_band_masks(&mut eff, &m_lo, &m_hi, paint);
            }
            let last = levels[levels.len() - 1];
            let live_idx = rings.len() + 1;
            let (lo, hi) = ordered(last, last + live);
            let m_lo = self.offset_level_mask_cached(lo);
            let m_hi = self.offset_level_mask_cached(hi);
            paint_band_masks(&mut eff, &m_lo, &m_hi, live_idx.is_multiple_of(2));
        }
        self.set_selection_from_crisp(eff);
    }
}

fn rasterize_ellipse_antigo(
    center: [f32; 2],
    u: [f32; 2],
    rx: f32,
    ry: f32,
    w: usize,
    h: usize,
    cov: &mut [u8],
) {
    let rx = rx.max(0.5);
    let ry = ry.max(0.5);
    let v = [-u[1], u[0]];
    for yy in 0..h {
        for xx in 0..w {
            let px = xx as f32 + 0.5 - center[0];
            let py = yy as f32 + 0.5 - center[1];
            let du = (px * u[0] + py * u[1]) / rx;
            let dv = (px * v[0] + py * v[1]) / ry;
            if du * du + dv * dv <= 1.0 {
                cov[yy * w + xx] = 255;
            }
        }
    }
}

type Estado = (Vec<u8>, Vec<u8>, Vec<u8>, bool);

fn estado(t: &PainterTool) -> Estado {
    (
        t.paint.selection_crisp.to_vec(),
        t.paint.selection_mask.to_vec(),
        t.paint.selection_offset_source.to_vec(),
        t.paint.selection_active,
    )
}

/// Hoje × oráculo sobre o MESMO estado: recompor (ou só re-aplicar o Offset) pelos dois caminhos.
fn confere(t: &mut PainterTool, so_offset: bool, passo: &str) {
    if so_offset {
        t.apply_selection_offset();
    } else {
        t.recompose_selection_mask();
    }
    let hoje = estado(t);
    // O oráculo rasteriza tudo de novo (sem a cache de hoje); a cache de hoje volta depois.
    let cache = std::mem::take(&mut t.paint.selection_raster_cache);
    if so_offset {
        t.apply_selection_offset_antigo();
    } else {
        t.recompose_selection_mask_antigo();
    }
    t.paint.selection_raster_cache = cache;
    let antigo = estado(t);
    assert!(
        hoje.0.iter().any(|&v| v > 0),
        "{passo}: a cena seleciona algo"
    );
    assert!(
        hoje.0 == antigo.0,
        "{passo}: selection_crisp difere do caminho antigo"
    );
    assert!(
        hoje.1 == antigo.1,
        "{passo}: selection_mask difere do caminho antigo"
    );
    assert!(hoje.2 == antigo.2, "{passo}: a fonte do Offset difere");
    assert_eq!(hoje.3, antigo.3, "{passo}: selection_active");
}

fn cena(size: u32) -> PainterTool {
    use super::super::curve_model::CurveModel;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let s = size as f32;
    let tri = CurveModel {
        points: vec![
            [0.30 * s, 0.25 * s],
            [0.70 * s, 0.35 * s],
            [0.45 * s, 0.80 * s],
        ],
        closed: true,
        ..Default::default()
    };
    let formas = [
        (
            SelectionShape::Ellipse {
                center: [0.40 * s, 0.45 * s],
                u: [0.8, 0.6],
                rx: 0.25 * s,
                ry: 0.15 * s,
            },
            0,
        ),
        (
            SelectionShape::Polygon {
                center: [0.65 * s, 0.55 * s],
                u: [1.0, 0.0],
                rx: 0.18 * s,
                ry: 0.12 * s,
                sides: 4,
            },
            1,
        ),
        (
            SelectionShape::Freehand {
                model: tri,
                u: [1.0, 0.0],
            },
            2,
        ),
        (
            SelectionShape::Ellipse {
                center: [0.5 * s, 0.5 * s],
                u: [1.0, 0.0],
                rx: 0.42 * s,
                ry: 0.40 * s,
            },
            3,
        ),
    ];
    for (shape, op) in formas {
        t.paint.selection_shapes.push(SelectionEntry { shape, op });
    }
    t
}

fn arrasta(t: &mut PainterTool, i: usize, dx: f32) {
    match &mut t.paint.selection_shapes[i].shape {
        SelectionShape::Ellipse { center, .. } | SelectionShape::Polygon { center, .. } => {
            center[0] += dx;
            center[1] -= 0.5 * dx;
        }
        SelectionShape::Freehand { model, .. } => {
            for p in &mut model.points {
                p[0] += dx;
            }
        }
        SelectionShape::Raster { .. } => {}
    }
}

#[test]
fn a_recomposicao_da_selecao_e_a_do_caminho_antigo_ao_bit() {
    for fios in [None, Some(1), Some(3), Some(7)] {
        crate::compositor::FIOS_FORCADOS.with(|c| c.set(fios));
        for feather in [0.0f32, 0.1] {
            let mut t = cena(384);
            t.paint.selection_feather = feather;
            confere(&mut t, false, "inicial");
            for k in 0..4 {
                arrasta(&mut t, k % 4, 9.5 - k as f32 * 3.0);
                confere(&mut t, false, &format!("arrasto {k}"));
            }
            // Offset simples: tiques do slider (níveis novos e um nível já visto).
            for norm in [0.56f32, 0.43, 0.56] {
                t.paint.selection_offset_norm = norm;
                confere(&mut t, true, &format!("offset {norm}"));
            }
            // Apply & Keep → anéis; um varrimento; Ctrl+Z volta ao Offset simples.
            t.paint.selection_offset_norm = 0.56;
            t.paint.selection_active = true;
            t.selection_offset_apply_keep();
            t.paint.selection_offset_norm = 0.45;
            confere(&mut t, true, "anéis");
            confere(&mut t, false, "anéis recompostos");
            assert!(t.undo_last(), "o Apply & Keep tem um degrau de undo");
            confere(&mut t, false, "Ctrl+Z");
        }
    }
    crate::compositor::FIOS_FORCADOS.with(|c| c.set(None));
}

/// Sonda 4096²: 3 formas, uma arrastada por quadro; e o tique do slider de Offset num nível já visto.
#[test]
#[ignore = "sonda B40.A3 — corra por nome com --ignored"]
fn sonda_recompose_selection_4096() {
    use std::time::Instant;
    let mut t = cena(4096);
    t.paint.selection_shapes.truncate(3);
    t.recompose_selection_mask();
    let (mut hoje, mut antigo) = (f64::MAX, f64::MAX);
    for volta in 0..12u32 {
        arrasta(&mut t, 1, if volta % 2 == 0 { 7.0 } else { -7.0 });
        let t0 = Instant::now();
        if volta % 2 == 0 {
            t.recompose_selection_mask_antigo();
            antigo = antigo.min(t0.elapsed().as_secs_f64() * 1e3);
        } else {
            t.recompose_selection_mask();
            hoje = hoje.min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    // Slider: dois níveis no cache, alternados (o custo é o de instalar, não o da geometria).
    t.paint.selection_offset_norm = 0.52;
    t.apply_selection_offset();
    t.paint.selection_offset_norm = 0.53;
    t.apply_selection_offset();
    let (mut s_hoje, mut s_antigo) = (f64::MAX, f64::MAX);
    for volta in 0..12u32 {
        t.paint.selection_offset_norm = if volta % 4 < 2 { 0.52 } else { 0.53 };
        let t0 = Instant::now();
        if volta % 2 == 0 {
            t.apply_selection_offset_antigo();
            s_antigo = s_antigo.min(t0.elapsed().as_secs_f64() * 1e3);
        } else {
            t.apply_selection_offset();
            s_hoje = s_hoje.min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    eprintln!(
        "B40.A3 4096²: arrasto antigo {antigo:.1} → hoje {hoje:.1} ms · tique Offset antigo {s_antigo:.2} → hoje {s_hoje:.2} ms (mínimos)"
    );
    assert!(
        hoje <= 25.0,
        "arrasto de gizmo 4096² custa {hoje:.1} ms (tecto 25 ms)"
    );
    assert!(
        s_hoje <= 2.0,
        "tique do Offset 4096² custa {s_hoje:.2} ms (tecto 2 ms)"
    );
}
