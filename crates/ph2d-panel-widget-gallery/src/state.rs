//! Widget Gallery panel state — owned by `ErasedPanel<WidgetGalleryPanel>` (ADR-0029 Phase C.3).
//!
//! - `rect` — a janela flutuante de hoje (`PH2D_CARDS=0`), criada no primeiro quadro.
//! - `choices` — o que os cartões mostram e que NÃO é número do store: os interruptores
//!   ([`Flag`]), as escolhas ([`Pick`]) e as cores. O paint lê; o evento escreve.
//! - `applied` — os valores de PARTIDA: o Apply grava-os, o Cancel e o ↺ voltam a eles.

use crate::ids as g;
use ph2d_editor_core::panel::cards::estilos::Estilo;
use ph2d_editor_core::zones::Rect;

/// Os interruptores do objeto fictício — índice de [`Choices`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Flag {
    Visible,
    Locked,
    Converted,
    ScaleLink,
    CornerEach,
    Clip,
    Physics,
    Fill0,
    Fill1,
    Fill2,
    FillEye0,
    FillEye1,
    FillEye2,
    Stroke0,
    Stroke1,
    Stroke2,
    StrokeEye0,
    StrokeEye1,
    StrokeEye2,
    Fx0,
    Fx1,
    Fx2,
    FxEye0,
    FxEye1,
    FxEye2,
    // Os casos reais.
    RbLock,
    ColTrigger,
    HpOverheal,
    CamPicking,
    CamActive,
    CamLook,
    AnimPlaying,
    Rule0,
    Rule1,
    Rule2,
    BoneLimit,
    BoneSmart,
    BoneIk,
    ConsRow,
    ConsEye,
    ConsPicking,
    LampLimit,
    LampShadows,
    Rack0,
    Rack1,
    Rack2,
    RackEye0,
    RackEye1,
    RackEye2,
    AudioMute,
    PixelGrid,
    ScriptEdit,
    ScriptDouble,
    // As amostras do Rive.
    Input0,
    Input1,
    Input2,
    InputBool0,
    InputBool1,
    InputBool2,
    Stop0,
    Stop1,
    Stop2,
    // O cartão Advanced.
    Meta0,
    Meta1,
    Meta2,
    Meta3,
    Meta4,
    Meta5,
    Prop0,
    Prop1,
    Prop2,
}
const N_FLAGS: usize = Flag::Prop2 as usize + 1;

/// Os que nascem ligados.
const FLAGS_ON: &[Flag] = &[
    Flag::Visible,
    Flag::Fill0,
    Flag::FillEye0,
    Flag::FillEye1,
    Flag::FillEye2,
    Flag::Stroke0,
    Flag::StrokeEye0,
    Flag::StrokeEye1,
    Flag::StrokeEye2,
    Flag::Fx0,
    Flag::FxEye0,
    Flag::FxEye1,
    Flag::FxEye2,
    Flag::CamActive,
    Flag::Rule0,
    Flag::Rule1,
    Flag::BoneLimit,
    Flag::BoneIk,
    Flag::ConsRow,
    Flag::ConsEye,
    Flag::LampShadows,
    Flag::Rack0,
    Flag::RackEye0,
    Flag::RackEye1,
    Flag::RackEye2,
    Flag::PixelGrid,
    Flag::Input0,
    Flag::Input1,
    Flag::Input2,
    Flag::Stop0,
    Flag::Stop1,
    Flag::Meta0,
    Flag::Meta1,
    Flag::Meta2,
    Flag::Prop0,
];

/// As linhas de cada lista («+» liga a primeira livre, «−» desliga a dela).
pub const FILLS: [Flag; 3] = [Flag::Fill0, Flag::Fill1, Flag::Fill2];
pub const STROKES: [Flag; 3] = [Flag::Stroke0, Flag::Stroke1, Flag::Stroke2];
pub const FXS: [Flag; 3] = [Flag::Fx0, Flag::Fx1, Flag::Fx2];
pub const RULES: [Flag; 3] = [Flag::Rule0, Flag::Rule1, Flag::Rule2];
pub const RACKS: [Flag; 3] = [Flag::Rack0, Flag::Rack1, Flag::Rack2];

/// As escolhas de uma-entre-várias (segmentos e listas suspensas) — índice de [`Choices`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Pick {
    Boolean,
    Direction,
    ResizeW,
    ResizeH,
    Grid,
    ConsH,
    ConsV,
    Blend,
    StrokePos,
    Cap,
    Join,
    Fx0,
    Fx1,
    Fx2,
    Font,
    FontWeight,
    TextAlign,
    ExportScale,
    Format,
    /// A linha cujas propriedades abriram ao lado (`SIDE_NONE` = nenhuma): `0..3` Effects, `3..6` o
    /// rack do Audio.
    SideOpen,
    // Os casos reais.
    ImgAsset,
    ImgUse,
    RbBody,
    ColShape,
    CamFormat,
    CamFrame,
    AnimWho,
    AnimClip,
    RuleWhen0,
    RuleWhen1,
    RuleWhen2,
    RulePlay0,
    RulePlay1,
    RulePlay2,
    BrushPreset,
    PaintType,
    BoneBend,
    ConsKind,
    ConsSpace,
    LampKind,
    Rack0,
    Rack1,
    Rack2,
    PixelTarget,
    PixelFg,
    PixelBg,
    ScriptFile,
    // As amostras do Rive.
    Interp,
    Input0,
    Input1,
    Input2,
    TrimMode,
    Solo,
    FillType,
    GradStop,
    // As peças do cromo.
    BarTool,
    ListRow,
    // Os cartões Advanced e Render.
    Meta0,
    Meta1,
    Meta2,
    Meta3,
    Meta4,
    Meta5,
    SliceMode,
    RenderFormat,
    RenderQuality,
}
const N_PICKS: usize = Pick::RenderQuality as usize + 1;

/// O valor de [`Pick::Boolean`] depois de achatar: nenhum modo aceso.
pub const BOOLEAN_NONE: u8 = u8::MAX;

/// O valor de [`Pick::SideOpen`] com o painel do lado fechado.
pub const SIDE_NONE: u8 = u8::MAX;

/// Quantas amostras de cor o objeto fictício tem (`controlos::SWATCHES`).
pub const N_SWATCHES: usize = 19;

/// As escolhas vivas dos cartões. `Default` = os valores de fábrica.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choices {
    flags: [bool; N_FLAGS],
    picks: [u8; N_PICKS],
    /// A cor escolhida em cada amostra; `None` = a de fábrica (um token da tinta do cartão).
    pub colors: [Option<[u8; 4]>; N_SWATCHES],
    /// O estilo de cor a que cada amostra está ligada (`None` = cor solta).
    pub linked: [Option<u32>; N_SWATCHES],
    /// As 32 camadas de colisão do cartão Advanced: o bit `n` é a camada `n + 1`.
    pub layers: u32,
}

/// As camadas de colisão de fábrica.
const LAYERS_DE_FABRICA: u32 = 0b1011_0101;

impl Default for Choices {
    fn default() -> Self {
        let mut c = Self {
            flags: [false; N_FLAGS],
            picks: [0; N_PICKS],
            colors: [None; N_SWATCHES],
            linked: [None; N_SWATCHES],
            layers: LAYERS_DE_FABRICA,
        };
        for f in FLAGS_ON {
            c.set(*f, true);
        }
        c.choose(Pick::ResizeH, 1);
        c.choose(Pick::Grid, 4);
        c.choose(Pick::SideOpen, SIDE_NONE);
        c.choose(Pick::AnimWho, 2);
        c.choose(Pick::RuleWhen1, 1);
        c.choose(Pick::RulePlay1, 1);
        c.choose(Pick::PixelBg, 15);
        c.choose(Pick::Interp, 1);
        c.choose(Pick::Input1, 1);
        c.choose(Pick::Input2, 2);
        c.choose(Pick::FillType, 1);
        c.choose(Pick::Meta1, 1);
        c.choose(Pick::Meta2, 2);
        c.choose(Pick::SliceMode, 1);
        c.choose(Pick::RenderQuality, 1);
        // A lâmpada de fábrica é branca (`Luz::default`).
        c.colors[9] = Some([u8::MAX; 4]);
        c
    }
}

impl Choices {
    #[must_use]
    pub fn on(&self, f: Flag) -> bool {
        self.flags[f as usize]
    }
    pub fn set(&mut self, f: Flag, v: bool) {
        self.flags[f as usize] = v;
    }
    #[must_use]
    pub fn pick(&self, p: Pick) -> u8 {
        self.picks[p as usize]
    }
    pub fn choose(&mut self, p: Pick, i: u8) {
        self.picks[p as usize] = i;
    }
}

/// Um retrato dos valores: as escolhas, os números dos campos (pela ordem de [`g::FIELDS`]) e os
/// textos (pela ordem de [`g::TEXTS`]). `Default` = a fábrica.
#[derive(Clone, Debug, PartialEq)]
pub struct Values {
    pub choices: Choices,
    pub numbers: [f64; g::FIELDS.len()],
    pub texts: [String; g::TEXTS.len()],
}

impl Default for Values {
    fn default() -> Self {
        Self {
            choices: Choices::default(),
            numbers: g::FIELDS.map(|f| f.1),
            texts: g::TEXTS.map(|(_, key)| ph2d_i18n::tr(key).to_owned()),
        }
    }
}

/// Widget Gallery retained state.
#[derive(Clone, Debug, Default)]
pub struct WidgetGalleryState {
    /// Rect of the floating gallery (`PH2D_CARDS=0`), lazily set on first paint.
    pub rect: Option<Rect>,
    pub choices: Choices,
    /// Os valores do último Apply (a fábrica antes do primeiro).
    pub applied: Values,
    /// O Export correu (o texto de estado do cartão); não conta como mudança por aplicar.
    pub exported: bool,
    /// A corrente do Scale: a proporção guardada (`None` = aberta, ou fecha no próximo quadro).
    pub link: Option<ph2d_editor_core::par_travado::ParTravado>,
    /// Os estilos de cor fictícios (nascem no 1.º quadro, das cores da tinta).
    pub estilos: Vec<Estilo>,
    /// A lista de estilos estava aberta no quadro anterior: os pedidos dela são desta galeria.
    pub styles_open: bool,
    pub styles_seeded: bool,
    /// As vezes que cada entrada Trigger disparou, e quando (o botão pisca).
    pub fires: [u32; 3],
    pub fired_at: [Option<std::time::Instant>; 3],
    /// O deslocamento já visto de cada pega da curva (`cards_rive::sync_pegas`).
    pub pegas: [(f32, f32); 2],
    /// A corrida fictícia do cartão Render.
    pub render: crate::cards_render::Corrida,
}
