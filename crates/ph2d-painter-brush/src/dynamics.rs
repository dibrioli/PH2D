//! Pressure dynamics — how pen pressure drives the dab size and coverage.
//!
//! Behavioural reference (clean-room, no code copied): Blender maps pen pressure onto brush
//! parameters via the per-channel pressure toggles shown in the brush popover (the pencil icons
//! next to Radius/Strength). Each enabled channel scales linearly from a per-channel minimum at
//! pressure 0 up to the full value at pressure 1. A mouse (no pressure) reports `1.0`, so dynamics
//! are a no-op there.

/// Which brush channels respond to pen pressure, and how far they fall at zero pressure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dynamics {
    /// Radius follows pressure.
    pub size_pressure: bool,
    /// Coverage (per-dab opacity) follows pressure.
    pub strength_pressure: bool,
    /// Radius multiplier at pressure 0 when [`Self::size_pressure`], `0..1`.
    pub size_min: f32,
    /// Coverage multiplier at pressure 0 when [`Self::strength_pressure`], `0..1`.
    pub strength_min: f32,
    /// The coverage CURVE: the pressure is raised to it before the ramp (`1` = linear, the law of
    /// always, bit for bit; `> 1` fades faster at a light touch — prompt 45).
    pub strength_curve: f32,
    /// The radius CURVE — the SAME law as [`Self::strength_curve`] ([`curve`]), on the radius ramp (W6).
    pub size_curve: f32,
    /// The pen before the curves: the floor/ceiling of the hand, the smoothing, the pulled string
    /// ([`crate::pen`]). [`crate::Pen::default`] = the hand's pressure as it comes, bit for bit.
    pub pen: crate::Pen,
    /// **A velocidade ESTREITA a marca** (L12, `docs/Painter/Pinceis/06_LEIS_DAS_LACUNAS.md` §10.4): o raio ×
    /// `(v / v₀)^−speed_size` acima de [`VELOCIDADE_DE_REFERENCIA_PX_S`]. `0` = a velocidade não conta (o
    /// Rebelle: 221 presets neutros), ao bit; Fineliner do Krita ≈ 0,06 · pen do MyPaint ≈ 0,14 · Marker 0,45.
    pub speed_size: f32,
    /// A mesma lei na cobertura do dab (o Thin_Pen do MyPaint: só a cobertura cai, 0,57 a 5 000 px/s ≈ 0,2).
    pub speed_opacity: f32,
    /// ⭐ **A pressão LEVE espalha** (J3 canetas 2.ª onda; a esferográfica do MyPaint `deevad/ballpen`, CC0:
    /// `offset_by_random` 0,354 na pressão 0 → 0 na 1): o raio do disco de Jitter SOMADO ao do pincel, em
    /// diâmetros do pincel (sem a pressão), na pressão 0 — e cai em recta até 0 na pressão 1. `0` = nada, ao bit.
    pub jitter_pressure: f32,
    /// ⭐ **A velocidade com ATRASO** (J4 efeitos; o `speed1_slowness` do libmypaint, ISC — 0,04 s em 164 dos 196
    /// pincéis do MyPaint): a velocidade que a L12 lê é a da mão passada por um filtro de 1.ª ordem com esta constante
    /// de tempo (s), e o traço NASCE a `v = 0` — largo no arranque e a afinar à medida que a mão acelera (a cabeça do
    /// traço rápido do `experimental/glow`). `0` = a velocidade medida desde o 1.º dab (a lei da J3), ao bit.
    pub speed_lag_s: f32,
    /// ⭐ **A velocidade AMACIA a queda** (J4 efeitos 2.ª ronda; o `hardness ← speed1` do libmypaint, ISC, no
    /// `experimental/glow`: 0,24 a mão parada · 0,13 a 300 px/s · 0,05 a 2 500): a dureza do dab soma
    /// `speed_hardness · ln((Γ + v₀)/(Γ + v))` ([`Self::dureza_da_velocidade`]) — o caroço onde a caneta pousa (o
    /// traço nasce a `v = 0` com o [`Self::speed_lag_s`]) e o rápido mais fino. `0` = a dureza do pincel, ao bit.
    pub speed_hardness: f32,
    /// ⭐ **A pressão ENDURECE a queda** (J4 efeitos 2.ª ronda; o `hardness ← pressure` do libmypaint: a esferográfica
    /// `deevad/ballpen` vai de 0,2 a 0,5): a dureza do dab soma `pressure_hardness · p` — o traço cheio de topo
    /// chato (sem contas a 200 px), o leve macio. `0` = a dureza do pincel, ao bit.
    pub pressure_hardness: f32,
}

impl Default for Dynamics {
    /// Size follows pressure (the iconic pen behaviour), coverage does not. `size_min = 0.1` keeps
    /// a faint mark at the lightest touch instead of vanishing.
    fn default() -> Self {
        Self {
            size_pressure: true,
            strength_pressure: false,
            size_min: 0.1,
            strength_min: 0.0,
            strength_curve: 1.0,
            size_curve: 1.0,
            pen: crate::Pen::default(),
            speed_size: 0.0,
            speed_opacity: 0.0,
            jitter_pressure: 0.0,
            speed_lag_s: 0.0,
            speed_hardness: 0.0,
            pressure_hardness: 0.0,
        }
    }
}

/// The pressure CURVE of a channel: `p` raised to `g` (`1` — or a non-positive/NaN `g` — is `p` itself, bit
/// for bit, without a `powf`). ONE law for the radius and the coverage.
#[inline]
#[must_use]
pub fn curve(pressure: f32, g: f32) -> f32 {
    if g == 1.0 || g.is_nan() || g <= 0.0 {
        pressure
    } else {
        pressure.clamp(0.0, 1.0).powf(g)
    }
}

#[inline]
fn ramp(enabled: bool, min: f32, pressure: f32) -> f32 {
    if enabled {
        let p = pressure.clamp(0.0, 1.0);
        let min = min.clamp(0.0, 1.0);
        min + (1.0 - min) * p
    } else {
        1.0
    }
}

impl Dynamics {
    /// Radius multiplier for a given pressure in `[0, 1]`.
    #[must_use]
    pub fn radius_scale(&self, pressure: f32) -> f32 {
        ramp(
            self.size_pressure,
            self.size_min,
            curve(pressure, self.size_curve),
        )
    }

    /// Coverage multiplier for a given pressure in `[0, 1]`.
    #[must_use]
    pub fn coverage_scale(&self, pressure: f32) -> f32 {
        ramp(
            self.strength_pressure,
            self.strength_min,
            curve(pressure, self.strength_curve),
        )
    }
}

/// A velocidade abaixo da qual a marca é a do pincel (px/s) — a mais LENTA que a bateria mede (3 px por
/// evento de 10 ms, `06` §10.4); todas as curvas medidas (Krita, MyPaint) normalizam-se a ela.
pub const VELOCIDADE_DE_REFERENCIA_PX_S: f32 = 300.0;

/// O teto do expoente da velocidade: o Marker do MyPaint (`w ∝ v^−0,45`, o mais forte medido) com margem.
pub const VELOCIDADE_EXPOENTE_MAX: f32 = 1.0;

/// O `Γ` da curva da dureza pela velocidade (px/s): o `e^speed1_gamma` do `experimental/glow` (`e^2,87`), o
/// único pincel medido que a pede — onde a curva logarítmica dele dobra.
pub const DUREZA_GAMA_PX_S: f32 = 17.6;

/// ⭐ **A lei da velocidade** (L12): `(v / v₀)^−p` acima de `v₀`, `1` abaixo dele e com `p = 0` (sem `powf`).
/// UMA potência serve as quatro curvas medidas — Fineliner `p 0,058` (0,96 · 0,92 · 0,88 · 0,85 contra 0,95 ·
/// 0,95 · 0,90 · 0,85), pen `0,136`, pen-note `0,18`, Marker `0,45` —, e nunca chega a zero (a log-linear sim).
#[inline]
#[must_use]
pub fn lei_da_velocidade(p: f32, v_px_s: f32) -> f32 {
    if p <= 0.0 || v_px_s <= VELOCIDADE_DE_REFERENCIA_PX_S || !v_px_s.is_finite() {
        return 1.0;
    }
    (v_px_s / VELOCIDADE_DE_REFERENCIA_PX_S).powf(-p.min(VELOCIDADE_EXPOENTE_MAX))
}

impl Dynamics {
    /// O raio (px) do disco de Jitter que a pressão `pressure` soma num pincel de `diametro_px` ([`Self::jitter_pressure`]).
    #[must_use]
    pub fn jitter_da_pressao(&self, pressure: f32, diametro_px: f32) -> f32 {
        if self.jitter_pressure <= 0.0 {
            return 0.0;
        }
        self.jitter_pressure.min(1.0) * (1.0 - pressure.clamp(0.0, 1.0)) * diametro_px.max(0.0)
    }

    /// Alguma lei da velocidade está ligada (só então o traço mede a velocidade de cada dab).
    #[must_use]
    pub fn speed_armed(&self) -> bool {
        self.speed_size > 0.0 || self.speed_opacity > 0.0 || self.speed_hardness > 0.0
    }

    /// A dureza que a velocidade `v` (px/s) SOMA à do pincel ([`Self::speed_hardness`]): a curva logarítmica do
    /// MyPaint (`m·ln(Γ + v) + q`, o `speed1` dele), zero em [`VELOCIDADE_DE_REFERENCIA_PX_S`] — o pincel calibra-se
    /// no traço lento —, positiva abaixo (a mão a pousar) e negativa acima.
    #[must_use]
    pub fn dureza_da_velocidade(&self, v: f32) -> f32 {
        if self.speed_hardness <= 0.0 || !v.is_finite() {
            return 0.0;
        }
        let k = self.speed_hardness.min(VELOCIDADE_EXPOENTE_MAX);
        k * ((DUREZA_GAMA_PX_S + VELOCIDADE_DE_REFERENCIA_PX_S) / (DUREZA_GAMA_PX_S + v.max(0.0))).ln()
    }

    /// A dureza que a MÃO soma à do pincel neste dab: a da pressão `p` ([`Self::pressure_hardness`]) e a da
    /// velocidade `v` ([`Self::dureza_da_velocidade`]). `0` sem as duas linhas.
    #[must_use]
    pub fn dureza_da_mao(&self, p: f32, v: f32) -> f32 {
        let pressao = if self.pressure_hardness > 0.0 {
            self.pressure_hardness.min(1.0) * p.clamp(0.0, 1.0)
        } else {
            0.0
        };
        pressao + self.dureza_da_velocidade(v)
    }

    /// O multiplicador do raio à velocidade `v` (px/s).
    #[must_use]
    pub fn speed_radius_scale(&self, v: f32) -> f32 {
        lei_da_velocidade(self.speed_size, v)
    }

    /// O multiplicador da cobertura à velocidade `v` (px/s).
    #[must_use]
    pub fn speed_coverage_scale(&self, v: f32) -> f32 {
        lei_da_velocidade(self.speed_opacity, v)
    }
}

#[cfg(test)]
mod tests {
    use super::Dynamics;

    #[test]
    fn mouse_pressure_is_noop() {
        let d = Dynamics {
            size_pressure: true,
            strength_pressure: true,
            ..Default::default()
        };
        assert!((d.radius_scale(1.0) - 1.0).abs() < 1e-6);
        assert!((d.coverage_scale(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn disabled_channel_is_full() {
        let d = Dynamics {
            size_pressure: false,
            strength_pressure: false,
            size_min: 0.1,
            strength_min: 0.1,
            ..Default::default()
        };
        assert!((d.radius_scale(0.0) - 1.0).abs() < 1e-6);
        assert!((d.coverage_scale(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn size_min_floor_at_zero_pressure() {
        let d = Dynamics {
            size_pressure: true,
            size_min: 0.25,
            ..Default::default()
        };
        assert!((d.radius_scale(0.0) - 0.25).abs() < 1e-6);
        assert!((d.radius_scale(0.5) - 0.625).abs() < 1e-6); // 0.25 + 0.75*0.5
    }

    /// The coverage curve (prompt 45): `1` is the linear ramp bit for bit; `2` squares the pressure before
    /// the ramp; it never touches the radius.
    #[test]
    fn the_coverage_curve_shapes_the_pressure_only() {
        let lin = Dynamics {
            strength_pressure: true,
            strength_min: 0.2,
            ..Default::default()
        };
        let sq = Dynamics {
            strength_curve: 2.0,
            ..lin
        };
        for p in [0.0, 0.15, 0.46, 0.5, 1.0] {
            assert_eq!(
                Dynamics {
                    strength_curve: 1.0,
                    ..lin
                }
                .coverage_scale(p)
                .to_bits(),
                (0.2 + 0.8 * p).to_bits()
            );
            assert!((sq.coverage_scale(p) - (0.2 + 0.8 * p * p)).abs() < 1e-6);
            assert_eq!(sq.radius_scale(p).to_bits(), lin.radius_scale(p).to_bits());
        }
    }

    /// ⭐ A dureza pela velocidade é a curva do MyPaint `experimental/glow` (`hardness ← speed1`, `k` 0,04 × 0,939):
    /// a dureza dele 0,238 a mão parada, 0,129 a 300 px/s e 0,051 a 2 500 (medido nos dabs do libmypaint) — a
    /// diferença à de 300 px/s, ± 0,005; e `0` sem a linha.
    #[test]
    fn a_dureza_da_velocidade_e_a_curva_do_mypaint() {
        let d = Dynamics { speed_hardness: 0.04 * 0.939, ..Dynamics::default() };
        for (v, mypaint) in [(0.0, 0.238), (300.0, 0.129), (2500.0, 0.051)] {
            let dh = d.dureza_da_velocidade(v);
            assert!((dh - (mypaint - 0.129)).abs() < 0.005, "v {v}: {dh} × {}", mypaint - 0.129);
        }
        assert_eq!(Dynamics::default().dureza_da_velocidade(0.0), 0.0);
    }
}
