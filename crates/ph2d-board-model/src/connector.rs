//! **A seta do quadro** (W2): a RELAÇÃO entre duas pontas e os pontos por onde o artista a fez
//! passar — a geometria não se guarda, é derivada (`ph2d-board-route`) e refaz-se quando a seta ou
//! uma das suas formas mexe. ⛔ Nunca por causa de OUTRA forma (ordem do dono, 06/10: «setas não se
//! reajustam sozinhas», o idioma do Miro).
//!
//! ⚠️ postcard é posicional: variante NOVA só no fim de cada enum.

use serde::{Deserialize, Serialize};

use crate::{ElementId, Style};

/// Uma ponta da seta.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum End {
    /// Solta, num ponto do mundo.
    Free([f64; 2]),
    /// Presa a uma forma.
    Bound { target: ElementId, anchor: Anchor },
}

/// Como a ponta presa acha o sítio onde encosta (a semântica do Miro, pesquisa 01 §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Anchor {
    /// Ao CENTRO: a linha sai pela face que olha para a outra ponta, re-escolhida quando as formas
    /// mexem, e nunca atravessa a forma.
    Center,
    /// Num PONTO FIXO `[u, v]` da caixa da forma antes de rodar (`0..1`, o `fixedPoint` do
    /// Excalidraw): roda e escala com ela, e a linha sai sempre dali.
    Fixed([f64; 2]),
    /// ⭐ **Em ÓRBITA do foco `[u, v]`** (W9, o `bindMode "orbit"` do Excalidraw, lei XB1 do
    /// [estudo 07](../../../docs/MiroClone/07_fidelidade_excalidraw.md) §2): a ponta fica na recta
    /// entre os DOIS focos, onde ela entra na faixa de [`ORBIT_GAP`] `+ traço da forma / 2` à volta
    /// do contorno — PÁRA antes da forma. Foco fora da forma e longe dela = a ponta é o foco.
    Orbit([f64; 2]),
}

/// ⭐ O vão entre a ponta de uma seta em [`Anchor::Orbit`] e a forma, ANTES de somar meio traço da
/// forma (XB1, as duas versões do oráculo): `5 + strokeWidth/2` ⇒ 5,5 · 6 · 7 para o traço 1 · 2 · 4.
pub const ORBIT_GAP: f64 = 5.0;

/// Como a rota se desenha.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    /// Segmentos rectos pelos pontos do artista.
    Straight,
    /// Só segmentos horizontais e verticais (contorna só as suas duas formas).
    Elbow,
    /// A curva suave do Miro, que sai e entra perpendicular às formas e passa pelos pontos do
    /// artista — a de nascença (ordem do dono, 06/10).
    #[default]
    Curved,
    /// ⭐ **A curva do EXCALIDRAW pelos pontos** (W9, lei XA1): Catmull-Rom uniforme com pegas de
    /// 1/6 — com só as duas pontas é RECTA. É a seta `roundness` que vem de um `.excalidraw` (a curva
    /// DELE pelos pontos DELE); a do quadro continua [`Route::Curved`] (ordem do dono, 10/10).
    Spline,
}

/// O desenho numa ponta (o catálogo de pontas do vectorial, `ph2d_vec_scene::Marker`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Head {
    None,
    /// Duas riscas em «V» — a seta de nascença do Excalidraw (`endArrowhead: "arrow"`).
    Arrow,
    Triangle,
    Diamond,
    DiamondOpen,
    Circle,
    CircleOpen,
    Bar,
    /// O triângulo VAZADO (`triangle_outline`, W9).
    TriangleOpen,
    /// As pontas de CARDINALIDADE dos diagramas entidade-relação (o pé-de-galinha do Excalidraw,
    /// `cardinality_*`, W9): um · muitos · um ou muitos · exactamente um · zero ou um · zero ou muitos.
    One,
    Many,
    OneOrMany,
    ExactlyOne,
    ZeroOrOne,
    ZeroOrMany,
}

impl Head {
    /// Todas, na ordem do enum. (A barra oferece as suas, `HEADS`; as outras entram pelo ficheiro.)
    pub const ALL: [Head; 15] = [
        Head::None,
        Head::Arrow,
        Head::Triangle,
        Head::Diamond,
        Head::DiamondOpen,
        Head::Circle,
        Head::CircleOpen,
        Head::Bar,
        Head::TriangleOpen,
        Head::One,
        Head::Many,
        Head::OneOrMany,
        Head::ExactlyOne,
        Head::ZeroOrOne,
        Head::ZeroOrMany,
    ];
}

/// Uma seta.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Connector {
    pub start: End,
    pub end: End,
    pub route: Route,
    /// `[início, fim]`.
    pub heads: [Head; 2],
    /// O traço (`stroke`, espessura, tracejado, opacidade) e a letra do rótulo; `fill` não se usa.
    pub style: Style,
    /// O rótulo, a meio da rota. Vazio = sem rótulo.
    pub label: String,
    /// Os pontos (mundo) por onde o artista fez a seta passar, do início para o fim — os «pontos de
    /// ajuste» do Miro. Vazio = a rota de nascença entre as pontas.
    pub waypoints: Vec<[f64; 2]>,
}

impl Connector {
    /// As pontas de nascença — as do Excalidraw (oráculo: `startArrowhead: null`,
    /// `endArrowhead: "arrow"`): nada no início, «V» no fim.
    pub const DEFAULT_HEADS: [Head; 2] = [Head::None, Head::Arrow];

    /// Uma seta nova com as pontas de nascença.
    #[must_use]
    pub fn new(start: End, end: End, route: Route, style: Style) -> Self {
        Self {
            start,
            end,
            route,
            heads: Self::DEFAULT_HEADS,
            style,
            label: String::new(),
            waypoints: Vec::new(),
        }
    }

    /// As duas pontas `[início, fim]`.
    #[must_use]
    pub fn ends(&self) -> [End; 2] {
        [self.start, self.end]
    }

    pub fn end_mut(&mut self, which: usize) -> &mut End {
        if which == 0 {
            &mut self.start
        } else {
            &mut self.end
        }
    }

    /// As formas a que está presa.
    pub fn targets(&self) -> impl Iterator<Item = ElementId> + '_ {
        self.ends().into_iter().filter_map(|e| match e {
            End::Bound { target, .. } => Some(target),
            End::Free(_) => None,
        })
    }
}
