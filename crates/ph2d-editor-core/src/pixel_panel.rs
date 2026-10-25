//! **O vocabulário do PIXEL ART que o chrome fala** (ADR-0192): o instantâneo do painel PIXEL, os
//! pedidos que ele faz e as escolhas do diálogo *New Sprite*. O painel e o diálogo não veem a família
//! (`ph2d-app-pixel`) nem o documento (`ph2d-pixel-core`); a família traduz.

/// A ferramenta na mão — as do Aseprite, na ordem da barra dele. A família reexporta-a como `Tool`
/// (um dono).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelTool {
    #[default]
    Pencil,
    Spray,
    Eraser,
    Eyedropper,
    /// O balde (*Paint Bucket*).
    Bucket,
    Line,
    Curve,
    Rectangle,
    FilledRectangle,
    Ellipse,
    FilledEllipse,
    /// Traço livre que FECHA e enche ao largar.
    Contour,
    /// Vértices por clique; fecha e enche.
    Polygon,
    /// A mão: arrastar move a vista (`Espaço` segurado faz o mesmo com qualquer ferramenta).
    Hand,
}

impl PixelTool {
    /// Todas, na ordem da coluna.
    pub const ALL: [Self; 14] = [
        Self::Pencil,
        Self::Spray,
        Self::Eraser,
        Self::Eyedropper,
        Self::Bucket,
        Self::Line,
        Self::Curve,
        Self::Rectangle,
        Self::FilledRectangle,
        Self::Ellipse,
        Self::FilledEllipse,
        Self::Contour,
        Self::Polygon,
        Self::Hand,
    ];

    /// A chave i18n do nome.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Pencil => "pixel.tool.pencil",
            Self::Spray => "pixel.tool.spray",
            Self::Eraser => "pixel.tool.eraser",
            Self::Eyedropper => "pixel.tool.eyedropper",
            Self::Bucket => "pixel.tool.bucket",
            Self::Line => "pixel.tool.line",
            Self::Curve => "pixel.tool.curve",
            Self::Rectangle => "pixel.tool.rectangle",
            Self::FilledRectangle => "pixel.tool.filled_rectangle",
            Self::Ellipse => "pixel.tool.ellipse",
            Self::FilledEllipse => "pixel.tool.filled_ellipse",
            Self::Contour => "pixel.tool.contour",
            Self::Polygon => "pixel.tool.polygon",
            Self::Hand => "pixel.tool.hand",
        }
    }

    /// O atalho por omissão (o do Aseprite; a mesma tecla de novo alterna a variante cheia).
    #[must_use]
    pub const fn shortcut(self) -> &'static str {
        match self {
            Self::Pencil => "B",
            Self::Spray => "Shift+B",
            Self::Eraser => "E",
            Self::Eyedropper => "I",
            Self::Bucket => "G",
            Self::Line => "L",
            Self::Curve => "Shift+L",
            Self::Rectangle | Self::FilledRectangle => "U",
            Self::Ellipse | Self::FilledEllipse => "Shift+U",
            Self::Contour => "D",
            Self::Polygon => "Shift+D",
            Self::Hand => "H",
        }
    }

    /// Deposita com o pincel (tamanho/forma/ângulo valem para ela).
    #[must_use]
    pub const fn uses_brush(self) -> bool {
        matches!(
            self,
            Self::Pencil
                | Self::Spray
                | Self::Eraser
                | Self::Line
                | Self::Curve
                | Self::Rectangle
                | Self::FilledRectangle
                | Self::Ellipse
                | Self::FilledEllipse
                | Self::Contour
                | Self::Polygon
        )
    }

    /// Pinta no documento (a simetria e o ladrilho valem para ela).
    #[must_use]
    pub const fn paints(self) -> bool {
        self.uses_brush() || matches!(self, Self::Bucket)
    }
}

/// A forma do pincel no vocabulário do chrome (a família traduz para o núcleo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelBrushShape {
    #[default]
    Circle,
    Square,
    Line,
}

impl PixelBrushShape {
    pub const ALL: [Self; 3] = [Self::Circle, Self::Square, Self::Line];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Circle => "panel.pixel.shape.circle",
            Self::Square => "panel.pixel.shape.square",
            Self::Line => "panel.pixel.shape.line",
        }
    }
}

/// O maior pincel — o `MAX_BRUSH` do `ph2d-pixel-core` (a família prova a igualdade).
pub const PIXEL_MAX_BRUSH: u32 = 64;

/// **As opções das ferramentas** (a barra de contexto do Aseprite): uma só por sessão, como no
/// Aseprite com *Same in all Tools*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelOptions {
    pub shape: PixelBrushShape,
    /// `1..=PIXEL_MAX_BRUSH`.
    pub size: u32,
    /// Graus, `-180..=180`, anti-horário.
    pub angle: i32,
    /// Só com pincel de 1 (Aseprite).
    pub pixel_perfect: bool,
    /// O raio do canto do retângulo (Aseprite 1.3.17).
    pub corner_radius: u32,
    /// `0..=255` (balde).
    pub tolerance: u8,
    pub contiguous: bool,
    /// Balde 8-conectado.
    pub eight: bool,
    /// O balde compara o que se VÊ (todas as camadas visíveis) em vez da camada activa.
    pub sample_visible: bool,
    /// O raio do jato do spray, em células.
    pub spray_width: u32,
    /// Pontos por jato.
    pub spray_speed: u32,
    /// Simetria: os espelhos (H = esquerda↔direita, V = cima↔baixo, as diagonais `\` e `/`).
    pub sym_h: bool,
    pub sym_v: bool,
    pub sym_d1: bool,
    pub sym_d2: bool,
    /// Tiled Mode por eixo.
    pub tile_x: bool,
    pub tile_y: bool,
}

impl Default for PixelOptions {
    /// As do Aseprite 1.3.18 ao abrir: pincel redondo de 1, pixel-perfect desligado, balde
    /// contíguo 4-conectado de tolerância 0 na camada activa.
    fn default() -> Self {
        Self {
            shape: PixelBrushShape::Circle,
            size: 1,
            angle: 0,
            pixel_perfect: false,
            corner_radius: 0,
            tolerance: 0,
            contiguous: true,
            eight: false,
            sample_visible: false,
            spray_width: 8,
            spray_speed: 8,
            sym_h: false,
            sym_v: false,
            sym_d1: false,
            sym_d2: false,
            tile_x: false,
            tile_y: false,
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
    pub options: PixelOptions,
}

/// Um gesto do painel PIXEL (`EditorAction::Pixel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelRequest {
    Tool(PixelTool),
    Fg([u8; 4]),
    Bg([u8; 4]),
    /// Troca frente e fundo (o `X` do Aseprite).
    Swap,
    /// As opções novas, inteiras (o painel escreve o campo que mudou sobre as que leu).
    Options(PixelOptions),
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
