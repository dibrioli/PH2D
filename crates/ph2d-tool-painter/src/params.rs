//! Painter host params — `PainterParams`.
//!
//! The painting/brush vocabulary (`PainterUiEdit` / `PainterUiSnapshot` /
//! `BrushParam` / `BrushStudioSnapshot` / size-px maps / color stubs) was
//! deleted with the brush engine. What remains is the takeover-mode flag the
//! layers + effects host needs: when the tool is active it suppresses the
//! normal PH2D chrome (ADR-0043 §1.1).

use serde::{Deserialize, Serialize};

/// Serializable state of the Painter (layers + effects) host.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PainterParams {
    /// When `true` the tool's takeover UI is installed (suppresses the normal
    /// PH2D chrome; ADR-0043 §1.1). Set in `Tool::on_activate`, cleared in
    /// `on_deactivate`.
    pub takeover_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn painter_params_default_is_inactive() {
        assert!(!PainterParams::default().takeover_active);
    }
}
