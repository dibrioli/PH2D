//! **A seta do quadro** (W2): a RELAÇÃO entre duas pontas — a geometria da rota não se guarda, é
//! derivada (`ph2d-board-route`) e refaz-se quando uma ponta ou um obstáculo mexe.
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
}

/// Como a rota se desenha.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    Straight,
    /// Só segmentos horizontais e verticais, desviando das formas no caminho — o de nascença.
    #[default]
    Elbow,
    /// A MESMA rota do cotovelo, suavizada: desvia das formas como ele.
    Curved,
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
}

impl Head {
    /// Todas, na ordem do enum — a que a barra oferece.
    pub const ALL: [Head; 8] = [
        Head::None,
        Head::Arrow,
        Head::Triangle,
        Head::Diamond,
        Head::DiamondOpen,
        Head::Circle,
        Head::CircleOpen,
        Head::Bar,
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
