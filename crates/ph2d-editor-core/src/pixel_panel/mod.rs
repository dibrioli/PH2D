//! **O vocabulário do PIXEL ART que o chrome fala** (ADR-0192): o instantâneo do painel PIXEL, os
//! pedidos que ele faz e as escolhas do diálogo *New Sprite*. O painel e o diálogo não veem a família
//! (`ph2d-app-pixel`) nem o documento (`ph2d-pixel-core`); a família traduz.

mod layers;
mod new_sprite;
mod selection;
mod shapes;
mod timeline;
mod tools;

pub use shapes::{
    PixelShape, PixelShapeValues, PixelShapes, SHAPE_COUNT, field_name, set_field_names,
    set_shape_names,
};

pub use layers::*;
pub use new_sprite::*;
pub use selection::*;
pub use timeline::*;
pub use tools::*;

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
    /// P6b: os valores das formas do Vector (o raio do canto do retângulo é o 1.º campo dele, o do
    /// Aseprite 1.3.17) e se as quatro enchem (P6c: a caixa «Fill» no lugar das ferramentas cheias).
    pub shapes: PixelShapes,
    pub shape_fill: bool,
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
    /// O seletor de cor por baixo da paleta (o menu ≡ da paleta do Aseprite).
    pub selector: PixelSelector,
    /// P6: o modo das ferramentas de selecção, a varinha (tolerância `0..=255`, contígua, 8-conectada)
    /// e a amostragem da moldura solta.
    pub select_mode: PixelSelectMode,
    pub wand_tolerance: u8,
    pub wand_contiguous: bool,
    pub wand_eight: bool,
    pub resample: PixelResample,
}

/// O seletor de cor abaixo da paleta — os cinco do menu ≡ da paleta do Aseprite 1.3.18, na ordem
/// dele.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelSelector {
    /// Quadrado saturação × brilho na matiz actual, barra de matiz.
    #[default]
    TintShadeTone,
    /// Matiz × claridade, barra de saturação.
    Spectrum,
    RgbWheel,
    RybWheel,
    NormalMap,
}

impl PixelSelector {
    pub const ALL: [Self; 5] = [
        Self::TintShadeTone,
        Self::Spectrum,
        Self::RgbWheel,
        Self::RybWheel,
        Self::NormalMap,
    ];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::TintShadeTone => "panel.pixel.selector.tint",
            Self::Spectrum => "panel.pixel.selector.spectrum",
            Self::RgbWheel => "panel.pixel.selector.rgb_wheel",
            Self::RybWheel => "panel.pixel.selector.ryb_wheel",
            Self::NormalMap => "panel.pixel.selector.normal_map",
        }
    }
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
    pub(crate) edit: PixelEditDialog,
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
            shapes: PixelShapes::default(),
            shape_fill: false,
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
            selector: PixelSelector::TintShadeTone,
            select_mode: PixelSelectMode::Replace,
            wand_tolerance: 0,
            wand_contiguous: true,
            wand_eight: false,
            resample: PixelResample::Fast,
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
    /// Há selecção (o *Copy* e o *Layer via Copy/Cut* acendem).
    pub has_selection: bool,
    /// As paletas gravadas com *Save Palette as Preset* (os nomes dos ficheiros, por ordem).
    pub presets: Vec<String>,
    /// A linha do tempo (o painel PIXEL TIMELINE): frames na ordem, o activo, os cels, as tags.
    pub frames: Vec<PixelFrameCol>,
    pub active_frame: Option<u32>,
    pub cels: Vec<PixelCel>,
    pub tags: Vec<PixelTagInfo>,
    /// A animação está a tocar / o papel vegetal está ligado (e com a tinta vermelha/azul).
    pub playing: bool,
    pub onion: bool,
    pub onion_tint: bool,
    /// P6: o tamanho do sprite (os diálogos abrem com ele), a moldura solta (os seus campos só
    /// existem com células levantadas) e se há o que o Reselect devolve.
    pub size: (u32, u32),
    pub transform: Option<PixelTransformInfo>,
    pub can_reselect: bool,
    /// P6b/P6c: a cara de cada botão de grupo e o flyout aberto.
    pub groups: PixelGroups,
    /// P6d: há uma forma VIVA à espera do `Enter` (a dica ensina as alças).
    pub live_shape: bool,
}

/// Um gesto do painel PIXEL (`EditorAction::Pixel`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PixelRequest {
    Tool(PixelTool),
    /// Um botão de grupo: a última usada na mão, e o flyout abre/fecha.
    Group(PixelToolGroup),
    Fg([u8; 4]),
    Bg([u8; 4]),
    /// Troca frente e fundo (o `X` do Aseprite).
    Swap,
    /// As opções novas, inteiras (o painel escreve o campo que mudou sobre as que leu).
    Options(Box<PixelOptions>),
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
    /// *Load Default Palette*: a gravada por *Save Palette as Default*, ou a DB32 de fábrica.
    DefaultPalette,
    /// Arrastar a amostra `from` para a célula `to`: a selecção (ou só ela, se não estava
    /// seleccionada) muda-se para lá; a imagem não muda.
    DragSwatch {
        from: usize,
        to: usize,
    },
    /// *Save Palette as Preset*: grava a paleta do sprite na pasta das predefinições.
    SavePreset,
    /// *Save Palette as Default*: a paleta com que nasce um Pixel Sprite novo.
    SaveDefault,
    /// Carrega a predefinição gravada com este nome (como o *Load Palette*).
    LoadPreset(String),
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
    /// Layer Properties ▸ Continuous.
    LayerContinuous(u32),
    /// Clique num cabeçalho de frame / num cel: o frame activo (e a camada activa).
    FrameSelect(u32),
    CelSelect {
        layer: u32,
        frame: u32,
    },
    FrameAction(PixelFrameAction),
    /// Frame Properties ▸ Duration do activo / *Constant Frame Rate* (todos).
    FrameDuration(u32),
    ConstantFrameRate(u32),
    /// Cel Properties do cel activo.
    CelOpacity(u8),
    CelZ(i16),
    /// Tag Properties da tag `index`.
    TagDirection {
        index: usize,
        direction: PixelDirection,
    },
    TagRepeat {
        index: usize,
        repeat: u16,
    },
    TagRange {
        index: usize,
        from: u32,
        to: u32,
    },
    TagRename {
        index: usize,
        name: String,
    },
    TagDelete(usize),
    /// P6: os menus Select, Edit e Sprite, e um campo da moldura solta.
    Select(PixelSelectAction),
    Edit(PixelEditAction),
    Sprite(PixelSpriteAction),
    TransformField {
        field: PixelTransformField,
        value: i32,
    },
}
