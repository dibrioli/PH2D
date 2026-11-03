//! Widget Gallery panel state — owned by `ErasedPanel<WidgetGalleryPanel>` (ADR-0029 Phase C.3).
//!
//! - `rect` — a janela flutuante de hoje (`PH2D_CARDS=0`), criada no primeiro quadro.
//! - `choices` — o que os cartões do vocabulário mostram e que NÃO é número do store (o segmento
//!   aceso, a caixa de marcar, a lista suspensa, a secção presente, a cor do Fill). O paint lê; o
//!   evento escreve.
//! - `applied` — os valores de PARTIDA: o Apply grava-os, o Cancel e o ↺ voltam a eles.

use crate::ids as g;
use ph2d_editor_core::zones::Rect;

/// As escolhas vivas dos cartões do vocabulário. `Default` = os valores de fábrica.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choices {
    /// Low · Mid · High (0..=2).
    pub level: u8,
    /// Butt · Round · Square (0..=2).
    pub cap: u8,
    pub visible: bool,
    pub locked: bool,
    pub snap: bool,
    /// Normal · Multiply · Screen (0..=2).
    pub blend: u8,
    pub fill_visible: bool,
    /// O Fill existe (o «−» tira-o; o «+» do cartão vazio devolve-o — à Figma).
    pub fill: bool,
    /// A cor escolhida no seletor; `None` = a de fábrica (o `Accent` da tinta do cartão).
    pub fill_rgba: Option<[u8; 4]>,
    /// A secção Physics existe (o «+» do cartão ausente cria-a).
    pub physics: bool,
}

impl Default for Choices {
    fn default() -> Self {
        Self {
            level: 0,
            cap: 0,
            visible: true,
            locked: false,
            snap: false,
            blend: 0,
            fill_visible: true,
            fill: true,
            fill_rgba: None,
            physics: false,
        }
    }
}

/// Um retrato dos valores do vocabulário: as escolhas, os números dos campos (pela ordem de
/// [`g::FIELDS`]) e o texto do Name. `Default` = a fábrica.
#[derive(Clone, Debug, PartialEq)]
pub struct Values {
    pub choices: Choices,
    pub numbers: [f64; g::FIELDS.len()],
    /// O texto do campo Name ([`g::NAME`]); a fábrica é o `name_default` da tabela.
    pub name: String,
}

impl Default for Values {
    fn default() -> Self {
        Self {
            choices: Choices::default(),
            numbers: g::FIELDS.map(|f| f.1),
            name: ph2d_i18n::tr("panel.widget_gallery.name_default").to_owned(),
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
}
