//! O **estilo do traço** — irmão de `lib.rs` pelo teto de 700 LOC daquele arquivo.
//!
//! A caneta: largura, ponta, junção, tracejado e as PONTAS (arrowheads). O que ela
//! DESENHA com isso é outra pergunta, e mora em [`crate::stroke_plan`].

use serde::{Deserialize, Serialize};

use crate::{Marker, Rgba8};

/// Ponta do traço (mapeia p/ `kurbo::Cap` no render). Default = `Butt`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// Junção entre segmentos (mapeia p/ `kurbo::Join`). Default = `Miter`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// **Qual contorno o Offset Path move** — num compound (uma forma com furos), a borda de fora
/// e as bordas dos furos são coisas separadas, e o artista pode querer mover só uma.
///
/// O offset é **por contorno**: `d > 0` empurra o contorno escolhido para FORA (ao longo da
/// sua normal externa, para longe do miolo que ele fecha); `d < 0` para dentro. Assim a quina
/// (Round/Bevel) aparece em QUALQUER contorno que se expanda — não só no de fora, que era a
/// queixa do smoke (`2026-07-20`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OffsetSide {
    /// Só o contorno de fora. `d > 0` cresce a forma.
    Outer,
    /// Só os contornos de furo. `d > 0` cresce os furos (para dentro da tinta).
    Inner,
    /// Todos os contornos. `d > 0` expande cada um pela sua normal externa.
    #[default]
    Both,
}

impl OffsetSide {
    /// Este modo move o contorno de FORA?
    #[must_use]
    pub fn hits_outer(self) -> bool {
        matches!(self, OffsetSide::Outer | OffsetSide::Both)
    }

    /// Este modo move os contornos de FURO?
    #[must_use]
    pub fn hits_inner(self) -> bool {
        matches!(self, OffsetSide::Inner | OffsetSide::Both)
    }
}

/// Estilo do traço de um path: cor + largura (world-units) + ponta/junção +
/// tracejado opcional. Substitui a tupla `(Rgba8, f64)` da Fase 0.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrokeSpec {
    pub color: Rgba8,
    pub width: f64,
    pub cap: LineCap,
    pub join: LineJoin,
    /// Tracejado como **múltiplos da largura**: `Some((dash, gap))` ⇒ traço de
    /// `dash·width` e vão de `gap·width`; `None` (ou `dash ≤ 0`) = contínuo.
    /// Width-aware: engrossar o traço alonga dash e vão na proporção, então a
    /// projeção da ponta nunca engole o vão.
    pub dash: Option<(f64, f64)>,
    /// **Ponta no COMEÇO** do caminho (arrowhead). Só vale em caminho aberto.
    /// Apendado por ÚLTIMO: o postcard é posicional, então um save anterior a este campo
    /// segue legível e lê `Marker::None` (que é o default — uma linha é uma linha).
    #[serde(default)]
    pub marker_start: Marker,
    /// **Ponta no FIM** do caminho. Idem.
    #[serde(default)]
    pub marker_end: Marker,
    /// **Tamanho da ponta**, como múltiplo do tamanho que a largura do traço já dita.
    /// `1.0` = o default.
    ///
    /// Por que um MÚLTIPLO e não um tamanho absoluto: a ponta tem de crescer com o traço,
    /// senão engrossar a linha faz a seta encolher visualmente até virar um cotoco. O
    /// multiplicador dá o ajuste fino sem quebrar essa proporção.
    #[serde(default = "unit_scale")]
    pub marker_scale: f64,
    /// **Arredondamento das quinas da ponta**: `0` = afiada, `1` = o máximo que a geometria
    /// da ponta comporta sem se descaracterizar.
    #[serde(default)]
    pub marker_round: f64,
}

/// O default do [`StrokeSpec::marker_scale`]. Precisa ser uma função porque o default de
/// `serde` para um `f64` é **zero** — e uma ponta de tamanho zero é uma ponta invisível.
fn unit_scale() -> f64 {
    1.0
}

impl StrokeSpec {
    /// Traço sólido, ponta/junção default (Butt/Miter), sem tracejado, sem setas.
    #[must_use]
    pub fn new(color: Rgba8, width: f64) -> Self {
        Self {
            color,
            width,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            dash: None,
            marker_start: Marker::None,
            marker_end: Marker::None,
            marker_scale: 1.0,
            marker_round: 0.0,
        }
    }

    /// `true` se o traço tem alguma ponta — o render então precisa recuar a linha.
    #[must_use]
    pub fn has_markers(&self) -> bool {
        self.marker_start != Marker::None || self.marker_end != Marker::None
    }

    /// **O tracejado em COMPRIMENTO** — `[traço, vão]` no mesmo espaço da geometria, ou
    /// `None` para linha contínua.
    ///
    /// O campo [`Self::dash`] guarda MÚLTIPLOS da largura (engrossar o traço alonga os dois
    /// na proporção, então a projeção da ponta nunca engole o vão); quem desenha ou assa
    /// precisa dos comprimentos. A conversão é UMA — o renderer e o Outline Stroke falam com
    /// versões diferentes da kurbo e cada um constrói o próprio `Stroke`, mas os dois têm de
    /// concordar sobre **quanto mede um traço**, senão a forma assada sai com o tracejado
    /// noutra cadência que a desenhada.
    ///
    /// O vão é afastado do zero: um vão de comprimento nulo é um elemento degenerado para a
    /// kurbo, e o que o usuário quis dizer com ele é "sólido".
    #[must_use]
    pub fn dash_lengths(&self) -> Option<[f64; 2]> {
        match self.dash {
            Some((d, g)) if d > 0.0 => Some([d * self.width, (g * self.width).max(f64::EPSILON)]),
            _ => None,
        }
    }
}
