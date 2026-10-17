//! **O `Style: Solid` e os fios no Wet Paint** (doc 46 item 9) — pela porta da MÁSCARA do motor
//! ([`ph2d_wet_paint::painter::Engine::deposit_mask`]): a região inteira pousa de uma vez, pela lei
//! do carimbo de tinta (o dente do papel, a cor por opacidade, os tectos), e o fluido age nela
//! quando a caneta sobe, como em qualquer traço.
//!
//! ⛔ **Nada de carimbos.** Medido 2026-10-05 (`diag_o_preco_da_agua`): os fios como carimbos de
//! água de 1 px custavam `283,6 ms` por quadro; uma máscara é UMA chamada por evento.
//!
//! ⚠️ **A mancha enche AO SOLTAR a caneta** (escolha do dono, 2026-10-05): durante o gesto a água
//! mostra só o traço. Medido em `4096²` com um laço de ~1 600 px: a mancha provisória refeita a cada
//! evento custava `21 ms` por quadro em média e `65` no pior (o Solid no Digital: `1,6` · `5,8`),
//! porque o recorte, o descasque e o depósito crescem com a ÁREA; pousada uma vez no pen-up ela custa
//! `~59 ms` uma vez só — e em telas normais quase nada. Os FIOS caem a cada evento (cumulativos):
//! `3,5 ms` médios ali, contra `5,5` do Sketchy no Digital.
//!
//! ⚠️ **Sem a corda.** A corda do Digital é carimbada com o pincel porque a aresta do rasterizador
//! é um corte duro; na água a corda seria tinta no rasto de uma faixa, que nenhum recorte desfaz, e
//! a borda da mancha amolece no fluido quando a caneta sobe.

use super::grid_map;
use super::*;
use crate::tool::paint::Region;
use ph2d_painter_brush::solid;
use ph2d_painter_brush::stroke::threads::Thread;
use ph2d_painter_brush::thread_raster::{ThreadInk, threads_alpha};

/// Folga em células à volta da máscara: a caixa de células cobre a AA das amostras da borda.
const FOLGA: i32 = 2;

impl PainterTool {
    /// A água pinta PIGMENTO neste gesto? — a ferramenta Paint, armada, sem a borracha. (Smear,
    /// Blend, Wet, Dry e Blow não depositam tinta: não há o que preencher nem o que costurar.)
    pub(in crate::tool::paint) fn agua_pinta_pigmento(&self) -> bool {
        self.paint.wetpaint.armed
            && self.paint.wetpaint.tool == WetTool::Paint
            && !self.paint.eraser
    }

    /// Pousa a janela de cobertura `cov` (`0..=255` por pixel, `rect` em pixels de canvas) na água,
    /// com a cor do pincel, e compõe.
    fn pousa_na_agua(&mut self, rect: Region, cov: &[u8]) {
        if rect.w == 0 || rect.h == 0 {
            return;
        }
        // A marca do artista (W9): a tinta desta cobertura aparece pela água.
        self.undo
            .marca
            .cobertura_na_agua((rect.x, rect.y, rect.w, rect.h), cov);
        let cor = self.paint.brush.color;
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        sess.bring_home();
        let ratio = sess.ratio;
        let cx0 = grid_map::px_to_cell(f64::from(rect.x), ratio).floor() as i32 - FOLGA;
        let cy0 = grid_map::px_to_cell(f64::from(rect.y), ratio).floor() as i32 - FOLGA;
        let cx1 = grid_map::px_to_cell(f64::from(rect.x + rect.w), ratio).ceil() as i32 + FOLGA;
        let cy1 = grid_map::px_to_cell(f64::from(rect.y + rect.h), ratio).ceil() as i32 + FOLGA;
        let (rx, ry, rw, rh) = (
            i64::from(rect.x),
            i64::from(rect.y),
            i64::from(rect.w),
            i64::from(rect.h),
        );
        let (aa_n, aa_step) = grid_map::cell_subsamples(ratio);
        let aa_inv = 1.0 / f64::from(u16::from(aa_n) * u16::from(aa_n));
        // A silhueta da célula: a média das amostras dela na janela (o AA por cobertura da rota do
        // dab, `dab_route`), `0` fora.
        let sil = |cx: i32, cy: i32| -> f64 {
            let mut acc = 0u32;
            for j in 0..aa_n {
                let sy = grid_map::cell_subsample_px(cy, ratio, j, aa_step).floor() as i64 - ry;
                for i in 0..aa_n {
                    let sx = grid_map::cell_subsample_px(cx, ratio, i, aa_step).floor() as i64 - rx;
                    if (0..rw).contains(&sx) && (0..rh).contains(&sy) {
                        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                        let k = (sy * rw + sx) as usize;
                        acc += u32::from(cov[k]);
                    }
                }
            }
            f64::from(acc) / 255.0 * aa_inv
        };
        let (mx, my) = (f64::from(cx0 + cx1) * 0.5, f64::from(cy0 + cy1) * 0.5);
        // O motor percorre o QUADRADO do raio (`shaped_bounds`), não o disco: meio lado maior cobre
        // a caixa — a meia diagonal percorria (e alocava na janela) o dobro das células.
        let r = f64::from((cx1 - cx0).max(cy1 - cy0)) * 0.5 + 1.0;
        let ink = [
            f64::from(cor[0]) * 255.0,
            f64::from(cor[1]) * 255.0,
            f64::from(cor[2]) * 255.0,
        ];
        sess.engine.deposit_mask(mx, my, r, 10.0, ink, &sil, None);
        self.wetpaint_composite();
    }

    /// **A mancha do `Style: Solid` na água, AO SOLTAR** (lisa; com ponta de imagem, os carimbos dela) — chamada pelo `paint_end` antes de o traço
    /// fechar (o depósito entra no mesmo passo de Undo). Durante o gesto o bracket do Solid só grava o
    /// caminho da tinta (`note_ink_path`), e o `stamp_solid_preview` não faz nada na água.
    pub(in crate::tool::paint) fn enche_a_mancha_na_agua(&mut self) {
        if !self.freehand_solid_fill_live()
            || !matches!(self.paint.paint_mode, super::PaintMode::WetPaint)
        {
            return;
        }
        // Uma ponta de IMAGEM enche a mancha com os carimbos dela, como no Digital (Bug #53; 26 C2,
        // plano 27 W4): o Shape é o pincel em toda mídia. ⚠️ Pela MÁSCARA, com a forma da união deles —
        // dab a dab cada carimbo pousava ~1/10 da tinta de um traço (o traço sobrepõe ~10 dabs por
        // ponto): medido, 39 de 400 píxeis do miolo tingidos contra 400 da mancha lisa.
        let carimbos = self.solid_fill_stamps();
        if !carimbos.is_empty() {
            if let Some((rect, cov)) = self.cobertura_dos_carimbos(&carimbos) {
                self.pousa_na_agua(rect, &cov);
            }
            return;
        }
        let loops = self.solid_fill_loops();
        let Some(rect) = self.solid_fill_rect(&loops) else {
            return;
        };
        #[allow(clippy::cast_precision_loss)]
        let origin = [rect.x as f32, rect.y as f32];
        let cov = solid::fill_coverage(&loops, rect.w as usize, rect.h as usize, origin);
        self.pousa_na_agua(rect, &cov);
    }

    /// A cobertura (`0..=255`) da mancha de CARIMBOS: a União `1 − Π(1 − c·s)` das silhuetas da rota do
    /// dab ([`ph2d_painter_brush::dab::silhouette_at`], cada carimbo com a rotação, a pegada e a força
    /// dele), na janela que os cobre — a tela inteira com Tiling, onde um carimbo na costura dá a volta.
    fn cobertura_dos_carimbos(&self, carimbos: &[Dab]) -> Option<(Region, Vec<u8>)> {
        let (w, h) = self.source_size;
        let brush = self.stroke_spec();
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let ramp_lut = (self.paint.shape_color_ramp_enabled && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
        let tiling = self.paint.tiling;
        let canvas = [w as f32, h as f32];
        let mut rng = self.paint.tex_rng;
        let pontas: Vec<_> = carimbos
            .iter()
            .map(|d| {
                let spec = BrushSpec {
                    radius_px: d.radius_px,
                    ..brush
                };
                let fp = spec.dab_footprint(spec.dab_rotor(d));
                let basis = ph2d_painter_brush::texture::shape_basis(
                    &spec.shape,
                    &mut rng,
                    canvas,
                    fp,
                    ph2d_painter_brush::texture::ShapeFrame::Stroke {
                        arc_len: d.arc_len,
                        unit_px: d.stroke_radius_px,
                    },
                );
                let alcance = d.radius_px * spec.dab_tip_reach(Some(&basis));
                (d, spec, fp, basis, alcance)
            })
            .collect();
        let rect = if tiling[0] || tiling[1] {
            Region { x: 0, y: 0, w, h }
        } else {
            let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for (d, .., a) in &pontas {
                x0 = x0.min(d.center[0] - a);
                y0 = y0.min(d.center[1] - a);
                x1 = x1.max(d.center[0] + a);
                y1 = y1.max(d.center[1] + a);
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let (x0, y0, x1, y1) = (
                x0.floor().max(0.0) as u32,
                y0.floor().max(0.0) as u32,
                (x1.ceil().max(0.0) as u32).min(w),
                (y1.ceil().max(0.0) as u32).min(h),
            );
            if x1 <= x0 || y1 <= y0 {
                return None;
            }
            Region {
                x: x0,
                y: y0,
                w: x1 - x0,
                h: y1 - y0,
            }
        };
        let (rw, rh) = (i64::from(rect.w), i64::from(rect.h));
        let mut resto = vec![1.0f32; (rect.w * rect.h) as usize];
        for (d, spec, fp, basis, a) in &pontas {
            let ponta = ph2d_painter_brush::ShapeInput {
                basis,
                image: shape_image.as_ref(),
                ramp_lut,
            };
            let inv_r = 1.0 / d.radius_px.max(0.01);
            #[allow(clippy::cast_possible_truncation)]
            let (px0, py0, px1, py1) = (
                (d.center[0] - a).floor() as i64,
                (d.center[1] - a).floor() as i64,
                (d.center[0] + a).ceil() as i64,
                (d.center[1] + a).ceil() as i64,
            );
            for py in py0..=py1 {
                let ty = if tiling[1] {
                    py.rem_euclid(i64::from(h))
                } else {
                    py
                } - i64::from(rect.y);
                if !(0..rh).contains(&ty) {
                    continue;
                }
                for px in px0..=px1 {
                    let tx = if tiling[0] {
                        px.rem_euclid(i64::from(w))
                    } else {
                        px
                    } - i64::from(rect.x);
                    if !(0..rw).contains(&tx) {
                        continue;
                    }
                    #[allow(clippy::cast_precision_loss)]
                    let (sx, sy) = (px as f32 + 0.5, py as f32 + 0.5);
                    let t = fp.falloff_t((sx - d.center[0]) * inv_r, (sy - d.center[1]) * inv_r);
                    let s = ph2d_painter_brush::dab::silhouette_at(
                        spec,
                        Some(ponta),
                        t,
                        px,
                        py,
                        d.center,
                        d.radius_px,
                    );
                    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                    let k = (ty * rw + tx) as usize;
                    resto[k] *= 1.0 - (s * d.coverage).clamp(0.0, 1.0);
                }
            }
        }
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let cov = resto
            .iter()
            .map(|r| ((1.0 - r) * 255.0).round() as u8)
            .collect();
        Some((rect, cov))
    }

    /// **Os fios na água** — cumulativos, na opacidade deles (o alfa do feixe é a máscara).
    pub(in crate::tool::paint) fn fios_na_agua(
        &mut self,
        threads: &[Thread],
        ink: ThreadInk,
        rect: Region,
    ) {
        #[allow(clippy::cast_precision_loss)]
        let origin = [rect.x as f32, rect.y as f32];
        let alpha = threads_alpha(threads, ink, rect.w as usize, rect.h as usize, origin);
        self.pousa_na_agua(rect, &alpha);
    }
}
