//! O corpo da MANCHA do Solid no impasto (★B52.R10): não é «fill» — é o planalto que fecha o envelope
//! do traço no pen-up. Mora ao lado do `impasto_fill.rs`, que fica com a cobertura da seleção e o corpo
//! do balde.

use crate::tool::PainterTool;

impl PainterTool {
    /// **O corpo da MANCHA, fundido no envelope do traço** (smoke do dono 2026-10-06: *«em impasto o
    /// relevo deveria preencher o centro com solid»*). Pela lei do dab (`height_walk`): onde a carga
    /// da mancha (cobertura × a carga que o traço atingiu) passa a do traço, ela fica com o texel —
    /// carga, grão, raio e a altura DERIVADA (`derive_height`, a mesma de um dab: o planalto tem a
    /// altura do miolo do traço, sem degrau na junção); o filme por máximo. Uma vez, no pen-up, como o
    /// corpo da corda.
    pub(in crate::tool::paint) fn assenta_o_corpo_da_mancha(
        &mut self,
        brush: &ph2d_painter_brush::BrushSpec,
    ) {
        use ph2d_painter_brush::height::{NO_GRAIN, derive_height};
        if !brush.deposits_height() || !self.impasto_batch_active() {
            return;
        }
        let loops = self.solid_fill_loops();
        let Some(r) = self.solid_fill_rect(&loops) else {
            return;
        };
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let n = w * h;
        // O GRÃO do pincel, amostrado como um dab o amostra (`grain_at`): com a fonte do relevo em
        // Grain ele esculpe o corpo do traço, e a mancha leva o mesmo grão (smoke do dono 2026-10-06:
        // *«o relevo do papel está corretamente sendo transmitido para o traço, mas no preenchimento
        // do solid não»*). Uma base só para a mancha inteira (ancorada à tela num Grain Tiled — o
        // papel), tirada de uma CÓPIA do sorteio, que não é avançado.
        let grao_ativo = brush.texture.is_active();
        let imagem = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let base_do_grao = grao_ativo.then(|| {
            let mut rng = self.paint.tex_rng;
            ph2d_painter_brush::texture::dab_basis(
                &brush.texture,
                &mut rng,
                [w as f32, h as f32],
                ph2d_painter_brush::footprint::FootprintDeform::identity(),
            )
        });
        #[allow(clippy::cast_precision_loss)]
        let centro = [r.x as f32 + r.w as f32 * 0.5, r.y as f32 + r.h as f32 * 0.5];
        let rel = &mut self.paint.relief;
        for (plano, vazio) in [
            (&mut rel.stroke_height, 0.0f32),
            (&mut rel.stroke_paint, 0.0),
            (&mut rel.stroke_radius, 0.0),
        ] {
            if plano.len() != n {
                *plano = vec![vazio; n];
            }
        }
        for plano in [&mut rel.stroke_grain, &mut rel.stroke_film] {
            if plano.len() != n {
                *plano = vec![0u8; n];
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let origin = [r.x as f32, r.y as f32];
        let cov =
            ph2d_painter_brush::solid::fill_coverage(&loops, r.w as usize, r.h as usize, origin);
        let raio = brush.clamped_radius();
        // A carga que o TRAÇO atingiu (a Strength e a pressão já estão nela): o planalto da mancha
        // encosta no corpo do traço sem degrau, como a cor encosta no tecto dele.
        let carga = (r.y as usize..(r.y + r.h) as usize)
            .flat_map(|y| {
                rel.stroke_paint[y * w + r.x as usize..y * w + (r.x + r.w) as usize]
                    .iter()
                    .copied()
            })
            .fold(0.0f32, f32::max);
        let carga = if carga > 0.0 { carga } else { 1.0 };
        for row in 0..r.h as usize {
            for cx in 0..r.w as usize {
                let c = cov[row * r.w as usize + cx];
                if c == 0 {
                    continue;
                }
                let i = (r.y as usize + row) * w + r.x as usize + cx;
                let p = f32::from(c) / 255.0 * carga;
                if p > rel.stroke_paint[i] {
                    rel.stroke_paint[i] = p;
                    let gq = base_do_grao.as_ref().map_or(NO_GRAIN, |b| {
                        let (x, y) = ((r.x as usize + cx) as i64, (r.y as usize + row) as i64);
                        let g = ph2d_painter_brush::dab::grain_at(
                            brush,
                            b,
                            imagem.as_ref(),
                            x,
                            y,
                            centro,
                            raio,
                        );
                        (g.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
                    });
                    rel.stroke_grain[i] = gq;
                    rel.stroke_radius[i] = raio;
                    // A altura VIVA, pela mesma lei; no pen-up o commit re-deriva-a da carga, do
                    // grão e do raio (`commit_stroke_height`) — é por isso que a mutação que tira
                    // o grão SÓ daqui sobrevive, e a que o tira do plano do grão sangra.
                    rel.stroke_height[i] = derive_height(brush, p, f32::from(gq) / 255.0);
                }
                rel.stroke_film[i] = rel.stroke_film[i].max(c);
            }
        }
        rel.stroke_relief_bbox = Some(
            rel.stroke_relief_bbox
                .map_or(r, |b| crate::tool::paint::union_region(b, r)),
        );
    }
}
