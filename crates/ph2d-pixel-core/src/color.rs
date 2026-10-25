//! **Cor**: RGB → cinza pelos três métodos do Aseprite 1.3 (Sprite ▸ Color Mode ▸ More Options ▸
//! Grayscale), medidos no Trial 1.3.18.6 por fotografia do ecrã, 0 erros em 216 cores cada
//! (`tests/fixtures/oracle_trial_color_mode.json`).

/// O método de RGB → cinza.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToGray {
    /// `(2126·R + 7152·G + 722·B) / 10000`, truncado (Rec. 709 sobre os bytes sRGB).
    #[default]
    Luminance,
    /// O V do HSV: o maior canal.
    Hsv,
    /// O L do HSL: `(maior + menor) / 2`, truncado.
    Hsl,
}

impl ToGray {
    pub const ALL: [ToGray; 3] = [ToGray::Luminance, ToGray::Hsv, ToGray::Hsl];

    /// O cinza de uma cor (o alfa não entra).
    #[must_use]
    pub fn of(self, c: [u8; 4]) -> u8 {
        let [r, g, b] = [c[0], c[1], c[2]].map(u32::from);
        match self {
            ToGray::Luminance => luma(c),
            ToGray::Hsv => r.max(g).max(b) as u8,
            ToGray::Hsl => ((r.max(g).max(b) + r.min(g).min(b)) / 2) as u8,
        }
    }
}

/// A luminância do Aseprite: `(2126·R + 7152·G + 722·B) / 10000`, truncada.
#[must_use]
pub fn luma(c: [u8; 4]) -> u8 {
    let [r, g, b] = [c[0], c[1], c[2]].map(u32::from);
    ((2126 * r + 7152 * g + 722 * b) / 10000) as u8
}
