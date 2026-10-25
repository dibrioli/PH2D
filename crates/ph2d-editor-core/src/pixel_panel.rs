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
