//! **A Borracha no Wet Paint apaga tudo o que está sob o círculo** (Bug #70, decisão do dono 05/11): além do fluido, a
//! tinta antiga, que mora na [`WetSession::base`] congelada. A cobertura sai da borracha normal, medida num escudo opaco
//! como a do Composite ([`Self::aplica_borracha_do_traco`]); o undo devolve a base pela chave do passo (`undo_vivo`).

use super::*;
use crate::tool::paint::region::dabs_bounds;

impl PainterTool {
    /// Apaga da base o que os `dabs` cobrem, sob a selecção e a protecção. O alfa preso não deixa apagar. A base só se
    /// copia quando há tinta a apagar; cada passo copia-a uma vez, por o undo segurar a de antes.
    pub(super) fn apaga_a_base_sob_os_dabs(&mut self, dabs: &[Dab]) {
        if self.paint.acumulando_no_plano {
            return;
        }
        let (w, h) = self.source_size;
        let stride = w as usize * 4;
        let len = stride * h as usize;
        let Some(r) = dabs_bounds(dabs, w, h, self.paint.brush.tip_reach()) else {
            return;
        };
        let (sel, prot, alock) = self.wet_splat_gates();
        if alock.is_some() || self.canvas_rgba.len() != len {
            return;
        }
        let Some(mut sess) = self.paint.wetpaint.session.take() else {
            return;
        };
        if sess.base.len() != len {
            self.paint.wetpaint.session = Some(sess);
            return;
        }
        if self.paint.pilha.escudo.len() != len {
            self.paint.pilha.escudo = Arc::new(vec![255u8; len]);
        }
        {
            let e = Arc::make_mut(&mut self.paint.pilha.escudo);
            for row in 0..r.h as usize {
                let o = (r.y as usize + row) * stride + r.x as usize * 4;
                e[o..o + r.w as usize * 4].fill(255);
            }
        }
        // Sem a sessão, os dabs caem na borracha normal; o escudo é um plano de medição, fora do journal do undo.
        let mut plano = std::mem::take(&mut self.paint.pilha.escudo);
        let rng = self.paint.tex_rng;
        crate::tool::paint::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut plano,
            &self.undo.write_state,
        );
        self.stamp_dabs_inner(dabs);
        crate::tool::paint::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut plano,
            &self.undo.write_state,
        );
        self.paint.tex_rng = rng;
        let linhas = || {
            (0..r.h as usize).flat_map(move |row| {
                let o = (r.y as usize + row) * stride + r.x as usize * 4;
                (0..r.w as usize).map(move |col| o + col * 4)
            })
        };
        let keep = |i: usize| {
            let c = 1.0 - f32::from(plano[i + 3]) / 255.0;
            let g = watercolor_accum::splat_keep(
                sel.as_deref().map(Vec::as_slice),
                prot.as_deref().map(Vec::as_slice),
                None,
                i / 4,
            );
            1.0 - c * g
        };
        if linhas().any(|i| sess.base[i + 3] > 0 && keep(i) < 1.0) {
            let base = Arc::make_mut(&mut sess.base);
            for i in linhas() {
                let k = keep(i);
                if k < 1.0 {
                    // Para baixo: com alfa 1 e k > 0,5 o arredondar nunca chegava a 0.
                    base[i + 3] = (f32::from(base[i + 3]) * k).floor() as u8;
                }
            }
            let q = i32::from(sess.ratio);
            sess.engine.merge_dirty_otica(
                r.x as i32 / q + 1,
                r.y as i32 / q + 1,
                (r.x + r.w) as i32 / q + 2,
                (r.y + r.h) as i32 / q + 2,
            );
        }
        self.paint.pilha.escudo = plano;
        self.paint.wetpaint.session = Some(sess);
    }
}
