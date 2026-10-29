//! **A linha INATIVA** — o `active = False` do Blender: desenha-se com menos opacidade e continua editável. A lei
//! mudou-se do `ph2d-panel-painter-layers` (28/10) para o painel do Tuning do Wet Paint esmaecer com o MESMO alfa.

use ph2d_tokens::{ColorToken, Theme};

/// **O alfa da linha inativa — DERIVADO, não escolhido**: a fração que leva o `Text1` sobre a superfície do painel
/// (`Bg2`) à luminância do `TextDisabled`. Um tema que mude um dos três tokens muda o esmaecido com ele.
#[must_use]
pub fn alfa_inativo(theme: Theme) -> f32 {
    let l = |t: ColorToken| t.resolve(theme).relative_luminance();
    let (fundo, texto, apagado) = (
        l(ColorToken::Bg2),
        l(ColorToken::Text1),
        l(ColorToken::TextDisabled),
    );
    if (texto - fundo).abs() < f64::EPSILON {
        return 1.0;
    }
    (((apagado - fundo) / (texto - fundo)) as f32).clamp(0.0, 1.0)
}
