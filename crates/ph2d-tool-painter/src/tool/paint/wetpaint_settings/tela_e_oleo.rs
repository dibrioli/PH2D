//! O Smooth Edges, o Óleo Real, a Live Water e a Heavy Edge do Wet Paint — filho de `wetpaint_settings` por LOC.

use super::*;

impl PainterTool {
    /// A intensidade do **Smooth Edges** do Wet Paint (`0..1`; NaN = `0`). É da TELA (dono 27/10, D1: «para todo o
    /// papel, de todos os traços já pintados ou futuros»; mora no cartão Paper): mudá-la recompõe a tela inteira.
    pub fn set_wet_smooth_edges(&mut self, forca: f32) {
        if self.guarda_smooth_edges(forca) {
            self.wet_recomposite_full();
        }
    }

    /// Guarda o Smooth Edges; `true` = mudou.
    pub(in crate::tool::paint) fn guarda_smooth_edges(&mut self, forca: f32) -> bool {
        let forca = if forca.is_nan() {
            0.0
        } else {
            forca.clamp(0.0, 1.0)
        };
        let muda = self.paint.wetpaint.smooth_edges != forca;
        self.paint.wetpaint.smooth_edges = forca;
        muda
    }

    /// A intensidade do **Smooth Edges** do Wet Paint (`0` = desligado).
    #[must_use]
    pub fn wet_smooth_edges(&self) -> f32 {
        self.paint.wetpaint.smooth_edges
    }

    /// O **Óleo Real** (doc 49): a caixa do pincel (`Some`) e, com ela, os números do cartão. A caixa é do TRAÇO (dono
    /// 28/10): vale para os traços seguintes e não recompõe; os números são da TELA e recompõem a tinta com óleo.
    pub fn set_wet_oleo_real(&mut self, oleo: Option<ph2d_wet_paint::oleo_real::OleoReal>) {
        if let Some(o) = oleo {
            self.guarda_oleo_valores(o);
        }
        self.paint.wetpaint.oleo_real = oleo.map(|_| self.paint.wetpaint.oleo_real_valores);
    }

    /// **Adia a óptica do óleo** até `false` (o pintor de TESTE — a bancada, o gerador —, como o preview do preset já
    /// faz, U4 W3b): o traço pousa o óleo, o composite a meio sai sem a luz, e ao desligar, com óleo na tela, a tela
    /// inteira recompõe-se com ela — a foto final ao bit (`exame_bancada` J6). O app nunca a liga.
    #[doc(hidden)]
    pub fn set_wet_oleo_adiado(&mut self, adiado: bool) {
        let era = std::mem::replace(&mut self.paint.wetpaint.oleo_adiado, adiado);
        // Só o desligar pergunta à tela (e traz o motor do worker): ligar, ou repetir, não toca na sessão.
        let tem_oleo = era
            && !adiado
            && self.paint.wetpaint.session.as_mut().is_some_and(|s| {
                s.bring_home();
                s.engine
                    .layers
                    .iter()
                    .any(|l| l.visible && !l.grid.oleo.is_empty())
            });
        if tem_oleo {
            self.wet_recomposite_full();
        }
    }

    /// O **Óleo Real** do Wet Paint (`None` = desligado).
    #[must_use]
    pub fn wet_oleo_real(&self) -> Option<ph2d_wet_paint::oleo_real::OleoReal> {
        self.paint.wetpaint.oleo_real
    }

    /// A caixa **Real Oil**: ligada = `Some` dos últimos valores dos knobs (os de fábrica na primeira vez),
    /// desligada = `None` — os valores ficam guardados para a próxima vez.
    pub fn set_wet_oleo_real_ligado(&mut self, ligado: bool) {
        let o = ligado.then_some(self.paint.wetpaint.oleo_real_valores);
        self.set_wet_oleo_real(o);
    }

    /// Os últimos valores dos knobs do **Real Oil** (os do painel, ligada ou não a caixa).
    #[must_use]
    pub fn wet_oleo_real_valores(&self) -> ph2d_wet_paint::oleo_real::OleoReal {
        self.paint.wetpaint.oleo_real_valores
    }

    /// Uma linha do **Real Oil** (`PAINTER_WETPAINT_OLEO_FIELDS`), nas unidades do painel — os ângulos em
    /// GRAUS. Guarda o valor e, com a caixa ligada, recompõe. `false` = o id não é dela; NaN = sem efeito.
    ///
    /// As faixas são as de `PAINTER_WETPAINT_OLEO_FAIXAS` (o porquê do piso de 5° da luz está lá); a
    /// direcção da luz dá a volta em vez de se prender.
    pub(super) fn set_wet_oleo_knob(&mut self, id: ph2d_a11y::NodeId, v: f64) -> bool {
        let ids = crate::ids::PAINTER_WETPAINT_OLEO_FIELDS;
        let Some(k) = ids.iter().position(|i| *i == id) else {
            return false;
        };
        if v.is_nan() {
            return true;
        }
        let (lo, hi) = crate::ids::PAINTER_WETPAINT_OLEO_FAIXAS[k];
        let mut o = self.paint.wetpaint.oleo_real_valores;
        match k {
            0 => o.sombra = v.clamp(lo, hi),
            1 => o.altitude = v.clamp(lo, hi).to_radians(),
            2 => o.azimute = v.rem_euclid(hi).to_radians(),
            3 => o.fenda = v.clamp(lo, hi),
            4 => o.alcance_fenda = v.clamp(lo, hi),
            5 => o.forca_ambiente = v.clamp(lo, hi),
            _ => o.material.poe(
                k - crate::ids::PAINTER_WETPAINT_OLEO_MATERIAL_DE,
                v.clamp(lo, hi),
            ),
        }
        self.guarda_oleo_valores(o);
        true
    }

    /// O **Environment** do Real Oil pelo índice em `Ambiente::TODOS` (o wire do chip). Fora da faixa =
    /// sem efeito (`false`). Guarda-o e, com a caixa ligada, recompõe.
    pub(super) fn set_wet_oleo_ambiente_indice(&mut self, i: u8) -> bool {
        let Some(a) = ph2d_wet_paint::oleo_real::Ambiente::de_indice(i) else {
            return false;
        };
        let mut o = self.paint.wetpaint.oleo_real_valores;
        o.ambiente = a;
        self.guarda_oleo_valores(o);
        true
    }

    /// Os valores novos dos knobs do Real Oil: ficam guardados e, com a caixa ligada, vão à tela NO QUADRO
    /// (um arrasto manda um valor por evento do rato — `wetpaint::oleo_no_quadro`).
    fn guarda_oleo_valores(&mut self, o: ph2d_wet_paint::oleo_real::OleoReal) {
        let w = &mut self.paint.wetpaint;
        let muda = w.oleo_real_valores != o;
        w.oleo_real_valores = o;
        if w.oleo_real.is_some() {
            w.oleo_real = Some(o);
        }
        // Os números do cartão são da TELA: mudam a tinta com óleo já pousada (e só ela).
        if muda && w.oleo_na_tela {
            self.wet_recompoe_no_quadro();
        }
    }

    /// A **Live Water** (W1g): a água corre com o traço aberto.
    pub fn set_wet_agua_viva(&mut self, ligada: bool) {
        self.paint.wetpaint.agua_viva = ligada;
    }

    /// A **Live Water** está ligada?
    #[must_use]
    pub fn wet_agua_viva(&self) -> bool {
        self.paint.wetpaint.agua_viva
    }

    /// A **Heavy Edge** (W1g, `0..1`; NaN = o neutro).
    pub fn set_wet_borda_que_pesa(&mut self, v: f64) {
        self.paint.wetpaint.borda_que_pesa = if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) };
    }

    /// A **Heavy Edge** de agora.
    #[must_use]
    pub fn wet_borda_que_pesa(&self) -> f64 {
        self.paint.wetpaint.borda_que_pesa
    }

    /// The authored knob values (the panel snapshot's source).
    #[must_use]
    pub fn wet_knobs(&self) -> WetKnobs {
        self.paint.wetpaint.knobs
    }
}
