//! **Grid Stamp** — como um traço preso a uma grade emite os seus dabs.
//!
//! A pergunta é outra da do `stroke.rs` pai: lá o traço PERCORRE um caminho e o espaçamento decide
//! onde cai o próximo dab; aqui o caminho só serve para descobrir que CÉLULAS foram atravessadas, e
//! quem decide onde o dab cai é a grade. Por isso o módulo é irmão do `stabilize`/`warmup` em vez de
//! mais um punhado de métodos no meio do caminhador.

use super::MAX_BRUSH_RADIUS_PX;
use super::Stroke;
use crate::Dab;

impl Stroke {
    /// **Grid Stamp** — percorre `from → to` e carimba toda célula NOVA que o caminho atravessa.
    ///
    /// O passo é meia célula (o menor lado), então amostras consecutivas nunca ficam a mais de uma
    /// célula de distância em cada eixo — é isso que impede um arrasto rápido de deixar buracos, sem
    /// precisar de um rasterizador supercover.
    pub(super) fn walk_grid_cells(&mut self, from: [f32; 2], to: [f32; 2], out: &mut Vec<Dab>) {
        let c = self.spec.grid_cell();
        let step = 0.5 * c[0].min(c[1]);
        let d = [to[0] - from[0], to[1] - from[1]];
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let n = if len > 0.0 {
            ((len / step).ceil() as u32).max(1)
        } else {
            1
        };
        for i in 1..=n {
            let t = f64::from(i) / f64::from(n);
            let p = [from[0] + d[0] * t as f32, from[1] + d[1] * t as f32];
            self.stamp_cell(self.spec.grid_cell_at(p), out);
        }
    }

    /// Carimba UMA célula — no centro dela — se ela não for a última carimbada.
    ///
    /// ⚠️ **A cópia de Symmetry é RE-ENCAIXADA na própria célula.** O espelho de um centro de célula
    /// não é, em geral, o centro de uma célula, e a promessa inteira deste método é que todo carimbo
    /// pousa num centro. Deixar a cópia onde o espelho a põe daria uma metade fora da grade — o único
    /// defeito que este método não pode ter. (A alternativa — não espelhar — mataria Symmetry aqui
    /// sem necessidade.)
    pub(super) fn stamp_cell(&mut self, cell: [i32; 2], out: &mut Vec<Dab>) {
        if self.last_cell == Some(cell) {
            return;
        }
        self.last_cell = Some(cell);
        let dab = self.grid_dab(self.spec.grid_cell_center(cell));
        let first = out.len();
        crate::symmetry::push_symmetric(out, dab, &self.spec.symmetry);
        for d in &mut out[first..] {
            d.center = self.spec.grid_cell_center(self.spec.grid_cell_at(d.center));
        }
        self.tot_samples = self.tot_samples.wrapping_add(1);
    }

    /// O dab de um carimbo de grade: o raio vem da CÉLULA, a pressão é cheia e não há heading — um
    /// carimbo de grade é alinhado à REDE, não ao traço, então girá-lo com a mão brigaria com a
    /// célula que ele preenche.
    ///
    /// ⚠️ **O raio é lido do spec, não re-derivado aqui.** O tool abre o traço com o spec já passado
    /// por [`BrushSpec::as_grid_stamp`] — a porta única, e a única que sabe se o slot de **Shape**
    /// carrega uma silhueta (os pixels dela vivem no tool). Chamar o frame daqui obrigava o motor a
    /// responder uma pergunta que ele não pode responder, e a resposta errada era um carimbo **1,64×**
    /// a célula. Um traço de grade aberto com um spec CRU carimba o tamanho do pincel, e é o
    /// `the_emitted_radius_comes_from_the_cell_not_the_brush` que prova que o produto não faz isso.
    fn grid_dab(&self, center: [f32; 2]) -> Dab {
        let radius = self.spec.radius_px;
        Dab {
            center,
            radius_px: radius.clamp(0.0, MAX_BRUSH_RADIUS_PX),
            coverage: self.spec.strength.clamp(0.0, 1.0),
            color: self.spec.color,
            rotation: [1.0, 0.0],
            dir: [0.0, 0.0],
            arc_len: 0.0,
            stroke_radius_px: radius,
        }
    }
}
