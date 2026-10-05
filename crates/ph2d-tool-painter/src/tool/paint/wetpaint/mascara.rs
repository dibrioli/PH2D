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

    /// **A mancha do `Style: Solid` na água, AO SOLTAR** — chamada pelo `paint_end` antes de o traço
    /// fechar (o depósito entra no mesmo passo de Undo). Durante o gesto o bracket do Solid só grava o
    /// caminho da tinta (`note_ink_path`), e o `stamp_solid_preview` não faz nada na água.
    pub(in crate::tool::paint) fn enche_a_mancha_na_agua(&mut self) {
        if !self.freehand_solid_fill_live()
            || !matches!(self.paint.paint_mode, super::PaintMode::WetPaint)
        {
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
