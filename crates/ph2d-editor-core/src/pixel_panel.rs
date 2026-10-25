//! **O vocabulário do PIXEL ART que o chrome fala** (ADR-0192): o instantâneo do painel PIXEL, os
//! pedidos que ele faz e as escolhas do diálogo *New Sprite*. O painel e o diálogo não veem a família
//! (`ph2d-app-pixel`) nem o documento (`ph2d-pixel-core`); a família traduz.

/// A ferramenta na mão (Aseprite: `B`, `E`, `I`, `H`). A família reexporta-a como `Tool` — um dono.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelTool {
    #[default]
    Pencil,
    Eraser,
    Eyedropper,
    /// A mão: arrastar move a vista (`Espaço` segurado faz o mesmo com qualquer ferramenta).
    Hand,
}

impl PixelTool {
    /// As quatro, na ordem da barra.
    pub const ALL: [Self; 4] = [Self::Pencil, Self::Eraser, Self::Eyedropper, Self::Hand];

    /// A chave i18n do nome.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Pencil => "pixel.tool.pencil",
            Self::Eraser => "pixel.tool.eraser",
            Self::Eyedropper => "pixel.tool.eyedropper",
            Self::Hand => "pixel.tool.hand",
        }
    }

    /// O atalho por omissão (o do Aseprite).
    #[must_use]
    pub const fn shortcut(self) -> &'static str {
        match self {
            Self::Pencil => "B",
            Self::Eraser => "E",
            Self::Eyedropper => "I",
            Self::Hand => "H",
        }
    }
}

/// **O instantâneo do painel PIXEL**, publicado pela shell uma vez por quadro.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PixelPanelInfo {
    /// Há um Pixel Sprite em Edit (sem ele o painel só diz como entrar).
    pub editing: bool,
    pub tool: PixelTool,
    /// Cores de frente e de fundo, RGBA de alfa reto.
    pub fg: [u8; 4],
    pub bg: [u8; 4],
    /// A paleta do sprite em edição.
    pub palette: Vec<[u8; 4]>,
}

/// Um gesto do painel PIXEL (`EditorAction::Pixel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelRequest {
    Tool(PixelTool),
    Fg([u8; 4]),
    Bg([u8; 4]),
    /// Troca frente e fundo (o `X` do Aseprite).
    Swap,
}

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
