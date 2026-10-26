//! O vocabulário das CAMADAS do Pixel (P3): a linha da lista, o modo de mistura e as acções do menu
//! Layer.

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
        &[
            Self::Lighten,
            Self::Screen,
            Self::ColorDodge,
            Self::Addition,
        ],
        &[Self::Overlay, Self::SoftLight, Self::HardLight],
        &[
            Self::Difference,
            Self::Exclusion,
            Self::Subtract,
            Self::Divide,
        ],
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
    /// *Layer via Copy* / *via Cut*: as células seleccionadas numa camada nova (pedem selecção,
    /// como no Aseprite). O Copy e o Select All/Deselect são os menus Edit e Select (P6).
    ViaCopy,
    ViaCut,
    PasteAsLayer,
    PasteAsReference,
}
