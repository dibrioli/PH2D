//! ⭐⭐⭐ **COMO A PEÇA É PINTADA** — o matcap (a luz do OLHO) ou o render (material, lâmpadas e céu) —
//! e o **olhar** da cena (`docs/Render3d/05`).
//!
//! # Duas perguntas, dois donos — os mesmos do Blender
//!
//! | o quê | de quem | porquê |
//! |---|---|---|
//! | [`Shading`] | de cada **viewport** | quatro vistas podem mostrar coisas diferentes — medir em matcap numa ortográfica, ver o material na perspectiva (o *Viewport Shading* do Blender é por área) |
//! | a exposição e a vista ([`Look`]) | da **cena** | é a *Color Management* do Blender: a mesma luz tem de chegar igual a todas as vistas, matcap incluído |
//!
//! ⚠️ **Os dois são VISTA e não documento** (`docs/Render3d/04` §4): sobrevivem a fechar o painel
//! (o [`crate::view::View`]) e não entram no undo nem no arquivo. Passam a documento no dia em que
//! houver uma saída de render que os grave.
//!
//! ⚠️ **A omissão é o que o módulo sempre mostrou** — matcap, `0` stops, `Standard` —, e com ela o
//! quadro é o de antes, byte a byte.

use ph2d_panel_model3d::ModeChip;
use ph2d_view_transform::{Look, ViewTransform};

/// **Como um viewport pinta a peça.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Shading {
    /// A fotografia de luz presa ao olho — lê **forma**, e é a omissão de um modelador.
    #[default]
    Matcap,
    /// O material sob as lâmpadas e o céu, com o olhar da cena.
    Render,
}

impl Shading {
    /// Todos, na ordem da fileira.
    pub const ALL: [Self; 2] = [Self::Matcap, Self::Render];

    /// A chave i18n do chip.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Matcap => "panel.model3d.shading.matcap",
            Self::Render => "panel.model3d.shading.render",
        }
    }
}

/// A chave i18n do chip de uma vista.
#[must_use]
pub const fn look_key(view: ViewTransform) -> &'static str {
    match view {
        ViewTransform::Standard => "panel.model3d.look.standard",
        ViewTransform::Neutral => "panel.model3d.look.neutral",
    }
}

/// **As exposições que a fileira oferece**, em stops, com a chave de cada chip — um par, e não duas
/// listas, para a contagem ser uma só.
///
/// ⚠️ **Cinco stops inteiros, e o alcance não é um tecto:** a lei aceita qualquer exposição finita
/// ([`ph2d_view_transform::exposure_scale`]). `±2` é o que o smoke do plano pede (*«a mesma peça com
/// exposição −2, 0, +2»*); um controlo contínuo é trabalho de painel com alcance medido, e fica
/// nomeado no doc da fatia.
pub const EXPOSURES: [(f32, &str); 5] = [
    (-2.0, "panel.model3d.exposure.minus2"),
    (-1.0, "panel.model3d.exposure.minus1"),
    (0.0, "panel.model3d.exposure.zero"),
    (1.0, "panel.model3d.exposure.plus1"),
    (2.0, "panel.model3d.exposure.plus2"),
];

/// O olhar com a exposição do chip `slot` — ou o mesmo olhar, se o slot não existe.
#[must_use]
pub fn with_exposure(look: Look, slot: usize) -> Look {
    EXPOSURES.get(slot).map_or(look, |(stops, _)| Look {
        exposure_stops: *stops,
        ..look
    })
}

/// O olhar com a vista do chip `slot` — ou o mesmo olhar, se o slot não existe.
#[must_use]
pub fn with_view(look: Look, slot: usize) -> Look {
    ViewTransform::ALL.get(slot).map_or(look, |view| Look {
        view: *view,
        ..look
    })
}

/// ⭐ **A fileira do modo**, com o aceso **derivado do estado** — nunca de um botão guardado.
#[must_use]
pub fn shading_chips(current: Shading) -> Vec<ModeChip> {
    Shading::ALL
        .iter()
        .map(|s| ModeChip {
            key: s.key(),
            active: *s == current,
        })
        .collect()
}

/// ⭐ **A fileira das vistas da cena**, derivada de [`ViewTransform::ALL`].
#[must_use]
pub fn look_chips(look: Look) -> Vec<ModeChip> {
    ViewTransform::ALL
        .iter()
        .map(|v| ModeChip {
            key: look_key(*v),
            active: *v == look.view,
        })
        .collect()
}

/// ⭐ **A fileira das exposições**, derivada de [`EXPOSURES`].
///
/// ⚠️ A comparação é EXACTA de propósito: a exposição da cena só é escrita a partir desta mesma
/// tabela, então um valor que não bata num chip é um valor que nenhum chip escreveu — e nenhum chip
/// acender é a resposta verdadeira.
#[must_use]
pub fn exposure_chips(look: Look) -> Vec<ModeChip> {
    EXPOSURES
        .iter()
        .map(|(stops, key)| ModeChip {
            key,
            active: stops.to_bits() == look.exposure_stops.to_bits(),
        })
        .collect()
}

#[cfg(test)]
#[path = "shading_tests.rs"]
mod tests;
