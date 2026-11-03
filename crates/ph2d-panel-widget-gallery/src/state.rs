//! Widget Gallery panel state — owned by `ErasedPanel<WidgetGalleryPanel>` (ADR-0029 Phase C.3).
//!
//! - `rect` — a janela flutuante de hoje (`PH2D_CARDS=0`), criada no primeiro quadro.
//! - `choices` — o que os cartões mostram e que NÃO é número do store: os interruptores
//!   ([`Flag`]), as escolhas ([`Pick`]) e as cores. O paint lê; o evento escreve.
//! - `applied` — os valores de PARTIDA: o Apply grava-os, o Cancel e o ↺ voltam a eles.

use crate::ids as g;
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
    FillStyles,
    StrokeStyles,
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
}
const N_FLAGS: usize = Flag::FxEye2 as usize + 1;

/// Os que nascem ligados.
const FLAGS_ON: [Flag; 13] = [
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
];

/// As linhas de cada lista («+» liga a primeira livre, «−» desliga a dela).
pub const FILLS: [Flag; 3] = [Flag::Fill0, Flag::Fill1, Flag::Fill2];
pub const STROKES: [Flag; 3] = [Flag::Stroke0, Flag::Stroke1, Flag::Stroke2];
pub const FXS: [Flag; 3] = [Flag::Fx0, Flag::Fx1, Flag::Fx2];

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
}
const N_PICKS: usize = Pick::Format as usize + 1;

/// O valor de [`Pick::Boolean`] depois de achatar: nenhum modo aceso.
pub const BOOLEAN_NONE: u8 = u8::MAX;

/// Quantas amostras de cor o objeto fictício tem (`controlos::SWATCHES`).
pub const N_SWATCHES: usize = 9;

/// As escolhas vivas dos cartões. `Default` = os valores de fábrica.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choices {
    flags: [bool; N_FLAGS],
    picks: [u8; N_PICKS],
    /// A cor escolhida em cada amostra; `None` = a de fábrica (um token da tinta do cartão).
    pub colors: [Option<[u8; 4]>; N_SWATCHES],
}

impl Default for Choices {
    fn default() -> Self {
        let mut c = Self {
            flags: [false; N_FLAGS],
            picks: [0; N_PICKS],
            colors: [None; N_SWATCHES],
        };
        for f in FLAGS_ON {
            c.set(f, true);
        }
        c.choose(Pick::ResizeH, 1);
        c.choose(Pick::Grid, 4);
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
    /// A corrente do Scale: os `(X, Y)` vistos no último quadro (`None` = armar de novo).
    pub link: Option<(f64, f64)>,
}
