//! B22.1 (decisão 8, ritmo fixo): o que o Painter INTEGRA no tempo (a secagem do papel, o encharque sob
//! a ponta) anda em passos fixos de [`PASSO_FIXO_S`], não no `dt` do quadro — o mesmo gesto pelo mesmo
//! tempo de parede dá o mesmo byte a 30, 60 ou 144 Hz. O `wall_dt` partilhado do shell fica intacto (C9).
//! O passo é o de 60 Hz: a lei que o dono aprovou é a que 60 Hz pintava.

/// O passo fixo, em segundos.
pub(super) const PASSO_FIXO_S: f32 = 1.0 / 60.0;
/// Batente por quadro: 8 passos = 133 ms de quadro. Recurso: o passe de secagem a 4096² custa 3,45 ms
/// (`watercolor_dry.rs`, medido 2026-08-02) ⇒ ≤ 28 ms a pagar num quadro preso; o excesso dobra-se no
/// último passo (a lei de um `dt` grande, como antes) para um engasgo não virar espiral.
pub(super) const PASSOS_POR_QUADRO_MAX: u32 = 8;

/// Os `dt` a correr neste quadro: `relogio` (em passos) recebe `dt_s` e devolve os passos inteiros.
pub(super) fn passos_fixos(relogio: &mut f64, dt_s: f32) -> impl Iterator<Item = f32> {
    *relogio += f64::from(dt_s.max(0.0)) / f64::from(PASSO_FIXO_S);
    // A folga absorve o arredondamento do `dt` em f32 (16,666666 ms ≠ 1/60 s ao último bit).
    let n = (*relogio + 1e-6).floor().max(0.0) as u32;
    *relogio -= f64::from(n);
    let m = n.min(PASSOS_POR_QUADRO_MAX);
    (0..m).map(move |i| {
        if i + 1 == m {
            PASSO_FIXO_S * (n - m + 1) as f32
        } else {
            PASSO_FIXO_S
        }
    })
}

impl super::PainterTool {
    /// O encharque sob a ponta em passos fixos ([`Self::grow_wet_soak`] por passo; a caixa é a união).
    pub(super) fn grow_wet_soak_fixo(&mut self, dt_s: f32) -> Option<super::Region> {
        if self.paint.wet_soak_pos.is_none() || self.paint.brush.wet_rewet <= 0.0 {
            self.paint.wet_soak_relogio = 0.0;
            return None;
        }
        let mut relogio = self.paint.wet_soak_relogio;
        let mut caixa = None;
        for dt in passos_fixos(&mut relogio, dt_s) {
            if let Some(r) = self.grow_wet_soak(dt) {
                caixa = Some(caixa.map_or(r, |c| super::union_region(c, r)));
            }
        }
        self.paint.wet_soak_relogio = relogio;
        caixa
    }
}
