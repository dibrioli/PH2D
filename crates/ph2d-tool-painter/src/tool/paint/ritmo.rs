//! B22.1 (decisão 8, ritmo fixo): o que o Painter INTEGRA no tempo (a secagem do papel, o encharque sob
//! a ponta, o tique do traço e o alcance do estabilizador) anda em passos fixos de [`PASSO_FIXO_S`], não no `dt` do quadro — o mesmo gesto pelo mesmo
//! tempo de parede dá o mesmo byte a 30, 60 ou 144 Hz. O `wall_dt` partilhado do shell fica intacto (C9).
//! O passo é o de 60 Hz: a lei que o dono aprovou é a que 60 Hz pintava.

/// O passo fixo, em segundos.
pub(super) const PASSO_FIXO_S: f32 = 1.0 / 60.0;
/// Batente por quadro: 8 passos = 133 ms de quadro. Recurso: o passe de secagem a 4096² custa 3,45 ms
/// (`watercolor_dry.rs`, medido 2026-08-02) ⇒ ≤ 28 ms a pagar num quadro preso; o excesso dobra-se no
/// último passo (a lei de um `dt` grande, como antes) para um engasgo não virar espiral.
pub(super) const PASSOS_POR_QUADRO_MAX: u32 = 8;

/// Quem pede os passos (a chave de medição [`ritmo_antigo`] distingue-os).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Ramo {
    Seca,
    Traco,
    /// O traço de um meio que não integra no tempo (Digital, Impasto): um passo com o `dt` do quadro, os
    /// bytes de antes do B22.1 (a lei J3 — a velocidade desde o 1.º dab — mede-se por quadro, e o banco de
    /// pincéis tica a 8 ms; integrador 13/11: a lei nova só nos meios molhados).
    Quadro,
}

/// Medição A×B (`PH2D_RITMO_FIXO`, lida uma vez): `0` = os dois ramos no `dt` do quadro (antes do B22.1),
/// `seca` / `traco` = só esse ramo. Ausente = o passo fixo.
fn ritmo_antigo(ramo: Ramo) -> bool {
    static V: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    match V
        .get_or_init(|| std::env::var("PH2D_RITMO_FIXO").ok())
        .as_deref()
    {
        Some("0") => true,
        Some("seca") => ramo == Ramo::Seca,
        Some("traco") => ramo == Ramo::Traco,
        _ => false,
    }
}

/// Os `dt` a correr neste quadro: `relogio` (em passos) recebe `dt_s` e devolve os passos inteiros.
pub(super) fn passos_fixos(relogio: &mut f64, dt_s: f32, ramo: Ramo) -> impl Iterator<Item = f32> {
    let (m, ultimo) = if ramo == Ramo::Quadro || ritmo_antigo(ramo) {
        (1, dt_s)
    } else {
        *relogio += f64::from(dt_s.max(0.0)) / f64::from(PASSO_FIXO_S);
        // A folga absorve o arredondamento do `dt` em f32 (16,666666 ms ≠ 1/60 s ao último bit).
        let n = (*relogio + 1e-6).floor().max(0.0) as u32;
        *relogio -= f64::from(n);
        let m = n.min(PASSOS_POR_QUADRO_MAX);
        (m, PASSO_FIXO_S * (n - m + 1) as f32)
    };
    (0..m).map(move |i| if i + 1 == m { ultimo } else { PASSO_FIXO_S })
}

use super::{Dab, Stroke, union_region};

impl super::PainterTool {
    /// Os passos fixos do traço aberto neste quadro: por passo, o relógio do traço ([`Stroke::tick`]:
    /// aerógrafo, fita, velocidade), o alcance do estabilizador se a mão não se moveu desde o passo
    /// anterior ([`Stroke::settle`]) e o encharque sob a ponta. Devolve se este quadro tem de recompor.
    ///
    /// Encharque: PARADO ⇒ recompõe já (o sangrado que cresce sob a ponta é o ponto). EM MOVIMENTO ⇒ só
    /// dobra a caixa no sujo do quadro: o flush do Move já recompõe (recompor de novo dobrava o custo,
    /// perfil 2026-07-07) e o bake do pen-up lê o campo inteiro.
    pub(super) fn passos_do_traco(
        &mut self,
        stroke: &mut Stroke,
        dabs: &mut Vec<Dab>,
        dt_s: f32,
        wet: bool,
    ) -> bool {
        let mut stamped = false;
        let mut relogio = self.paint.traco_relogio;
        // Só os meios molhados (aguada, Wet Paint) integram no tempo e andam no passo fixo.
        let molhado = wet || self.paint_media() == super::media::PaintMedia::WetPaint;
        let ramo = if molhado { Ramo::Traco } else { Ramo::Quadro };
        for dt in passos_fixos(&mut relogio, dt_s, ramo) {
            let parked = !std::mem::take(&mut self.paint.movido_desde_o_passo);
            stroke.tick(dt, dabs);
            self.dual_passo(|m, o| m.tick(dt, o));
            super::ribbon_diag::note(super::ribbon_diag::Source::Tick, dabs.len());
            stamped |= !dabs.is_empty();
            self.stamp_dabs(dabs);
            if parked {
                stroke.settle(dabs); // clears `dabs` first
                self.dual_passo(|m, o| m.settle(o));
                super::ribbon_diag::note(super::ribbon_diag::Source::Settle, dabs.len());
                stamped |= !dabs.is_empty();
                self.stamp_dabs(dabs);
            }
            if wet && let Some(r) = self.grow_wet_soak(dt) {
                let p = &mut self.paint;
                for caixa in [
                    &mut p.wet_frame_dirty,
                    &mut p.wet_cum_dirty,
                    &mut p.wet_stroke_dirty,
                ] {
                    *caixa = Some(caixa.map_or(r, |c| union_region(c, r)));
                }
                stamped |= parked;
            }
        }
        self.paint.traco_relogio = relogio;
        stamped
    }
}
