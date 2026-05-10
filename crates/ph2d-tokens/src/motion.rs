//! Motion tokens. Source: `docs/design/tokens.json` → `motion.*`.
//!
//! Easing como cubic-bezier control points (4 floats); duration em ms
//! como `f32`. Vello/animação é responsabilidade do consumidor — este
//! módulo só fornece os parâmetros canônicos.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Easing {
    /// `out` — cubic-bezier(0.2, 0.7, 0.1, 1). Padrão para entrada de
    /// elementos / transições single-shot.
    Out,
    /// `inout` — cubic-bezier(0.4, 0.0, 0.2, 1). Material standard.
    /// Usado em transições reversíveis (toggle, abertura/fechamento).
    InOut,
    /// `spring` — cubic-bezier(0.34, 1.56, 0.64, 1). Overshoot leve;
    /// para feedback tátil de press / drop.
    Spring,
}

impl Easing {
    /// 4 control points (x1, y1, x2, y2) of the cubic-bezier curve.
    /// Compatible com CSS `cubic-bezier(...)` e parley/vello custom
    /// easing implementations.
    pub const fn bezier(self) -> [f32; 4] {
        match self {
            Self::Out => [0.2, 0.7, 0.1, 1.0],
            Self::InOut => [0.4, 0.0, 0.2, 1.0],
            Self::Spring => [0.34, 1.56, 0.64, 1.0],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Duration {
    /// `instant` — 80 ms (state flicker, hover feedback).
    Instant,
    /// `fast` — 150 ms (button press, icon swap).
    Fast,
    /// `default` — 240 ms (panel open/close, page transitions).
    Default,
    /// `slow` — 400 ms (hero animations, onboarding).
    Slow,
}

impl Duration {
    pub const fn ms(self) -> f32 {
        match self {
            Self::Instant => 80.0,
            Self::Fast => 150.0,
            Self::Default => 240.0,
            Self::Slow => 400.0,
        }
    }

    pub const fn secs(self) -> f32 {
        self.ms() / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_strictly_increasing() {
        assert!(Duration::Instant.ms() < Duration::Fast.ms());
        assert!(Duration::Fast.ms() < Duration::Default.ms());
        assert!(Duration::Default.ms() < Duration::Slow.ms());
    }

    #[test]
    fn easing_bezier_in_unit_range_for_x() {
        // x1 e x2 (índices 0 e 2) DEVEM estar em [0, 1] para cubic-bezier
        // monotônico em CSS. y pode passar (spring overshoot).
        for ease in [Easing::Out, Easing::InOut, Easing::Spring] {
            let [x1, _y1, x2, _y2] = ease.bezier();
            assert!((0.0..=1.0).contains(&x1), "{ease:?} x1={x1}");
            assert!((0.0..=1.0).contains(&x2), "{ease:?} x2={x2}");
        }
    }

    #[test]
    fn spring_overshoots_y() {
        // Spring deve ter y1 > 1.0 (passa do destino para voltar).
        let [_, y1, _, _] = Easing::Spring.bezier();
        assert!(y1 > 1.0, "spring y1 = {y1}, expected > 1.0");
    }
}
