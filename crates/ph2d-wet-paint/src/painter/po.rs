//! **O GRÃO DE PÓ SOLTO** (J4 `pastel_carvao`, `docs/Painter/Pinceis/exames/pastel_carvao.md` 2.ª ronda): a porta por
//! onde o hospedeiro pousa um grão do pó À PARTE do rasto do traço (filho de [`super`], LOC cap).

use super::*;

impl Engine {
    /// Pousa UM grão de pó solto em `(x, y)` com raio `r` (células) e a silhueta `sil` do hospedeiro, pela lei do
    /// dab ([`Trail::accumulate_paint_shaped_cor`]: o portão do papel, a carga, o pouso e a recolha da cor) — mas num
    /// rasto PRÓPRIO (`trail`, do hospedeiro), recomeçado com a cor `color` (sRGB `0..255`) em cada grão e pousado
    /// logo. O grão não entra nos sub-passos nem na janela da lane: não arrasta a corda, a âncora nem a carga do
    /// traço (dentro da lane os grãos pintavam nódoas — `exames/texturas.md` «Motor J4»). Liso: sem cerdas nem grão
    /// do hospedeiro (o grão É o grão).
    #[allow(clippy::too_many_arguments)]
    pub fn pousa_grao_solto(
        &mut self,
        trail: &mut Trail,
        color: [f64; 3],
        x: f64,
        y: f64,
        b: f64,
        r: f64,
        opacidade: f64,
        sil: crate::brush::CellFn<'_>,
    ) {
        let mut p = self
            .sim
            .gather_params(&self.tuning)
            .com_pigmento(self.sliders.pigmento);
        p.agua_do_dab = self.agua_do_dab;
        let (mut dab, _) = self.pressure_dab(&p, x, y, b, r);
        dab.opacity = opacidade;
        dab.water_amount *= self.agua_do_dab;
        // A silhueta do grão tem a orla de AA de uma célula além do raio.
        dab.reach = (r + 1.5) / r.max(1e-3);
        dab.window_reach = dab.reach;
        let ext_bypass = self.sim.ext_bypass;
        self.texture();
        let tex = self.brush_tex.take().unwrap();
        trail.start_stroke(x, y, color, TrailMode::Paint);
        let lisa = |_: i32, _: i32| -> f64 { 1.0 };
        let layer = &mut self.layers[self.active_layer];
        let acc = trail.accumulate_paint_shaped_cor(
            &mut layer.grid,
            &p,
            &tex,
            &dab,
            ext_bypass,
            sil,
            Some(&lisa),
            None,
            None,
            None,
        );
        let landed = if acc.window_full {
            trail.transfer_paint(&mut layer.grid, &p)
        } else {
            None
        };
        trail.drop_remainder();
        self.brush_tex = Some(tex);
        if let Some(rc) = TouchedRect::union(acc.wrote, landed) {
            self.merge_dirty(rc.x0, rc.y0, rc.x1, rc.y1);
        }
    }
}
