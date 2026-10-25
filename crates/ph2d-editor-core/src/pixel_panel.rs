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

    /// A tecla do atalho por omissão (a do Aseprite; a mesma tecla de novo alterna a variante cheia).
    #[must_use]
    pub const fn shortcut(self) -> &'static str {
        match self {
            Self::Pencil | Self::Spray => "B",
            Self::Eraser => "E",
            Self::Eyedropper => "I",
            Self::Bucket => "G",
            Self::Line | Self::Curve => "L",
            Self::Rectangle | Self::FilledRectangle | Self::Ellipse | Self::FilledEllipse => "U",
            Self::Contour | Self::Polygon => "D",
            Self::Hand => "H",
        }
    }

    /// O atalho leva `Shift` (a sub-ferramenta do Aseprite).
    #[must_use]
    pub const fn shifted(self) -> bool {
        matches!(
            self,
            Self::Spray | Self::Curve | Self::Ellipse | Self::FilledEllipse | Self::Polygon
        )
    }

    /// O atalho como o artista o lê: «B», «Shift+B».
    #[must_use]
    pub fn shortcut_label(self) -> String {
        if self.shifted() {
            ph2d_i18n::tr_with("pixel.shortcut_shift", &[("key", &self.shortcut())])
        } else {
            self.shortcut().to_owned()
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

/// O maior tamanho de paleta — o `MAX_COLORS` do `ph2d-pixel-core` (a família prova a igualdade).
pub const PIXEL_MAX_COLORS: u32 = 256;

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
    /// A tinta das ferramentas que pintam (a borracha pinta sempre com a `Simple`).
    pub ink: PixelInk,
    /// `0..=255`: a tolerância do *Replace Color* (por canal RGBA).
    pub replace_tolerance: u8,
}

/// A tinta (*Ink* do Aseprite) no vocabulário do chrome; a família traduz para o núcleo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelInk {
    #[default]
    Simple,
    AlphaCompositing,
    CopyAlphaColor,
    LockAlpha,
    /// Anda um degrau na rampa = as cores SELECCIONADAS na paleta.
    Shading,
}

impl PixelInk {
    pub const ALL: [Self; 5] = [
        Self::Simple,
        Self::AlphaCompositing,
        Self::CopyAlphaColor,
        Self::LockAlpha,
        Self::Shading,
    ];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Simple => "panel.pixel.ink.simple",
            Self::AlphaCompositing => "panel.pixel.ink.alpha",
            Self::CopyAlphaColor => "panel.pixel.ink.copy",
            Self::LockAlpha => "panel.pixel.ink.lock",
            Self::Shading => "panel.pixel.ink.shading",
        }
    }
}

/// O critério do *Sort* da paleta (a ordem do menu do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelSortKey {
    #[default]
    Hue,
    Saturation,
    Brightness,
    Luminance,
    Red,
    Green,
    Blue,
    Alpha,
}

impl PixelSortKey {
    pub const ALL: [Self; 8] = [
        Self::Hue,
        Self::Saturation,
        Self::Brightness,
        Self::Luminance,
        Self::Red,
        Self::Green,
        Self::Blue,
        Self::Alpha,
    ];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Hue => "panel.pixel.sort.hue",
            Self::Saturation => "panel.pixel.sort.saturation",
            Self::Brightness => "panel.pixel.sort.brightness",
            Self::Luminance => "panel.pixel.sort.luminance",
            Self::Red => "panel.pixel.sort.red",
            Self::Green => "panel.pixel.sort.green",
            Self::Blue => "panel.pixel.sort.blue",
            Self::Alpha => "panel.pixel.sort.alpha",
        }
    }
}

/// Como um clique esquerdo numa amostra da paleta mexe na selecção (sempre põe a cor de frente).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelSelect {
    /// Só esta.
    Only,
    /// `Shift`: da âncora até esta.
    Range,
    /// `Ctrl`: liga/desliga esta.
    Toggle,
}

/// O dither da conversão para Indexed — a lista do diálogo *Color Mode* do Aseprite 1.3.18.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelDither {
    #[default]
    None,
    Ordered2,
    Ordered4,
    Ordered8,
    Old2,
    Old4,
    Old8,
    FloydSteinberg,
}

/// O método de RGB → cinza do diálogo *Color Mode*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelToGray {
    #[default]
    Luminance,
    Hsv,
    Hsl,
}

/// **As escolhas do diálogo *Color Mode*** (Sprite ▸ Color Mode ▸ More Options).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ColorModeChoice {
    pub target: NewSpriteMode,
    pub dither: PixelDither,
    pub gray: PixelToGray,
}

/// O estado do diálogo *Color Mode*: a última escolha, o modo do sprite ao abrir (o alvo que não se
/// escolhe) e o pedido do OK.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColorModeDialog {
    pub choice: ColorModeChoice,
    pub current: NewSpriteMode,
    pub(crate) request: Option<ColorModeChoice>,
}

/// Os diálogos do Pixel Art no `WidgetStore`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PixelDialogs {
    pub(crate) new_sprite: NewSpriteDialog,
    pub(crate) color_mode: ColorModeDialog,
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
            ink: PixelInk::Simple,
            replace_tolerance: 0,
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
    /// O modo de cor do sprite em edição.
    pub mode: NewSpriteMode,
    /// As amostras seleccionadas, em ordem de paleta (a rampa do Shading).
    pub selected: Vec<usize>,
    /// A última amostra clicada: a que o *Edit color* edita.
    pub anchor: Option<usize>,
    pub sort_key: PixelSortKey,
    /// As camadas na ordem da lista (de CIMA para baixo); os filhos de um grupo recolhido não vêm.
    pub layers: Vec<PixelLayerRow>,
    /// A camada activa (a que as ferramentas pintam).
    pub active_layer: Option<u32>,
    /// Há uma imagem copiada (o *Paste as New Layer* acende).
    pub has_clip: bool,
}

/// Uma linha da lista de camadas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelLayerRow {
    pub id: u32,
    pub name: String,
    /// Quantos grupos a contêm (o recuo da linha).
    pub depth: u8,
    pub group: bool,
    pub collapsed: bool,
    pub visible: bool,
    pub locked: bool,
    pub reference: bool,
    pub background: bool,
    pub opacity: u8,
    pub blend: PixelBlend,
}

/// O modo de mistura no vocabulário do chrome (os 19 do Aseprite, na ordem dos ids do `.ase`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelBlend {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    Addition,
    Subtract,
    Divide,
}

impl PixelBlend {
    pub const ALL: [Self; 19] = [
        Self::Normal,
        Self::Multiply,
        Self::Screen,
        Self::Overlay,
        Self::Darken,
        Self::Lighten,
        Self::ColorDodge,
        Self::ColorBurn,
        Self::HardLight,
        Self::SoftLight,
        Self::Difference,
        Self::Exclusion,
        Self::Hue,
        Self::Saturation,
        Self::Color,
        Self::Luminosity,
        Self::Addition,
        Self::Subtract,
        Self::Divide,
    ];

    /// Os grupos do menu (escurecer, clarear, contraste, diferença, cor), na ordem do Aseprite.
    pub const GROUPS: [&'static [Self]; 6] = [
        &[Self::Normal],
        &[Self::Darken, Self::Multiply, Self::ColorBurn],
        &[Self::Lighten, Self::Screen, Self::ColorDodge, Self::Addition],
        &[Self::Overlay, Self::SoftLight, Self::HardLight],
        &[Self::Difference, Self::Exclusion, Self::Subtract, Self::Divide],
        &[Self::Hue, Self::Saturation, Self::Color, Self::Luminosity],
    ];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Normal => "panel.pixel.blend.normal",
            Self::Multiply => "panel.pixel.blend.multiply",
            Self::Screen => "panel.pixel.blend.screen",
            Self::Overlay => "panel.pixel.blend.overlay",
            Self::Darken => "panel.pixel.blend.darken",
            Self::Lighten => "panel.pixel.blend.lighten",
            Self::ColorDodge => "panel.pixel.blend.color_dodge",
            Self::ColorBurn => "panel.pixel.blend.color_burn",
            Self::HardLight => "panel.pixel.blend.hard_light",
            Self::SoftLight => "panel.pixel.blend.soft_light",
            Self::Difference => "panel.pixel.blend.difference",
            Self::Exclusion => "panel.pixel.blend.exclusion",
            Self::Hue => "panel.pixel.blend.hue",
            Self::Saturation => "panel.pixel.blend.saturation",
            Self::Color => "panel.pixel.blend.color",
            Self::Luminosity => "panel.pixel.blend.luminosity",
            Self::Addition => "panel.pixel.blend.addition",
            Self::Subtract => "panel.pixel.blend.subtract",
            Self::Divide => "panel.pixel.blend.divide",
        }
    }
}

/// As acções do menu Layer (e do Edit que fazem camadas).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelLayerAction {
    NewLayer,
    NewGroup,
    Delete,
    Duplicate,
    /// Um degrau acima/abaixo entre irmãos.
    MoveUp,
    MoveDown,
    MergeDown,
    Flatten,
    FlattenVisible,
    /// *Layer via Copy* / *via Cut* (sem selecção: o cel inteiro do frame).
    ViaCopy,
    ViaCut,
    /// *Edit ▸ Copy* do cel activo.
    Copy,
    PasteAsLayer,
    PasteAsReference,
}

/// Um gesto do painel PIXEL (`EditorAction::Pixel`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PixelRequest {
    Tool(PixelTool),
    Fg([u8; 4]),
    Bg([u8; 4]),
    /// Troca frente e fundo (o `X` do Aseprite).
    Swap,
    /// As opções novas, inteiras (o painel escreve o campo que mudou sobre as que leu).
    Options(PixelOptions),
    /// Clique esquerdo na amostra `index`: a cor de frente e a selecção.
    Swatch {
        index: usize,
        how: PixelSelect,
    },
    /// *Palette Size*: corta ou estende com preto.
    PaletteSize(u16),
    SortKey(PixelSortKey),
    /// Ordena a selecção (2+ amostras) ou a paleta inteira; a imagem não muda.
    Sort {
        descending: bool,
    },
    /// Desloca as amostras seleccionadas um lugar (`-1` esquerda, `+1` direita); a imagem não muda.
    MoveSelection(i8),
    /// *Select ▸ Used Colors* (Indexed).
    SelectUsed,
    /// *New Palette from Sprite*.
    PaletteFromSprite,
    LoadPalette,
    SavePalette,
    /// *Load Default Palette* (DB32).
    DefaultPalette,
    /// *Edit ▸ Replace Color*: a cor de frente vira a de fundo no cel activo.
    ReplaceColor,
    /// Clique no nome: a camada activa.
    LayerSelect(u32),
    /// O olho / o cadeado / o triângulo do grupo.
    LayerVisible(u32),
    LayerLocked(u32),
    LayerCollapsed(u32),
    /// *Solo*: só esta camada se vê (outra vez: todas voltam como estavam).
    LayerSolo(u32),
    /// As propriedades da camada activa.
    LayerOpacity(u8),
    LayerBlend(PixelBlend),
    LayerAction(PixelLayerAction),
    /// O nome novo (o campo do duplo clique; vazio = fica o antigo).
    LayerRename {
        id: u32,
        name: String,
    },
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
