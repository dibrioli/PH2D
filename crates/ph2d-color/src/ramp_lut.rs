//! **O DEGRADÊ ASSADO EM TABELA** — a lei que assa uma [`ColorRamp`] em `n` amostras uniformes, e a
//! régua que mede o erro da tabela no espaço em que a saída é GRAVADA (sRGB de 8 bits).
//!
//! Dois consumidores, cada um com o seu leitor: o Gradient Map do `fx.filter` (tabela num buffer,
//! interpolação à mão no shader) e a rampa do halo do `fx.glow` (textura `Rgba16Float` lida pelo
//! sampler). A tabela é a mesma lei; o que cada um mede é a SUA reconstrução.

use crate::srgb::linear_to_srgb_unit;
use crate::{ColorRamp, GradientPreset, RampColorMode, RampHue, RampInterp};

/// A lei da tabela: a amostra `k` é `eval(k / (n − 1))`, então as pontas são as paradas das pontas
/// e o índice contínuo de `t` é `t · (n − 1)`.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "n <= 4096")]
pub fn bake(ramp: &ColorRamp, n: usize) -> Vec<[f32; 4]> {
    (0..n)
        .map(|k| ramp.eval(k as f32 / (n - 1) as f32))
        .collect()
}

/// **A régua.** O erro de uma tabela é medido onde se vê: depois de cada canal passar a sRGB e ser
/// preso a `[0, 1]`, como a gravação o faz — perto do preto um erro linear de `1/255` são vários
/// degraus do ecrã.
pub mod regua {
    use super::{
        ColorRamp, GradientPreset, RampColorMode, RampHue, RampInterp, bake, linear_to_srgb_unit,
    };

    /// O passo da saída de 8 bits.
    pub const DISPLAY_STEP: f32 = 1.0 / 255.0;

    /// O erro de uma reconstrução linear é máximo no MEIO de uma célula: amostrar nos nós daria zero.
    pub const SWEEP: usize = 8192;

    /// O corpus SUAVE com o pior do editor dentro: os presets (todos `Linear`), o `Ease`, o
    /// `Cardinal` (que ultrapassa as paradas) e uma matiz que dá a volta LONGA em OKLCH.
    #[must_use]
    pub fn corpus() -> Vec<(String, ColorRamp)> {
        let mut out: Vec<(String, ColorRamp)> = GradientPreset::ALL
            .iter()
            .map(|p| (p.label_key().to_string(), p.ramp()))
            .collect();
        let mut eased = GradientPreset::Heat.ramp();
        eased.interp = RampInterp::Ease;
        out.push(("Heat/Ease".into(), eased));
        let mut cardinal = GradientPreset::Rainbow.ramp();
        cardinal.interp = RampInterp::Cardinal;
        out.push(("Rainbow/Cardinal".into(), cardinal));
        let mut longe = GradientPreset::Heat.ramp();
        longe.color_mode = RampColorMode::Oklch;
        longe.hue = RampHue::Far;
        out.push(("Heat/OKLCH far".into(), longe));
        out
    }

    /// A rampa em DEGRAU (`Constant`) — medida à parte: num salto o erro máximo é metade do salto
    /// a qualquer densidade, e o que a densidade encolhe é a LARGURA da banda errada.
    #[must_use]
    pub fn degrau() -> ColorRamp {
        let mut r = GradientPreset::Rainbow.ramp();
        r.interp = RampInterp::Constant;
        r
    }

    /// O pior erro no espaço da saída e a FRACÇÃO do percurso acima do passo do ecrã, para a tabela
    /// de `n` amostras lida por `le(tabela, t)` — a reconstrução do device que a lê.
    #[expect(clippy::cast_precision_loss, reason = "SWEEP <= 8192")]
    pub fn erro_no_ecra(
        ramp: &ColorRamp,
        n: usize,
        le: impl Fn(&[[f32; 4]], f32) -> [f32; 4],
    ) -> (f32, f32) {
        let lut = bake(ramp, n);
        let tela = |v: f32| linear_to_srgb_unit(v.clamp(0.0, 1.0));
        let (mut worst, mut bad) = (0.0f32, 0usize);
        for i in 0..=SWEEP {
            let t = i as f32 / SWEEP as f32;
            let (got, want) = (le(&lut, t), ramp.eval(t));
            let e = (0..3).fold(0.0f32, |m, c| m.max((tela(got[c]) - tela(want[c])).abs()));
            let e = e.max((got[3] - want[3]).abs());
            worst = worst.max(e);
            bad += usize::from(e > DISPLAY_STEP);
        }
        (worst, bad as f32 / (SWEEP + 1) as f32)
    }

    /// O pior erro do [`corpus`] inteiro.
    #[must_use]
    pub fn pior(n: usize, le: impl Fn(&[[f32; 4]], f32) -> [f32; 4] + Copy) -> f32 {
        corpus()
            .iter()
            .map(|(_, r)| erro_no_ecra(r, n, le).0)
            .fold(0.0, f32::max)
    }
}
