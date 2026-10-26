//! As escolhas do diálogo *New Sprite* (P0).

/// O lado máximo de um sprite — o mesmo `MAX_SIDE` do `ph2d-pixel-core` (a família prova a
/// igualdade): o formato grava o lado em `u16`.
pub const NEW_SPRITE_MAX_SIDE: u32 = u16::MAX as u32;

/// O modo de cor do diálogo *New Sprite*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewSpriteMode {
    #[default]
    Rgba,
    Gray,
    Indexed,
}

/// O fundo do diálogo *New Sprite*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewSpriteBackground {
    #[default]
    Transparent,
    White,
    Black,
}

/// **As escolhas do diálogo *New Sprite*** — por omissão as do Aseprite: 32 × 32, RGBA, transparente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewSpriteChoice {
    pub width: u32,
    pub height: u32,
    pub mode: NewSpriteMode,
    pub background: NewSpriteBackground,
}

impl Default for NewSpriteChoice {
    fn default() -> Self {
        Self {
            width: 32,
            height: 32,
            mode: NewSpriteMode::Rgba,
            background: NewSpriteBackground::Transparent,
        }
    }
}

/// O estado do diálogo no `WidgetStore`: a última escolha (lembrada na sessão) e o pedido do OK, que
/// a shell drena. Cancelar é a AUSÊNCIA de pedido.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NewSpriteDialog {
    pub choice: NewSpriteChoice,
    pub(crate) request: Option<NewSpriteChoice>,
}

/// Um lado escrito no campo, como o documento o aceita: inteiro em `1..=NEW_SPRITE_MAX_SIDE`.
#[must_use]
pub fn side_of(v: f64) -> u32 {
    if !v.is_finite() {
        return 1;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "presa a 1..=65535 antes da conversão"
    )]
    let s = v.round().clamp(1.0, f64::from(NEW_SPRITE_MAX_SIDE)) as u32;
    s
}
