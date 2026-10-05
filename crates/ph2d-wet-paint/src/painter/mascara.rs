//! **A porta da MÁSCARA** (filho de [`super`]): pousa uma região inteira de uma vez — a mancha do
//! `Style: Solid` e os fios de um evento (o Painter, doc 46 item 9).
//!
//! ⭐ **Nenhuma aritmética nova.** É uma janela de rasto PRÓPRIA (nunca a de uma faixa do traço),
//! do tamanho da máscara, que acumula pela lei da célula de sempre ([`Trail::accumulate_paint_shaped`]
//! — o dente do papel, a umidade) e pousa NA HORA ([`Trail::transfer_paint`] — a cor por opacidade,
//! os tectos suaves, a ponta que apanha a cor molhada). O arrasto do pouso é nulo por construção:
//! a âncora anterior de uma janela nova é a própria.
//!
//! ⛔ **Por que não pelas faixas** (medido 2026-10-05, `diag_o_preco_da_agua` no Painter): os fios
//! como carimbos de 1 px pela rota do dab custavam `283,6 ms` por quadro — o preço é o dab, não a
//! tinta. E a janela de uma faixa ANDA com o pincel: uma mancha de 400 px por ela pousaria quando
//! a janela enchesse, não quando o artista a fechou.

use super::*;

impl Engine {
    /// Pousa a máscara `sil` (cobertura `0..1` por CÉLULA; `0` = fora) com a cor `color`
    /// (`0..255`), num disco de raio `r` células em `(x, y)` que a contém.
    ///
    /// A máscara acumula UMA vez — o corpo de um carimbo da faixa: medido (Painter,
    /// `a_mancha_tem_o_corpo_do_traco`), o miolo de um traço lê `0,75`–`1,3×` uma passada lisa, porque
    /// a janela da faixa não soma dab a dab (tectos suaves, a perda no pouso). `b` é a pressão, a
    /// mesma escala da [`Self::dispatch_pressure_dab_lane`].
    ///
    /// `grain` (`None` = lisa) substitui a textura de cerdas, como na porta da faixa.
    #[allow(clippy::too_many_arguments)]
    pub fn deposit_mask(
        &mut self,
        x: f64,
        y: f64,
        r: f64,
        b: f64,
        color: [f64; 3],
        sil: crate::brush::CellFn<'_>,
        grain: Option<crate::brush::CellFn<'_>>,
    ) {
        if r <= 0.0 {
            return;
        }
        let p = self.sim.gather_params(&self.tuning);
        let (dab, _water_unit) = self.pressure_dab(&p, x, y, b, 1.0, 0.0, r);
        let ext_bypass = self.sim.ext_bypass;
        self.texture();
        let tex = self.brush_tex.take().unwrap();
        let layer = &mut self.layers[self.active_layer];
        // ⚠️ **Sem grão do hospedeiro, a máscara é LISA** — e não as cerdas do pincel: num traço as
        // cerdas andam com o dab e cobrem tudo; num dab parado do tamanho da região elas ficam
        // como salpicos FIXOS (medido: metade do miolo, 200 de 400 pixels). O papel continua a
        // granular — o dente é da célula, não do pincel.
        let plano = |_: i32, _: i32| 1.0;
        let grain = Some(grain.unwrap_or(&plano));
        let mut trail = Trail::default();
        trail.start_stroke(x, y, color, TrailMode::Paint);
        let acc =
            trail.accumulate_paint_shaped(&mut layer.grid, &p, &tex, &dab, ext_bypass, sil, grain);
        let landed = trail.transfer_paint(&mut layer.grid, &p);
        self.brush_tex = Some(tex);
        if let Some(rc) = TouchedRect::union(acc.wrote, landed) {
            self.merge_dirty(rc.x0, rc.y0, rc.x1, rc.y1);
        }
    }
}
